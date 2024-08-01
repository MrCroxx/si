use axum::{
    http::HeaderValue,
    response::{IntoResponse, Json, Response},
    routing::get,
    Router,
};
use hyper::header;
use hyper::Method;
use hyper::StatusCode;
use serde_json::{json, Value};
use si_data_nats::NatsError;
use si_data_pg::PgError;
use telemetry::prelude::*;
use thiserror::Error;
use tower_http::cors::CorsLayer;
use tower_http::{compression::CompressionLayer, cors::AllowOrigin};

use super::{server::ServerError, state::AppState};

#[allow(clippy::too_many_arguments)]
pub fn routes(state: AppState) -> Router {
    let mut router: Router<AppState> = Router::new();
    router = router
        // root health route is currently pinged by auth portal to check if backend is up and running so we need permissive CORS headers
        .nest(
            "/api/",
            Router::new().route("/", get(system_status_route).layer(CorsLayer::permissive())),
        )
        .nest("/api/action", crate::server::service::action::routes())
        .nest(
            "/api/node_debug",
            crate::server::service::node_debug::routes(),
        )
        .nest(
            "/api/attribute",
            crate::server::service::attribute::routes(),
        )
        .nest(
            "/api/change_set",
            crate::server::service::change_set::routes(),
        )
        .nest(
            "/api/component",
            crate::server::service::component::routes(),
        )
        .nest("/api/diagram", crate::server::service::diagram::routes())
        .nest("/api/graphviz", crate::server::service::graphviz::routes())
        .nest(
            "/api/qualification",
            crate::server::service::qualification::routes(),
        )
        .nest("/api/secret", crate::server::service::secret::routes())
        .nest("/api/session", crate::server::service::session::routes())
        .nest("/api/ws", crate::server::service::ws::routes())
        .nest("/api/module", crate::server::service::module::routes())
        .nest("/api/variant", crate::server::service::variant::routes())
        .nest("/api/v2", crate::server::service::v2::routes())
        .layer(CompressionLayer::new())
        // allows us to be permissive about cors from our owned subdomains
        .layer(
            CorsLayer::new()
                .allow_origin(AllowOrigin::predicate(|origin: &HeaderValue, _| {
                    origin.as_bytes().ends_with(b".systeminit.com")
                }))
                .allow_credentials(true)
                .allow_headers(vec![
                    header::ACCEPT,
                    header::ACCEPT_LANGUAGE,
                    header::AUTHORIZATION,
                    header::CONTENT_LANGUAGE,
                    header::CONTENT_TYPE,
                ])
                .allow_methods(vec![
                    Method::GET,
                    Method::POST,
                    Method::PUT,
                    Method::DELETE,
                    Method::HEAD,
                    Method::OPTIONS,
                    Method::CONNECT,
                    Method::PATCH,
                    Method::TRACE,
                ]),
        );

    // Load dev routes if we are in dev mode (decided by "opt-level" at the moment).
    router = dev_routes(router);

    router.with_state(state)
}

async fn system_status_route() -> Json<Value> {
    Json(json!({ "ok": true }))
}

#[cfg(debug_assertions)]
pub fn dev_routes(mut router: Router<AppState>) -> Router<AppState> {
    router = router.nest("/api/dev", crate::server::service::dev::routes());
    router
}

#[cfg(not(debug_assertions))]
pub fn dev_routes(router: Router<AppState>) -> Router<AppState> {
    telemetry::prelude::debug!("skipping dev routes...");
    router
}

#[allow(clippy::large_enum_variant)]
#[remain::sorted]
#[derive(Debug, Error)]
pub enum AppError {
    #[error(transparent)]
    Nats(#[from] NatsError),
    #[error(transparent)]
    Pg(#[from] PgError),
    #[error(transparent)]
    Server(#[from] ServerError),
}

impl IntoResponse for AppError {
    fn into_response(self) -> Response {
        let (status, error_message) = (StatusCode::INTERNAL_SERVER_ERROR, self.to_string());

        let body = Json(serde_json::json!({
            "error": {
                "message": error_message,
                "code": 42,
                "statusCode": status.as_u16(),
            },
        }));
        error!(si.error.message = error_message);
        (status, body).into_response()
    }
}
