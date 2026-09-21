use std::collections::{BTreeMap, BTreeSet};

use anyhow::{Context as _, Result};
use async_graphql::{
    Context, EmptyMutation, EmptySubscription, Enum, Error, ID, InputObject, Object,
    Result as GraphqlResult, Schema, SimpleObject, http::GraphiQLSource,
};
use async_graphql_axum::{GraphQLRequest, GraphQLResponse};
use axum::{
    Router,
    extract::State,
    http::{HeaderValue, Method, header::CONTENT_TYPE},
    response::Html,
    routing::{get, post},
};
use chrono::{DateTime, Utc};
use sqlx::{FromRow, PgPool, Postgres, QueryBuilder};
use tower_http::cors::{AllowOrigin, CorsLayer};

const OSM_ATTRIBUTION: &str = "© OpenStreetMap contributors";
const OSM_LICENSE: &str = "ODbL-1.0";

type PlaygroundSchema = Schema<QueryRoot, EmptyMutation, EmptySubscription>;

#[derive(Clone, Copy, Debug, Eq, Enum, PartialEq)]
pub enum Capability {
    Swing,
    Slide,
    ClimbingFrame,
    Sandpit,
    Seesaw,
    Springy,
    Playhouse,
    Roundabout,
}

impl Capability {
    fn database_value(self) -> &'static str {
        match self {
            Self::Swing => "swing",
            Self::Slide => "slide",
            Self::ClimbingFrame => "climbing_frame",
            Self::Sandpit => "sandpit",
            Self::Seesaw => "seesaw",
            Self::Springy => "springy",
            Self::Playhouse => "playhouse",
            Self::Roundabout => "roundabout",
        }
    }

    fn from_database(value: &str) -> Option<Self> {
        Some(match value {
            "swing" => Self::Swing,
            "slide" => Self::Slide,
            "climbing_frame" => Self::ClimbingFrame,
            "sandpit" => Self::Sandpit,
            "seesaw" => Self::Seesaw,
            "springy" => Self::Springy,
            "playhouse" => Self::Playhouse,
            "roundabout" => Self::Roundabout,
            _ => return None,
        })
    }
}

#[derive(Clone, Copy, Debug, InputObject)]
pub struct CoordinateInput {
    pub longitude: f64,
    pub latitude: f64,
}

#[derive(Clone, Copy, Debug, InputObject)]
pub struct BoundsInput {
    pub south_west: CoordinateInput,
    pub north_east: CoordinateInput,
}

#[derive(Clone, Debug, Default, InputObject)]
pub struct PlaygroundFilter {
    pub bounds: Option<BoundsInput>,
    pub center: Option<CoordinateInput>,
    pub radius_meters: Option<f64>,
    pub neighborhood_id: Option<String>,
    pub child_age: Option<i32>,
    pub required_capabilities: Option<Vec<Capability>>,
}

#[derive(SimpleObject)]
pub struct Coordinate {
    pub longitude: f64,
    pub latitude: f64,
}

#[derive(SimpleObject)]
pub struct Neighborhood {
    pub id: ID,
    pub name: String,
}

#[derive(SimpleObject)]
pub struct SourceMetadata {
    pub id: ID,
    pub url: String,
    pub updated_at: Option<String>,
    pub attribution: String,
    pub license: String,
}

#[derive(SimpleObject)]
pub struct Playground {
    pub id: ID,
    pub name: Option<String>,
    pub location: Coordinate,
    pub neighborhoods: Vec<Neighborhood>,
    pub capabilities: Vec<Capability>,
    pub equipment: Vec<Equipment>,
    pub min_age: Option<i32>,
    pub max_age: Option<i32>,
    pub photo_urls: Vec<String>,
    pub source: SourceMetadata,
    pub distance_meters: Option<f64>,
}

#[derive(SimpleObject)]
pub struct Equipment {
    pub capability: Capability,
    pub count: Option<i32>,
}

#[derive(FromRow)]
struct PlaygroundRow {
    id: String,
    name: Option<String>,
    longitude: f64,
    latitude: f64,
    neighborhood_ids: Vec<String>,
    neighborhood_names: Vec<String>,
    capabilities: Vec<String>,
    equipment_counts: sqlx::types::Json<BTreeMap<String, i32>>,
    min_age: Option<i16>,
    max_age: Option<i16>,
    photo_urls: Vec<String>,
    source_url: String,
    source_updated_at: Option<DateTime<Utc>>,
    distance_meters: Option<f64>,
}

