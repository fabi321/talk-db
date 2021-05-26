#[macro_use]
extern crate lazy_static;
extern crate reqwest;

use compute::{Compute, ComputeCount};
use person::{Person, Entry};

mod anne_will;
mod hart_aber_fair;
mod maischberger;
mod zdf;
mod maybrit_illner;
mod markus_lanz;
mod compute;
mod person;

#[tokio::main]
async fn main() {
    let pool = sqlx::SqlitePool::connect("data.db").await.unwrap();
    sqlx::query_file!("src/init.sql").execute(&pool).await.unwrap();
    anne_will::scrape(&pool).await;
    hart_aber_fair::scrape(&pool).await;
    maischberger::scrape(&pool).await;
    maybrit_illner::scrape(&pool).await;
    markus_lanz::scrape(&pool).await;
    pool.close().await;
}

