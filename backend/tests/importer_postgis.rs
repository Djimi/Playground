use std::path::Path;

use axum::{Router, http::StatusCode, response::IntoResponse, routing::post};
use serde_json::Value;
use sofia_playgrounds_backend::{
    enrichment::{match_sources, merge_catalog, normalize_sofiaplan},
    importer::{
        load_neighborhoods, normalize_overpass, replace_snapshot, run_import, SnapshotCounts,
    },
};
use sqlx::{PgPool, Row};
use tokio::{net::TcpListener, task::JoinHandle};

const VALID: &str = r#"{"elements":[
  {"type":"node","id":1,"lat":0,"lon":0,"tags":{"leisure":"playground","name":"Boundary"}},
  {"type":"node","id":2,"lat":0,"lon":1.5,"tags":{"leisure":"playground"}},
  {"type":"node","id":3,"lat":5,"lon":5,"tags":{"leisure":"playground"}}
]}"#;
const EQUIPMENT: &str = r#"{"elements":[
  {"type":"way","id":10,"geometry":[{"lat":-0.9,"lon":-0.9},{"lat":-0.9,"lon":-0.2},{"lat":-0.2,"lon":-0.2},{"lat":-0.2,"lon":-0.9},{"lat":-0.9,"lon":-0.9}],"tags":{"leisure":"playground","playground:swing":"yes"}},
  {"type":"node","id":11,"lat":-0.8,"lon":-0.8,"tags":{"playground":"slide"}},
  {"type":"node","id":12,"lat":-0.7,"lon":-0.7,"tags":{"playground":"slide"}},
  {"type":"node","id":13,"lat":-0.6,"lon":1.5,"tags":{"playground":"seesaw"}}
]}"#;
const INVALID: &str =
    r#"{"elements":[{"type":"node","id":1,"lat":100,"lon":0,"tags":{"leisure":"playground"}}]}"#;
const REMARKED: &str = r#"{"remark":"runtime error","elements":[]}"#;
const ENRICHED: &str = r#"{"elements":[
  {"type":"node","id":1,"lat":42.7070,"lon":23.3444,"tags":{"leisure":"playground","name":"Matched"}},
  {"type":"node","id":2,"lat":42.7000,"lon":23.3000,"tags":{"leisure":"playground"}},
  {"type":"node","id":3,"lat":42.7500,"lon":23.3500,"tags":{"leisure":"playground"}}
]}"#;

async fn valid() -> &'static str {
    VALID
}

async fn equipment() -> &'static str {
    EQUIPMENT
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
        .route("/equipment", post(equipment))
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

async fn count(pool: &PgPool, table: &str) -> i64 {
    sqlx::query_scalar(&format!("SELECT count(*) FROM {table}"))
        .fetch_one(pool)
        .await
        .unwrap()
}

async fn replace_enriched_snapshot(pool: &PgPool) -> anyhow::Result<SnapshotCounts> {
    let osm = normalize_overpass(ENRICHED)?;
    let sofia = normalize_sofiaplan(include_str!("fixtures/sofiaplan-playgrounds.geojson"))?;
    let matches = match_sources(&osm, &sofia);
    let catalog = merge_catalog(&osm, &sofia, &matches);
    let mut sources = osm;
    sources.extend(sofia);

    replace_snapshot(
        pool,
        &load_neighborhoods(Path::new("tests/fixtures/import-neighborhoods.geojson"))?,
        &sources,
        &catalog.playgrounds,
        &catalog.source_links,
    )
    .await
}

async fn seed_prior_catalog(pool: &PgPool) {
    sqlx::query("DELETE FROM playground_neighborhoods")
        .execute(pool)
        .await
        .unwrap();
    sqlx::query("DELETE FROM playground_source_links")
        .execute(pool)
        .await
        .unwrap();
    sqlx::query("DELETE FROM playgrounds")
        .execute(pool)
        .await
        .unwrap();
    sqlx::query("DELETE FROM source_playgrounds")
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
    sqlx::query(
        "INSERT INTO source_playgrounds (source, external_id, raw_data, location) VALUES ('openstreetmap', 'prior/1', '{}'::jsonb, ST_SetSRID(ST_MakePoint(23.3, 42.7), 4326)::geography)",
    )
    .execute(pool)
    .await
    .unwrap();
    sqlx::query(
        "INSERT INTO playground_source_links (playground_id, source, external_id, match_method) VALUES ('prior/1', 'openstreetmap', 'prior/1', 'unmatched')",
    )
    .execute(pool)
    .await
    .unwrap();
}

