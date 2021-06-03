#[macro_use]
extern crate diesel;
extern crate dotenv;
extern crate r2d2;
extern crate reqwest;
extern crate serde;
extern crate serenity;
extern crate tokio;

use crate::models::{Response, Stocks};
use bershka_notify::scan_stocks;
use diesel::prelude::*;
use diesel::r2d2::ConnectionManager;
use dotenv::dotenv;
use r2d2::Pool;
use r2d2_postgres::postgres::NoTls;
use r2d2_postgres::PostgresConnectionManager;
use serde::Deserialize;
use std::env;

mod lib;
mod models;
mod schema;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    dotenv().ok();
    let database_url = env::var("DATABASE_URL").expect("Missing required DATABASE_URL");
    let connection_manager = ConnectionManager::<PgConnection>::new(database_url);
    let pool = Pool::builder()
        .build(connection_manager)
        .expect("Failed to create connection pool");

    let stocks = scan_stocks(
        "https://www.bershka.com/itxrest/2/catalog/store/45109555/40259532/product/102872244/stock",
    )
    .await?;
    println!("{:?}", stocks);

    Ok(())
}
