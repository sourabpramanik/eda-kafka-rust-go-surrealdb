// mod kafka;
mod schema;

use crate::schema::Product;
use actix_web::{web, App, HttpResponse, HttpServer, Responder};
use futures_util::StreamExt;
use schema::{StockEvent, UpdateProductStock};
use surrealdb::{
    engine::remote::ws::{Client, Ws},
    opt::auth::Root,
    Notification, Result, Surreal,
};
use tokio::task;

struct State {
    db: Surreal<Client>,
}

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    let db = Surreal::new::<Ws>("127.0.0.1:8000")
        .await
        .expect("Failed to connect to the Surreal client");

    db.signin(Root {
        username: "root",
        password: "root",
    })
    .await
    .expect("Failed to authenticate");

    db.use_ns("foo")
        .use_db("ecommerce")
        .await
        .expect("Failed to access the Database");

    let db_clone = db.clone();
    task::spawn(async move {
        stream_stock_changes(&db_clone)
            .await
            .expect("failed to stream");
    });

    HttpServer::new(move || {
        App::new()
            .app_data(web::Data::new(State { db: db.to_owned() }))
            .service(
                web::scope("/inventory")
                    .service(web::resource("").route(web::get().to(get_inventory_products)))
                    .service(
                        web::scope("/{product_id}")
                            .service(web::resource("").route(web::patch().to(update_stock))),
                    ),
            )
    })
    .bind(("127.0.0.1", 3000))?
    .run()
    .await
}

async fn get_inventory_products(state: web::Data<State>) -> impl Responder {
    let db = &state.db;

    let products: Vec<Product> = match db.select("inventory").await {
        Ok(val) => val,
        Err(e) => {
            dbg!(e);
            return HttpResponse::InternalServerError().body("Server problems!!");
        }
    };
    HttpResponse::Ok().json(products)
}

async fn stream_stock_changes(db: &Surreal<Client>) -> surrealdb::Result<()> {
    let mut stream = db.select("notification").live().await?;

    // Process updates as they come in.
    while let Some(result) = stream.next().await {
        // Do something with the notification
        handle(result);
    }

    // Handle the result of the live query notification
    fn handle(result: Result<Notification<StockEvent>>) {
        println!("Received notification: {:?}", result);
    }

    Ok(())
}

async fn update_stock(
    product_id: web::Path<String>,
    state: web::Data<State>,
    payload: web::Json<UpdateProductStock>,
) -> impl Responder {
    let db = &state.db;

    match db
        .query(format!(
            "UPDATE inventory:{} SET units={}",
            product_id, payload.units,
        ))
        .await
    {
        Ok(_) => HttpResponse::Ok().body("Stock Updated"),
        Err(_) => HttpResponse::InternalServerError().body("Server error"),
    }
}
