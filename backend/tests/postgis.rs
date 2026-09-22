use axum::{
    Router,
    body::Body,
    http::{Request, header::CONTENT_TYPE},
};
use http_body_util::BodyExt;
use serde_json::{Value, json};
use sofia_playgrounds_backend::graphql;
use sqlx::PgPool;
use tower::ServiceExt;

async fn graphql(app: &Router, query: &str) -> Value {
    let response = app
        .clone()
        .oneshot(
            Request::post("/graphql")
                .header(CONTENT_TYPE, "application/json")
                .body(Body::from(json!({ "query": query }).to_string()))
                .unwrap(),
        )
        .await
        .unwrap();
    serde_json::from_slice(&response.into_body().collect().await.unwrap().to_bytes()).unwrap()
}

async fn clean(pool: &PgPool) {
    sqlx::query("DELETE FROM playgrounds WHERE id LIKE 'graphql-test/%'")
        .execute(pool)
        .await
        .unwrap();
    sqlx::query("DELETE FROM neighborhoods WHERE id LIKE 'graphql-test/%'")
        .execute(pool)
        .await
        .unwrap();
}

async fn seed(pool: &PgPool) {
    clean(pool).await;
    sqlx::query(
        r#"
        INSERT INTO neighborhoods (id, name, boundary) VALUES
          ('graphql-test/parent', 'Parent', ST_GeomFromText('MULTIPOLYGON(((-1 -1,1 -1,1 1,-1 1,-1 -1)))', 4326)),
          ('graphql-test/child', 'Child', ST_GeomFromText('MULTIPOLYGON(((-0.5 -0.5,0.5 -0.5,0.5 0.5,-0.5 0.5,-0.5 -0.5)))', 4326))
        "#,
    )
    .execute(pool)
    .await
    .unwrap();
    sqlx::query(
        r#"
        INSERT INTO playgrounds
          (id, location, capabilities, photo_urls, source_url)
        SELECT format('graphql-test/default-%s', lpad(series::text, 3, '0')),
               ST_SetSRID(ST_MakePoint(10, 10), 4326)::geography,
               ARRAY[]::text[], ARRAY[]::text[],
               format('https://www.openstreetmap.org/node/%s', series + 100)
        FROM generate_series(1, 201) AS series
        "#,
    )
    .execute(pool)
    .await
    .unwrap();
    sqlx::query(
        r#"
        INSERT INTO playgrounds
          (id, name, location, capabilities, equipment_counts, min_age, max_age, photo_urls, source_url, source_updated_at)
        VALUES
          ('graphql-test/a', 'Alpha', ST_SetSRID(ST_MakePoint(0, 0), 4326)::geography,
           ARRAY['swing', 'slide'], '{"swing": 2, "slide": 1}'::jsonb, 3, 8, ARRAY['https://example.test/a.jpg'],
           'https://www.openstreetmap.org/node/1', '2026-01-01T00:00:00Z'),
          ('graphql-test/b', NULL, ST_SetSRID(ST_MakePoint(0.001, 0), 4326)::geography,
           ARRAY['swing'], '{}'::jsonb, NULL, 5, ARRAY[]::text[],
           'https://www.openstreetmap.org/node/2', NULL),
          ('graphql-test/c', 'Unknown metadata', ST_SetSRID(ST_MakePoint(0.003, 0), 4326)::geography,
           ARRAY[]::text[], '{}'::jsonb, NULL, NULL, ARRAY[]::text[],
           'https://www.openstreetmap.org/node/3', NULL),
          ('graphql-test/d', 'Open maximum', ST_SetSRID(ST_MakePoint(0.002, 0), 4326)::geography,
           ARRAY['swing'], '{"swing": 3}'::jsonb, 10, NULL, ARRAY[]::text[],
           'https://www.openstreetmap.org/node/4', NULL)
        "#,
    )
    .execute(pool)
    .await
    .unwrap();
    sqlx::query(
        r#"
        INSERT INTO playground_neighborhoods (playground_id, neighborhood_id) VALUES
          ('graphql-test/a', 'graphql-test/parent'),
          ('graphql-test/a', 'graphql-test/child'),
          ('graphql-test/b', 'graphql-test/parent'),
          ('graphql-test/c', 'graphql-test/parent'),
          ('graphql-test/d', 'graphql-test/parent')
        "#,
    )
    .execute(pool)
    .await
    .unwrap();
}

