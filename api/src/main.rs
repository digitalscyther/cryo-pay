use tracing::Level;

mod monitor;
mod utils;
mod api;
mod db;
mod events;
mod network;
mod telegram;
mod mailer;
mod payments;

#[tokio::main]
async fn main() -> Result<(), String> {
    tracing_subscriber::fmt().json()
        .with_max_level(Level::ERROR)
        .with_env_filter(tracing_subscriber::EnvFilter::from_default_env())
        .init();

    let test = false;

    let networks = network::Network::default_vec()?;
    let db = api::state::DB::new().await?;
    let telegram_client = telegram::TelegramClient::new().await?;

    let monitor_networks = networks.clone();
    let (monitor_db, monitor_telegram_client) = (db.clone(), telegram_client.clone());
    let monitor_handle = tokio::spawn(async move {
        monitor::run_monitor(test, monitor_networks, monitor_db, monitor_telegram_client).await
    });

    let (api_db, api_telegram_client) = (db.clone(), telegram_client.clone());
    let api_handle = tokio::spawn(async move {
        api::run_api(networks, api_db, api_telegram_client).await
    });

    let (bot_db, bot_telegram_client) = (db.clone(), telegram_client.clone());
    let bot_handle = tokio::spawn(async move {
        bot_telegram_client.run_as_bot(bot_db).await
    });

    let _ = tokio::join!(api_handle, monitor_handle, bot_handle);

    Ok(())
}
