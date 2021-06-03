#[macro_use]
extern crate diesel;
extern crate dotenv;
extern crate r2d2;
extern crate reqwest;
extern crate serde;
extern crate serenity;
extern crate tokio;

use crate::models::{JsonResponse, JsonStocks, NewStocks, Stocks};
use diesel::associations::HasTable;
use diesel::prelude::*;
use diesel::r2d2::ConnectionManager;
use dotenv::dotenv;
use r2d2::Pool;
use std::env;

mod models;
mod schema;

pub async fn scan_stocks(url: &str) -> anyhow::Result<JsonStocks> {
    let response = reqwest::get(url).await?;
    let mut json = response.json::<JsonResponse>().await?;
    Ok(json.stocks.remove(0))
}

pub fn find_stocks(
    pool: &Pool<ConnectionManager<PgConnection>>,
    foreign_product_id: &i32,
) -> anyhow::Result<Option<Stocks>> {
    use crate::schema::stocks::dsl::*;

    let existing_stocks = stocks
        .filter(product_id.eq(foreign_product_id))
        .limit(1)
        .load::<Stocks>(&pool.get().expect("Could not get connection"))
        .expect("Could not get existing stocks");

    let mut result: Option<Stocks> = None;

    match existing_stocks.len() {
        0 => result = None,
        1 => result = Some(existing_stocks.remove(0).unwrap()),
        _ => Err(()),
    };

    Ok(result)
}

pub fn create_stocks(
    pool: &Pool<ConnectionManager<PgConnection>>,
    foreign_product_id: &i32,
) -> anyhow::Result<Stocks> {
    use crate::schema::stocks;
    use crate::schema::stocks::columns::product_id;

    let new_stocks = NewStocks {
        product_id: foreign_product_id,
    };

    let result: Stocks = diesel::insert_into(stocks::table)
        .values(&new_stocks)
        .get_result(&pool.get().expect("Could not get connection"))?;

    Ok(result)
}

pub fn update_stocks(
    pool: &Pool<ConnectionManager<PgConnection>>,
    current: Stocks,
) -> anyhow::Result<()> {
    Ok(())
}

async fn stocks(pool: &Pool<ConnectionManager<PgConnection>>, s: &str) -> anyhow::Result<()> {
    let stocks = scan_stocks(s).await?;

    match find_stocks(pool, &stocks.product_id)? {
        None => create_stocks(pool, &stocks.product_id),
        Some(found_stocks) => update_stocks(pool, found_stocks),
    };

    Ok(())
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    dotenv().ok();
    let database_url = env::var("DATABASE_URL").expect("Missing required DATABASE_URL");
    let connection_manager = ConnectionManager::<PgConnection>::new(database_url);
    let pool = Pool::builder()
        .limit(15)
        .build(connection_manager)
        .expect("Failed to create connection pool");

    stocks(
        &pool,
        "https://www.bershka.com/itxrest/2/catalog/store/45109555/40259532/product/102872244/stock",
    );

    Ok(())
}