#[sqlx::test(migrations = "./migrations")]
async fn equipment_counts_default_preserves_capabilities(pool: PgPool) {
    sqlx::query(
        "INSERT INTO playgrounds (id, location, capabilities, source_url) VALUES ('migration-test/one', ST_SetSRID(ST_MakePoint(23.3, 42.7), 4326)::geography, ARRAY['swing', 'slide'], 'https://example.test/migration')",
    )
    .execute(&pool)
    .await
    .unwrap();

    let (capabilities, counts): (Vec<String>, Value) = sqlx::query_as(
        "SELECT capabilities, equipment_counts FROM playgrounds WHERE id = 'migration-test/one'",
    )
    .fetch_one(&pool)
    .await
    .unwrap();
    assert_eq!(capabilities, ["swing", "slide"]);
    assert_eq!(counts, json!({}));
}

#[sqlx::test(migrations = "./migrations")]
async fn enrichment_schema_keeps_current_sources_and_safe_defaults(pool: PgPool) {
    sqlx::query(
        r#"INSERT INTO source_playgrounds
           (source, external_id, raw_data, location, source_date, date_meaning)
           VALUES ('sofiaplan', '06.129', '{}'::jsonb,
                   ST_SetSRID(ST_MakePoint(23.3444, 42.7070), 4326)::geography,
                   '2019-04-18T00:00:00Z', 'observation')"#,
    )
    .execute(&pool)
    .await
    .unwrap();
    sqlx::query(
        "INSERT INTO playgrounds (id, location, source_url, primary_source) VALUES ('sofiaplan/06.129', ST_SetSRID(ST_MakePoint(23.3444, 42.7070), 4326)::geography, 'https://urbandata.sofia.bg/dataset/playgrounds', 'sofiaplan')",
    )
    .execute(&pool)
    .await
    .unwrap();
    sqlx::query(
        "INSERT INTO playground_source_links (playground_id, source, external_id, match_method) VALUES ('sofiaplan/06.129', 'sofiaplan', '06.129', 'unmatched')",
    )
    .execute(&pool)
    .await
    .unwrap();

    let (photos, values): (Value, Value) = sqlx::query_as(
        "SELECT photos, source_values FROM playgrounds WHERE id = 'sofiaplan/06.129'",
    )
    .fetch_one(&pool)
    .await
    .unwrap();
    assert_eq!(photos, json!([]));
    assert_eq!(values, json!([]));

    assert!(sqlx::query(
        "INSERT INTO playground_source_links (playground_id, source, external_id, match_method) VALUES ('missing-playground', 'sofiaplan', '06.129', 'unmatched')",
    )
    .execute(&pool)
    .await
    .is_err());
    assert!(sqlx::query(
        "INSERT INTO playground_source_links (playground_id, source, external_id, match_method) VALUES ('sofiaplan/06.129', 'sofiaplan', 'missing', 'unmatched')",
    )
    .execute(&pool)
    .await
    .is_err());
    assert!(sqlx::query(
        "INSERT INTO playgrounds (id, location, source_url, photos) VALUES ('invalid-photos', ST_SetSRID(ST_MakePoint(23.3444, 42.7070), 4326)::geography, 'https://example.test/invalid-photos', '{}'::jsonb)",
    )
    .execute(&pool)
    .await
    .is_err());
    assert!(sqlx::query(
        "INSERT INTO playgrounds (id, location, source_url, source_values) VALUES ('invalid-source-values', ST_SetSRID(ST_MakePoint(23.3444, 42.7070), 4326)::geography, 'https://example.test/invalid-source-values', '{}'::jsonb)",
    )
    .execute(&pool)
    .await
    .is_err());

    sqlx::query("DELETE FROM source_playgrounds WHERE source = 'sofiaplan' AND external_id = '06.129'")
        .execute(&pool)
        .await
        .unwrap();
    let (source_links,): (i64,) = sqlx::query_as(
        "SELECT count(*)::bigint FROM playground_source_links WHERE source = 'sofiaplan' AND external_id = '06.129'",
    )
    .fetch_one(&pool)
    .await
    .unwrap();
    assert_eq!(source_links, 0);

    sqlx::query(
        r#"INSERT INTO source_playgrounds
           (source, external_id, raw_data, location)
           VALUES ('openstreetmap', 'node/1', '{}'::jsonb,
                   ST_SetSRID(ST_MakePoint(23.3444, 42.7070), 4326)::geography)"#,
    )
    .execute(&pool)
    .await
    .unwrap();
    sqlx::query(
        "INSERT INTO playgrounds (id, location, source_url) VALUES ('openstreetmap/node/1', ST_SetSRID(ST_MakePoint(23.3444, 42.7070), 4326)::geography, 'https://www.openstreetmap.org/node/1')",
    )
    .execute(&pool)
    .await
    .unwrap();
    sqlx::query(
        "INSERT INTO playground_source_links (playground_id, source, external_id, match_method) VALUES ('openstreetmap/node/1', 'openstreetmap', 'node/1', 'unmatched')",
    )
    .execute(&pool)
    .await
    .unwrap();
    sqlx::query("DELETE FROM playgrounds WHERE id = 'openstreetmap/node/1'")
        .execute(&pool)
        .await
        .unwrap();
    let (playground_links,): (i64,) = sqlx::query_as(
        "SELECT count(*)::bigint FROM playground_source_links WHERE playground_id = 'openstreetmap/node/1'",
    )
    .fetch_one(&pool)
    .await
    .unwrap();
    assert_eq!(playground_links, 0);
}

