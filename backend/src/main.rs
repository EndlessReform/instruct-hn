use axum::{routing::get, Router};
use backend_lib::{
    config::Config, db, firebase_listener::FirebaseListener, sync_service::SyncService,
};
use std::time::Instant;

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
    let pool_config =
        AsyncDieselConnectionManager::<diesel_async::AsyncPgConnection>::new(&config.db_url);
    let pool = Pool::builder(pool_config)
        .build()
        .expect("Could not establish connection!");

    // Temporary
    let sync_service = SyncService::new(config.hn_api_url, pool.clone(), 200);

    let start_time = Instant::now();
    sync_service
        .fetch_all_data(None)
        .await
        .expect("Catchup failed");
    let elapsed_time = start_time.elapsed();
    info!("Catchup time elapsed: {:?}", elapsed_time);
    // let text: &str = "When I was a young boy, my father took me into the city to see a marching band";

    /*
    let embedder = E5Embedder::new(&config.triton_server_addr)
        .await
        .expect("Cannot connect to Triton!");

    debug!("Embedder initialized");
    let embedding = embedder.encode(text).await.expect("Embedding failed!");
    println!("{:?}", embedding); */
    /*
    let app = Router::new().route("/", get(|| async { "Hello, world!" }));

    axum::Server::bind(&"0.0.0.0:3000".parse().unwrap())
        .serve(app.into_make_service())
        .await
        .unwrap(); */
}