impl PlaygroundRow {
    fn into_graphql(self) -> GraphqlResult<Playground> {
        if self.neighborhood_ids.len() != self.neighborhood_names.len() {
            return Err(Error::new("playground data is temporarily unavailable"));
        }
        let capabilities = self
            .capabilities
            .iter()
            .map(|value| Capability::from_database(value))
            .collect::<Option<Vec<_>>>()
            .ok_or_else(|| Error::new("playground data is temporarily unavailable"))?;
        let mut capabilities = capabilities;
        capabilities.sort_unstable_by_key(|value| value.database_value());
        capabilities.dedup();

        let capability_names = capabilities
            .iter()
            .map(|value| value.database_value())
            .collect::<BTreeSet<_>>();
        if self.equipment_counts.0.iter().any(|(value, count)| {
            *count <= 0
                || Capability::from_database(value).is_none()
                || !capability_names.contains(value.as_str())
        }) {
            return Err(Error::new("playground data is temporarily unavailable"));
        }
        let equipment = capabilities
            .iter()
            .map(|capability| Equipment {
                capability: *capability,
                count: self
                    .equipment_counts
                    .0
                    .get(capability.database_value())
                    .copied(),
            })
            .collect();

        Ok(Playground {
            source: SourceMetadata {
                id: self.id.clone().into(),
                url: self.source_url,
                updated_at: self.source_updated_at.map(|value| value.to_rfc3339()),
                attribution: OSM_ATTRIBUTION.into(),
                license: OSM_LICENSE.into(),
            },
            id: self.id.into(),
            name: self.name,
            location: Coordinate {
                longitude: self.longitude,
                latitude: self.latitude,
            },
            neighborhoods: self
                .neighborhood_ids
                .into_iter()
                .zip(self.neighborhood_names)
                .map(|(id, name)| Neighborhood {
                    id: id.into(),
                    name,
                })
                .collect(),
            capabilities,
            equipment,
            min_age: self.min_age.map(i32::from),
            max_age: self.max_age.map(i32::from),
            photo_urls: self.photo_urls,
            distance_meters: self.distance_meters,
        })
    }
}

pub struct QueryRoot;

#[Object]
impl QueryRoot {
    async fn playgrounds(
        &self,
        context: &Context<'_>,
        filter: Option<PlaygroundFilter>,
        #[graphql(default = 200)] limit: i32,
        #[graphql(default = 0)] offset: i32,
    ) -> GraphqlResult<Vec<Playground>> {
        validate_search(filter.as_ref(), limit, offset)?;
        let pool = context.data::<PgPool>()?;
        let rows = search(pool, filter.as_ref(), limit, offset)
            .await
            .map_err(internal_error)?;
        rows.into_iter().map(PlaygroundRow::into_graphql).collect()
    }

    async fn playground(&self, context: &Context<'_>, id: ID) -> GraphqlResult<Option<Playground>> {
        let pool = context.data::<PgPool>()?;
        detail(pool, id.as_str())
            .await
            .map_err(internal_error)?
            .map(PlaygroundRow::into_graphql)
            .transpose()
    }
}

fn validate_coordinate(value: CoordinateInput) -> GraphqlResult<()> {
    if !value.longitude.is_finite()
        || !value.latitude.is_finite()
        || !(-180.0..=180.0).contains(&value.longitude)
        || !(-90.0..=90.0).contains(&value.latitude)
    {
        return Err(Error::new("coordinates are outside valid ranges"));
    }
    Ok(())
}

fn validate_search(
    filter: Option<&PlaygroundFilter>,
    limit: i32,
    offset: i32,
) -> GraphqlResult<()> {
    if !(1..=500).contains(&limit) {
        return Err(Error::new("limit must be between 1 and 500"));
    }
    if offset < 0 {
        return Err(Error::new("offset must not be negative"));
    }

    let Some(filter) = filter else {
        return Ok(());
    };

    if let Some(bounds) = filter.bounds {
        validate_coordinate(bounds.south_west)?;
        validate_coordinate(bounds.north_east)?;
        if bounds.south_west.longitude > bounds.north_east.longitude
            || bounds.south_west.latitude > bounds.north_east.latitude
        {
            return Err(Error::new("bounds are inverted"));
        }
    }

    match (filter.center, filter.radius_meters) {
        (Some(center), Some(radius)) => {
            validate_coordinate(center)?;
            if !radius.is_finite() || radius <= 0.0 {
                return Err(Error::new("radiusMeters must be positive"));
            }
        }
        (None, None) => {}
        _ => {
            return Err(Error::new(
                "center and radiusMeters must be supplied together",
            ));
        }
    }

    if filter.child_age.is_some_and(|age| !(0..=18).contains(&age)) {
        return Err(Error::new("childAge must be between 0 and 18"));
    }

    Ok(())
}

