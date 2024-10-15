use std::fs::File;
use std::io;
use std::io::Read;
use std::path::Path;
use std::sync::Arc;
use axum::http::StatusCode;
use axum::response::IntoResponse;
use axum::{Json, Router};
use axum::routing::{get};
use serde::Serialize;
use crate::api::ping_pong::ping_pong;
use crate::api::state::AppState;
use crate::utils;

pub fn get_router(app_state: Arc<AppState>) -> Router {
    Router::new()
        .route("/ping", get(ping_pong))
        .route("/info", get(get_info))
        .with_state(app_state)
}

#[derive(Serialize)]
struct ContractInfo {
    address: String,
    abi: serde_json::Value,
}

#[derive(Serialize)]
struct BlockchainInfo {
    erc20: ContractInfo,
    invoice: ContractInfo,
}

async fn get_info() -> Result<impl IntoResponse, StatusCode> {
    let erc20_abi = load_json_from_file(utils::get_env_var("ERC20_ABI_PATH")
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?)
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    let invoice_abi = load_json_from_file(utils::get_env_var("INVOICE_ABI_PATH")
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?)
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    let response = BlockchainInfo {
        erc20: ContractInfo {
            address: utils::get_env_var("ERC20_ADDRESS")
                .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?,
            abi: erc20_abi,
        },
        invoice: ContractInfo {
            address: utils::get_env_var("INVOICE_ADDRESS")
                .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?,
            abi: invoice_abi,
        },
    };

    // Return the response as JSON
    Ok(Json(response).into_response())
}

fn load_json_from_file<P: AsRef<Path>>(path: P) -> Result<serde_json::Value, io::Error> {
    let mut file = File::open(path)?;
    let mut contents = String::new();
    file.read_to_string(&mut contents)?;
    let json: serde_json::Value = serde_json::from_str(&contents)?;
    Ok(json)
}