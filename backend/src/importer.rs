use std::{
    collections::{BTreeMap, BTreeSet},
    fs,
    path::Path,
};

use anyhow::{Context, Result, anyhow, bail};
use chrono::{DateTime, Utc};
use geo::{Contains, InteriorPoint, Intersects, LineString, MultiPolygon, Point, Polygon};
use serde::Deserialize;
use serde_json::{Value, json};
use sqlx::PgPool;

use crate::commons::{LicensedPhoto, resolve_commons};
use crate::enrichment::{
    CanonicalPlayground, DateMeaning, SourceKind, SourceLink, SourcePlayground, match_sources,
    merge_catalog, normalize_sofiaplan,
};

pub const DEFAULT_OVERPASS_URL: &str = "https://overpass-api.de/api/interpreter";
pub const DEFAULT_NEIGHBORHOODS_PATH: &str = "../public/data/sofia-neighborhoods.geojson";

const OVERPASS_QUERY: &str = r#"[out:json][timeout:90];
area(3604283101)->.sofia;
(
  nwr["leisure"="playground"](area.sofia);
  nwr["playground"](area.sofia);
);
out meta center geom;"#;

const CAPABILITIES: &[&str] = &[
    "climbing_frame",
    "playhouse",
    "roundabout",
    "sandpit",
    "seesaw",
    "slide",
    "springy",
    "swing",
];