#[sqlx::test(migrations = "./migrations")]
async fn graphql_queries_use_postgis_and_combine_catalog_filters(pool: PgPool) {
    seed(&pool).await;
    let app = graphql::router(pool.clone(), "http://127.0.0.1:5173").unwrap();

    let default_limit = graphql(&app, "{ playgrounds { id } }").await;
    assert_eq!(
        default_limit["data"]["playgrounds"]
            .as_array()
            .unwrap()
            .len(),
        200
    );

    let bounds = graphql(
        &app,
        r#"{
          playgrounds(
            filter: { bounds: { southWest: { longitude: -0.01, latitude: -0.01 }, northEast: { longitude: 0.01, latitude: 0.01 } } }
            limit: 2
          ) { id distanceMeters }
        }"#,
    )
    .await;
    assert_eq!(
        bounds["data"]["playgrounds"],
        json!([
            { "id": "graphql-test/a", "distanceMeters": null },
            { "id": "graphql-test/b", "distanceMeters": null }
        ])
    );

    let second_page = graphql(
        &app,
        r#"{
          playgrounds(
            filter: { bounds: { southWest: { longitude: -0.01, latitude: -0.01 }, northEast: { longitude: 0.01, latitude: 0.01 } } }
            limit: 2
            offset: 2
          ) { id }
        }"#,
    )
    .await;
    assert_eq!(
        second_page["data"]["playgrounds"],
        json!([{ "id": "graphql-test/c" }, { "id": "graphql-test/d" }])
    );

    let radius = graphql(
        &app,
        r#"{
          playgrounds(filter: { center: { longitude: 0, latitude: 0 }, radiusMeters: 250 }) {
            id distanceMeters
          }
        }"#,
    )
    .await;
    let radius_rows = radius["data"]["playgrounds"].as_array().unwrap();
    assert_eq!(
        radius_rows
            .iter()
            .map(|row| row["id"].as_str().unwrap())
            .collect::<Vec<_>>(),
        ["graphql-test/a", "graphql-test/b", "graphql-test/d"]
    );
    assert!(
        radius_rows
            .iter()
            .all(|row| row["distanceMeters"].is_number())
    );

    let nested = graphql(
        &app,
        r#"{
          playgrounds(filter: { neighborhoodId: "graphql-test/child" }) {
            id neighborhoods { id name }
          }
        }"#,
    )
    .await;
    assert_eq!(nested["data"]["playgrounds"].as_array().unwrap().len(), 1);
    assert_eq!(
        nested["data"]["playgrounds"][0]["neighborhoods"]
            .as_array()
            .unwrap()
            .len(),
        2
    );

    let combined = graphql(
        &app,
        r#"{
          playgrounds(filter: {
            bounds: { southWest: { longitude: -0.01, latitude: -0.01 }, northEast: { longitude: 0.01, latitude: 0.01 } }
            neighborhoodId: "graphql-test/parent"
            childAge: 4
            requiredCapabilities: [SWING, SLIDE]
          }) { id }
        }"#,
    )
    .await;
    assert_eq!(
        combined["data"]["playgrounds"],
        json!([{ "id": "graphql-test/a" }])
    );

    let equipment = graphql(
        &app,
        r#"{
          playgrounds(filter: { neighborhoodId: "graphql-test/parent" }) {
            id equipment { capability count }
          }
        }"#,
    )
    .await;
    assert_eq!(
        equipment["data"]["playgrounds"][0]["equipment"],
        json!([
            { "capability": "SLIDE", "count": 1 },
            { "capability": "SWING", "count": 2 }
        ])
    );
    assert_eq!(
        equipment["data"]["playgrounds"][1]["equipment"],
        json!([{ "capability": "SWING", "count": null }])
    );
    assert_eq!(equipment["data"]["playgrounds"][2]["equipment"], json!([]));

    let one_sided_and_unknown = graphql(
        &app,
        r#"{
          ageThree: playgrounds(filter: { neighborhoodId: "graphql-test/parent", childAge: 3 }) { id }
          ageEight: playgrounds(filter: { neighborhoodId: "graphql-test/parent", childAge: 8 }) { id }
          ageFour: playgrounds(filter: { neighborhoodId: "graphql-test/parent", childAge: 4 }) { id }
          ageEighteen: playgrounds(filter: { neighborhoodId: "graphql-test/parent", childAge: 18 }) { id }
          slide: playgrounds(filter: { neighborhoodId: "graphql-test/parent", requiredCapabilities: [SLIDE] }) { id }
        }"#,
    )
    .await;
    assert_eq!(
        one_sided_and_unknown["data"]["ageThree"],
        json!([{ "id": "graphql-test/a" }, { "id": "graphql-test/b" }])
    );
    assert_eq!(
        one_sided_and_unknown["data"]["ageEight"],
        json!([{ "id": "graphql-test/a" }])
    );
    assert_eq!(
        one_sided_and_unknown["data"]["ageFour"],
        json!([{ "id": "graphql-test/a" }, { "id": "graphql-test/b" }])
    );
    assert_eq!(
        one_sided_and_unknown["data"]["ageEighteen"],
        json!([{ "id": "graphql-test/d" }])
    );
    assert_eq!(
        one_sided_and_unknown["data"]["slide"],
        json!([{ "id": "graphql-test/a" }])
    );

    let detail = graphql(
        &app,
        r#"{
          found: playground(id: "graphql-test/b") {
            id name minAge maxAge photoUrls capabilities equipment { capability count }
            location { longitude latitude }
            source { id url updatedAt attribution license }
          }
          missing: playground(id: "graphql-test/missing") { id }
        }"#,
    )
    .await;
    assert_eq!(detail["data"]["found"]["name"], Value::Null);
    assert_eq!(detail["data"]["found"]["minAge"], Value::Null);
    assert_eq!(detail["data"]["found"]["photoUrls"], json!([]));
    assert_eq!(
        detail["data"]["found"]["equipment"],
        json!([{ "capability": "SWING", "count": null }])
    );
    assert_eq!(detail["data"]["found"]["source"]["id"], "graphql-test/b");
    assert_eq!(detail["data"]["found"]["source"]["updatedAt"], Value::Null);
    assert_eq!(
        detail["data"]["found"]["source"]["attribution"],
        "© OpenStreetMap contributors"
    );
    assert_eq!(detail["data"]["found"]["source"]["license"], "ODbL-1.0");
    assert_eq!(detail["data"]["missing"], Value::Null);

    clean(&pool).await;
}