const SELECT_PLAYGROUND: &str = r#"
SELECT p.id,
       p.name,
       ST_X(p.location::geometry)::double precision AS longitude,
       ST_Y(p.location::geometry)::double precision AS latitude,
       ARRAY(SELECT n.id
             FROM playground_neighborhoods pn
             JOIN neighborhoods n ON n.id = pn.neighborhood_id
             WHERE pn.playground_id = p.id
             ORDER BY n.id) AS neighborhood_ids,
       ARRAY(SELECT n.name
             FROM playground_neighborhoods pn
             JOIN neighborhoods n ON n.id = pn.neighborhood_id
             WHERE pn.playground_id = p.id
             ORDER BY n.id) AS neighborhood_names,
       p.capabilities,
       p.equipment_counts,
       p.min_age,
       p.max_age,
       p.photo_urls,
       p.source_url,
       p.source_updated_at,
"#;

async fn detail(pool: &PgPool, id: &str) -> sqlx::Result<Option<PlaygroundRow>> {
    let mut query = QueryBuilder::<Postgres>::new(SELECT_PLAYGROUND);
    query.push(" NULL::double precision AS distance_meters FROM playgrounds p WHERE p.id = ");
    query.push_bind(id);
    query
        .build_query_as::<PlaygroundRow>()
        .fetch_optional(pool)
        .await
}

async fn search(
    pool: &PgPool,
    filter: Option<&PlaygroundFilter>,
    limit: i32,
    offset: i32,
) -> sqlx::Result<Vec<PlaygroundRow>> {
    let mut query = QueryBuilder::<Postgres>::new(SELECT_PLAYGROUND);
    let radius = filter.and_then(|value| value.center.zip(value.radius_meters));

    if let Some((center, _)) = radius {
        query.push(" ST_Distance(p.location, ST_SetSRID(ST_MakePoint(");
        query.push_bind(center.longitude);
        query.push(", ");
        query.push_bind(center.latitude);
        query.push("), 4326)::geography) AS distance_meters");
    } else {
        query.push(" NULL::double precision AS distance_meters");
    }
    query.push(" FROM playgrounds p WHERE TRUE");

    if let Some(filter) = filter {
        if let Some(bounds) = filter.bounds {
            query.push(" AND ST_Covers(ST_MakeEnvelope(");
            query.push_bind(bounds.south_west.longitude);
            query.push(", ");
            query.push_bind(bounds.south_west.latitude);
            query.push(", ");
            query.push_bind(bounds.north_east.longitude);
            query.push(", ");
            query.push_bind(bounds.north_east.latitude);
            query.push(", 4326), p.location::geometry)");
        }
        if let Some((center, radius_meters)) = radius {
            query.push(" AND ST_DWithin(p.location, ST_SetSRID(ST_MakePoint(");
            query.push_bind(center.longitude);
            query.push(", ");
            query.push_bind(center.latitude);
            query.push("), 4326)::geography, ");
            query.push_bind(radius_meters);
            query.push(")");
        }
        if let Some(neighborhood_id) = &filter.neighborhood_id {
            query.push(
                " AND EXISTS (SELECT 1 FROM playground_neighborhoods pn WHERE pn.playground_id = p.id AND pn.neighborhood_id = ",
            );
            query.push_bind(neighborhood_id);
            query.push(")");
        }
        if let Some(age) = filter.child_age {
            query.push(" AND (p.min_age IS NOT NULL OR p.max_age IS NOT NULL)");
            query.push(" AND (p.min_age IS NULL OR p.min_age <= ");
            query.push_bind(age);
            query.push(") AND (p.max_age IS NULL OR p.max_age >= ");
            query.push_bind(age);
            query.push(")");
        }
        if let Some(capabilities) = &filter.required_capabilities
            && !capabilities.is_empty()
        {
            query.push(" AND p.capabilities @> ");
            query.push_bind(
                capabilities
                    .iter()
                    .map(|value| value.database_value().to_owned())
                    .collect::<Vec<_>>(),
            );
        }
    }

    if radius.is_some() {
        query.push(" ORDER BY distance_meters, p.id");
    } else {
        query.push(" ORDER BY p.id");
    }
    query.push(" LIMIT ");
    query.push_bind(i64::from(limit));
    query.push(" OFFSET ");
    query.push_bind(i64::from(offset));
    query
        .build_query_as::<PlaygroundRow>()
        .fetch_all(pool)
        .await
}

