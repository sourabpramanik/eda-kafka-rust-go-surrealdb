mod kafka;
mod schema;

use std::time::Duration;

use crate::schema::Product;
use actix_web::{web, App, HttpResponse, HttpServer, Responder};
use futures_util::StreamExt;
use kafka::producer;
use rdkafka::{
    message::{Header, OwnedHeaders},
    producer::{FutureProducer, FutureRecord},
};
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
    let stock_producer = producer("localhost:9092").await;

    task::spawn(async move {
        stream_stock_changes(&db_clone, &stock_producer)
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

async fn stream_stock_changes(
    db: &Surreal<Client>,
    stock_producer: &FutureProducer,
) -> surrealdb::Result<()> {
    let mut stream = db.select("notification").live().await?;

    // Process updates as they come in.
    while let Some(result) = stream.next().await {
        println!("ddsadasasdad");
        let res: Result<Notification<StockEvent>> = result;
        let data = &res.unwrap().data;

        stock_producer
            .send(
                FutureRecord::to("stock_update")
                    .payload(&format!(
                        "Message {}",
                        &serde_json::to_string(data).unwrap()
                    ))
                    .key(&format!("Key {}", 1))
                    .headers(OwnedHeaders::new().insert(Header {
                        key: "header_key",
                        value: Some("header_value"),
                    })),
                Duration::from_secs(0),
            )
            .await
            .expect("Failed to send message");
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
