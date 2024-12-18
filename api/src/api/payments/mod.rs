use std::sync::Arc;
use axum::extract::{Path, Query, State};
use axum::http::StatusCode;
use axum::{Extension, Json, middleware, Router};
use axum::response::IntoResponse;
use axum::routing::{delete, get, post};
use bigdecimal::BigDecimal;
use chrono::NaiveDateTime;
use serde::{Deserialize, Serialize};
use uuid::Uuid;
use crate::db::{Invoice, User};
use crate::api::middleware::{AppUser, extract_user, only_auth, rate_limit};
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
        .route(
            "/invoice",
            post(create_invoice_handler)
                .layer(middleware::from_fn_with_state(app_state.clone(), rate_limit)))
        .route("/invoice/:invoice_id", get(get_invoice_handler))
        .route(
            "/invoice/:invoice_id",
            delete(delete_invoice_handler)
                .layer(middleware::from_fn_with_state(app_state.clone(), only_auth)))
        .layer(middleware::from_fn_with_state(app_state.clone(), extract_user))
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
    fn to_user_id(&self, app_user: AppUser) -> Option<Uuid> {
        match self {
            UserIdFilter::All => None,
            UserIdFilter::My => app_user.user_id()
        }
    }
}

#[derive(Deserialize)]
struct Filter {
    #[serde(default = "default_user_id")]
    user_id: UserIdFilter,
}

fn default_user_id() -> UserIdFilter {
    UserIdFilter::All
}

async fn get_invoices_handler(
    State(state): State<Arc<AppState>>,
    Extension(app_user): Extension<AppUser>,
    Query(pagination): Query<Pagination>,
    Query(filter): Query<Filter>,
) -> Result<Json<Vec<InvoiceResponse>>, StatusCode> {
    let limit = pagination.limit;
    let offset = pagination.offset;

    let invoices = state.db.list_invoices(limit, offset, filter.user_id.to_user_id(app_user))
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
    networks: Vec<i32>,
}

async fn create_invoice_handler(
    State(state): State<Arc<AppState>>,
    Extension(app_user): Extension<AppUser>,
    Json(payload): Json<CreateInvoiceRequest>,
) -> Result<Json<InvoiceResponse>, StatusCode> {
    let invoice = state.db.create_invoice(payload.amount, &payload.seller, &payload.networks, app_user.user_id())
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    Ok(Json(invoice.into()))
}

async fn get_invoice_handler(
    State(state): State<Arc<AppState>>,
    Path(invoice_id): Path<Uuid>,
    Extension(app_user): Extension<AppUser>,
) -> Result<impl IntoResponse, StatusCode> {
    let (own, invoice) = state.db.get_own_invoice(invoice_id, app_user.user_id())
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    match invoice {
        None => Err(StatusCode::NOT_FOUND),
        Some(invoice) => Ok(Json(
            OwnInvoiceResponse { own, invoice: invoice.into() }
        ))
    }
}

async fn delete_invoice_handler(
    State(state): State<Arc<AppState>>,
    Path(invoice_id): Path<Uuid>,
    Extension(user): Extension<User>,
) -> impl IntoResponse {
    match state.db.delete_own_invoice(&invoice_id, &user.id).await {
        Ok(true) => StatusCode::NO_CONTENT,
        Ok(false) => StatusCode::NOT_FOUND,
        Err(_) => StatusCode::INTERNAL_SERVER_ERROR,
    }
}