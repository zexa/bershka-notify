#[macro_use]
extern crate diesel;
extern crate dotenv;
extern crate job_scheduler;
extern crate r2d2;
extern crate reqwest;
extern crate serde;
extern crate serenity;
extern crate tokio;

use crate::models::{JsonResponse, JsonStock, JsonStocks, NewStock, NewStocks, Stock, Stocks};
use anyhow::Context;
use chrono::{DateTime, Local};
use diesel::prelude::*;
use diesel::r2d2::ConnectionManager;
use dotenv::dotenv;
use job_scheduler::{Job, JobScheduler};
use r2d2::Pool;
use serenity::client::EventHandler;
use serenity::http::Http;
use serenity::model::id::ChannelId;
use serenity::Client;
use std::env;
use std::sync::Arc;
use std::thread::sleep;
use std::time::{Duration, SystemTime};

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
    http: Arc<Http>,
    channel: &ChannelId,
) -> anyhow::Result<(Stocks, Vec<Stock>)> {
    let stocks = scan_stocks(s).await?;

    let stocks_entity = match find_stocks(pool, &stocks.product_id)? {
        None => create_stocks(pool, &stocks.product_id)?,
        Some(found_stocks) => update_stocks(pool, found_stocks)?,
    };

    let mut stock_entities: Vec<Stock> = vec![];
    for st in stocks.stocks {
        stock_entities.push(stock(pool, &stocks_entity, &st, http.clone(), &channel).await?);
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

pub fn is_stock_same(stock: &Stock, json: &JsonStock) -> bool {
    stock.availability == json.availability && stock.type_threshold == json.type_threshold
}

pub async fn stock(
    pool: &Pool<ConnectionManager<PgConnection>>,
    stocks: &Stocks,
    json: &JsonStock,
    http: Arc<Http>,
    channel: &ChannelId,
) -> anyhow::Result<Stock> {
    let stock_entity = match find_stock(pool, stocks, json)? {
        None => create_stock(pool, stocks, json)?,
        Some(stock) => {
            if !is_stock_same(&stock, json) {
                let updated = update_stock(pool, &stock, json)?;

                let _ = channel.say(http, "@Oggy Updated!").await?;

                updated
            } else {
                stock
            }
        }
    };

    Ok(stock_entity)
}

pub fn get_pool(database_url: String) -> anyhow::Result<Pool<ConnectionManager<PgConnection>>> {
    let connection_manager = ConnectionManager::<PgConnection>::new(database_url);
    let pool = Pool::builder()
        .max_size(15)
        .build(connection_manager)
        .expect("Failed to create connection pool");

    Ok(pool)
}

struct Handler;

impl EventHandler for Handler {}

pub async fn get_discord(token: String) -> anyhow::Result<Client> {
    let mut client = Client::builder(token).event_handler(Handler).await?;

    let _ = client.start();

    Ok(client)
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    dotenv().ok();
    let database_url = env::var("DATABASE_URL")?;
    let pool = get_pool(database_url)?;

    let discord_token = env::var("DISCORD_TOKEN")?;
    let discord = get_discord(discord_token).await?;

    let discord_cache_and_http = discord.cache_and_http.clone();
    let discord_http = discord_cache_and_http.http.clone();

    let discord_channel = env::var("DISCORD_CHANNEL")?.parse::<u64>()?;
    let channel = ChannelId(discord_channel);

    channel
        .say(discord_http.clone(), "Starting...")
        .await
        .context("Announcing bot start")?;

    let mut scheduler = JobScheduler::new();
    scheduler.add(Job::new("0 0,5,10,15,20,25,30,35,40,45,50,55 * * * * *".parse().unwrap(), || {
        let datetime: DateTime<Local> = SystemTime::now().into();
        println!("Last scan at {}", datetime.format("%Y-%m-%d %T"));

        let _ = stocks(
            &pool,
            "https://www.bershka.com/itxrest/2/catalog/store/45109555/40259532/product/102872244/stock",
            discord_http.clone(),
            &channel,
        );
    }));

    loop {
        scheduler.tick();

        sleep(Duration::from_millis(1000));
    }

    Ok(())
}
