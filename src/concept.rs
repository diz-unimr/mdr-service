use std::sync::Arc;
use axum::{debug_handler, Json, Router};
use axum::extract::{Path, State};
use axum::routing::get;
use serde_derive::{Deserialize, Serialize};
use sqlx::FromRow;
use crate::error::ApiError;
use crate::server::ApiContext;
use sqlx::types::Uuid;

#[derive(Deserialize, Serialize, FromRow)]
struct Concept {
    id: Uuid,
    name: String,
    parent_id: Option<Uuid>,
    module_id: Option<Uuid>,
    // ...
 }

pub(crate) fn router() -> Router<Arc<ApiContext>> {
    Router::new()
        .route("/ontology/{id}", get(ontology))
}

#[debug_handler]
async fn ontology(
    State(ctx): State<Arc<ApiContext>>,
    Path(id): Path<Uuid>,
) -> Result<Json<Vec<Concept>>, ApiError> {
    let result = sqlx::query_as!(Concept,
        // "select * from concepts where id = $1",
        r#"WITH RECURSIVE ontology AS
            ( select id,name,parent_id,module_id
               from concepts where module_id = $1 and parent_id is null
               UNION ALL SELECT c.* FROM concepts c
               JOIN ontology on c.parent_id = ontology.id )
           SELECT id as "id!", name as "name!",parent_id,module_id from ontology"#,
        id)
        .fetch_all(&ctx.db)
        .await?;

    Ok(Json(result))
}