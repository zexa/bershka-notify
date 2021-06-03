#[macro_use]
extern crate diesel;

use crate::models::{JsonResponse, JsonStocks, NewStocks, Stocks};
use diesel::prelude::*;
use diesel::r2d2::ConnectionManager;
use r2d2::Pool;

mod models;
mod schema;

pub async fn scan_stocks(url: &str) -> anyhow::Result<JsonStocks> {
    let response = reqwest::get(url).await?;
    let mut json = response.json::<JsonResponse>().await?;
    Ok(json.stocks.remove(0))
}

pub async fn update_stocks(
    pool: Pool<ConnectionManager<PgConnection>>,
    foreign_product_id: &i32,
) -> anyhow::Result<()> {
    use schema::stocks::dsl::*;

    let _new_stocks = NewStocks {
        product_id: foreign_product_id,
    };

    let existing_stocks = stocks
        .filter(product_id.eq(foreign_product_id))
        .limit(1)
        .load::<Stocks>(&pool.get().expect("Could not get connection"));

    println!("{:?}", existing_stocks);

    // let stocks = match existing_stocks {
    //     Some(es) => es,
    //     None => diesel::insert_into(stocks::table)
    //         .values(&new_stocks)
    //         .get_result(&pool.get().expect("Could not get connection")),
    // };

    Ok(())
}