#[sqlx::test(migrations = "./migrations")]
async fn enriched_snapshot_persists_sources_links_and_canonical_provenance(pool: PgPool) {
    let first = replace_enriched_snapshot(&pool).await.unwrap();
    let second = replace_enriched_snapshot(&pool).await.unwrap();

    assert_eq!(first, second);
    assert_eq!(
        second,
        SnapshotCounts {
            source_records: 6,
            neighborhoods: 3,
            playgrounds: 4,
            memberships: 0,
        }
    );
    assert_eq!(count(&pool, "source_playgrounds").await, 6);
    assert_eq!(count(&pool, "playground_source_links").await, 5);
    assert_eq!(count(&pool, "playgrounds").await, 4);
    assert_eq!(
        sqlx::query_scalar::<_, String>(
            "SELECT raw_data #>> '{properties,nobekt_new}' FROM source_playgrounds WHERE source = 'sofiaplan' AND external_id = '06.129'",
        )
        .fetch_one(&pool)
        .await
        .unwrap(),
        "06.129"
    );
    assert_eq!(
        sqlx::query_scalar::<_, String>("SELECT primary_source FROM playgrounds WHERE id = 'node/1'")
            .fetch_one(&pool)
            .await
            .unwrap(),
        "openstreetmap"
    );
    assert!(
        sqlx::query_scalar::<_, bool>(
            "SELECT source_values @> '[{\"field\":\"address\",\"source_id\":\"06.129\",\"selected\":true}]'::jsonb FROM playgrounds WHERE id = 'node/1'",
        )
        .fetch_one(&pool)
        .await
        .unwrap()
    );
    let (method, distance): (String, f64) = sqlx::query_as(
        "SELECT match_method, match_distance_meters FROM playground_source_links WHERE source = 'sofiaplan' AND external_id = '06.129'",
    )
    .fetch_one(&pool)
    .await
    .unwrap();
    assert_eq!(method, "proximity");
    assert_eq!(distance, 0.0);
    assert_eq!(
        sqlx::query_scalar::<_, i64>(
            "SELECT count(*) FROM source_playgrounds WHERE source = 'sofiaplan' AND external_id = '06.131'",
        )
        .fetch_one(&pool)
        .await
        .unwrap(),
        1
    );
}

#[sqlx::test(migrations = "./migrations")]
async fn enriched_snapshot_rolls_back_sources_and_canonical_records_on_insert_failure(pool: PgPool) {
    seed_prior_catalog(&pool).await;
    sqlx::query(
        "CREATE FUNCTION importer_test_fail() RETURNS trigger LANGUAGE plpgsql AS $$ BEGIN RAISE EXCEPTION 'forced canonical failure'; END $$",
    )
    .execute(&pool)
    .await
    .unwrap();
    sqlx::query(
        "CREATE TRIGGER importer_test_fail BEFORE INSERT ON playgrounds FOR EACH ROW EXECUTE FUNCTION importer_test_fail()",
    )
    .execute(&pool)
    .await
    .unwrap();

    assert!(replace_enriched_snapshot(&pool).await.is_err());
    assert_eq!(catalog_ids(&pool).await, ["prior/1"]);
    assert_eq!(
        sqlx::query_scalar::<_, String>(
            "SELECT external_id FROM source_playgrounds WHERE source = 'openstreetmap'",
        )
        .fetch_one(&pool)
        .await
        .unwrap(),
        "prior/1"
    );
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

#[sqlx::test(migrations = "./migrations")]
async fn importer_persists_known_counts_and_unknown_boolean_equipment(pool: PgPool) {
    let (base_url, server) = fixture_server().await;
    let neighborhoods = Path::new("tests/fixtures/import-neighborhoods.geojson");

    run_import(&pool, &format!("{base_url}/equipment"), neighborhoods)
        .await
        .unwrap();

    let (capabilities, counts): (Vec<String>, Value) = sqlx::query_as(
        "SELECT capabilities, equipment_counts FROM playgrounds WHERE id = 'way/10'",
    )
    .fetch_one(&pool)
    .await
    .unwrap();
    assert_eq!(capabilities, ["slide", "swing"]);
    assert_eq!(counts, serde_json::json!({"slide": 2}));

    run_import(&pool, &format!("{base_url}/valid"), neighborhoods)
        .await
        .unwrap();
    assert_eq!(
        sqlx::query_scalar::<_, i64>(
            "SELECT count(*) FROM playgrounds WHERE equipment_counts <> '{}'::jsonb",
        )
        .fetch_one(&pool)
        .await
        .unwrap(),
        0
    );

    server.abort();
}
