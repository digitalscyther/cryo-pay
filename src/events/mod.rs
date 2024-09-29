use std::fmt::Debug;
use ethers::prelude::*;


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
        .map_err(|err| crate::utils::make_err(Box::new(err), "decode log"))?;

    Ok(())
}