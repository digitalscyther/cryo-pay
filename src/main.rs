mod monitor;
mod utils;
mod api;
mod events;

#[tokio::main]
async fn main() -> Result<(), String> {
    let api_handle = tokio::spawn(async move {
        api::run_api().await
    });

    // let _ = api_handle.await;

    let monitor_handle = tokio::spawn(async move {
        monitor::run_monitor(events::just_print_log).await
    });

    let _ = tokio::join!(api_handle, monitor_handle);

    Ok(())
}
