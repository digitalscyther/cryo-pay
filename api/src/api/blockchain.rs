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
struct Addresses {
    erc20: String,
    contract: String,
}

#[derive(Serialize)]
struct Network {
    id: i64,
    addresses: Addresses,
}

#[derive(Serialize)]
struct Abi {
    erc20: serde_json::Value,
    contract: serde_json::Value,
}

#[derive(Serialize)]
struct Info {
    networks: Vec<Network>,
    abi: Abi,
}

async fn get_info() -> Result<impl IntoResponse, StatusCode> {
    let erc20_abi = load_json_from_file(utils::get_env_var("ERC20_ABI_PATH")
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?)
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    let contract_abi = load_json_from_file(utils::get_env_var("CONTRACT_ABI_PATH")
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?)
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    let response = Info {
        networks: vec![
            Network {
                id: 11155420,
                addresses: Addresses {
                    erc20: utils::get_env_var("ERC20_ADDRESS")
                        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?,
                    contract: utils::get_env_var("CONTRACT_ADDRESS")
                        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?,
                },
            },
            Network {   // TODO real optimism
                id: 10,
                addresses: Addresses {
                    erc20: utils::get_env_var("ERC20_ADDRESS")
                        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?,
                    contract: utils::get_env_var("CONTRACT_ADDRESS")
                        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?,
                },
            },
        ],
        abi: Abi {
            erc20: erc20_abi,
            contract: contract_abi,
        },
    };

    Ok(Json(response).into_response())
}

fn load_json_from_file<P: AsRef<Path>>(path: P) -> Result<serde_json::Value, io::Error> {
    let mut file = File::open(path)?;
    let mut contents = String::new();
    file.read_to_string(&mut contents)?;
    let json: serde_json::Value = serde_json::from_str(&contents)?;
    Ok(json)
}