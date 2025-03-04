use crate::server::ApiContext;
use axum::extract::Path;
use axum::http::StatusCode;
use axum::response::{IntoResponse, Response};
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
        .route("/modules", get(all).post(create))
        .route("/modules/{id}", get(read))
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
    .fetch_one(&ctx.db)
    .await?;

    Ok(Json(result))
}

struct ApiError(anyhow::Error);

impl IntoResponse for ApiError {
    fn into_response(self) -> Response {
        (
            StatusCode::INTERNAL_SERVER_ERROR,
            format!("Something went wrong: {}", self.0),
        )
            .into_response()
    }
}

impl<E> From<E> for ApiError
where
    E: Into<anyhow::Error>,
{
    fn from(err: E) -> Self {
        Self(err.into())
    }
}
