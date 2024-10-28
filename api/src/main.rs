use std::sync::Arc;

mod monitor;
mod utils;
mod api;
mod events;

#[tokio::main]
async fn main() -> Result<(), String> {
    let test = false;

    let postgres_db = Arc::new(api::state::DB::new().await?);

    let monitor_handle = match test {
        false => tokio::spawn(async move {
            let _ = monitor::run_monitor(move |log| {
                let postgres_db = Arc::clone(&postgres_db);
                events::process_log(postgres_db, log)
            }).await;
        }),
        true => tokio::spawn(async move {
            let _ = monitor::run_monitor(events::just_print_log).await;
        })
    };

    let api_handle = tokio::spawn(async move {
        api::run_api().await
    });

    let _ = tokio::join!(api_handle, monitor_handle);

    Ok(())
}
