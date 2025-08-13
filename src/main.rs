use std::{path::PathBuf, process::exit};

use axum::{Extension, Router, http::StatusCode, response::Response, routing::get, extract::Query};
use clap::Parser;
use serde::Deserialize;
use sub_util::{
    AppConfig, ConfigError, generate_clash_config_with_validation, get_available_region_groups,
};
use tracing::error;
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};

#[derive(Parser, Debug)]
#[command(version, about, long_about = None)]
struct Args {
    #[arg(short, long, default_value = "config.toml")]
    config: PathBuf,

    #[arg(short, long, default_value = "0.0.0.0:3000")]
    bind: String,
}

#[derive(Deserialize)]
struct TokenQuery {
    token: Option<String>,
}

async fn hello_world(
    Extension(app_config): Extension<AppConfig>,
    Query(query): Query<TokenQuery>,
) -> Response<String> {
    // Check token authentication
    if let Err(response) = validate_token(&app_config, &query) {
        return response;
    }

    match generate_clash_config_with_validation(app_config) {
        Ok(clash_config) => match serde_yaml::to_string(&clash_config) {
            Ok(yaml) => Response::builder()
                .status(StatusCode::OK)
                .header("content-type", "text/yaml; charset=utf-8")
                .header("content-disposition", "attachment; filename=sub.yaml")
                .body(yaml)
                .unwrap(),
            Err(err) => {
                error!("Failed to serialize clash config: {}", err);
                Response::builder()
                    .status(StatusCode::INTERNAL_SERVER_ERROR)
                    .body("Failed to serialize clash config".to_string())
                    .unwrap()
            }
        },
        Err(err) => {
            error!("Failed to generate clash config: {}", err);
            create_error_response(&err)
        }
    }
}

async fn get_config_info(
    Extension(app_config): Extension<AppConfig>,
    Query(query): Query<TokenQuery>,
) -> Response<String> {
    // Check token authentication
    if let Err(response) = validate_token(&app_config, &query) {
        return response;
    }

    use serde_json::json;

    let region_groups = get_available_region_groups(&app_config);
    let proxy_providers: Vec<String> = app_config.proxies.keys().cloned().collect();

    let info = json!({
        "proxy_providers": proxy_providers,
        "region_groups": region_groups,
        "user_groups": app_config.groups.iter().map(|g| match g {
            sub_util::ProxyGroup::Select(s) => &s.common.name,
            sub_util::ProxyGroup::UrlTest(u) => &u.common.name,
            sub_util::ProxyGroup::Fallback(f) => &f.common.name,
            sub_util::ProxyGroup::LoadBalance(l) => &l.common.name,
            sub_util::ProxyGroup::Relay(r) => &r.common.name,
        }).collect::<Vec<_>>(),
        "rules_count": app_config.rules.len(),
        "region_groups_enabled": app_config.region_groups.as_ref().map(|r| r.enabled).unwrap_or(false)
    });

    match serde_json::to_string_pretty(&info) {
        Ok(json) => Response::builder()
            .status(StatusCode::OK)
            .header("content-type", "application/json")
            .body(json)
            .unwrap(),
        Err(err) => {
            error!("Failed to serialize config info: {}", err);
            Response::builder()
                .status(StatusCode::INTERNAL_SERVER_ERROR)
                .body("Failed to serialize config info".to_string())
                .unwrap()
        }
    }
}

#[tokio::main]
async fn main() {
    tracing_subscriber::registry()
        .with(
            tracing_subscriber::EnvFilter::try_from_default_env().unwrap_or_else(|_| {
                // axum logs rejections from built-in extractors with the `axum::rejection`
                // target, at `TRACE` level. `axum::rejection=trace` enables showing those events
                format!(
                    "{}=debug,tower_http=debug,axum::rejection=trace,axum=trace",
                    env!("CARGO_CRATE_NAME")
                )
                .into()
            }),
        )
        .with(tracing_subscriber::fmt::layer())
        .init();

    let args = Args::parse();
    let app_config = match AppConfig::load_from_file(args.config) {
        Ok(config) => config,
        Err(err) => {
            error!("Failed to load config: {}", err);
            exit(1);
        }
    };
    let router = Router::new()
        .route("/", get(hello_world))
        .route("/config", get(get_config_info))
        .layer(Extension(app_config));

    tracing::info!("Server is running on http://{}", &args.bind);
    let listener = match tokio::net::TcpListener::bind(&args.bind).await {
        Ok(listener) => listener,
        Err(err) => {
            error!("Failed to bind to address: {}", err);
            exit(1);
        }
    };

    if let Err(err) = axum::serve(listener, router).await {
        error!("Server error: {}", err);
        exit(1);
    }
}
fn create_error_response(error: &ConfigError) -> Response<String> {
    use serde_json::json;

    let (status_code, error_type) = match error {
        ConfigError::InvalidSubscriptionUrl(_) => {
            (StatusCode::BAD_REQUEST, "invalid_subscription_url")
        }
        ConfigError::ProxyGroupGenerationFailed(_) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            "proxy_group_generation_failed",
        ),
        ConfigError::RuleProcessingFailed(_) => (StatusCode::BAD_REQUEST, "rule_processing_failed"),
        ConfigError::ConfigValidationFailed(_) => {
            (StatusCode::BAD_REQUEST, "config_validation_failed")
        }
    };

    let error_response = json!({
        "error": {
            "type": error_type,
            "message": error.to_string()
        }
    });

    Response::builder()
        .status(status_code)
        .header("content-type", "application/json")
        .body(error_response.to_string())
        .unwrap()
}

fn validate_token(app_config: &AppConfig, query: &TokenQuery) -> Result<(), Response<String>> {
    // Check if authentication is enabled
    if let Some(auth_config) = &app_config.auth {
        if auth_config.enabled {
            // Authentication is enabled, check for token
            let provided_token = query.token.as_deref().unwrap_or("");
            let expected_token = auth_config.token.as_deref().unwrap_or("");
            
            if expected_token.is_empty() {
                error!("Authentication is enabled but no token is configured");
                return Err(create_unauthorized_response("Server configuration error"));
            }
            
            if provided_token.is_empty() {
                return Err(create_unauthorized_response("Token required"));
            }
            
            if provided_token != expected_token {
                return Err(create_unauthorized_response("Invalid token"));
            }
        }
    }
    // If authentication is not configured or disabled, allow access
    Ok(())
}

fn create_unauthorized_response(message: &str) -> Response<String> {
    use serde_json::json;
    
    let error_response = json!({
        "error": {
            "type": "unauthorized",
            "message": message
        }
    });

    Response::builder()
        .status(StatusCode::UNAUTHORIZED)
        .header("content-type", "application/json")
        .body(error_response.to_string())
        .unwrap()
}
