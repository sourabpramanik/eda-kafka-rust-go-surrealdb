use serde::{Deserialize, Serialize};
use surrealdb::sql::{Datetime, Id};

#[allow(dead_code)]
#[derive(Debug, Deserialize, Serialize)]
pub struct ProductThing {
    id: Id,
}
#[derive(Debug, Deserialize, Serialize)]
pub struct Product {
    pub id: ProductThing,
    pub name: String,
    pub price: u16,
    pub units: u16,
}

#[derive(Debug, Deserialize)]
pub struct UpdateProductStock {
    pub units: u16,
}

#[allow(dead_code)]
#[derive(Debug, Deserialize, Serialize)]
pub struct EventThing {
    id: Id,
}
#[derive(Debug, Deserialize, Serialize)]
pub struct StockEvent {
    pub id: EventThing,
    pub time: Datetime,
    pub action: String,
    pub product: ProductThing,
    pub before_update: u16,
    pub after_update: u16,
}
