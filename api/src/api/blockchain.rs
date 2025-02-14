use std::fs::File;
use std::io;
use std::io::Read;
use std::path::Path as StdPath;
use std::sync::Arc;
use axum::response::IntoResponse;
use axum::{Json, Router};
use axum::extract::{Path, State};
use axum::routing::get;
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};
use crate::api::ping_pong::ping_pong;
use crate::api::response_error::ResponseError;
use crate::api::state::AppState;
use crate::network::{Addresses, Network};
use crate::utils;

pub fn get_router(app_state: Arc<AppState>) -> Router {
    Router::new()
        .route("/ping", get(ping_pong))
        .route("/info", get(get_info))
        .route("/suggested_gas_fees/:network_id", get(get_suggested_gas_fees))
        .with_state(app_state)
}

#[derive(Serialize)]
struct Abi {
    erc20: Value,
    contract: Value,
}

#[derive(Serialize)]
struct Info {
    networks: Vec<BlockChainNetwork>,
    abi: Abi,
}

async fn get_info(
    State(state): State<Arc<AppState>>
) -> Result<impl IntoResponse, ResponseError> {
    let erc20_abi = load_json_from_file(
        utils::get_env_var("ERC20_ABI_PATH")
            .map_err(ResponseError::from_error)?
    )
        .map_err(|err| ResponseError::from_error(format!("{err:?}")))?;

    let contract_abi = load_json_from_file(
        utils::get_env_var("CONTRACT_ABI_PATH")
            .map_err(ResponseError::from_error)?
    )
        .map_err(|err| ResponseError::from_error(format!("{err:?}")))?;

    let response = Info {
        networks: state.networks
            .clone()
            .into_iter()
            .map(BlockChainNetwork::from)
            .collect(),
        abi: Abi {
            erc20: erc20_abi,
            contract: contract_abi,
        },
    };

    Ok(Json(response).into_response())
}

#[derive(Serialize)]
struct BlockChainNetwork {
    pub name: String,
    pub id: i64,
    pub addresses: Addresses,
}

impl From<Network> for BlockChainNetwork {
    fn from(n: Network) -> Self {
        Self {
            name: n.name,
            id: n.id,
            addresses: n.addresses,
        }
    }
}

fn load_json_from_file<P: AsRef<StdPath>>(path: P) -> Result<Value, io::Error> {
    let mut file = File::open(path)?;
    let mut contents = String::new();
    file.read_to_string(&mut contents)?;
    let json: Value = serde_json::from_str(&contents)?;
    Ok(json)
}

#[derive(Clone, Serialize, Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
struct GasPriceResponse {
    low: GasFeeDetails,
    medium: GasFeeDetails,
    high: GasFeeDetails,
    estimated_base_fee: String,
    network_congestion: f64,
}

#[derive(Clone, Serialize, Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
struct GasFeeDetails {
    suggested_max_priority_fee_per_gas: String,
    suggested_max_fee_per_gas: String,
    min_wait_time_estimate: u64,
    max_wait_time_estimate: u64,
}

async fn get_suggested_gas_fees(
    State(state): State<Arc<AppState>>,
    Path(network_id): Path<i64>,
) -> Result<impl IntoResponse, ResponseError> {
    let allowed_ids = state
        .networks
        .iter()
        .map(|n| n.id)
        .collect::<Vec<i64>>();
    if !allowed_ids.contains(&network_id) {
        return Err(ResponseError::Bad("Unknown network_id".to_string()));
    }

    if let Ok(Some(value)) = state.redis.get_suggested_gas_fees(&network_id).await {
        if let Ok(gas_prices) = serde_json::from_str::<GasPriceResponse>(&value) {
            return Ok(Json(json!({ "source": "cache", "data": gas_prices })));
        }
    }

    let value = utils::get_suggested_gas_fees(&state.infura_token, network_id)
        .await
        .map_err(ResponseError::from_error)?;

    let response = serde_json::from_value::<GasPriceResponse>(value)
        .map_err(|err| ResponseError::from_error(format!("{err:?}")))?;

    state.redis.set_suggested_gas_fees(
        &network_id, serde_json::to_string(&response.clone())
            .map_err(|err| ResponseError::from_error(format!("{err:?}")))?,
    ).
        await
        .map_err(ResponseError::from_error)?;

    Ok(Json(json!({ "source": "api", "data": response })))
}