fn internal_error(error: sqlx::Error) -> Error {
    tracing::error!(%error, "GraphQL database query failed");
    Error::new("playground data is temporarily unavailable")
}

pub fn router(pool: PgPool, frontend_origin: &str) -> Result<Router> {
    let origin = HeaderValue::from_str(frontend_origin).context("FRONTEND_ORIGIN is invalid")?;
    let schema = Schema::build(QueryRoot, EmptyMutation, EmptySubscription)
        .data(pool)
        .finish();
    let cors = CorsLayer::new()
        .allow_origin(AllowOrigin::predicate(move |request_origin, _| {
            request_origin == origin
        }))
        .allow_methods([Method::POST])
        .allow_headers([CONTENT_TYPE]);

    Ok(Router::new()
        .route("/graphql", post(graphql_handler))
        .route("/graphiql", get(graphiql))
        .layer(cors)
        .with_state(schema))
}

async fn graphql_handler(
    State(schema): State<PlaygroundSchema>,
    request: GraphQLRequest,
) -> GraphQLResponse {
    schema.execute(request.into_inner()).await.into()
}

async fn graphiql() -> Html<String> {
    Html(GraphiQLSource::build().endpoint("/graphql").finish())
}

#[cfg(test)]
mod tests {
    use super::*;
    use async_graphql::Request;
    use axum::{
        body::Body,
        http::{Request as HttpRequest, StatusCode, header},
    };
    use http_body_util::BodyExt;
    use sqlx::postgres::PgPoolOptions;
    use tower::ServiceExt;

    fn lazy_pool() -> PgPool {
        PgPoolOptions::new()
            .acquire_timeout(std::time::Duration::from_millis(50))
            .connect_lazy("postgres://postgres:postgres@127.0.0.1/playgrounds")
            .unwrap()
    }

    fn valid_filter() -> PlaygroundFilter {
        PlaygroundFilter {
            center: Some(CoordinateInput {
                longitude: 23.32,
                latitude: 42.69,
            }),
            radius_meters: Some(1_000.0),
            child_age: Some(18),
            ..Default::default()
        }
    }

    #[test]
    fn validates_search_inputs_before_querying() {
        assert!(validate_search(Some(&valid_filter()), 1, 0).is_ok());
        assert!(validate_search(Some(&valid_filter()), 500, 1_000).is_ok());

        let invalid = [
            PlaygroundFilter {
                center: Some(CoordinateInput {
                    longitude: 181.0,
                    latitude: 42.0,
                }),
                radius_meters: Some(1.0),
                ..Default::default()
            },
            PlaygroundFilter {
                center: Some(CoordinateInput {
                    longitude: 23.0,
                    latitude: 91.0,
                }),
                radius_meters: Some(1.0),
                ..Default::default()
            },
            PlaygroundFilter {
                center: Some(CoordinateInput {
                    longitude: 23.0,
                    latitude: 42.0,
                }),
                ..Default::default()
            },
            PlaygroundFilter {
                radius_meters: Some(1.0),
                ..Default::default()
            },
            PlaygroundFilter {
                center: Some(CoordinateInput {
                    longitude: 23.0,
                    latitude: 42.0,
                }),
                radius_meters: Some(0.0),
                ..Default::default()
            },
            PlaygroundFilter {
                bounds: Some(BoundsInput {
                    south_west: CoordinateInput {
                        longitude: 24.0,
                        latitude: 43.0,
                    },
                    north_east: CoordinateInput {
                        longitude: 23.0,
                        latitude: 42.0,
                    },
                }),
                ..Default::default()
            },
            PlaygroundFilter {
                child_age: Some(-1),
                ..Default::default()
            },
            PlaygroundFilter {
                child_age: Some(19),
                ..Default::default()
            },
        ];
        for filter in &invalid {
            assert!(validate_search(Some(filter), 200, 0).is_err());
        }
        assert!(validate_search(None, 0, 0).is_err());
        assert!(validate_search(None, 501, 0).is_err());
        assert!(validate_search(None, 200, -1).is_err());
    }

