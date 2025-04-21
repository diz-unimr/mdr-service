use crate::error::ApiError;
use crate::server::ApiContext;
use anyhow::anyhow;
use axum::extract::Path;
use axum::http::StatusCode;
use axum::{debug_handler, extract::State, routing::get, Json, Router};
use serde::{Deserialize, Serialize};
use sqlx::types::Uuid;
use sqlx::FromRow;
use std::sync::Arc;

#[derive(Deserialize, Serialize, FromRow)]
struct Module {
    id: Uuid,
    name: String,
    fdpg_cds_code: String,
    fdpg_cds_system: String,
    fdpg_cds_version: String,
    version: String,
}

pub(crate) fn router() -> Router<Arc<ApiContext>> {
    Router::new()
        .route("/ontology/modules", get(all).post(create))
        .route("/ontology/modules/{id}", get(read))
}

#[debug_handler]
async fn create(
    State(ctx): State<Arc<ApiContext>>,
    module: Json<Module>,
) -> Result<(StatusCode, Json<Uuid>), ApiError> {
    let result = sqlx::query!(
        r#"insert into modules (id,name,fdpg_cds_code,fdpg_cds_system,fdpg_cds_version,version)
           values ($1,$2,$3,$4,$5,$6)
           RETURNING id"#,
        module.id,
        module.name,
        module.fdpg_cds_code,
        module.fdpg_cds_system,
        module.fdpg_cds_version,
        module.version
    )
    .fetch_one(&ctx.db)
    .await?;

    Ok((StatusCode::CREATED, Json(result.id)))
}

#[debug_handler]
async fn all(State(ctx): State<Arc<ApiContext>>) -> Result<Json<Vec<Module>>, ApiError> {
    let modules = sqlx::query_as!(
        Module,
        r#"select id, name, fdpg_cds_code,fdpg_cds_system,fdpg_cds_version, version
           from modules"#
    )
    .fetch_all(&ctx.db)
    .await?;

    Ok(Json(modules))
}

#[debug_handler]
async fn read(
    State(ctx): State<Arc<ApiContext>>,
    Path(id): Path<Uuid>,
) -> Result<Json<Module>, ApiError> {
    let result = sqlx::query_as!(
        Module,
        r#"select id, name, fdpg_cds_code,fdpg_cds_system,fdpg_cds_version, version
           from modules where id = $1"#,
        id
    )
    .fetch_optional(&ctx.db)
    .await?;

    match result {
        Some(module) => Ok(Json(module)),
        None => Err(ApiError(
            anyhow!(format!("No module found with id: {}", id)),
            StatusCode::NOT_FOUND,
        )),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use axum::body::Body;
    use axum::http::{self, Request, StatusCode};
    use http_body_util::BodyExt;
    use serde_json::{json, Value};
    use sqlx::PgPool;
    use tower::ServiceExt;

    #[sqlx::test(fixtures("modules"))]
    async fn read_test(pool: PgPool) {
        let state = Arc::new(ApiContext { db: pool });
        let router = router().with_state(state);

        let response = router
            .oneshot(
                Request::builder()
                    .method(http::Method::GET)
                    .uri("/ontology/modules/0b6e62ccf4e328ceef0e653f4dc8c088")
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();

        assert_eq!(response.status(), StatusCode::OK);

        let body = response.into_body().collect().await.unwrap().to_bytes();
        let body: Value = serde_json::from_slice(&body).unwrap();

        assert_eq!(
            body,
            json!({
                "id":"0b6e62cc-f4e3-28ce-ef0e-653f4dc8c088",
                "name":"Person",
                "fdpg_cds_code": "Patient",
                "fdpg_cds_system": "fdpg.mii.cds",
                "fdpg_cds_version": "1.0.0",
                "version": "2.2.0",
            })
        );
    }
}
