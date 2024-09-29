use std::sync::Arc;

mod monitor;
mod utils;
mod api;
mod events;

#[tokio::main]
async fn main() -> Result<(), String> {
    let postgres_db = Arc::new(api::state::DB::new().await?);

    let monitor_handle = tokio::spawn(async move {
        let _ = monitor::run_monitor(move |log| {
            let postgres_db = Arc::clone(&postgres_db);
            events::set_invoice_paid(postgres_db, log)
        }).await;
    });

    let api_handle = tokio::spawn(async move {
        api::run_api().await
    });

    let _ = tokio::join!(api_handle, monitor_handle);

    Ok(())
}
