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
    #[serde(skip_serializing_if = "Vec::is_empty")]
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

impl PartialEq for ConceptTree {
    fn eq(&self, other: &Self) -> bool {
        self.id == other.id
    }
}
impl From<&Concept> for ConceptTree {
    fn from(c: &Concept) -> Self {
        c.clone().into()
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
#[serde(rename_all = "kebab-case")]
pub enum SearchResult {
    Tree,
}

#[derive(Deserialize, Serialize)]
struct Search {
    module_id: Uuid,
    search_term: String,
    display: Option<SearchResult>,
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
) -> Result<axum::Json<Vec<ConceptTree>>, ApiError> {
    if search.search_term.len() < 2 {
        return Err(ApiError(
            anyhow!("Search term must consist of at least 2 characters"),
            StatusCode::BAD_REQUEST,
        ));
    }

    let term_like = format!("%{}%", search.search_term.to_lowercase());
    let result: Vec<Concept> = sqlx::query_as!(
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

    let tree: Vec<ConceptTree> = if search.display.is_some() {
        to_tree(result)
    } else {
        result.into_iter().map(|c| c.into()).collect()
    };
    Ok(axum::Json(tree))
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

fn to_tree(concepts: Vec<Concept>) -> Vec<ConceptTree> {
    let mut tree: Vec<ConceptTree> = concepts.iter().map(|c| c.into()).collect();
    // reorder children with existing parents
    let reorder = tree
        .clone()
        .into_iter()
        .filter(|c| {
            tree.iter()
                .any(|o| c.parent_id.is_some() && o.id == c.parent_id.unwrap())
        })
        .collect::<Vec<ConceptTree>>();

    // remove children from flat vector
    tree.retain(|c| !reorder.contains(c));
    for c in reorder {
        // add child to its parent
        tree.iter_mut().any(|t| t.add_child_to_tree(&c));
    }

    tree
}

#[cfg(test)]
mod tests {
    use crate::concept::{build_concept_tree, router, Concept, Search, StatusCode};
    use crate::server::ApiContext;
    use axum::body::Body;
    use axum::http::{Method, Request};
    use axum::response::Response;
    use axum::{http, Router};
    use http_body_util::BodyExt;
    use serde_json::{json, Value};
    use sqlx::PgPool;
    use std::sync::Arc;
    use tower::ServiceExt;
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

    #[sqlx::test(fixtures("concepts"))]
    async fn read_test(pool: PgPool) {
        let router = setup_router(pool);

        let response = send_request(
            router,
            "/ontology/concepts/a52b18659011fe8adeb112ce01327a2d".to_owned(),
            Method::GET,
            Body::empty(),
        )
        .await;

        assert_eq!(response.status(), StatusCode::OK);

        let body = parse_json(response).await.unwrap();

        assert_eq!(
            body,
            json!({
              "id": "a52b1865-9011-fe8a-deb1-12ce01327a2d",
              "display": "Vancomycin",
              "parent_id": "ce3e2ac8-6da7-4b36-7e7d-57a628022aca",
              "module_id": "4bfd4e2e-caf5-f7ae-3ef8-400ab0858ec7",
              "term_codes": [
                {
                  "code": "VANC",
                  "system": "https://fhir.diz.uni-marburg.de/CodeSystem/swisslab-code",
                  "display": "Vancomycin",
                  "version": null
                },
                {
                  "code": "20578-1",
                  "system": "http://loinc.org",
                  "display": "Vancomycin [Mass/volume] in Serum or Plasma",
                  "version": "2.73"
                }
              ],
              "leaf": true,
              "time_restriction_allowed": true,
              "filter_type": null,
              "selectable": true,
              "filter_options": null,
              "version": "2.2.0"
            })
        );
    }

    #[sqlx::test(fixtures("concepts"))]
    async fn ontology_test(pool: PgPool) {
        let router = setup_router(pool);

        let response = send_request(
            router,
            "/ontology/tree/4bfd4e2ecaf5f7ae3ef8400ab0858ec7".to_owned(),
            Method::GET,
            Body::empty(),
        )
        .await;

        assert_eq!(response.status(), StatusCode::OK);

        let body: Value = parse_json(response).await.unwrap();

        assert_eq!(body.pointer("/0/display").unwrap(), &json!("Medikamente"));
        assert_eq!(
            body.pointer("/0/children/0/children/0/display").unwrap(),
            &json!("Voriconazol [Fremdlabor]")
        );
    }

    #[sqlx::test(fixtures("concepts"))]
    async fn search_test(pool: PgPool) {
        let router = setup_router(pool);

        // search lab module for code
        let search = Search {
            module_id: Uuid::parse_str("4bfd4e2ecaf5f7ae3ef8400ab0858ec7").unwrap(),
            search_term: "VORI".to_owned(),
            display: None,
        };

        let response = send_request(
            router,
            "/ontology/concepts/search".to_owned(),
            Method::POST,
            Body::from(serde_json::to_string(&search).unwrap()),
        )
        .await;

        assert_eq!(response.status(), StatusCode::OK);

        let body = parse_json(response).await.unwrap();
        assert_eq!(
            body,
            json!([{
              "id": "6f12427c-7db3-5328-e268-206113ac1c69",
              "display": "Voriconazol [Fremdlabor]",
              "parent_id": "ce3e2ac8-6da7-4b36-7e7d-57a628022aca",
              "module_id": "4bfd4e2e-caf5-f7ae-3ef8-400ab0858ec7",
              "term_codes": [
                {
                  "code": "VORI",
                  "system": "https://fhir.diz.uni-marburg.de/CodeSystem/swisslab-code",
                  "display": "Voriconazol [Fremdlabor]",
                  "version": null
                },
                {
                  "code": "38370-3",
                  "system": "http://loinc.org",
                  "display": "Voriconazole [Mass/volume] in Serum or Plasma",
                  "version": "2.42"
                }
              ],
              "leaf": true,
              "time_restriction_allowed": true,
              "filter_type": null,
              "selectable": true,
              "filter_options": null,
              "version": "2.2.0"
            }])
        );
    }

    #[sqlx::test(fixtures("concepts"))]
    async fn search_fails_test(pool: PgPool) {
        let router = setup_router(pool);

        // below minimum search term length
        let search = Search {
            module_id: Uuid::parse_str("4bfd4e2ecaf5f7ae3ef8400ab0858ec7").unwrap(),
            search_term: "x".to_owned(),
            display: None,
        };

        let response = send_request(
            router,
            "/ontology/concepts/search".to_owned(),
            Method::POST,
            Body::from(serde_json::to_string(&search).unwrap()),
        )
        .await;

        assert_eq!(response.status(), StatusCode::BAD_REQUEST);

        let body = parse_string(response).await.unwrap();

        assert_eq!(body, "Search term must consist of at least 2 characters");
    }

    async fn send_request(router: Router, uri: String, method: Method, body: Body) -> Response {
        router
            .oneshot(
                Request::builder()
                    .method(method)
                    .uri(uri)
                    .header(http::header::CONTENT_TYPE, mime::APPLICATION_JSON.as_ref())
                    .body(body)
                    .unwrap(),
            )
            .await
            .unwrap()
    }

    fn setup_router(pool: PgPool) -> Router {
        let state = Arc::new(ApiContext { db: pool });
        router().with_state(state)
    }

    async fn parse_json(response: Response) -> Result<Value, anyhow::Error> {
        let body = response.into_body().collect().await?.to_bytes();
        serde_json::from_slice(&body).map_err(|e| e.into())
    }

    async fn parse_string(response: Response) -> Result<String, anyhow::Error> {
        let body = response.into_body().collect().await?.to_bytes().to_vec();
        String::from_utf8(body).map_err(|e| e.into())
    }
}
