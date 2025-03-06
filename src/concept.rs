use crate::error::ApiError;
use crate::server::ApiContext;
use axum::extract::{Path, State};
use axum::routing::get;
use axum::{debug_handler, Router};
use serde_derive::{Deserialize, Serialize};
use sqlx::types::{JsonValue, Uuid};
use sqlx::FromRow;
use std::sync::Arc;

// #[derive(Deserialize, Serialize, FromRow)]
// struct Coding {
//     code: String,
//     system: String,
//     display: String,
//     version: String,
// }

#[derive(Deserialize, Serialize, FromRow)]
struct Concept {
    id: Uuid,
    display: String,
    parent_id: Option<Uuid>,
    module_id: Uuid,
    term_codes: Option<JsonValue>,
    leaf: bool,
    time_restriction_allowed: Option<bool>,
    filter_type: Option<String>,
    selectable: bool,
    //    filter_options: Option<String>,
    filter_options: Option<JsonValue>,
    // filter_options: Option<Json<Vec<Coding>>>,
    version: String,
}

pub(crate) fn router() -> Router<Arc<ApiContext>> {
    Router::new().route("/ontology/{id}", get(ontology))
}

#[debug_handler]
async fn ontology(
    State(ctx): State<Arc<ApiContext>>,
    Path(id): Path<Uuid>,
) -> Result<axum::Json<Vec<Concept>>, ApiError> {
    let result = sqlx::query_as!(
        Concept,
        r#"WITH RECURSIVE ontology AS
            ( select *
               from concepts where module_id = $1 and parent_id is null
               UNION ALL SELECT c.* FROM concepts c
               JOIN ontology on c.parent_id = ontology.id )
           SELECT id as "id!", display as "display!",parent_id,module_id as "module_id!",
                  term_codes,leaf as "leaf!",time_restriction_allowed,filter_type,
                  selectable as "selectable!",filter_options,version as "version!" from ontology"#,
        id
    )
    .fetch_all(&ctx.db)
    .await?;

    Ok(axum::Json(result))
}
