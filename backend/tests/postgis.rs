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
    sqlx::query(
        r#"INSERT INTO playgrounds
          (id, name, location, capabilities, photo_urls, source_url, source_updated_at,
           address, surface, fenced, ownership, access, fee, municipal_status,
           ordinance_compliant, repairs, notes, primary_source, photos, source_values)
        VALUES ('graphql-test/enriched', 'Enriched', ST_SetSRID(ST_MakePoint(2, 2), 4326)::geography,
          ARRAY[]::text[], ARRAY['https://example.test/photo.jpg'],
          'https://www.openstreetmap.org/node/42', '2018-01-01T00:00:00Z',
          'Park entrance', 'rubber', false, 'municipal', 'public', 'free', 'planned',
          false, 'Replace slide', 'Inspection pending', 'openstreetmap',
          '[{"url":"https://example.test/photo.jpg","author":"A Person","license":"CC BY 4.0","license_url":"https://creativecommons.org/licenses/by/4.0/","attribution":"A Person / CC BY 4.0","source_url":"https://commons.wikimedia.org/wiki/File:Photo.jpg"}]'::jsonb,
          '[{"field":"fenced","value":true,"source":"open_street_map","source_id":"node/42","date":"2018-01-01T00:00:00Z","date_meaning":"source_update","selected":false},
            {"field":"fenced","value":false,"source":"sofia_plan","source_id":"06.129","date":"2019-04-18T00:00:00Z","date_meaning":"observation","selected":true}]'::jsonb)"#,
    )
    .execute(pool)
    .await
    .unwrap();
    sqlx::query(
        r#"INSERT INTO source_playgrounds (source, external_id, raw_data, location, source_date, date_meaning) VALUES
          ('sofiaplan', '06.129', '{}'::jsonb, ST_SetSRID(ST_MakePoint(2, 2), 4326)::geography, '2019-04-18T00:00:00Z', 'observation'),
          ('openstreetmap', 'node/42', '{}'::jsonb, ST_SetSRID(ST_MakePoint(2, 2), 4326)::geography, '2018-01-01T00:00:00Z', 'source_update')"#,
    )
    .execute(pool)
    .await
    .unwrap();
    sqlx::query(
        r#"INSERT INTO playground_source_links (playground_id, source, external_id, match_method) VALUES
          ('graphql-test/enriched', 'sofiaplan', '06.129', 'proximity'),
          ('graphql-test/enriched', 'openstreetmap', 'node/42', 'proximity')"#,
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

    sqlx::query(
        "DELETE FROM source_playgrounds WHERE source = 'sofiaplan' AND external_id = '06.129'",
    )
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
async fn graphql_keeps_unknown_equipment_count_available_in_detail_and_list(pool: PgPool) {
    sqlx::query(
        r#"INSERT INTO playgrounds
           (id, location, capabilities, source_url, source_values)
           VALUES ('graphql-test/unknown-count', ST_SetSRID(ST_MakePoint(2, 2), 4326)::geography,
             ARRAY['swing'], 'https://www.openstreetmap.org/node/43',
             '[{"field":"equipment.swing","value":null,"source":"open_street_map",
               "source_id":"node/43","date":null,"date_meaning":null,"selected":true}]'::jsonb)"#,
    )
    .execute(&pool)
    .await
    .unwrap();
    let app = graphql::router(pool.clone(), "http://127.0.0.1:5173").unwrap();

    let detail = graphql(
        &app,
        r#"{ playground(id: "graphql-test/unknown-count") {
          id equipment { capability count }
          sourceValues { field value source sourceId selected }
        } }"#,
    )
    .await;
    assert_eq!(detail["errors"], Value::Null, "{detail}");
    assert_eq!(
        detail["data"]["playground"],
        json!({
            "id": "graphql-test/unknown-count",
            "equipment": [{"capability":"SWING", "count":null}],
            "sourceValues": [{"field":"equipment.swing", "value":"null",
                "source":"OPEN_STREET_MAP", "sourceId":"node/43", "selected":true}]
        })
    );

    let list = graphql(
        &app,
        r#"{ playgrounds(filter: { bounds: {
          southWest: { longitude: 1, latitude: 1 },
          northEast: { longitude: 3, latitude: 3 }
        } }) { id } }"#,
    )
    .await;
    assert_eq!(list["errors"], Value::Null, "{list}");
    assert_eq!(
        list["data"]["playgrounds"],
        json!([{"id":"graphql-test/unknown-count"}])
    );

    sqlx::query("UPDATE playgrounds SET source_values = jsonb_set(source_values, '{0,value}', '[]'::jsonb) WHERE id = 'graphql-test/unknown-count'")
        .execute(&pool).await.unwrap();
    let malformed = graphql(
        &app,
        r#"{ playground(id: "graphql-test/unknown-count") { id } }"#,
    )
    .await;
    assert_eq!(
        malformed["errors"][0]["message"],
        "playground data is temporarily unavailable"
    );
    sqlx::query("UPDATE playgrounds SET source_values = jsonb_set(source_values, '{0,value}', '{}'::jsonb) WHERE id = 'graphql-test/unknown-count'")
        .execute(&pool).await.unwrap();
    let malformed = graphql(
        &app,
        r#"{ playground(id: "graphql-test/unknown-count") { id } }"#,
    )
    .await;
    assert_eq!(
        malformed["errors"][0]["message"],
        "playground data is temporarily unavailable"
    );
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

    let enriched = graphql(
        &app,
        r#"query {
          playground(id: "graphql-test/enriched") {
            id name address surface fenced ownership access fee
            municipalStatus ordinanceCompliant repairs notes
            neighborhoods { id name }
            photos { url author license licenseUrl attribution sourceUrl }
            photoUrls
            source { id kind url updatedAt dateMeaning attribution license }
            sources { id kind url updatedAt dateMeaning attribution license }
            sourceValues { field value source sourceId date dateMeaning selected }
          }
        }"#,
    )
    .await;
    assert_eq!(enriched["errors"], Value::Null, "{enriched}");
    assert_eq!(
        enriched["data"]["playground"],
        json!({
            "id": "graphql-test/enriched", "name": "Enriched", "address": "Park entrance",
            "surface": "rubber", "fenced": false, "ownership": "municipal", "access": "public",
            "fee": "free", "municipalStatus": "planned", "ordinanceCompliant": false,
            "repairs": "Replace slide", "notes": "Inspection pending", "neighborhoods": [],
            "photos": [{"url":"https://example.test/photo.jpg","author":"A Person","license":"CC BY 4.0",
                "licenseUrl":"https://creativecommons.org/licenses/by/4.0/","attribution":"A Person / CC BY 4.0",
                "sourceUrl":"https://commons.wikimedia.org/wiki/File:Photo.jpg"}],
            "photoUrls": ["https://example.test/photo.jpg"],
            "source": {"id":"openstreetmap/node/42","kind":"OPEN_STREET_MAP","url":"https://www.openstreetmap.org/node/42",
                "updatedAt":"2018-01-01T00:00:00+00:00","dateMeaning":"SOURCE_UPDATE","attribution":"© OpenStreetMap contributors","license":"ODbL-1.0"},
            "sources": [
                {"id":"openstreetmap/node/42","kind":"OPEN_STREET_MAP","url":"https://www.openstreetmap.org/node/42",
                    "updatedAt":"2018-01-01T00:00:00+00:00","dateMeaning":"SOURCE_UPDATE","attribution":"© OpenStreetMap contributors","license":"ODbL-1.0"},
                {"id":"sofiaplan/06.129","kind":"SOFIA_PLAN","url":"https://urbandata.sofia.bg/dataset/playgrounds",
                    "updatedAt":"2019-04-18T00:00:00+00:00","dateMeaning":"OBSERVATION","attribution":"SofiaPlan","license":"Reuse terms need confirmation"}
            ],
            "sourceValues": [
                {"field":"fenced","value":"true","source":"OPEN_STREET_MAP","sourceId":"node/42",
                    "date":"2018-01-01T00:00:00+00:00","dateMeaning":"SOURCE_UPDATE","selected":false},
                {"field":"fenced","value":"false","source":"SOFIA_PLAN","sourceId":"06.129",
                    "date":"2019-04-18T00:00:00+00:00","dateMeaning":"OBSERVATION","selected":true}
            ]
        })
    );

    sqlx::query("DELETE FROM playground_source_links WHERE playground_id = 'graphql-test/enriched' AND source = 'openstreetmap'")
        .execute(&pool).await.unwrap();
    let sofia_only = graphql(&app, r#"{ playground(id: "graphql-test/enriched") { source { kind url dateMeaning license } sources { kind } } }"#).await;
    assert_eq!(
        sofia_only["data"]["playground"],
        json!({
            "source": {"kind":"SOFIA_PLAN", "url":"https://urbandata.sofia.bg/dataset/playgrounds",
                "dateMeaning":"OBSERVATION", "license":"Reuse terms need confirmation"},
            "sources": [{"kind":"SOFIA_PLAN"}]
        })
    );

    sqlx::query("UPDATE playgrounds SET source_values = '[{\"field\":\"fenced\",\"value\":\"secret-raw-json\"}]'::jsonb WHERE id = 'graphql-test/enriched'")
        .execute(&pool).await.unwrap();
    let malformed = graphql(
        &app,
        r#"{ playground(id: "graphql-test/enriched") { sourceValues { value } } }"#,
    )
    .await;
    assert_eq!(
        malformed["errors"][0]["message"],
        "playground data is temporarily unavailable"
    );
    assert!(!malformed.to_string().contains("secret-raw-json"));

    clean(&pool).await;
}
