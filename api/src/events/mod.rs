pub mod notifications;

use std::fmt::Debug;
use std::sync::Arc;
use bigdecimal::BigDecimal;
use chrono::{DateTime, Utc};
use ethers::{abi::RawLog, prelude::*};
use futures::future::join_all;
use tracing::info;
use uuid::Uuid;
use crate::api::state::DB;
use crate::db::Invoice;
use crate::events::notifications::Notifier;
use crate::monitoring::app_state::MonitorAppState;
use crate::utils;


#[derive(Debug, Clone, EthEvent)]
struct NewTransaction {
    #[ethevent(indexed)]
    block_timestamp: U256,
}

#[derive(Debug, Clone, EthEvent)]
pub struct PayInvoiceEvent {
    pub invoice_id: String,
    #[ethevent(indexed)]
    pub seller: Address,
    #[ethevent(indexed)]
    pub payer: Address,
    pub paid_at: U128,
    pub amount: U128,
}

pub async fn just_print_log(log: &Log) -> Result<(), String> {
    parse_event(log)
        .map(|event| info!("New transaction event: {:?}", event))
}


pub async fn set_invoice_paid(postgres_db: &DB, event: PayInvoiceEvent) -> Result<Invoice, String> {
    let invoice_id = Uuid::parse_str(&event.invoice_id)
        .map_err(|err| utils::make_err(Box::new(err), "parse invoice_id"))?;

    let paid_at = DateTime::<Utc>::from_timestamp(event.paid_at.as_u64() as i64, 0)
        .map(|dt| dt.naive_utc())
        .ok_or_else(|| "Invalid timestamp".to_string())?;

    postgres_db.set_invoice_paid(
        invoice_id,
        &format!("{:#020x}", event.seller),
        BigDecimal::from(event.amount.as_u128()) / BigDecimal::from(1_000_000),
        &format!("{:#020x}", event.payer),
        paid_at,
    ).await
}

pub fn parse_event(log: &Log) -> Result<PayInvoiceEvent, String> {
    let log: RawLog = log.clone().into();
    <PayInvoiceEvent as EthEvent>::decode_log(&log)
        .map_err(|err| utils::make_err(Box::new(err), "decode log"))
}

pub async fn process_log(app_state: &MonitorAppState, log: &Log) -> Result<(), String> {
    let event = parse_event(log)?;
    let invoice = set_invoice_paid(&app_state.db, event).await?;

    if let Some(user_id) = invoice.user_id {
        let tasks = Notifier::get_notifiers(&app_state.db, &user_id)
            .await?
            .into_iter()
            .map(|n| {
                let app_state = Arc::new(app_state.clone());
                let invoice = invoice.clone();
                tokio::spawn(async move {
                    n.notify(app_state, invoice).await
                })
            }
            )
            .collect::<Vec<_>>();

        let results = join_all(tasks).await;

        for result in results {
            result.map_err(|e| format!("Notify failed: {:?}", e))??;
        }
    }

    Ok(())
}
