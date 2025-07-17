use std::{path::PathBuf, process::exit};

use axum::{extract::Query, routing::get, Extension, Router};
use clap::Parser;
use serde::Deserialize;
use sub_util::{generate_clash_config, AppConfig};
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
struct QueryParams {
    url: Option<String>,
}

async fn hello_world(
    Extension(mut app_config): Extension<AppConfig>,
    Query(params): Query<QueryParams>,
) -> String {
    // 如果提供了 url 参数，替换 proxy provider
    if let Some(url) = params.url {
        app_config.proxies.insert("miaona".to_string(), url);
    }

    let clash_config = generate_clash_config(app_config);
    match serde_yaml::to_string(&clash_config) {
        Ok(yaml) => yaml,
        Err(err) => {
            error!("Failed to serialize clash config: {}", err);
            "Failed to generate clash config".to_string()
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
