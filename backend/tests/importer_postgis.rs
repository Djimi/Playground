use std::path::Path;

use axum::{Router, http::StatusCode, response::IntoResponse, routing::post};
use serde_json::Value;
use sofia_playgrounds_backend::importer::run_import;
use sqlx::{PgPool, Row};
use tokio::{net::TcpListener, task::JoinHandle};

const VALID: &str = r#"{"elements":[
  {"type":"node","id":1,"lat":0,"lon":0,"tags":{"leisure":"playground","name":"Boundary"}},
  {"type":"node","id":2,"lat":0,"lon":1.5,"tags":{"leisure":"playground"}},
  {"type":"node","id":3,"lat":5,"lon":5,"tags":{"leisure":"playground"}}
]}"#;
const INVALID: &str =
    r#"{"elements":[{"type":"node","id":1,"lat":100,"lon":0,"tags":{"leisure":"playground"}}]}"#;
const REMARKED: &str = r#"{"remark":"runtime error","elements":[]}"#;

async fn valid() -> &'static str {
    VALID
}

async fn invalid() -> &'static str {
    INVALID
}

async fn remarked() -> &'static str {
    REMARKED
}

async fn failed() -> impl IntoResponse {
    (StatusCode::SERVICE_UNAVAILABLE, "fixture unavailable")
}

async fn fixture_server() -> (String, JoinHandle<()>) {
    let listener = TcpListener::bind(("127.0.0.1", 0)).await.unwrap();
    let address = listener.local_addr().unwrap();
    let app = Router::new()
        .route("/valid", post(valid))
        .route("/invalid", post(invalid))
        .route("/remarked", post(remarked))
        .route("/failed", post(failed));
    let server = tokio::spawn(async move {
        axum::serve(listener, app).await.unwrap();
    });
    (format!("http://{address}"), server)
}

async fn catalog_ids(pool: &PgPool) -> Vec<String> {
    sqlx::query("SELECT id FROM playgrounds ORDER BY id")
        .fetch_all(pool)
        .await
        .unwrap()
        .into_iter()
        .map(|row| row.get("id"))
        .collect()
}

async fn seed_prior_catalog(pool: &PgPool) {
    sqlx::query("DELETE FROM playground_neighborhoods")
        .execute(pool)
        .await
        .unwrap();
    sqlx::query("DELETE FROM playgrounds")
        .execute(pool)
        .await
        .unwrap();
    sqlx::query("DELETE FROM neighborhoods")
        .execute(pool)
        .await
        .unwrap();
    sqlx::query(
        "INSERT INTO playgrounds (id, location, source_url) VALUES ('prior/1', ST_SetSRID(ST_MakePoint(23.3, 42.7), 4326)::geography, 'https://example.test/prior')",
    )
    .execute(pool)
    .await
    .unwrap();
}

#[sqlx::test(migrations = "./migrations")]
async fn importer_is_atomic_idempotent_and_assigns_every_covering_neighborhood(pool: PgPool) {
    seed_prior_catalog(&pool).await;
    let (base_url, server) = fixture_server().await;
    let valid_neighborhoods = Path::new("tests/fixtures/import-neighborhoods.geojson");
    let invalid_neighborhoods = Path::new("tests/fixtures/invalid-neighborhoods.geojson");

    for path in ["failed", "remarked", "invalid"] {
        assert!(
            run_import(&pool, &format!("{base_url}/{path}"), valid_neighborhoods)
                .await
                .is_err()
        );
        assert_eq!(catalog_ids(&pool).await, ["prior/1"]);
    }

    assert!(
        run_import(&pool, &format!("{base_url}/valid"), invalid_neighborhoods)
            .await
            .is_err()
    );
    assert_eq!(catalog_ids(&pool).await, ["prior/1"]);

    let first = run_import(&pool, &format!("{base_url}/valid"), valid_neighborhoods)
        .await
        .unwrap();
    let second = run_import(&pool, &format!("{base_url}/valid"), valid_neighborhoods)
        .await
        .unwrap();
    assert_eq!(first, second);
    assert_eq!(
        (second.playgrounds, second.neighborhoods, second.memberships),
        (3, 3, 4)
    );
    assert_eq!(catalog_ids(&pool).await, ["node/1", "node/2", "node/3"]);

    let memberships: Value = sqlx::query_scalar(
        "SELECT jsonb_object_agg(id, neighborhoods) FROM (SELECT p.id, COALESCE(jsonb_agg(pn.neighborhood_id ORDER BY pn.neighborhood_id) FILTER (WHERE pn.neighborhood_id IS NOT NULL), '[]'::jsonb) neighborhoods FROM playgrounds p LEFT JOIN playground_neighborhoods pn ON pn.playground_id = p.id GROUP BY p.id) rows",
    )
    .fetch_one(&pool)
    .await
    .unwrap();
    assert_eq!(
        memberships,
        serde_json::json!({
            "node/1": ["child", "east", "parent"],
            "node/2": ["parent"],
            "node/3": []
        })
    );

    server.abort();
}