#[derive(Debug, Clone)]
pub struct Neighborhood {
    pub id: String,
    pub name: String,
    geometry_json: String,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ImportCounts {
    pub osm_source_records: u64,
    pub sofia_source_records: u64,
    pub canonical_playgrounds: u64,
    pub clear_matches: u64,
    pub ambiguous_records: u64,
    pub excluded_source_records: u64,
    pub accepted_photos: u64,
    pub rejected_photos: u64,
    pub neighborhoods: u64,
    pub memberships: u64,
}

pub struct ImportEndpoints {
    pub overpass_url: String,
    pub sofiaplan_url: String,
    pub commons_api_url: String,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct SnapshotCounts {
    pub source_records: u64,
    pub neighborhoods: u64,
    pub playgrounds: u64,
    pub memberships: u64,
}

#[derive(Debug, Deserialize)]
struct OverpassResponse {
    elements: Vec<Value>,
}

#[derive(Debug, Deserialize)]
struct Element {
    #[serde(rename = "type")]
    kind: String,
    id: i64,
    lat: Option<f64>,
    lon: Option<f64>,
    center: Option<RawPoint>,
    bounds: Option<Bounds>,
    geometry: Option<Vec<RawPoint>>,
    members: Option<Vec<Member>>,
    #[serde(default)]
    tags: serde_json::Map<String, Value>,
    timestamp: Option<String>,
}

#[derive(Debug, Clone, Copy, Deserialize)]
struct RawPoint {
    lat: f64,
    lon: f64,
}

#[derive(Debug, Clone, Copy, Deserialize)]
struct Bounds {
    minlat: f64,
    minlon: f64,
    maxlat: f64,
    maxlon: f64,
}

impl Bounds {
    fn center(self) -> RawPoint {
        RawPoint {
            lat: (self.minlat + self.maxlat) / 2.0,
            lon: (self.minlon + self.maxlon) / 2.0,
        }
    }
}

#[derive(Debug, Deserialize)]
struct Member {
    #[serde(rename = "type")]
    kind: String,
    #[serde(default)]
    role: String,
    geometry: Option<Vec<RawPoint>>,
}

struct Root {
    playground: SourcePlayground,
    area: Option<MultiPolygon<f64>>,
    raw_root: Value,
    raw_equipment: Vec<Value>,
}

pub async fn run_import(
    pool: &PgPool,
    endpoints: &ImportEndpoints,
    neighborhoods_path: &Path,
) -> Result<ImportCounts> {
    let client = reqwest::Client::new();
    let osm = normalize_overpass(&fetch_overpass(&client, &endpoints.overpass_url).await?)?;
    let sofia_body = client
        .get(&endpoints.sofiaplan_url)
        .send()
        .await
        .with_context(|| format!("request SofiaPlan endpoint {}", endpoints.sofiaplan_url))?
        .error_for_status()
        .context("SofiaPlan returned an HTTP error")?
        .text()
        .await
        .context("read SofiaPlan response")?;
    let sofia = normalize_sofiaplan(&sofia_body)?;
    let neighborhoods = load_neighborhoods(neighborhoods_path)?;

    let titles = osm
        .iter()
        .flat_map(|record| record.commons_titles.iter().cloned())
        .collect::<Vec<_>>();
    let (photos, rejected_photos) =
        match resolve_commons(&client, &endpoints.commons_api_url, &titles).await {
            Ok(result) => (
                result
                    .accepted
                    .into_iter()
                    .map(|resolved| (resolved.requested_title, resolved.photo))
                    .collect::<BTreeMap<String, LicensedPhoto>>(),
                result.rejected as u64,
            ),
            Err(error) => {
                eprintln!(
                    "Warning: Commons photo resolution failed for {} referenced photos: {error}",
                    titles.len()
                );
                (BTreeMap::new(), titles.len() as u64)
            }
        };
    let mut osm = osm;
    let mut accepted_photos = 0;
    for record in &mut osm {
        for title in &record.commons_titles {
            if let Some(photo) = photos.get(title) {
                record.photos.push(photo.clone());
                accepted_photos += 1;
            }
        }
    }

    let matches = match_sources(&osm, &sofia);
    let catalog = merge_catalog(&osm, &sofia, &matches);
    let excluded_source_records = osm
        .iter()
        .chain(&sofia)
        .filter(|record| record.excluded_from_catalog)
        .count() as u64;
    let osm_source_records = osm.len() as u64;
    let sofia_source_records = sofia.len() as u64;
    let mut source_records = osm;
    source_records.extend(sofia);
    let counts = replace_snapshot(
        pool,
        &neighborhoods,
        &source_records,
        &catalog.playgrounds,
        &catalog.source_links,
    )
    .await?;
    Ok(ImportCounts {
        osm_source_records,
        sofia_source_records,
        canonical_playgrounds: counts.playgrounds,
        clear_matches: matches.clear_pairs.len() as u64,
        ambiguous_records: matches.ambiguous_source_ids.len() as u64,
        excluded_source_records,
        accepted_photos,
        rejected_photos,
        neighborhoods: counts.neighborhoods,
        memberships: counts.memberships,
    })
}

async fn fetch_overpass(client: &reqwest::Client, url: &str) -> Result<String> {
    let response = client
        .post(url)
        .header("content-type", "application/x-www-form-urlencoded")
        .header("user-agent", "SofiaPlaygrounds/0.1 (local import)")
        .body(format!("data={}", form_encode(OVERPASS_QUERY)))
        .send()
        .await
        .with_context(|| format!("request Overpass endpoint {url}"))?
        .error_for_status()
        .context("Overpass returned an HTTP error")?;

    response.text().await.context("read Overpass response")
}

pub fn normalize_overpass(body: &str) -> Result<Vec<SourcePlayground>> {
    let value: Value = serde_json::from_str(body).context("Overpass response is not valid JSON")?;
    let object = value
        .as_object()
        .ok_or_else(|| anyhow!("Overpass response must be a JSON object"))?;
    if object.contains_key("error") || object.contains_key("remark") {
        bail!("Overpass response contains an error or remark marker");
    }

    let response: OverpassResponse =
        serde_json::from_value(value).context("Overpass response has invalid elements")?;
    let mut roots = Vec::new();
    let mut equipment = Vec::new();

    for raw_element in response.elements {
        let element: Element = serde_json::from_value(raw_element.clone())
            .context("Overpass response has an invalid element")?;
        let leisure = tag(&element, "leisure");
        let equipment_kind = tag(&element, "playground").and_then(normalize_capability);

        if leisure == Some("playground") {
            roots.push(normalize_root(&element, raw_element)?);
        } else if let Some(capability) = equipment_kind {
            equipment.push((element_point(&element)?, capability, raw_element));
        }
    }

    if roots.is_empty() {
        bail!("Overpass response contains no Sofia playgrounds");
    }

    for (point, capability, raw_equipment) in equipment {
        for root in &mut roots {
            if root
                .area
                .as_ref()
                .is_some_and(|area| area.intersects(&point))
            {
                let count = root
                    .playground
                    .equipment
                    .entry(capability.to_owned())
                    .or_insert(None);
                *count = Some(count.unwrap_or(0) + 1);
                root.raw_equipment.push(raw_equipment.clone());
            }
        }
    }

    let mut playgrounds = Vec::with_capacity(roots.len());
    for mut root in roots {
        root.playground.raw_data = json!({
            "root": root.raw_root,
            "equipment": root.raw_equipment,
        });
        playgrounds.push(root.playground);
    }
    playgrounds.sort_by(|left, right| left.external_id.cmp(&right.external_id));

    let mut ids = BTreeSet::new();
    if playgrounds
        .iter()
        .any(|playground| !ids.insert(&playground.external_id))
    {
        bail!("Overpass response contains duplicate playground identifiers");
    }
    Ok(playgrounds)
}

pub fn load_neighborhoods(path: &Path) -> Result<Vec<Neighborhood>> {
    let contents = fs::read_to_string(path)
        .with_context(|| format!("read neighborhood GeoJSON at {}", path.display()))?;
    let value: Value =
        serde_json::from_str(&contents).context("neighborhood file is invalid JSON")?;
    if value.get("type").and_then(Value::as_str) != Some("FeatureCollection") {
        bail!("neighborhood GeoJSON must be a FeatureCollection");
    }
    let features = value
        .get("features")
        .and_then(Value::as_array)
        .ok_or_else(|| anyhow!("neighborhood GeoJSON must contain a features array"))?;

    let mut neighborhoods = Vec::with_capacity(features.len());
    let mut ids = BTreeSet::new();
    for feature in features {
        let properties = feature
            .get("properties")
            .and_then(Value::as_object)
            .ok_or_else(|| anyhow!("neighborhood feature is missing properties"))?;
        let id = properties
            .get("id")
            .and_then(Value::as_str)
            .filter(|value| !value.is_empty())
            .ok_or_else(|| anyhow!("neighborhood feature is missing id"))?;
        let name = properties
            .get("name")
            .and_then(Value::as_str)
            .filter(|value| !value.is_empty())
            .ok_or_else(|| anyhow!("neighborhood feature {id} is missing name"))?;
        let geometry = feature
            .get("geometry")
            .ok_or_else(|| anyhow!("neighborhood feature {id} is missing geometry"))?;
        match geometry.get("type").and_then(Value::as_str) {
            Some("Polygon" | "MultiPolygon") => {}
            _ => bail!("neighborhood feature {id} must be Polygon or MultiPolygon"),
        }
        if !ids.insert(id.to_owned()) {
            bail!("duplicate neighborhood identifier {id}");
        }
        neighborhoods.push(Neighborhood {
            id: id.to_owned(),
            name: name.to_owned(),
            geometry_json: serde_json::to_string(geometry)?,
        });
    }
    Ok(neighborhoods)
}

pub async fn replace_snapshot(
    pool: &PgPool,
    neighborhoods: &[Neighborhood],
    source_records: &[SourcePlayground],
    playgrounds: &[CanonicalPlayground],
    source_links: &[SourceLink],
) -> Result<SnapshotCounts> {
    let mut transaction = pool.begin().await.context("begin import transaction")?;
    sqlx::query(
        "CREATE TEMP TABLE import_neighborhoods (LIKE neighborhoods INCLUDING ALL) ON COMMIT DROP",
    )
    .execute(&mut *transaction)
    .await
    .context("create neighborhood import staging table")?;
    sqlx::query(
        "CREATE TEMP TABLE import_playgrounds (LIKE playgrounds INCLUDING ALL) ON COMMIT DROP",
    )
    .execute(&mut *transaction)
    .await
    .context("create playground import staging table")?;
    sqlx::query(
        "CREATE TEMP TABLE import_source_playgrounds (LIKE source_playgrounds INCLUDING ALL) ON COMMIT DROP",
    )
    .execute(&mut *transaction)
    .await
    .context("create source playground import staging table")?;
    sqlx::query(
        "CREATE TEMP TABLE import_playground_source_links (LIKE playground_source_links INCLUDING ALL) ON COMMIT DROP",
    )
    .execute(&mut *transaction)
    .await
    .context("create source link import staging table")?;

    for neighborhood in neighborhoods {
        sqlx::query(
            "INSERT INTO import_neighborhoods (id, name, boundary) VALUES ($1, $2, ST_Multi(ST_SetSRID(ST_GeomFromGeoJSON($3), 4326)))",
        )
        .bind(&neighborhood.id)
        .bind(&neighborhood.name)
        .bind(&neighborhood.geometry_json)
        .execute(&mut *transaction)
        .await
        .with_context(|| format!("stage neighborhood {}", neighborhood.id))?;
    }

    for source_record in source_records {
        sqlx::query(
            "INSERT INTO import_source_playgrounds (source, external_id, raw_data, normalized_name, location, source_date, date_meaning) VALUES ($1, $2, $3, $4, ST_SetSRID(ST_MakePoint($5, $6), 4326)::geography, $7, $8)",
        )
        .bind(source_kind(source_record.source))
        .bind(&source_record.external_id)
        .bind(serde_json::to_value(&source_record.raw_data)?)
        .bind(&source_record.name)
        .bind(source_record.longitude)
        .bind(source_record.latitude)
        .bind(source_record.source_date)
        .bind(source_record.date_meaning.map(date_meaning))
        .execute(&mut *transaction)
        .await
        .with_context(|| format!("stage source playground {}", source_record.external_id))?;
    }

    for playground in playgrounds {
        sqlx::query(
            "INSERT INTO import_playgrounds (id, name, location, capabilities, equipment_counts, min_age, max_age, address, surface, fenced, ownership, access, fee, municipal_status, ordinance_compliant, repairs, notes, primary_source, photo_urls, photos, source_values, source_url, source_updated_at) VALUES ($1, $2, ST_SetSRID(ST_MakePoint($3, $4), 4326)::geography, $5, $6, $7, $8, $9, $10, $11, $12, $13, $14, $15, $16, $17, $18, $19, $20, $21, $22, $23, $24)",
        )
        .bind(&playground.id)
        .bind(&playground.name)
        .bind(playground.longitude)
        .bind(playground.latitude)
        .bind(&playground.capabilities)
        .bind(serde_json::to_value(&playground.equipment_counts)?)
        .bind(playground.min_age)
        .bind(playground.max_age)
        .bind(&playground.address)
        .bind(&playground.surface)
        .bind(playground.fenced)
        .bind(&playground.ownership)
        .bind(&playground.access)
        .bind(&playground.fee)
        .bind(&playground.municipal_status)
        .bind(playground.ordinance_compliant)
        .bind(&playground.repairs)
        .bind(&playground.notes)
        .bind(source_kind(playground.primary_source))
        .bind(playground.photos.iter().map(|photo| photo.url.clone()).collect::<Vec<_>>())
        .bind(serde_json::to_value(&playground.photos)?)
        .bind(serde_json::to_value(&playground.source_values)?)
        .bind(&playground.source_url)
        .bind(playground.source_updated_at)
        .execute(&mut *transaction)
        .await
        .with_context(|| format!("stage playground {}", playground.id))?;
    }

    for source_link in source_links {
        sqlx::query(
            "INSERT INTO import_playground_source_links (playground_id, source, external_id, match_method, match_distance_meters) VALUES ($1, $2, $3, $4, $5)",
        )
        .bind(&source_link.playground_id)
        .bind(source_kind(source_link.source))
        .bind(&source_link.external_id)
        .bind(source_link.match_method)
        .bind(source_link.match_distance_meters)
        .execute(&mut *transaction)
        .await
        .with_context(|| format!("stage source link {}", source_link.external_id))?;
    }

    sqlx::query("DELETE FROM playground_neighborhoods")
        .execute(&mut *transaction)
        .await?;
    sqlx::query("DELETE FROM playground_source_links")
        .execute(&mut *transaction)
        .await?;
    sqlx::query("DELETE FROM playgrounds")
        .execute(&mut *transaction)
        .await?;
    sqlx::query("DELETE FROM source_playgrounds")
        .execute(&mut *transaction)
        .await?;
    sqlx::query("DELETE FROM neighborhoods")
        .execute(&mut *transaction)
        .await?;
    sqlx::query("INSERT INTO neighborhoods SELECT * FROM import_neighborhoods")
        .execute(&mut *transaction)
        .await?;
    sqlx::query("INSERT INTO source_playgrounds SELECT * FROM import_source_playgrounds")
        .execute(&mut *transaction)
        .await?;
    sqlx::query("INSERT INTO playgrounds SELECT * FROM import_playgrounds")
        .execute(&mut *transaction)
        .await?;
    sqlx::query("INSERT INTO playground_source_links SELECT * FROM import_playground_source_links")
        .execute(&mut *transaction)
        .await?;
    let memberships = sqlx::query(
        "INSERT INTO playground_neighborhoods (playground_id, neighborhood_id) SELECT p.id, n.id FROM playgrounds p CROSS JOIN neighborhoods n WHERE ST_Covers(n.boundary, p.location::geometry)",
    )
    .execute(&mut *transaction)
    .await
    .context("assign playground neighborhood memberships")?
    .rows_affected();

    transaction
        .commit()
        .await
        .context("commit imported snapshot")?;
    Ok(SnapshotCounts {
        source_records: source_records.len() as u64,
        neighborhoods: neighborhoods.len() as u64,
        playgrounds: playgrounds.len() as u64,
        memberships,
    })
}

fn normalize_root(element: &Element, raw_root: Value) -> Result<Root> {
    if element.id < 0 || !matches!(element.kind.as_str(), "node" | "way" | "relation") {
        bail!("playground has invalid OpenStreetMap type or identifier");
    }
    let area = element_area(element)?;
    let point = match &area {
        Some(area) => area
            .interior_point()
            .ok_or_else(|| anyhow!("{}/{} has no interior point", element.kind, element.id))?,
        None => element_point(element)?,
    };
    validate_point(point)?;

    let external_id = format!("{}/{}", element.kind, element.id);
    let mut equipment = CAPABILITIES
        .iter()
        .filter(|capability| {
            tag(element, &format!("playground:{capability}")).is_some_and(is_truthy)
                || (**capability == "climbing_frame"
                    && tag(element, "playground:climbingframe").is_some_and(is_truthy))
        })
        .map(|capability| ((*capability).to_owned(), None))
        .collect::<BTreeMap<_, _>>();
    if let Some(capability) = tag(element, "playground").and_then(normalize_capability) {
        equipment.insert(capability.to_owned(), None);
    }

    let mut min_age = tag(element, "min_age").and_then(parse_age);
    let mut max_age = tag(element, "max_age").and_then(parse_age);
    if min_age.zip(max_age).is_some_and(|(min, max)| min > max) {
        min_age = None;
        max_age = None;
    }
    let mut values = BTreeMap::new();
    for field in ["surface", "access", "fee"] {
        if let Some(value) = tag(element, field)
            .map(str::trim)
            .filter(|value| !value.is_empty())
        {
            values.insert(field.to_owned(), Value::String(value.to_owned()));
        }
    }
    if let Some(min_age) = min_age {
        values.insert("min_age".into(), Value::from(min_age));
    } else if let Some(value) = tag(element, "min_age") {
        warn_invalid_osm_value(&external_id, "min_age", value);
    }
    if let Some(max_age) = max_age {
        values.insert("max_age".into(), Value::from(max_age));
    } else if let Some(value) = tag(element, "max_age") {
        warn_invalid_osm_value(&external_id, "max_age", value);
    }

    let timestamp = element
        .timestamp
        .as_deref()
        .or_else(|| tag(element, "timestamp"));
    let source_date = timestamp.and_then(|value| {
        DateTime::parse_from_rfc3339(value)
            .ok()
            .map(|parsed| parsed.with_timezone(&Utc))
    });
    if source_date.is_none()
        && let Some(timestamp) = timestamp
    {
        warn_invalid_osm_value(&external_id, "source_date", timestamp);
    }

    Ok(Root {
        playground: SourcePlayground {
            source: SourceKind::OpenStreetMap,
            external_id,
            raw_data: Value::Null,
            name: tag(element, "name")
                .map(str::trim)
                .filter(|name| !name.is_empty())
                .map(str::to_owned),
            longitude: point.x(),
            latitude: point.y(),
            source_date,
            date_meaning: source_date.map(|_| DateMeaning::SourceUpdate),
            values,
            equipment,
            commons_titles: tag(element, "wikimedia_commons")
                .into_iter()
                .flat_map(|titles| titles.split(';'))
                .map(str::trim)
                .filter(|title| title.starts_with("File:"))
                .map(str::to_owned)
                .collect(),
            photos: Vec::new(),
            excluded_from_catalog: false,
        },
        area,
        raw_root,
        raw_equipment: Vec::new(),
    })
}

fn source_kind(source: SourceKind) -> &'static str {
    match source {
        SourceKind::OpenStreetMap => "openstreetmap",
        SourceKind::SofiaPlan => "sofiaplan",
    }
}

fn date_meaning(meaning: DateMeaning) -> &'static str {
    match meaning {
        DateMeaning::Observation => "observation",
        DateMeaning::SourceUpdate => "source_update",
    }
}

