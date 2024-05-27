use serde::{Deserialize, Serialize};
use surrealdb::sql::Id;

#[derive(Debug, Deserialize)]
pub struct Product {
    pub id: ProductThing,
    pub name: String,
    pub price: u16,
    pub units: u16,
}

#[derive(Debug, Deserialize)]
struct ProductThing {
    id: Id,
}

#[derive(Debug, Deserialize)]
pub struct UpdateProductStock {
    pub units: u16,
}
