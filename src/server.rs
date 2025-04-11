use crate::config::AppConfig;
use crate::{concept, module};
use axum::{routing::get, Router};
use sqlx::postgres::PgPoolOptions;
use sqlx::PgPool;
use std::sync::Arc;
use std::time::Duration;
use tower_http::cors::CorsLayer;
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
    let state = Arc::new(ApiContext { db: pool.clone() });
    let router = api_router(state);

    sqlx::migrate!().run(&pool).await?;

    let listener = tokio::net::TcpListener::bind("0.0.0.0:3000").await?;
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
        .layer(CorsLayer::permissive())
}

#[cfg(test)]
mod tests {
    use super::*;
    use axum::body::Body;
    use axum::http::{self, Request, StatusCode};
    use http_body_util::BodyExt;
    use std::str::from_utf8;
    use tower::ServiceExt;

    #[sqlx::test]
    async fn root_test(pool: PgPool) {
        let state = Arc::new(ApiContext { db: pool });
        let router = api_router(state);

        let response = router
            .oneshot(
                Request::builder()
                    .method(http::Method::GET)
                    .uri("/")
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();

        assert_eq!(response.status(), StatusCode::OK);

        let body = response.into_body().collect().await.unwrap().to_bytes();
        let resp_body = from_utf8(&body).unwrap();

        assert_eq!(resp_body, "DIZ Marburg MDR Web API");
    }
}
