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

#[derive(Deserialize, Serialize, FromRow, Clone, Debug)]
struct Coding {
    code: String,
    system: String,
    display: String,
    version: Option<String>,
}

#[derive(Deserialize, Serialize, FromRow, Clone, Default)]
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

#[derive(Deserialize, Serialize, Clone, Debug)]
struct ConceptTree {
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
    children: Vec<ConceptTree>,
}

impl From<Concept> for ConceptTree {
    fn from(c: Concept) -> Self {
        ConceptTree {
            id: c.id,
            display: c.display,
            parent_id: c.parent_id,
            module_id: c.module_id,
            term_codes: c.term_codes,
            leaf: c.leaf,
            time_restriction_allowed: c.time_restriction_allowed,
            filter_type: c.filter_type,
            selectable: c.selectable,
            filter_options: c.filter_options,
            version: c.version,
            children: vec![],
        }
    }
}

impl ConceptTree {
    pub(crate) fn add_child(&mut self, child: &ConceptTree) {
        self.children.push(child.clone());
    }

    pub(crate) fn add_child_to_tree(&mut self, child: &ConceptTree) -> bool {
        if self.id == child.parent_id.unwrap() {
            self.add_child(child);
            return true;
        }

        for c in self.children.iter_mut() {
            c.add_child_to_tree(child);
        }
        false
    }
}

#[derive(Deserialize, Serialize)]
struct Search {
    module_id: Uuid,
    search_term: String,
}

pub(crate) fn router() -> Router<Arc<ApiContext>> {
    Router::new()
        .route("/ontology/tree/{module_id}", get(ontology))
        .route("/ontology/concepts/search", post(search))
        .route("/ontology/concepts/{concept_id}", get(read))
}

#[debug_handler]
async fn ontology(
    State(ctx): State<Arc<ApiContext>>,
    Path(module_id): Path<Uuid>,
) -> Result<axum::Json<Vec<ConceptTree>>, ApiError> {
    let result = sqlx::query_as!(
        Concept,
        r#"with recursive ontology as (
               (select * from concepts where module_id = $1 and parent_id is null
                order by leaf,display)
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

    // build tree
    let tree = build_concept_tree(result);

    Ok(axum::Json(tree))
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

#[debug_handler]
async fn read(
    State(ctx): State<Arc<ApiContext>>,
    Path(id): Path<Uuid>,
) -> Result<axum::Json<Concept>, ApiError> {
    let result = sqlx::query_as!(
        Concept,
        r#"select id as "id!", display as "display!",parent_id,module_id as "module_id!",
                  term_codes as "term_codes: Json<Vec<Coding>>",leaf as "leaf!",
                  time_restriction_allowed,filter_type,selectable as "selectable!",
                  filter_options as "filter_options: Json<Vec<Coding>>", version as "version!"
           from concepts where id = $1"#,
        id
    )
    .fetch_optional(&ctx.db)
    .await?;

    match result {
        Some(concept) => Ok(axum::Json(concept)),
        None => Err(ApiError(
            anyhow!(format!("No concept found with id: {}", id)),
            StatusCode::NOT_FOUND,
        )),
    }
}

fn build_concept_tree(concepts: Vec<Concept>) -> Vec<ConceptTree> {
    let mut tree: Vec<ConceptTree> = vec![];
    for c in concepts {
        match c.parent_id {
            Some(_) => {
                tree.iter_mut()
                    .any(|t| t.add_child_to_tree(&c.clone().into()));
            }
            None => tree.push(c.into()),
        }
    }
    tree
}

#[cfg(test)]
mod tests {
    use crate::concept::{build_concept_tree, Concept};
    use uuid::Uuid;

    #[test]
    fn builds_nested_concepts() {
        let c1 = Concept {
            id: Uuid::new_v4(),
            ..Concept::default()
        };
        let c2 = Concept {
            id: Uuid::new_v4(),
            parent_id: Some(c1.id),
            ..Concept::default()
        };
        let c3 = Concept {
            id: Uuid::new_v4(),
            ..Concept::default()
        };
        let c4 = Concept {
            id: Uuid::new_v4(),
            parent_id: Some(c2.id),
            ..Concept::default()
        };

        // act
        let result = build_concept_tree(vec![c1.clone(), c2, c3, c4.clone()]);

        // check deeply nested
        let nested = result
            .iter()
            // c2
            .find(|c| c.id == c1.id)
            .unwrap()
            // nested child
            .children
            .first()
            .unwrap()
            // nested child
            .children
            .first()
            .unwrap();

        // two root elements
        assert_eq!(result.len(), 2);
        assert_eq!(nested.id, c4.id);
    }
}
