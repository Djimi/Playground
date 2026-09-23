use std::path::Path;

use axum::{
    Json, Router,
    http::StatusCode,
    response::IntoResponse,
    routing::{get, post},
};
use serde_json::{Value, json};
use sofia_playgrounds_backend::{
    enrichment::{match_sources, merge_catalog, normalize_sofiaplan},
    importer::{
        ImportEndpoints, SnapshotCounts, load_neighborhoods, normalize_overpass, replace_snapshot,
        run_import,
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
const PHOTO_OSM: &str = r#"{"elements":[
  {"type":"node","id":1,"lat":42.7070,"lon":23.3444,"tags":{"leisure":"playground","wikimedia_commons":"File:Requested title.jpg"}},
  {"type":"node","id":2,"lat":42.7000,"lon":23.3000,"tags":{"leisure":"playground","wikimedia_commons":"File:NC.jpg"}}
]}"#;
const AMBIGUOUS_OSM: &str = r#"{"elements":[
  {"type":"node","id":1,"lat":42.7070,"lon":23.3444,"tags":{"leisure":"playground"}},
  {"type":"node","id":2,"lat":42.7070,"lon":23.3444,"tags":{"leisure":"playground"}}
]}"#;

async fn valid() -> &'static str {
    VALID
}

async fn equipment() -> &'static str {
    EQUIPMENT
}

async fn photo_osm() -> &'static str {
    PHOTO_OSM
}

async fn ambiguous_osm() -> &'static str {
    AMBIGUOUS_OSM
}

async fn sofiaplan() -> &'static str {
    include_str!("fixtures/sofiaplan-playgrounds.geojson")
}

async fn sofiaplan_malformed() -> &'static str {
    "not json"
}

async fn sofiaplan_empty() -> &'static str {
    r#"{"type":"FeatureCollection","features":[]}"#
}

async fn sofiaplan_ambiguous() -> &'static str {
    r#"{"type":"FeatureCollection","features":[
      {"type":"Feature","properties":{"nobekt_new":"06.129"},"geometry":{"type":"MultiPoint","coordinates":[[23.3444,42.7070]]}},
      {"type":"Feature","properties":{"nobekt_new":"06.130"},"geometry":{"type":"MultiPoint","coordinates":[[23.3444,42.7070]]}}
    ]}"#
}

async fn commons() -> Json<Value> {
    let mut fixture: Value =
        serde_json::from_str(include_str!("fixtures/commons-imageinfo.json")).unwrap();
    fixture["query"]["normalized"] =
        json!([{"from":"File:Requested title.jpg","to":"File:CC-BY-SA.jpg"}]);
    Json(fixture)
}

async fn invalid() -> &'static str {
    INVALID
}

async fn remarked() -> &'static str {
    REMARKED
}

async fn empty_osm() -> &'static str {
    r#"{"elements":[]}"#
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
        .route("/photo-osm", post(photo_osm))
        .route("/ambiguous-osm", post(ambiguous_osm))
        .route("/invalid", post(invalid))
        .route("/remarked", post(remarked))
        .route("/empty-osm", post(empty_osm))
        .route("/failed", post(failed))
        .route("/sofiaplan", get(sofiaplan))
        .route("/sofiaplan-malformed", get(sofiaplan_malformed))
        .route("/sofiaplan-empty", get(sofiaplan_empty))
        .route("/sofiaplan-ambiguous", get(sofiaplan_ambiguous))
        .route("/sofiaplan-failed", get(failed))
        .route("/commons", get(commons))
        .route("/commons-failed", get(failed));
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

fn endpoints(base_url: &str, osm: &str, sofia: &str, commons: &str) -> ImportEndpoints {
    ImportEndpoints {
        overpass_url: format!("{base_url}/{osm}"),
        sofiaplan_url: format!("{base_url}/{sofia}"),
        commons_api_url: format!("{base_url}/{commons}"),
    }
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
        sqlx::query_scalar::<_, String>(
            "SELECT primary_source FROM playgrounds WHERE id = 'node/1'"
        )
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
async fn enriched_snapshot_rolls_back_sources_and_canonical_records_on_insert_failure(
    pool: PgPool,
) {
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

    for path in ["failed", "remarked", "invalid", "empty-osm"] {
        assert!(
            run_import(
                &pool,
                &endpoints(&base_url, path, "sofiaplan", "commons"),
                valid_neighborhoods
            )
            .await
            .is_err()
        );
        assert_eq!(catalog_ids(&pool).await, ["prior/1"]);
        assert_eq!(count(&pool, "source_playgrounds").await, 1);
    }

    for path in ["sofiaplan-failed", "sofiaplan-malformed", "sofiaplan-empty"] {
        assert!(
            run_import(
                &pool,
                &endpoints(&base_url, "valid", path, "commons"),
                valid_neighborhoods
            )
            .await
            .is_err()
        );
        assert_eq!(catalog_ids(&pool).await, ["prior/1"]);
        assert_eq!(count(&pool, "source_playgrounds").await, 1);
    }

    assert!(
        run_import(
            &pool,
            &endpoints(&base_url, "valid", "sofiaplan", "commons"),
            invalid_neighborhoods
        )
        .await
        .is_err()
    );
    assert_eq!(catalog_ids(&pool).await, ["prior/1"]);

    let first = run_import(
        &pool,
        &endpoints(&base_url, "valid", "sofiaplan", "commons"),
        valid_neighborhoods,
    )
    .await
    .unwrap();
    let second = run_import(
        &pool,
        &endpoints(&base_url, "valid", "sofiaplan", "commons"),
        valid_neighborhoods,
    )
    .await
    .unwrap();
    assert_eq!(first, second);
    assert_eq!(
        (
            second.osm_source_records,
            second.sofia_source_records,
            second.canonical_playgrounds,
            second.clear_matches,
            second.ambiguous_records,
            second.excluded_source_records,
            second.accepted_photos,
            second.rejected_photos,
            second.neighborhoods,
            second.memberships
        ),
        (3, 3, 5, 0, 0, 1, 0, 0, 3, 4)
    );
    assert_eq!(
        catalog_ids(&pool).await,
        [
            "node/1",
            "node/2",
            "node/3",
            "sofiaplan/06.129",
            "sofiaplan/06.130"
        ]
    );
    assert_eq!(count(&pool, "source_playgrounds").await, 6);

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
            "node/3": [],
            "sofiaplan/06.129": [],
            "sofiaplan/06.130": []
        })
    );

    server.abort();
}

