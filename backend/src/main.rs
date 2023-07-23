pub mod config;
pub mod db;
pub mod firebase_listener;
pub mod hn_processor;
pub mod triton;

use crate::config::Config;
use crate::db::models::Item;
// use crate::hn_processor::embedder::E5Embedder;
use axum::{routing::get, Router};

use clap::Parser;
use diesel::prelude::*;
use diesel_async::{AsyncConnection, AsyncPgConnection, RunQueryDsl};
use dotenv::dotenv;
use log::{debug, info};

#[derive(Parser, Debug)]
#[clap(about = "Backend server for instruct-hn")]
struct Cli {
    #[clap(short, long)]
    /// Catch up on previous data
    catchup: Option<bool>,

    #[clap(short, long)]
    /// Listen for HN updates and persist them to DB
    realtime: Option<bool>,
}

#[tokio::main]
async fn main() {
    use self::db::schema::items::dsl::*;

    info!("Starting embedding backend");
    dotenv().ok();

    let config = Config::from_env().expect("Config incorrectly specified");
    env_logger::init();
    // let args = Cli::parse();
    debug!("Config loaded");
    let conn = &mut AsyncPgConnection::establish(&config.db_url).await.unwrap();

    /*
    let results: Vec<Item> = items.filter(id.eq(30302618)).load(conn).await.unwrap();
    println!("{:?}", results.len()); */

    // let text: &str = "When I was a young boy, my father took me into the city to see a marching band";

    /*
    let embedder = E5Embedder::new(&config.triton_server_addr)
        .await
        .expect("Cannot connect to Triton!");

    debug!("Embedder initialized");
    let embedding = embedder.encode(text).await.expect("Embedding failed!");
    println!("{:?}", embedding); */
    let app = Router::new().route("/", get(|| async { "Hello, world!" }));

    axum::Server::bind(&"0.0.0.0:3000".parse().unwrap())
        .serve(app.into_make_service())
        .await
        .unwrap();
}