fn warn_invalid_osm_value(external_id: &str, field: &str, value: &str) {
    tracing::warn!(
        source = "openstreetmap",
        external_id,
        field,
        rejected_value = value,
        "reject invalid OpenStreetMap value"
    );
}

fn tag<'a>(element: &'a Element, name: &str) -> Option<&'a str> {
    element.tags.get(name).and_then(Value::as_str)
}

fn parse_age(value: &str) -> Option<i16> {
    value
        .trim()
        .parse()
        .ok()
        .filter(|age| (0..=18).contains(age))
}

fn normalize_capability(value: &str) -> Option<&'static str> {
    let normalized = value.trim().to_ascii_lowercase().replace([' ', '-'], "_");
    if normalized == "climbingframe" {
        return Some("climbing_frame");
    }
    CAPABILITIES
        .iter()
        .copied()
        .find(|capability| *capability == normalized)
}

fn is_truthy(value: &str) -> bool {
    matches!(
        value.trim().to_ascii_lowercase().as_str(),
        "yes" | "true" | "1"
    )
}

fn element_point(element: &Element) -> Result<Point<f64>> {
    let raw = match element.kind.as_str() {
        "node" => match (element.lon, element.lat) {
            (Some(lon), Some(lat)) => RawPoint { lon, lat },
            _ => bail!("node/{} is missing coordinates", element.id),
        },
        // Overpass omits `center` when `out ... geom` is requested; its `bounds`
        // center is the same point, so use it as the fallback.
        "way" | "relation" => element
            .center
            .or_else(|| element.bounds.map(Bounds::center))
            .ok_or_else(|| {
                anyhow!(
                    "{}/{} is missing center and bounds",
                    element.kind,
                    element.id
                )
            })?,
        _ => bail!("unsupported OpenStreetMap element type {}", element.kind),
    };
    let point = Point::new(raw.lon, raw.lat);
    validate_point(point)?;
    Ok(point)
}