#[sqlx::test(migrations = "./migrations")]
async fn importer_persists_known_counts_and_unknown_boolean_equipment(pool: PgPool) {
    let (base_url, server) = fixture_server().await;
    let neighborhoods = Path::new("tests/fixtures/import-neighborhoods.geojson");

    run_import(
        &pool,
        &endpoints(&base_url, "equipment", "sofiaplan", "commons"),
        neighborhoods,
    )
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

    run_import(
        &pool,
        &endpoints(&base_url, "valid", "sofiaplan", "commons"),
        neighborhoods,
    )
    .await
    .unwrap();
    assert_eq!(
        sqlx::query_scalar::<_, i64>(
            "SELECT count(*) FROM playgrounds WHERE primary_source = 'openstreetmap' AND equipment_counts <> '{}'::jsonb",
        )
        .fetch_one(&pool)
        .await
        .unwrap(),
        0
    );

    server.abort();
}

#[sqlx::test(migrations = "./migrations")]
async fn importer_attaches_only_licensed_photos_to_the_referencing_playground(pool: PgPool) {
    let (base_url, server) = fixture_server().await;
    let counts = run_import(
        &pool,
        &endpoints(&base_url, "photo-osm", "sofiaplan", "commons"),
        Path::new("tests/fixtures/import-neighborhoods.geojson"),
    )
    .await
    .unwrap();

    assert_eq!(
        (
            counts.osm_source_records,
            counts.sofia_source_records,
            counts.canonical_playgrounds,
            counts.clear_matches,
            counts.excluded_source_records,
            counts.accepted_photos,
            counts.rejected_photos
        ),
        (2, 3, 3, 1, 1, 1, 1)
    );
    let photos: Value = sqlx::query_scalar("SELECT photos FROM playgrounds WHERE id = 'node/1'")
        .fetch_one(&pool)
        .await
        .unwrap();
    assert_eq!(
        photos[0]["url"],
        "https://upload.wikimedia.org/example/cc-by-sa.jpg"
    );
    assert_eq!(photos[0]["author"], "Example Author");
    assert_eq!(
        sqlx::query_scalar::<_, Vec<String>>(
            "SELECT photo_urls FROM playgrounds WHERE id = 'node/1'"
        )
        .fetch_one(&pool)
        .await
        .unwrap(),
        ["https://upload.wikimedia.org/example/cc-by-sa.jpg"]
    );
    assert_eq!(
        sqlx::query_scalar::<_, Value>("SELECT photos FROM playgrounds WHERE id = 'node/2'")
            .fetch_one(&pool)
            .await
            .unwrap(),
        serde_json::json!([])
    );
    server.abort();
}

#[sqlx::test(migrations = "./migrations")]
async fn failed_commons_keeps_primary_catalog_and_counts_unavailable_photos(pool: PgPool) {
    seed_prior_catalog(&pool).await;
    let (base_url, server) = fixture_server().await;
    let counts = run_import(
        &pool,
        &endpoints(&base_url, "photo-osm", "sofiaplan", "commons-failed"),
        Path::new("tests/fixtures/import-neighborhoods.geojson"),
    )
    .await
    .unwrap();

    assert_eq!(
        (
            counts.canonical_playgrounds,
            counts.accepted_photos,
            counts.rejected_photos
        ),
        (3, 0, 2)
    );
    assert_eq!(
        catalog_ids(&pool).await,
        ["node/1", "node/2", "sofiaplan/06.130"]
    );
    assert_eq!(count(&pool, "source_playgrounds").await, 5);
    assert_eq!(
        sqlx::query_scalar::<_, i64>(
            "SELECT count(*) FROM playgrounds WHERE jsonb_array_length(photos) > 0"
        )
        .fetch_one(&pool)
        .await
        .unwrap(),
        0
    );
    server.abort();
}

#[sqlx::test(migrations = "./migrations")]
async fn ambiguous_records_are_counted_once_across_multiple_candidate_edges(pool: PgPool) {
    let (base_url, server) = fixture_server().await;
    let counts = run_import(
        &pool,
        &endpoints(&base_url, "ambiguous-osm", "sofiaplan-ambiguous", "commons"),
        Path::new("tests/fixtures/import-neighborhoods.geojson"),
    )
    .await
    .unwrap();

    assert_eq!(
        (
            counts.clear_matches,
            counts.ambiguous_records,
            counts.canonical_playgrounds
        ),
        (0, 4, 4)
    );
    assert_eq!(count(&pool, "playground_source_links").await, 4);
    server.abort();
}
