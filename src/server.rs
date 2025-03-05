use crate::config::AppConfig;
use crate::{concept, module};
use axum::{Router, routing::get};
use sqlx::PgPool;
use sqlx::postgres::PgPoolOptions;
use std::sync::Arc;
use std::time::Duration;
use tower_http::trace::TraceLayer;
use tracing_subscriber::EnvFilter;

#[derive(Clone)]
pub(crate) struct ApiContext {
    pub(crate) db: PgPool,
}

pub async fn serve(config: AppConfig) -> anyhow::Result<()> {
    let filter = format!(
        "{}={level},tower_http={level}",
        env!("CARGO_CRATE_NAME"),
        level = config.app.log_level
    );
    tracing_subscriber::fmt()
        .with_env_filter(EnvFilter::try_from_default_env().unwrap_or_else(|_| filter.into()))
        // .with_span_events(FmtSpan::CLOSE)
        .init();

    let mut db_opts = PgPoolOptions::new();
    // max connections
    if let Some(max) = config.database.max_connections {
        db_opts = db_opts.max_connections(max);
    }
    // timeout
    if let Some(timeout) = config.database.timeout {
        db_opts = db_opts.acquire_timeout(Duration::from_secs(timeout));
    }

    let pool = db_opts
        .clone()
        .connect(&config.database.url)
        .await
        .expect("Could not connect to database url");
    let state = Arc::new(ApiContext { db: pool });
    let router = api_router(state);

    // sqlx::migrate!().run(&db).await?;

    let listener = tokio::net::TcpListener::bind("127.0.0.1:3000").await?;
    tracing::debug!("listening on {}", listener.local_addr()?);
    axum::serve(listener, router).await.map_err(|e| e.into())
}

async fn root() -> &'static str {
    "DIZ Marburg MDR Web API"
}

fn api_router(state: Arc<ApiContext>) -> Router {
    Router::new()
        .route("/", get(root))
        .merge(module::router())
        .merge(concept::router())
        .with_state(state)
        .layer(TraceLayer::new_for_http())
}
