use axum::extract::{Request, State};
use std::sync::Arc;
use axum::Extension;
use axum::middleware::Next;
use axum::response::IntoResponse;
use axum::http::StatusCode;
use tracing::warn;
use crate::api::middleware::auth::{AppUser, AuthType};
use crate::api::middleware::rate_limiting::{Limit, Period, RateLimit, Target};
use crate::api::state::{AppState, DB};
use crate::api::utils;
use crate::payments::payable::SubscriptionTarget;


pub enum RateLimitType {
    CreateProductInvoice,
    CreateUserInvoice,
}

impl RateLimitType {
    async fn rate_limit(&self, app_user: &AppUser, db: &DB) -> Result<RateLimit, String> {
        match self {
            RateLimitType::CreateProductInvoice => CreateProductInvoiceRateLimitGetter::get(app_user, db).await,
            RateLimitType::CreateUserInvoice => CreateUserInvoiceRateLimitGetter::get(app_user, db).await,
        }
    }

    async fn check(
        &self,
        state: &Arc<AppState>,
        app_user: &AppUser,
        req: Request,
        next: Next,
    ) -> Result<impl IntoResponse, StatusCode> {
        match self
            .rate_limit(app_user, &state.db)
            .await
            .map_err(utils::log_and_error)?
            .is_ok(&state.redis, app_user)
            .await
            .map_err(utils::log_and_error)? {
            true => Ok(next.run(req).await),
            false => Err(StatusCode::TOO_MANY_REQUESTS),
        }
    }

    pub async fn product_invoice(
        State(state): State<Arc<AppState>>,
        Extension(app_user): Extension<AppUser>,
        req: Request,
        next: Next,
    ) -> Result<impl IntoResponse, StatusCode> {
        Self::CreateProductInvoice.check(&state, &app_user, req, next).await
    }

    pub async fn user_invoice(
        State(state): State<Arc<AppState>>,
        Extension(app_user): Extension<AppUser>,
        req: Request,
        next: Next,
    ) -> Result<impl IntoResponse, StatusCode> {
        Self::CreateUserInvoice.check(&state, &app_user, req, next).await
    }
}

trait RateLimitGetter {
    async fn get(app_user: &AppUser, db: &DB) -> Result<RateLimit, String>;
}

struct CreateProductInvoiceRateLimitGetter {}

struct CreateUserInvoiceRateLimitGetter {}

impl RateLimitGetter for CreateProductInvoiceRateLimitGetter {
    async fn get(app_user: &AppUser, db: &DB) -> Result<RateLimit, String> {
        Ok(RateLimit {
            target: Target::ProductInvoice,
            period: Period::Day,
            limit: {
                const API_LIMIT: i32 = 10;
                const WEB_LIMIT: i32 = 3;
                const ANONYMUS_LIMIT: i32 = WEB_LIMIT;

                let get_default_limit = || {
                    Limit::Limited(match &app_user.auth {
                        None => ANONYMUS_LIMIT,
                        Some(auth) => match auth.auth_type {
                            AuthType::API => API_LIMIT,
                            AuthType::WEB => WEB_LIMIT,
                        },
                    } as u16)
                };
                match app_user.user_id() {
                    Some(user_id) => {
                        let target: String = SubscriptionTarget::UnlimitedInvoices.into();
                        match db.get_user_active_subscription(&user_id, &target).await {
                            Err(err) => {
                                warn!("Failed to get_user_active_subscription: {err}");
                                get_default_limit()
                            }
                            Ok(None) => get_default_limit(),
                            Ok(Some(_)) => Limit::Unlimited,
                        }
                    }
                    None => get_default_limit()
                }
            },
        })
    }
}

impl RateLimitGetter for CreateUserInvoiceRateLimitGetter {
    async fn get(_: &AppUser, _: &DB) -> Result<RateLimit, String> {
        Ok(RateLimit {
            target: Target::UserInvoice,
            period: Period::Day,
            limit: Limit::Limited(10),
        })
    }
}
