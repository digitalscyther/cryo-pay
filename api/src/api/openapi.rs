use utoipa::OpenApi;

#[derive(OpenApi)]
#[openapi(
    paths(
        crate::api::ping_pong::ping_pong,
        crate::api::ping_pong::health_check,
        crate::api::auth::login,
        crate::api::auth::logout,
        crate::api::payments::get_invoices_handler,
        crate::api::payments::create_invoice_handler,
        crate::api::payments::get_invoice_handler,
        crate::api::payments::delete_invoice_handler,
        crate::api::user::get_user,
        crate::api::user::update_user,
        crate::api::user::api_key::list,
        crate::api::user::api_key::create,
        crate::api::user::api_key::read,
        crate::api::user::api_key::destroy,
        crate::api::user::webhook::list,
        crate::api::user::webhook::create,
        crate::api::user::webhook::destroy,
        crate::api::user::callback_url::list,
        crate::api::user::callback_url::create,
        crate::api::user::callback_url::destroy,
        crate::api::user::analytics::get_analytics,
    ),
    components(
        schemas(
            crate::db::User,
            crate::db::Invoice,
            crate::db::ApiKey,
            crate::db::CallbackUrl,
            crate::db::Webhook,
            crate::api::ping_pong::PongResponse,
            crate::api::payments::InvoiceResponse,
            crate::api::payments::CreateInvoiceRequest,
            crate::api::auth::FirebaseTokenRequest,
            crate::api::user::UserRequest,
            crate::api::user::UserResponse,
            crate::api::user::api_key::GetApiKeyResponse,
            crate::api::user::api_key::CreateApiKeyResponse,
            crate::api::user::webhook::GetWebhookResponse,
            crate::api::user::webhook::CreateWebhookRequest,
            crate::api::user::callback_url::GetCallbackUrlResponse,
            crate::api::user::callback_url::CreateCallbackUrlRequest,
            crate::api::user::analytics::AnalyticsResponse,
            crate::db::analytics::InvoicePeriodStats,
            crate::db::analytics::InvoiceSummary,
        )
    ),
    tags(
        (name = "invoices", description = "Invoice management"),
        (name = "user", description = "User account"),
        (name = "auth", description = "Authentication"),
        (name = "system", description = "Health checks"),
    ),
    info(
        title = "Cryo Pay API",
        description = "USDT payment gateway for EVM chains",
        version = "1.0.0"
    )
)]
pub struct ApiDoc;