fn validate_point(point: Point<f64>) -> Result<()> {
    if !point.x().is_finite()
        || !point.y().is_finite()
        || !(-180.0..=180.0).contains(&point.x())
        || !(-90.0..=90.0).contains(&point.y())
    {
        bail!("invalid longitude or latitude");
    }
    Ok(())
}

fn element_area(element: &Element) -> Result<Option<MultiPolygon<f64>>> {
    match element.kind.as_str() {
        "node" => Ok(None),
        "way" => {
            let geometry = element
                .geometry
                .as_deref()
                .ok_or_else(|| anyhow!("way/{} is missing geometry", element.id))?;
            Ok(Some(MultiPolygon(vec![Polygon::new(
                ring(geometry)?,
                vec![],
            )])))
        }
        "relation" => relation_area(element).map(Some),
        _ => bail!("unsupported OpenStreetMap element type {}", element.kind),
    }
}

fn relation_area(element: &Element) -> Result<MultiPolygon<f64>> {
    let members = element
        .members
        .as_deref()
        .ok_or_else(|| anyhow!("relation/{} is missing members", element.id))?;
    let mut outers = Vec::new();
    let mut inners = Vec::new();
    for role in ["outer", "inner"] {
        let lines = members
            .iter()
            .filter(|member| member.kind == "way" && member.role == role)
            .map(|member| {
                member
                    .geometry
                    .as_deref()
                    .ok_or_else(|| anyhow!("relation/{} member is missing geometry", element.id))
                    .and_then(line)
            })
            .collect::<Result<Vec<_>>>()?;
        let stitched = stitch_rings(lines)
            .with_context(|| format!("assemble relation/{} {role} rings", element.id))?;
        if role == "outer" {
            outers = stitched;
        } else {
            inners = stitched;
        }
    }
    if outers.is_empty() {
        bail!("relation/{} has no outer geometry", element.id);
    }

    let mut holes = vec![Vec::new(); outers.len()];
    for inner in inners {
        let sample = Point::from(inner.0[0]);
        let index = outers
            .iter()
            .position(|outer| Polygon::new(outer.clone(), vec![]).contains(&sample))
            .ok_or_else(|| anyhow!("relation/{} has an unmatched inner ring", element.id))?;
        holes[index].push(inner);
    }
    Ok(MultiPolygon(
        outers
            .into_iter()
            .zip(holes)
            .map(|(outer, holes)| Polygon::new(outer, holes))
            .collect(),
    ))
}

