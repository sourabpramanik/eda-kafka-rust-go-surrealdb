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
    pub created_at: Datetime,
    pub message: String,
    pub product_id: ProductThing,
    pub units_before: u16,
    pub units_after: u16,
}
