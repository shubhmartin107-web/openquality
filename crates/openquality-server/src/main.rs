mod auth_extractor;
mod routes;
mod state;

use axum::{
    Router,
    routing::{delete, get, post, put},
};
use std::path::PathBuf;
use std::sync::Arc;
use tokio::net::TcpListener;
use tower_http::cors::CorsLayer;
use tower_http::services::ServeDir;
use tracing_subscriber::EnvFilter;

use crate::state::AppState;

#[tokio::main]
async fn main() {
    tracing_subscriber::fmt()
        .with_env_filter(
            EnvFilter::try_from_default_env().unwrap_or_else(|_| EnvFilter::new("info")),
        )
        .init();

    let app_state = Arc::new(AppState::new_with_defaults().await);

    let routes_health = Router::new().route("/health", get(routes::health::health));

    let routes_auth = Router::new()
        .route("/api/v1/auth/register", post(routes::auth::register))
        .route("/api/v1/auth/login", post(routes::auth::login))
        .route("/api/v1/auth/refresh", post(routes::auth::refresh_token))
        .route("/api/v1/auth/api-keys", get(routes::auth::list_api_keys))
        .route("/api/v1/auth/api-keys", post(routes::auth::create_api_key))
        .route(
            "/api/v1/auth/api-keys/{id}",
            delete(routes::auth::revoke_api_key),
        );

    let routes_workspaces = Router::new()
        .route(
            "/api/v1/workspaces",
            get(routes::workspaces::list_workspaces),
        )
        .route(
            "/api/v1/workspaces",
            post(routes::workspaces::create_workspace),
        )
        .route(
            "/api/v1/workspaces/{id}",
            get(routes::workspaces::get_workspace),
        );

    let routes_users = Router::new()
        .route(
            "/api/v1/workspaces/{ws_id}/users",
            get(routes::users::list_users),
        )
        .route(
            "/api/v1/workspaces/{ws_id}/users",
            post(routes::users::create_user),
        )
        .route("/api/v1/users/{id}", get(routes::users::get_user));

    let routes_data_sources = Router::new()
        .route(
            "/api/v1/workspaces/{ws_id}/data-sources",
            get(routes::data_sources::list_data_sources),
        )
        .route(
            "/api/v1/workspaces/{ws_id}/data-sources",
            post(routes::data_sources::create_data_source),
        )
        .route(
            "/api/v1/data-sources/{id}",
            get(routes::data_sources::get_data_source),
        )
        .route(
            "/api/v1/data-sources/{id}",
            delete(routes::data_sources::delete_data_source),
        );

    let routes_suites = Router::new()
        .route(
            "/api/v1/workspaces/{ws_id}/suites",
            get(routes::suites::list_suites),
        )
        .route(
            "/api/v1/workspaces/{ws_id}/suites",
            post(routes::suites::create_suite),
        )
        .route("/api/v1/suites/{id}", get(routes::suites::get_suite))
        .route("/api/v1/suites/{id}", delete(routes::suites::delete_suite))
        .route("/api/v1/suites/{id}/run", post(routes::suites::run_suite))
        .route(
            "/api/v1/suites/{id}/runs",
            get(routes::suites::list_suite_runs),
        )
        .route(
            "/api/v1/suites/{id}/expectations",
            get(routes::suites::list_expectations),
        )
        .route(
            "/api/v1/suites/{id}/expectations",
            post(routes::suites::create_expectation),
        );

    let routes_monitors = Router::new()
        .route(
            "/api/v1/workspaces/{ws_id}/monitors",
            get(routes::monitors::list_monitors),
        )
        .route(
            "/api/v1/workspaces/{ws_id}/monitors",
            post(routes::monitors::create_monitor),
        )
        .route("/api/v1/monitors/{id}", get(routes::monitors::get_monitor))
        .route(
            "/api/v1/monitors/{id}",
            put(routes::monitors::update_monitor),
        )
        .route(
            "/api/v1/monitors/{id}",
            delete(routes::monitors::delete_monitor),
        )
        .route(
            "/api/v1/monitors/{id}/run",
            post(routes::monitors::run_monitor),
        )
        .route(
            "/api/v1/monitors/{id}/history",
            get(routes::monitors::get_monitor_history),
        );

    let routes_incidents = Router::new()
        .route("/api/v1/incidents", get(routes::incidents::list_incidents))
        .route(
            "/api/v1/incidents/{id}",
            get(routes::incidents::get_incident),
        )
        .route(
            "/api/v1/incidents/{id}/resolve",
            post(routes::incidents::resolve_incident),
        )
        .route(
            "/api/v1/incidents/{id}/acknowledge",
            post(routes::incidents::acknowledge_incident),
        )
        .route(
            "/api/v1/incidents/{id}/snooze",
            post(routes::incidents::snooze_incident),
        );

    let routes_profiler = Router::new()
        .route("/api/v1/profile", post(routes::profiler::profile_table))
        .route(
            "/api/v1/suggest",
            post(routes::profiler::suggest_expectations),
        );

    let routes_mcp = Router::new().route("/api/v1/mcp/execute", post(routes::mcp::execute_mcp));

    let routes_integrations = Router::new()
        .route(
            "/api/v1/integrations/dbt/parse-manifest",
            post(routes::integrations::dbt_parse_manifest),
        )
        .route(
            "/api/v1/integrations/airflow/webhook",
            post(routes::integrations::airflow_webhook),
        )
        .route(
            "/api/v1/integrations/ge/translate",
            post(routes::integrations::ge_translate),
        )
        .route(
            "/api/v1/integrations/lineage/parse-sql",
            post(routes::integrations::lineage_parse_sql),
        )
        .route(
            "/api/v1/integrations/lineage/build-graph",
            post(routes::integrations::lineage_build_graph),
        );

    let api_routes = routes_health
        .merge(routes_auth)
        .merge(routes_workspaces)
        .merge(routes_users)
        .merge(routes_data_sources)
        .merge(routes_suites)
        .merge(routes_monitors)
        .merge(routes_incidents)
        .merge(routes_profiler)
        .merge(routes_mcp)
        .merge(routes_integrations)
        .with_state(app_state);

    let cors_origins = std::env::var("OQ_CORS_ORIGINS")
        .unwrap_or_else(|_| "*".to_string());
    let cors = if cors_origins == "*" {
        CorsLayer::permissive()
    } else {
        CorsLayer::new()
            .allow_origin(cors_origins.split(',').map(|o| o.trim().parse::<axum::http::HeaderValue>().unwrap()).collect::<Vec<_>>())
            .allow_methods([axum::http::Method::GET, axum::http::Method::POST, axum::http::Method::PUT, axum::http::Method::DELETE])
            .allow_headers([axum::http::header::AUTHORIZATION, axum::http::header::CONTENT_TYPE])
    };

    let static_dir = std::env::var("OQ_STATIC_DIR")
        .ok()
        .map(PathBuf::from)
        .unwrap_or_else(|| {
            PathBuf::from(env!("CARGO_MANIFEST_DIR"))
                .parent()
                .unwrap()
                .parent()
                .unwrap()
                .join("web")
                .join("dist")
        });
    let serve_static = ServeDir::new(&static_dir).append_index_html_on_directories(true);

    let app = api_routes.layer(cors).fallback_service(serve_static);

    let addr = std::env::var("OQ_LISTEN_ADDR").unwrap_or_else(|_| "0.0.0.0:8080".to_string());
    tracing::info!("OpenQuality server v2 listening on {addr}");
    let listener = TcpListener::bind(&addr).await.unwrap();
    axum::serve(listener, app).await.unwrap();
}
