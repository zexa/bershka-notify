use crate::schema::stock;
use crate::schema::stocks;
use diesel::Associations;
use diesel::Identifiable;
use diesel::Insertable;
use diesel::Queryable;
use serde::Deserialize;

// Queryable variants
#[derive(Identifiable, Debug, PartialEq, Queryable, Associations)]
#[belongs_to(Stocks)]
#[table_name = "stock"]
pub struct Stock {
    id: i32,
    foreign_id: i32,
    availability: String,
    type_threshold: String,
    stocks_id: Option<i32>,
}

#[derive(Identifiable, Debug, PartialEq, Queryable)]
#[table_name = "stocks"]
pub struct Stocks {
    pub id: i32,
    pub product_id: i32,
}

// Insertable variants
#[derive(Insertable)]
#[table_name = "stock"]
pub struct NewStock<'a> {
    pub foreign_id: &'a i32,
    pub availability: &'a String,
    pub type_threshold: &'a String,
    pub stocks_id: &'a i32,
}

#[derive(Insertable)]
#[table_name = "stocks"]
pub struct NewStocks<'a> {
    pub product_id: &'a i32,
}

// Deserializable variants
#[derive(Debug, Deserialize, PartialEq)]
pub struct JsonStock {
    #[serde(rename(deserialize = "id"))]
    pub foreign_id: i32,
    pub availability: String,
    #[serde(rename(deserialize = "typeThreshold"))]
    pub type_threshold: String,
}

#[derive(Debug, Deserialize, PartialEq)]
pub struct JsonStocks {
    #[serde(rename(deserialize = "productId"))]
    pub product_id: i32,
    pub stocks: Vec<JsonStock>,
}

#[derive(Debug, Deserialize)]
pub struct JsonResponse {
    pub stocks: Vec<JsonStocks>,
}
