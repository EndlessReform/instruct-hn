pub mod config;
pub mod db;
pub mod firebase_listener;
pub mod hn_processor;
pub mod sync_service;
pub mod triton;

use crate::config::Config;
use crate::db::models::Item;
use crate::firebase_listener::FirebaseListener;
// use crate::hn_processor::embedder::E5Embedder;
use axum::{routing::get, Router};

use clap::Parser;
use diesel::dsl::max;
use diesel::prelude::*;
use diesel_async::pooled_connection::deadpool::Pool;
use diesel_async::pooled_connection::AsyncDieselConnectionManager;
use diesel_async::RunQueryDsl;
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

    let fb = FirebaseListener::new(&config.hn_api_url).unwrap();
    println!("{:?}", fb.get_max_id().await);
    let config =
        AsyncDieselConnectionManager::<diesel_async::AsyncPgConnection>::new(&config.db_url);
    let pool = Pool::builder(config)
        .build()
        .expect("Could not establish connection!");

    // Temporary
    let mut conn = pool.get().await.unwrap();
    //let results: Vec<Item> = items.filter(id.eq(30302618)).load(&mut conn).await.unwrap();
    let max_db_item: Option<i64> = items.select(max(id)).first(&mut conn).await.unwrap();
    println!("{:?}", max_db_item);

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
