use std::sync::Arc;
use tracing::Level;

mod monitor;
mod utils;
mod api;
mod events;
mod network;

#[tokio::main]
async fn main() -> Result<(), String> {
    tracing_subscriber::fmt().json()
        .with_max_level(Level::ERROR)
        .with_env_filter(tracing_subscriber::EnvFilter::from_default_env())
        .init();

    let test = false;

    let networks = network::Network::vec_from_env("NETWORKS")?;
    let db = api::state::DB::new().await?;

    let monitor_networks = networks.clone();
    let monitor_handle = tokio::spawn(async move {
        monitor::run_monitor(test, monitor_networks, Arc::new(db)).await
    });

    let api_handle = tokio::spawn(async move {
        api::run_api(networks).await
    });

    let _ = tokio::join!(api_handle, monitor_handle);

    Ok(())
}
