extern crate serenity;
extern crate reqwest;
extern crate tokio;
extern crate serde;
extern crate dotenv;
extern crate r2d2;
extern crate r2d2_postgres;

use serde::Deserialize;
use dotenv::dotenv;
use std::env;
use r2d2_postgres::PostgresConnectionManager;
use r2d2_postgres::postgres::NoTls;
use r2d2::Pool;

#[derive(Debug, Deserialize)]
struct Stock {
    id: i32,
    availability: String,
    #[serde(rename(deserialize = "typeThreshold"))]
    type_threshold: String,
}

#[derive(Debug, Deserialize, PartialEq)]
struct Stocks {
    #[serde(rename(deserialize = "productId"))]
    product_id: i32,
    stocks: Vec<Stock>,
}

#[derive(Debug, Deserialize)]
struct Response {
    stocks: Vec<Stocks>,
}

async fn scan_stocks(url: &str) -> anyhow::Result<Stocks> {
    let response = reqwest::get(url).await?;
    let mut json = response.json::<Response>().await?;
    Ok(json.stocks.remove(0))
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    dotenv().ok();
    let database_url = env::var("DATABASE_URL").expect("Missing required DATABASE_URL");
    let connection_manager = PostgresConnectionManager::new(
        database_url.as_str().parse().expect("Failed parsing Postgres connection manager configs"),
        NoTls
    );
    let pool = Pool::new(connection_manager).expect("Could not initiate database connection pool");

    let stocks = scan_stocks("https://www.bershka.com/itxrest/2/catalog/store/45109555/40259532/product/102872244/stock").await?;
    println!("{:?}", stocks);
    Ok(())
}
