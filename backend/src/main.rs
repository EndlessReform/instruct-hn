pub mod config;
pub mod hn_processor;
pub mod triton;

use crate::config::Config;

use clap::Parser;
use dotenv::dotenv;

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
    dotenv().ok();

    let config = Config::from_env().expect("Config incorrectly specified");
    let args = Cli::parse();
    println!("Hello, world!");
}
