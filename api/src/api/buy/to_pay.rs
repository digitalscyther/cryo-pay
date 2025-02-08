use tracing::warn;
use uuid::Uuid;
use serde_json::Value;
use crate::api::state::DB;
use crate::db::billing::Payment;
use crate::payments::payable::Payable;
use crate::payments::{ToPay, ToPayId};

struct ToPayAdapterPayment {
    invoice_id: Uuid,
    data: Value,
}

impl TryFrom<&ToPay> for ToPayAdapterPayment {
    type Error = ();

    fn try_from(value: &ToPay) -> Result<ToPayAdapterPayment, Self::Error> {
        let invoice_id = match value.id {
            ToPayId::CryoPay(id) => id
        };
        let data = serde_json::to_value(&value.payable)
            .map_err(|err| {
                warn!("Failed parse value to Payable: {err}");
                ()
            })?;

        Ok(ToPayAdapterPayment { invoice_id, data })
    }
}

impl TryFrom<Payment> for ToPay {
    type Error = ();

    fn try_from(value: Payment) -> Result<ToPay, Self::Error> {
        let payable: Payable = serde_json::from_value(value.data)
            .map_err(|err| {
                warn!("Failed parse value to Payable: {err}");
                ()
            })?;

        Ok(ToPay { id: ToPayId::CryoPay(value.id), payable })
    }
}

pub async fn create_payment_url(to_pay: &ToPay, db: &DB, user_id: Option<Uuid>) -> Result<String, String> {
    let payment_url = to_pay.payment_url()?;

    let payment_adapter: ToPayAdapterPayment = to_pay
        .try_into()
        .map_err(|_| "Failed to_pay to Adapter".to_string())?;

    db.create_payment(&payment_adapter.invoice_id, user_id, &payment_adapter.data).await?;

    Ok(payment_url)
}
