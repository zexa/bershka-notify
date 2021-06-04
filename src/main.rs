#[macro_use]
extern crate diesel;
extern crate dotenv;
extern crate r2d2;
extern crate reqwest;
extern crate serde;
extern crate serenity;
extern crate tokio;

use crate::models::{JsonResponse, JsonStock, JsonStocks, NewStock, NewStocks, Stock, Stocks};
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

    let existing_stocks: Option<Stocks> = stocks
        .filter(product_id.eq(foreign_product_id))
        .first::<Stocks>(&pool.get()?)
        .optional()?;

    Ok(existing_stocks)
}

pub fn create_stocks(
    pool: &Pool<ConnectionManager<PgConnection>>,
    product_id: &i32,
) -> anyhow::Result<Stocks> {
    use crate::schema::stocks;

    let new_stocks = NewStocks { product_id };

    let result: Stocks = diesel::insert_into(stocks::table)
        .values(&new_stocks)
        .get_result(&pool.get().expect("Could not get connection"))?;

    Ok(result)
}

// Placeholder for the future I guess?
pub fn update_stocks(
    _pool: &Pool<ConnectionManager<PgConnection>>,
    current: Stocks,
) -> anyhow::Result<Stocks> {
    Ok(current)
}

pub async fn stocks(
    pool: &Pool<ConnectionManager<PgConnection>>,
    s: &str,
) -> anyhow::Result<(Stocks, Vec<Stock>)> {
    let stocks = scan_stocks(s).await?;

    let stocks_entity = match find_stocks(pool, &stocks.product_id)? {
        None => create_stocks(pool, &stocks.product_id)?,
        Some(found_stocks) => update_stocks(pool, found_stocks)?,
    };

    let mut stock_entities: Vec<Stock> = vec![];
    for st in stocks.stocks {
        stock_entities.push(stock(pool, &stocks_entity, &st)?);
    }

    Ok((stocks_entity, stock_entities))
}

pub fn find_stock(
    pool: &Pool<ConnectionManager<PgConnection>>,
    stocks: &Stocks,
    json: &JsonStock,
) -> anyhow::Result<Option<Stock>> {
    use crate::schema::stock::dsl::*;

    let existing_stock: Option<Stock> = Stock::belonging_to(stocks)
        .filter(foreign_id.eq(json.foreign_id))
        .first(&pool.get()?)
        .optional()?;

    Ok(existing_stock)
}

pub fn create_stock(
    pool: &Pool<ConnectionManager<PgConnection>>,
    stocks: &Stocks,
    json: &JsonStock,
) -> anyhow::Result<Stock> {
    use crate::schema::stock;

    let new_stock = NewStock {
        foreign_id: &json.foreign_id,
        availability: &json.availability,
        type_threshold: &json.type_threshold,
        stocks_id: &stocks.id,
    };

    let result: Stock = diesel::insert_into(stock::table)
        .values(&new_stock)
        .get_result(&pool.get()?)?;

    Ok(result)
}

pub fn update_stock(
    pool: &Pool<ConnectionManager<PgConnection>>,
    st: &Stock,
    json: &JsonStock,
) -> anyhow::Result<Stock> {
    use crate::schema::stock::dsl::*;

    let updated_stock: Stock = diesel::update(st)
        .set((
            availability.eq(&json.availability),
            type_threshold.eq(&json.type_threshold),
        ))
        .get_result(&pool.get()?)?;

    Ok(updated_stock)
}

pub fn stock(
    pool: &Pool<ConnectionManager<PgConnection>>,
    stocks: &Stocks,
    json: &JsonStock,
) -> anyhow::Result<Stock> {
    let stock_entity = match find_stock(pool, stocks, json)? {
        None => create_stock(pool, stocks, json)?,
        Some(stock) => update_stock(pool, &stock, json)?,
    };

    Ok(stock_entity)
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    dotenv().ok();
    let database_url = env::var("DATABASE_URL").expect("Missing required DATABASE_URL");
    let connection_manager = ConnectionManager::<PgConnection>::new(database_url);
    let pool = Pool::builder()
        .max_size(15)
        .build(connection_manager)
        .expect("Failed to create connection pool");

    let _ = stocks(
        &pool,
        "https://www.bershka.com/itxrest/2/catalog/store/45109555/40259532/product/102872244/stock",
    )
    .await?;

    Ok(())
}
