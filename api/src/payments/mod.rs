use bigdecimal::BigDecimal;
use uuid::Uuid;
use crate::api::get_cryo_pay_callback_full_path;
use crate::network::Network;
use crate::payments::cryo_pay::{CryoPayApi, CryoPayRecipient, get_payment_path};
use crate::payments::payable::Payable;
use crate::utils;

pub mod cryo_pay;
pub mod payable;

pub struct ToPay {
    pub id: ToPayId,
    pub payable: Payable,
}

pub enum ToPayId {
    CryoPay(Uuid)
}

impl ToPay {
    pub async fn create_donation(amount: BigDecimal) -> Result<Self, String> {
        let payable = Payable::create_anonymus_no_target_donation(&amount);

        Self::create(amount.clone(), Some(format!("Donation of {amount}")), payable).await
    }

    pub async fn create(amount: BigDecimal, custom_id: Option<String>, payable: Payable) -> Result<Self, String> {
        let cryo_pay_api = CryoPayApi::default();
        let cryo_pay_recipient = CryoPayRecipient::default(
            &Network::default_vec()?
        )?;

        let invoice_id = cryo_pay_api
            .create_invoice(&cryo_pay_recipient.seller, &cryo_pay_recipient.networks, custom_id, &amount)
            .await?;

        Ok(Self::new(ToPayId::CryoPay(invoice_id), payable))
    }

    fn new(id: ToPayId, payable: Payable) -> Self {
        Self { id, payable }
    }

    pub fn payment_url(&self) -> Result<String, String> {
        let global_api_url = utils::ApiGlobalUrl::get()?.url()?;
        let callback_path = get_cryo_pay_callback_full_path();
        let callback_url = utils::combine_paths(&[&global_api_url, &callback_path]);

        let payment_path = match self.id {
            ToPayId::CryoPay(id) => get_payment_path(&id, Some(callback_url))?
        };

        Ok(utils::combine_paths(&[&utils::web_base_url()?, &payment_path]))
    }
}