fn line(points: &[RawPoint]) -> Result<LineString<f64>> {
    if points.len() < 2 {
        bail!("geometry line has fewer than two points");
    }
    let mut coordinates = Vec::with_capacity(points.len());
    for raw in points {
        let point = Point::new(raw.lon, raw.lat);
        validate_point(point)?;
        coordinates.push(point.0);
    }
    Ok(LineString(coordinates))
}

fn ring(points: &[RawPoint]) -> Result<LineString<f64>> {
    let ring = line(points)?;
    validate_ring(&ring)?;
    Ok(ring)
}

fn validate_ring(ring: &LineString<f64>) -> Result<()> {
    if ring.0.len() < 4 || ring.0.first() != ring.0.last() {
        bail!("area geometry is not a closed ring");
    }
    let points = &ring.0[..ring.0.len() - 1];
    let mut distinct = Vec::with_capacity(3);
    for point in points {
        if !distinct.contains(point) {
            distinct.push(*point);
        }
    }
    if distinct.len() < 3 {
        bail!("area geometry has fewer than three distinct points");
    }
    Ok(())
}

fn stitch_rings(mut lines: Vec<LineString<f64>>) -> Result<Vec<LineString<f64>>> {
    let mut rings = Vec::new();
    while let Some(line) = lines.pop() {
        let mut coordinates = line.0;
        while coordinates.first() != coordinates.last() {
            let first = *coordinates.first().unwrap();
            let last = *coordinates.last().unwrap();
            let Some((index, prepend, reverse)) =
                lines.iter().enumerate().find_map(|(index, line)| {
                    let line_first = *line.0.first()?;
                    let line_last = *line.0.last()?;
                    if line_first == last {
                        Some((index, false, false))
                    } else if line_last == last {
                        Some((index, false, true))
                    } else if line_last == first {
                        Some((index, true, false))
                    } else if line_first == first {
                        Some((index, true, true))
                    } else {
                        None
                    }
                })
            else {
                bail!("relation geometry contains an open ring");
            };
            let mut next = lines.swap_remove(index).0;
            if reverse {
                next.reverse();
            }
            if prepend {
                next.pop();
                next.extend(coordinates);
                coordinates = next;
            } else {
                coordinates.pop();
                coordinates.extend(next);
            }
        }
        let ring = LineString(coordinates);
        validate_ring(&ring)?;
        rings.push(ring);
    }
    Ok(rings)
}

