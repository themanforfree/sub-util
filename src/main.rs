use axum::{Router, extract::Query, routing::get};
use serde::Deserialize;
use sub_util::{AppConfig, generate_clash_config};
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};

#[derive(Deserialize)]
struct QueryParams {
    url: Option<String>,
}

async fn hello_world(Query(params): Query<QueryParams>) -> String {
    let mut app_config = AppConfig::load_from_file("config.toml").unwrap();

    // 如果提供了 url 参数，替换 proxy provider
    if let Some(url) = params.url {
        app_config.proxies.insert("miaona".to_string(), url);
    }

    let clash_config = generate_clash_config(app_config);
    serde_yaml::to_string(&clash_config).unwrap()
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

    let arg1 = std::env::args().nth(1);
    let bind = arg1.as_deref().unwrap_or("0.0.0.0:3000").to_string();
    let router = Router::new().route("/", get(hello_world));

    tracing::info!("Server is running on http://{}", bind);
    let listener = tokio::net::TcpListener::bind(bind).await.unwrap();

    axum::serve(listener, router).await.unwrap();
}
