//! Animal facts API server
//!
//! To test it, just run `cargo run` and request `http://127.0.0.1:8888/fact`

#![deny(missing_docs)]

use axum::Server;
use std::env;
use tracing::{info, instrument, trace, warn};

mod config;
mod facts;
mod handlers;

#[tokio::main]
#[instrument]
async fn main() {
  tracing_subscriber::fmt::init();

  trace!("load json config file");
  let config_file_name =
    env::var("RUST_FACTS_CONFIG_FILE").unwrap_or("config.json".to_string());
  let confg_data = config::ConfigData::new(&config_file_name)
    .expect("configuration file loading error");

  trace!("constructing fact resolver");
  let fact_resolver = facts::FactResolver::new(&confg_data.animals);

  info!(
    "server is starting to listen on {}",
    confg_data.server.address
  );
  Server::bind(
    &confg_data
      .server
      .address
      .parse()
      .expect("failed to parse listening address value"),
  )
  .serve(handlers::app(fact_resolver).into_make_service())
  .await
  .expect("failed to start axum web server")
}