fn form_encode(value: &str) -> String {
    const HEX: &[u8; 16] = b"0123456789ABCDEF";
    let mut encoded = String::with_capacity(value.len());
    for byte in value.bytes() {
        if byte.is_ascii_alphanumeric() || matches!(byte, b'-' | b'_' | b'.' | b'~') {
            encoded.push(byte as char);
        } else {
            encoded.push('%');
            encoded.push(HEX[(byte >> 4) as usize] as char);
            encoded.push(HEX[(byte & 0xf) as usize] as char);
        }
    }
    encoded
}

#[cfg(test)]
mod tests {
    use super::*;
    use geo::Coord;

    #[test]
    fn normalizes_inline_and_contained_equipment_without_guessing_bad_tags() {
        let playgrounds = normalize_overpass(
            r#"{
              "elements": [
                {"type":"way","id":20,"version":3,"changeset":4,"user":"mapper","uid":5,"timestamp":"2026-09-20T12:00:00Z",
                 "center":{"lat":42.5,"lon":23.5},
                 "geometry":[{"lat":42.0,"lon":23.0},{"lat":42.0,"lon":24.0},{"lat":43.0,"lon":24.0},{"lat":43.0,"lon":23.0},{"lat":42.0,"lon":23.0}],
                 "tags":{"leisure":"playground","name":" Test ","playground:swing":"yes","playground:slide":"no","min_age":"-1","max_age":"12","image":"https://example.test/photo.jpg","wikimedia_commons":"File:Playground.jpg","surface":"rubber","access":"yes","fee":"no"}},
                {"type":"node","id":21,"version":6,"changeset":7,"user":"equipment-mapper","uid":8,"lat":42.25,"lon":23.25,"tags":{"playground":"slide"}},
                {"type":"node","id":22,"lat":42.5,"lon":23.75,"tags":{"playground":"slide"}},
                {"type":"node","id":23,"lat":41.0,"lon":23.25,"tags":{"playground":"seesaw"}}
              ]
            }"#,
        )
        .unwrap();