    #[test]
    fn database_errors_do_not_expose_internal_details() {
        assert_eq!(
            internal_error(sqlx::Error::RowNotFound).message,
            "playground data is temporarily unavailable"
        );
    }

    #[tokio::test]
    async fn schema_is_query_only_and_uses_documented_default_limit() {
        let schema = Schema::build(QueryRoot, EmptyMutation, EmptySubscription)
            .data(lazy_pool())
            .finish();
        let sdl = schema.sdl();
        assert!(sdl.contains(
            "playgrounds(filter: PlaygroundFilter, limit: Int! = 200, offset: Int! = 0)"
        ));
        assert!(sdl.contains("playground(id: ID!)"));
        assert!(!sdl.contains("type Mutation"));

        let response = schema
            .execute(Request::new("{ __schema { mutationType { name } } }"))
            .await;
        assert!(response.errors.is_empty());
        assert_eq!(
            response.data.into_json().unwrap()["__schema"]["mutationType"],
            serde_json::Value::Null
        );
    }

    #[tokio::test]
    async fn invalid_graphql_input_does_not_touch_database() {
        let schema = Schema::build(QueryRoot, EmptyMutation, EmptySubscription)
            .data(lazy_pool())
            .finish();
        let response = schema
            .execute(Request::new(
                "{ playgrounds(filter: { center: { longitude: 23, latitude: 42 } }) { id } }",
            ))
            .await;
        assert_eq!(response.errors.len(), 1);
        assert!(response.errors[0].message.contains("supplied together"));
    }

    #[tokio::test]
    async fn router_exposes_graphql_graphiql_and_exact_cors_origin() {
        let app = router(lazy_pool(), "http://127.0.0.1:5173").unwrap();
        let graphql = app
            .clone()
            .oneshot(
                HttpRequest::post("/graphql")
                    .header(CONTENT_TYPE, "application/json")
                    .body(Body::from(r#"{"query":"{ __typename }"}"#))
                    .unwrap(),
            )
            .await
            .unwrap();
        assert_eq!(graphql.status(), StatusCode::OK);

        let unavailable = app
            .clone()
            .oneshot(
                HttpRequest::post("/graphql")
                    .header(CONTENT_TYPE, "application/json")
                    .body(Body::from(r#"{"query":"{ playgrounds { id } }"}"#))
                    .unwrap(),
            )
            .await
            .unwrap();
        let unavailable: serde_json::Value =
            serde_json::from_slice(&unavailable.into_body().collect().await.unwrap().to_bytes())
                .unwrap();
        assert_eq!(
            unavailable["errors"][0]["message"],
            "playground data is temporarily unavailable"
        );

        let graphiql = app
            .clone()
            .oneshot(HttpRequest::get("/graphiql").body(Body::empty()).unwrap())
            .await
            .unwrap();
        assert_eq!(graphiql.status(), StatusCode::OK);
        assert!(
            String::from_utf8(
                graphiql
                    .into_body()
                    .collect()
                    .await
                    .unwrap()
                    .to_bytes()
                    .to_vec()
            )
            .unwrap()
            .contains("/graphql")
        );

        let allowed = app
            .clone()
            .oneshot(
                HttpRequest::builder()
                    .method(Method::OPTIONS)
                    .uri("/graphql")
                    .header(header::ORIGIN, "http://127.0.0.1:5173")
                    .header(header::ACCESS_CONTROL_REQUEST_METHOD, "POST")
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();
        assert_eq!(
            allowed.headers()[header::ACCESS_CONTROL_ALLOW_ORIGIN],
            "http://127.0.0.1:5173"
        );

        let blocked = app
            .oneshot(
                HttpRequest::builder()
                    .method(Method::OPTIONS)
                    .uri("/graphql")
                    .header(header::ORIGIN, "http://evil.example")
                    .header(header::ACCESS_CONTROL_REQUEST_METHOD, "POST")
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();
        assert!(
            !blocked
                .headers()
                .contains_key(header::ACCESS_CONTROL_ALLOW_ORIGIN)
        );
    }
}
