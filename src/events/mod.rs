use std::fmt::Debug;
use std::sync::Arc;
use bigdecimal::BigDecimal;
use chrono::{DateTime, NaiveDateTime, Utc};
use ethers::prelude::*;
use uuid::Uuid;
use crate::api::state::DB;
use crate::utils;


#[derive(Debug, Clone, EthEvent)]
struct NewTransaction {
    #[ethevent(indexed)]
    block_timestamp: U256,
}

pub async fn just_print_log(log: Log) -> Result<(), String> {
    <NewTransaction as EthEvent>::decode_log(&log.into())
        .map(|ns| {
            println!("New transaction event: {:?}", ns);
            ns
        })
        .map_err(|err| utils::make_err(Box::new(err), "decode log"))?;

    Ok(())
}

#[derive(Debug, Clone, EthEvent)]
struct PayInvoiceEvent {
    invoice_id: String,
    #[ethevent(indexed)]
    seller: Address,
    #[ethevent(indexed)]
    payer: Address,
    paid_at: U128,
    amount: U128,
}


pub async fn set_invoice_paid(postgres_db: Arc<DB>, log: Log) -> Result<(), String> {
    match <PayInvoiceEvent as EthEvent>::decode_log(&log.into()) {
        Ok(event) => {
            if let Err(err) = postgres_db.set_invoice_paid(
                Uuid::parse_str(&event.invoice_id).unwrap(),
                &format!("{:#020x}", event.seller),
                BigDecimal::from(event.amount.as_u128()) / BigDecimal::from(1_000_000),
                &format!("{:#020x}", event.payer),
                DateTime::<Utc>::from_timestamp(event.paid_at.as_u64() as i64, 0).unwrap().naive_utc(),
            ).await {
                return Err(err);
            }

            Ok(())
        }
        Err(err) => Err(utils::make_err(Box::new(err), "decode log"))
    }
}
