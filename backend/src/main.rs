pub mod config;
pub mod hn_processor;
pub mod triton;

use crate::config::Config;
use crate::hn_processor::embedder::E5Embedder;

use clap::Parser;
use dotenv::dotenv;
use log::{debug, error, info, warn};

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
    info!("Starting embedding backend");
    dotenv().ok();

    let config = Config::from_env().expect("Config incorrectly specified");
    env_logger::init();
    // let args = Cli::parse();
    debug!("Config loaded");

    let text = "When I was a young boy, my father took me into the city to see a marching band";

    let embedder = E5Embedder::new(&config.triton_server_addr)
        .await
        .expect("Cannot connect to Triton!");

    debug!("Embedder initialized");
    let embedding = embedder.encode(text).await.expect("Embedding failed!");
    println!("{:?}", embedding);
}