        assert_eq!(playgrounds.len(), 1);
        let playground = &playgrounds[0];
        assert_eq!(
            playground.source,
            crate::enrichment::SourceKind::OpenStreetMap
        );
        assert_eq!(playground.external_id, "way/20");
        assert_eq!(playground.name.as_deref(), Some("Test"));
        assert_eq!(
            playground.equipment,
            BTreeMap::from([
                (String::from("slide"), Some(2)),
                (String::from("swing"), None),
            ])
        );
        assert_eq!(playground.values["max_age"], 12);
        assert_eq!(playground.values["surface"], "rubber");
        assert_eq!(playground.values["access"], "yes");
        assert_eq!(playground.values["fee"], "no");
        assert_eq!(playground.commons_titles, ["File:Playground.jpg"]);
        assert_eq!(playground.raw_data["root"]["id"], 20);
        assert_eq!(playground.raw_data["root"]["version"], 3);
        assert_eq!(playground.raw_data["root"]["changeset"], 4);
        assert_eq!(playground.raw_data["root"]["user"], "mapper");
        assert_eq!(playground.raw_data["root"]["uid"], 5);
        assert_eq!(
            playground.raw_data["equipment"].as_array().unwrap().len(),
            2
        );
        assert_eq!(playground.raw_data["equipment"][0]["version"], 6);
        assert_eq!(playground.raw_data["equipment"][0]["changeset"], 7);
        assert_eq!(
            playground.raw_data["equipment"][0]["user"],
            "equipment-mapper"
        );
        assert_eq!(playground.raw_data["equipment"][0]["uid"], 8);
        assert_eq!((playground.longitude, playground.latitude), (23.5, 42.5));
        assert_eq!(
            playground.date_meaning,
            Some(crate::enrichment::DateMeaning::SourceUpdate)
        );
    }

    #[test]
    fn splits_multiple_wikimedia_commons_titles_and_keeps_file_prefixes() {
        let playgrounds = normalize_overpass(
            r#"{"elements":[{"type":"node","id":9,"lat":42.7,"lon":23.3,"tags":{"leisure":"playground","wikimedia_commons":"File:A.jpg; File:B.jpg;https://example.test/C.jpg;Category:D"}}]}"#,
        )
        .unwrap();

        assert_eq!(playgrounds[0].commons_titles, ["File:A.jpg", "File:B.jpg"]);
    }

    #[test]
    fn area_point_is_inside_concave_playground() {
        let playgrounds = normalize_overpass(
            r#"{"elements":[{"type":"way","id":1,"center":{"lat":1.5,"lon":1.5},"geometry":[{"lat":0,"lon":0},{"lat":0,"lon":3},{"lat":1,"lon":3},{"lat":1,"lon":1},{"lat":3,"lon":1},{"lat":3,"lon":0},{"lat":0,"lon":0}],"tags":{"leisure":"playground"}}]}"#,
        )
        .unwrap();
        let area = Polygon::new(
            LineString(vec![
                Coord { x: 0., y: 0. },
                Coord { x: 3., y: 0. },
                Coord { x: 3., y: 1. },
                Coord { x: 1., y: 1. },
                Coord { x: 1., y: 3. },
                Coord { x: 0., y: 3. },
                Coord { x: 0., y: 0. },
            ]),
            vec![],
        );
        assert!(area.contains(&Point::new(
            playgrounds[0].longitude,
            playgrounds[0].latitude
        )));
    }

    #[test]
    fn normalizes_nodes_and_stitched_relations_with_stable_ids() {
        let playgrounds = normalize_overpass(
            r#"{"elements":[
              {"type":"node","id":7,"lat":42.7,"lon":23.3,"tags":{"leisure":"playground"}},
              {"type":"relation","id":8,"center":{"lat":42.5,"lon":23.5},"members":[
                {"type":"way","ref":1,"role":"outer","geometry":[{"lat":42,"lon":23},{"lat":42,"lon":24},{"lat":43,"lon":24}]},
                {"type":"way","ref":2,"role":"outer","geometry":[{"lat":43,"lon":24},{"lat":43,"lon":23},{"lat":42,"lon":23}]}
              ],"tags":{"leisure":"playground","playground:climbingframe":"yes"}}
            ]}"#,
        )
        .unwrap();

        assert_eq!(playgrounds[0].external_id, "node/7");
        assert_eq!(
            (playgrounds[0].longitude, playgrounds[0].latitude),
            (23.3, 42.7)
        );
        assert_eq!(playgrounds[1].external_id, "relation/8");
        assert_eq!(playgrounds[1].equipment["climbing_frame"], None);
        assert_eq!(playgrounds[1].raw_data["root"]["members"][0]["ref"], 1);
    }

    #[test]
    fn falls_back_to_bounds_center_when_overpass_omits_center() {
        let playgrounds = normalize_overpass(
            r#"{"elements":[
              {"type":"way","id":30,
               "bounds":{"minlat":42.0,"minlon":23.0,"maxlat":42.2,"maxlon":23.4},
               "geometry":[{"lat":42.0,"lon":23.0},{"lat":42.0,"lon":23.4},{"lat":42.2,"lon":23.4},{"lat":42.2,"lon":23.0},{"lat":42.0,"lon":23.0}],
               "tags":{"leisure":"playground"}},
              {"type":"node","id":31,"lat":42.1,"lon":23.2,"tags":{"playground":"swing"}}
            ]}"#,
        )
        .unwrap();

        assert_eq!(playgrounds.len(), 1);
        assert_eq!(
            (playgrounds[0].longitude, playgrounds[0].latitude),
            (23.2, 42.1)
        );
        assert_eq!(playgrounds[0].equipment["swing"], Some(1));
        assert_eq!(
            playgrounds[0].raw_data["equipment"]
                .as_array()
                .unwrap()
                .len(),
            1
        );
    }

    #[test]
    fn rejects_malformed_and_remarked_responses() {
        assert!(normalize_overpass("not json").is_err());
        assert!(normalize_overpass(r#"{"remark":"runtime error","elements":[]}"#).is_err());
        assert!(normalize_overpass(r#"{"error":"bad","elements":[]}"#).is_err());
    }

    #[test]
    fn parses_neighborhood_source() {
        let path = std::env::temp_dir().join(format!(
            "playground-neighborhoods-{}.geojson",
            std::process::id()
        ));
        fs::write(
            &path,
            r#"{"type":"FeatureCollection","features":[{"type":"Feature","properties":{"id":"one","name":"One"},"geometry":{"type":"Polygon","coordinates":[[[23,42],[24,42],[24,43],[23,42]]]}}]}"#,
        )
        .unwrap();
        let neighborhoods = load_neighborhoods(&path).unwrap();
        fs::remove_file(path).unwrap();
        assert_eq!(neighborhoods.len(), 1);
        assert_eq!(neighborhoods[0].id, "one");
    }
}
