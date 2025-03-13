use crate::error::ApiError;
use crate::server::ApiContext;
use anyhow::anyhow;
use axum::extract::{Path, State};
pub use axum::http::StatusCode;
use axum::routing::{get, post};
use axum::{debug_handler, Router};
use serde_derive::{Deserialize, Serialize};
use sqlx::types::{Json, Uuid};
use sqlx::FromRow;
use std::sync::Arc;

#[derive(Deserialize, Serialize, FromRow)]
struct Coding {
    code: String,
    system: String,
    display: String,
    version: String,
}

#[derive(Deserialize, Serialize, FromRow)]
struct Concept {
    id: Uuid,
    display: String,
    parent_id: Option<Uuid>,
    module_id: Uuid,
    term_codes: Option<Json<Vec<Coding>>>,
    leaf: bool,
    time_restriction_allowed: Option<bool>,
    filter_type: Option<String>,
    selectable: bool,
    filter_options: Option<Json<Vec<Coding>>>,
    version: String,
}

#[derive(Deserialize, Serialize)]
struct Search {
    module_id: Uuid,
    search_term: String,
}

pub(crate) fn router() -> Router<Arc<ApiContext>> {
    Router::new()
        .route("/ontology/{module_id}", get(ontology))
        .route("/concepts/search", post(search))
}

#[debug_handler]
async fn ontology(
    State(ctx): State<Arc<ApiContext>>,
    Path(module_id): Path<Uuid>,
) -> Result<axum::Json<Vec<Concept>>, ApiError> {
    let result = sqlx::query_as!(
        Concept,
        r#"with recursive ontology as (
                select *
                from concepts where module_id = $1 and parent_id is null
                union all select c.* from concepts c
                join ontology on c.parent_id = ontology.id 
           )
           select id as "id!", display as "display!",parent_id,module_id as "module_id!",
                term_codes as "term_codes: Json<Vec<Coding>>",leaf as "leaf!",
                time_restriction_allowed,filter_type,selectable as "selectable!",
                filter_options as "filter_options: Json<Vec<Coding>>", version as "version!"
                from ontology"#,
        module_id
    )
    .fetch_all(&ctx.db)
    .await?;

    Ok(axum::Json(result))
}

#[debug_handler]
async fn search(
    State(ctx): State<Arc<ApiContext>>,
    search: axum::Json<Search>,
) -> Result<axum::Json<Vec<Concept>>, ApiError> {
    if search.search_term.len() < 2 {
        return Err(ApiError(
            anyhow!("Search term must consist of at least 2 characters"),
            StatusCode::BAD_REQUEST,
        ));
    }

    let term_like = format!("%{}%", search.search_term.to_lowercase());
    let result = sqlx::query_as!(
        Concept,
        r#"select id as "id!", display as "display!",parent_id,module_id as "module_id!",
                  term_codes as "term_codes: Json<Vec<Coding>>",leaf as "leaf!",
                  time_restriction_allowed,filter_type,selectable as "selectable!",
                  filter_options as "filter_options: Json<Vec<Coding>>", version as "version!"
           from concepts
           where module_id = $1
           and selectable is true
           and (lower(display) like lower($2)
           or exists(select 1 from jsonb_array_elements(term_codes) o(obj) where lower(o.obj ->> 'code') like $3)
           )"#,
        search.module_id,
        term_like,
        term_like,
    )
    .fetch_all(&ctx.db)
    .await?;

    Ok(axum::Json(result))
}
