mod kafka;
mod schema;

use std::{env, time::Duration};

use crate::schema::Product;
use actix_web::{web, App, HttpResponse, HttpServer, Responder};
use dotenv::dotenv;
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
    sql::Value,
    Notification, Result, Surreal,
};
use tokio::task;

struct State {
    db: Surreal<Client>,
}

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    dotenv().ok();

    // LOAD ENV VARS
    let surreal_url = env::var("SURREAL_URL").expect("SURREAL_URL must be set.");
    let surreal_ns = env::var("SURREAL_NS").expect("SURREAL_NS must be set.");
    let surreal_db = env::var("SURREAL_DB").expect("SURREAL_DB must be set.");
    let surreal_user = env::var("SURREAL_USER").expect("SURREAL_USER must be set.");
    let surreal_password = env::var("SURREAL_PW").expect("SURREAL_PW must be set.");
    let kafka_broker = env::var("KAFKA_BROKER").expect("KAFKA_BROKER must be set.");

    // INIT DATABASE
    let db = Surreal::new::<Ws>(&surreal_url)
        .await
        .expect("Failed to connect to the Surreal client");
    db.signin(Root {
        username: &surreal_user,
        password: &surreal_password,
    })
    .await
    .expect("Failed to authenticate");
    db.use_ns(surreal_ns)
        .use_db(surreal_db)
        .await
        .expect("Failed to access the Database");

    let db_clone = db.clone();

    // CREATE KAFKA PRODUCER
    let stock_producer = producer(&kafka_broker).await;

    // SPAWN A NEW THREAD TO EXECUTE KAFKA PRODUCER
    task::spawn(async move {
        stream_stock_changes(&db_clone, &stock_producer)
            .await
            .expect("failed to stream");
    });

    // EXECUTE SERVER
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

// FETCH ALL PRODUCTS FROM INVENTORY
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

// LIVE STREAM STOCK UPDATES
// PRODUCE MESSAGES FOR EVERY STOCK UPDATE EVENT
async fn stream_stock_changes(
    db: &Surreal<Client>,
    stock_producer: &FutureProducer,
) -> surrealdb::Result<()> {
    let kafka_topic = env::var("KAFKA_TOPIC").expect("KAFKA_TOPIC must be set.");

    if let Ok(mut stream) = db.select("inventory_stock_events").live().await {
        while let Some(result) = stream.next().await {
            let res: Result<Notification<StockEvent>> = result;
            let data = &res.unwrap().data;

            stock_producer
                .send(
                    FutureRecord::to(&kafka_topic)
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
                .expect("FAILED TO PRODUCE THE MESSAGE");
        }
    } else {
        println!("Failed to stream")
    }

    Ok(())
}

// VALIDATE AND UPDATE STOCKS
async fn update_stock(
    product_id: web::Path<String>,
    state: web::Data<State>,
    payload: web::Json<UpdateProductStock>,
) -> impl Responder {
    if product_id.is_empty() {
        return HttpResponse::BadRequest().body("Invalid Product Id");
    }

    let db = &state.db;
    let mut available_units: u16 = 0;

    if let Ok(mut query_product) = db
        .query(format!(
            "SELECT units FROM inventory:{} WHERE units>={}",
            product_id, payload.units
        ))
        .await
    {
        if let Ok(Value::Array(arr)) = query_product.take(0) {
            if !arr.is_empty() {
                if let Value::Object(obj) = &arr[0] {
                    if let Some(Value::Number(units)) = obj.get("units") {
                        available_units = units.to_usize() as u16;
                    }
                }
            } else {
                return HttpResponse::NotFound().body("Product not found or insufficient units");
            }
        } else {
            return HttpResponse::InternalServerError().body("Unexpected query response format");
        }
    } else {
        return HttpResponse::InternalServerError().body("Server Error");
    }

    if let Ok(mut update_product) = db
        .query(format!(
            "UPDATE inventory:{} SET units={}",
            product_id,
            available_units - payload.units,
        ))
        .await
    {
        if let Ok(Value::Array(arr)) = update_product.take(0) {
            if !arr.is_empty() {
                HttpResponse::Ok().body("Product stock updated")
            } else {
                HttpResponse::NotFound().body("Product not found or insufficient units")
            }
        } else {
            HttpResponse::InternalServerError().body("Unexpected query response format")
        }
    } else {
        HttpResponse::InternalServerError().body("Server Error")
    }
}
