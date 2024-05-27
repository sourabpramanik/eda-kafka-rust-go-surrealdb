// mod kafka;
mod schema;

use crate::schema::Product;
use actix_web::{web, App, HttpResponse, HttpServer, Responder};
use schema::UpdateProductStock;
use surrealdb::{
    engine::remote::ws::{Client, Ws},
    opt::auth::Root,
    Notification, Surreal,
};

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

    dbg!(products);
    HttpResponse::Ok().body("Hey there!")
}

async fn update_stock(
    product_id: web::Path<String>,
    state: web::Data<State>,
    payload: web::Json<UpdateProductStock>,
) -> impl Responder {
    let db = &state.db;

    let update = db
        .query(format!(
            "UPDATE inventory:{} SET units={}",
            product_id, payload.units,
        ))
        .await;
    dbg!(update);
    HttpResponse::Ok().body("Hey there!")
}
