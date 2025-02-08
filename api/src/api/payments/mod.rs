use std::sync::Arc;
use axum::extract::{Path, Query, State};
use axum::http::StatusCode;
use axum::{Extension, Json, middleware, Router};
use axum::response::{IntoResponse, Redirect};
use axum::routing::{delete, get, post};
use bigdecimal::BigDecimal;
use chrono::NaiveDateTime;
use serde::{Deserialize, Serialize};
use url::Url;
use uuid::Uuid;
use crate::api::INVOICE_PATH;
use crate::db::{Invoice, User};
use crate::api::middleware::{extract_user, only_auth, only_bill_owner};
use crate::api::middleware::auth::AppUser;
use crate::api::middleware::rate_limiting::middleware::RateLimitType;
use crate::api::ping_pong::ping_pong;
use crate::api::response_error::ResponseError;
use crate::api::state::AppState;

#[derive(Serialize)]
struct InvoiceResponse {
    pub id: Uuid,
    pub created_at: NaiveDateTime,
    pub amount: BigDecimal,
    pub seller: String,
    pub paid_at: Option<NaiveDateTime>,
    pub networks: Vec<i32>,
    pub external_id: Option<String>,
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
            external_id: i.external_id,
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
        .route(INVOICE_PATH, get(get_invoices_handler))
        .route(
            INVOICE_PATH,
            post(create_invoice_handler)
                .layer(middleware::from_fn_with_state(app_state.clone(), RateLimitType::product_invoice)))
        .route(&format!("{INVOICE_PATH}/:invoice_id"), get(get_invoice_handler))
        .route(
            &format!("{INVOICE_PATH}/:invoice_id"),
            delete(delete_invoice_handler)
                .layer(middleware::from_fn_with_state(app_state.clone(), only_auth))
                .layer(middleware::from_fn_with_state(app_state.clone(), only_bill_owner)))
        .layer(middleware::from_fn_with_state(app_state.clone(), extract_user))
        .route(&format!("{INVOICE_PATH}/:invoice_id/redirect"), get(redirect_invoice_handler))
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
) -> Result<Json<Vec<InvoiceResponse>>, ResponseError> {
    let limit = pagination.limit;
    let offset = pagination.offset;

    let invoices = state.db.list_invoices(limit, offset, filter.user_id.to_user_id(app_user))
        .await
        .map_err(ResponseError::from_error)?
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
    external_id: Option<String>,
}

async fn create_invoice_handler(
    State(state): State<Arc<AppState>>,
    Extension(app_user): Extension<AppUser>,
    Json(payload): Json<CreateInvoiceRequest>,
) -> Result<Json<InvoiceResponse>, ResponseError> {
    let invoice = state.db.create_invoice(
        payload.amount,
        &payload.seller,
        &payload.networks,
        app_user.user_id(),
        payload.external_id)
        .await
        .map_err(ResponseError::from_error)?;

    Ok(Json(invoice.into()))
}

async fn get_invoice_handler(
    State(state): State<Arc<AppState>>,
    Path(invoice_id): Path<Uuid>,
    Extension(app_user): Extension<AppUser>,
) -> Result<impl IntoResponse, ResponseError> {
    let (own, invoice) = state.db.get_own_invoice(invoice_id, app_user.user_id())
        .await
        .map_err(ResponseError::from_error)?;

    match invoice {
        None => Err(ResponseError::NotFound),
        Some(invoice) => Ok(Json(
            OwnInvoiceResponse { own, invoice: invoice.into() }
        ))
    }
}

async fn delete_invoice_handler(
    State(state): State<Arc<AppState>>,
    Path(invoice_id): Path<Uuid>,
    Extension(user): Extension<User>,
) -> Result<impl IntoResponse, ResponseError> {
    Ok(match state.db.delete_own_invoice(&invoice_id, &user.id)
        .await
        .map_err(ResponseError::from_error)? {
        true => StatusCode::NO_CONTENT,
        false => return Err(ResponseError::NotFound),
    })
}

#[derive(Deserialize)]
struct RedirectInvoiceQuery {
    url: Option<String>,
}

async fn redirect_invoice_handler(
    State(state): State<Arc<AppState>>,
    Path(invoice_id): Path<Uuid>,
    query: Query<RedirectInvoiceQuery>,
) -> Result<impl IntoResponse, ResponseError> {
    match &query.url {
        None => Err(ResponseError::Bad("`url` query_param required")),
        Some(url) => match state.db.get_invoice(&invoice_id)
            .await
            .map_err(ResponseError::from_error)? {
            None => Err(ResponseError::NotFound),
            Some(invoice) => {
                let was_paid = invoice.paid_at.is_some();
                let get_success_response = || get_redirect_url(&url, &invoice_id, was_paid);
                match invoice.user_id {
                    None => get_success_response(),
                    Some(user_id) => match state.db.exists_callback_url(&url, &user_id)
                        .await
                        .map_err(ResponseError::from_error)? {
                        false => Err(ResponseError::Bad("`url` not found in callback_urls")),
                        true => get_success_response(),
                    }
                }
            }
        }
    }
}

fn get_redirect_url(url: &str, invoice_id: &Uuid, was_paid: bool) -> Result<impl IntoResponse, ResponseError> {
    let mut parsed_url = Url::parse(url).map_err(|err| ResponseError::from_error(format!("{err:?}")))?;
    parsed_url.query_pairs_mut().append_pair("invoice_id", &invoice_id.to_string());
    parsed_url.query_pairs_mut().append_pair("status", if was_paid { "SUCCESS" } else { "PENDING" });

    Ok(Redirect::to(parsed_url.as_str()))
}
