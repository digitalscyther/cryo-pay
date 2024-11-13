use std::sync::Arc;
use axum::extract::{Path, Query, State};
use axum::http::StatusCode;
use axum::{Extension, Json, middleware, Router};
use axum::response::IntoResponse;
use axum::routing::{get, post};
use bigdecimal::BigDecimal;
use chrono::NaiveDateTime;
use serde::{Deserialize, Serialize};
use uuid::Uuid;
use crate::db::Invoice;
use crate::api::middleware::{extract_jwt, MaybeUser};
use crate::api::ping_pong::ping_pong;
use crate::api::state::AppState;

#[derive(Serialize)]
struct InvoiceResponse {
    pub id: Uuid,
    pub created_at: NaiveDateTime,
    pub amount: BigDecimal,
    pub seller: String,
    pub paid_at: Option<NaiveDateTime>,
    pub networks: Vec<i32>,
}

impl From<Invoice> for InvoiceResponse {
    fn from(i: Invoice) -> Self {
        Self {
            id: i.id,
            created_at: i.created_at,
            amount: i.amount,
            seller: i.seller,
            paid_at: i.paid_at,
            networks: i.networks,
        }
    }
}

#[derive(Serialize)]
struct OwnInvoiceResponse {
    own: bool,
    invoice: InvoiceResponse,
}

pub fn get_router(app_state: Arc<AppState>) -> Router {
    Router::new()
        .route("/ping", get(ping_pong))
        .route("/invoice", get(get_invoices_handler))
        .route("/invoice", post(create_invoice_handler))
        .route("/invoice/:invoice_id", get(get_invoice_handler))
        .layer(middleware::from_fn_with_state(app_state.clone(), extract_jwt))
        .with_state(app_state)
}

#[derive(Deserialize)]
struct Pagination {
    #[serde(default = "default_limit")]
    limit: i64,
    #[serde(default = "default_offset")]
    offset: i64,
}

fn default_limit() -> i64 {
    10
}

fn default_offset() -> i64 {
    0
}

#[derive(Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum UserIdFilter {
    All,
    My,
}

impl UserIdFilter {
    fn to_user_id(&self, maybe_user: MaybeUser) -> Option<Uuid> {
        match self {
            UserIdFilter::All => None,
            UserIdFilter::My => maybe_user.user_id()
        }
    }
}

#[derive(Deserialize)]
struct Filter {
    #[serde(default = "default_user_id")]
    user_id: UserIdFilter
}

fn default_user_id() -> UserIdFilter {
    UserIdFilter::All
}

async fn get_invoices_handler(
    State(state): State<Arc<AppState>>,
    Extension(maybe_user): Extension<MaybeUser>,
    Query(pagination): Query<Pagination>,
    Query(filter): Query<Filter>,
) -> Result<Json<Vec<InvoiceResponse>>, StatusCode> {
    let limit = pagination.limit;
    let offset = pagination.offset;

    let invoices = state.db.list_invoices(limit, offset, filter.user_id.to_user_id(maybe_user))
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?
        .into_iter()
        .map(|i| i.into())
        .collect();

    Ok(Json(invoices))
}

#[derive(Deserialize)]
struct CreateInvoiceRequest {
    amount: BigDecimal,
    seller: String,
    networks: Vec<i32>
}

async fn create_invoice_handler(
    State(state): State<Arc<AppState>>,
    Extension(maybe_user): Extension<MaybeUser>,
    Json(payload): Json<CreateInvoiceRequest>,
) -> Result<Json<InvoiceResponse>, StatusCode> {
    let invoice = state.db.create_invoice(payload.amount, &payload.seller, &payload.networks, maybe_user.user_id())
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    Ok(Json(invoice.into()))
}

async fn get_invoice_handler(
    State(state): State<Arc<AppState>>,
    Path(invoice_id): Path<Uuid>,
    Extension(maybe_user): Extension<MaybeUser>,
) -> Result<impl IntoResponse, StatusCode> {
    let (own, invoice) = state.db.get_own_invoice(invoice_id, maybe_user.user_id())
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    let own_invoice = OwnInvoiceResponse { own, invoice: invoice.into() };

    Ok(Json(own_invoice))
}