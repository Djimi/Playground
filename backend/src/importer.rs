use std::{collections::BTreeSet, fs, path::Path};

use anyhow::{Context, Result, anyhow, bail};
use chrono::{DateTime, Utc};
use geo::{Contains, InteriorPoint, Intersects, LineString, MultiPolygon, Point, Polygon};
use serde::Deserialize;
use serde_json::Value;
use sqlx::PgPool;

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

#[derive(Debug, Clone, PartialEq)]
pub struct Playground {
    pub id: String,
    pub name: Option<String>,
    pub longitude: f64,
    pub latitude: f64,
    pub capabilities: Vec<String>,
    pub min_age: Option<i16>,
    pub max_age: Option<i16>,
    pub photo_urls: Vec<String>,
    pub source_url: String,
    pub source_updated_at: Option<DateTime<Utc>>,
}

#[derive(Debug, Clone)]
pub struct Neighborhood {
    pub id: String,
    pub name: String,
    geometry_json: String,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ImportCounts {
    pub neighborhoods: u64,
    pub playgrounds: u64,
    pub memberships: u64,
}

#[derive(Debug, Deserialize)]
struct OverpassResponse {
    elements: Vec<Element>,
}

#[derive(Debug, Deserialize)]
struct Element {
    #[serde(rename = "type")]
    kind: String,
    id: i64,
    lat: Option<f64>,
    lon: Option<f64>,
    center: Option<RawPoint>,
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

#[derive(Debug, Deserialize)]
struct Member {
    #[serde(rename = "type")]
    kind: String,
    #[serde(default)]
    role: String,
    geometry: Option<Vec<RawPoint>>,
}

struct Root {
    playground: Playground,
    area: Option<MultiPolygon<f64>>,
}

pub async fn run_import(
    pool: &PgPool,
    overpass_url: &str,
    neighborhoods_path: &Path,
) -> Result<ImportCounts> {
    let body = fetch_overpass(overpass_url).await?;
    apply_import(pool, &body, neighborhoods_path).await
}

pub async fn apply_import(
    pool: &PgPool,
    overpass_body: &str,
    neighborhoods_path: &Path,
) -> Result<ImportCounts> {
    let playgrounds = normalize_overpass(overpass_body)?;
    let neighborhoods = load_neighborhoods(neighborhoods_path)?;
    replace_snapshot(pool, &neighborhoods, &playgrounds).await
}

pub async fn fetch_overpass(url: &str) -> Result<String> {
    let response = reqwest::Client::new()
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

pub fn normalize_overpass(body: &str) -> Result<Vec<Playground>> {
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

    for element in &response.elements {
        let leisure = tag(element, "leisure");
        let equipment_kind = tag(element, "playground").and_then(normalize_capability);

        if leisure == Some("playground") {
            roots.push(normalize_root(element)?);
        } else if let Some(capability) = equipment_kind {
            equipment.push((element_point(element)?, capability));
        }
    }

    if roots.is_empty() {
        bail!("Overpass response contains no Sofia playgrounds");
    }

    for (point, capability) in equipment {
        for root in &mut roots {
            if root
                .area
                .as_ref()
                .is_some_and(|area| area.intersects(&point))
            {
                root.playground.capabilities.push(capability.to_owned());
            }
        }
    }

    let mut playgrounds = Vec::with_capacity(roots.len());
    for mut root in roots {
        root.playground.capabilities.sort();
        root.playground.capabilities.dedup();
        playgrounds.push(root.playground);
    }
    playgrounds.sort_by(|left, right| left.id.cmp(&right.id));

    let mut ids = BTreeSet::new();
    if playgrounds
        .iter()
        .any(|playground| !ids.insert(&playground.id))
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
    playgrounds: &[Playground],
) -> Result<ImportCounts> {
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

    for playground in playgrounds {
        sqlx::query(
            "INSERT INTO import_playgrounds (id, name, location, capabilities, min_age, max_age, photo_urls, source_url, source_updated_at) VALUES ($1, $2, ST_SetSRID(ST_MakePoint($3, $4), 4326)::geography, $5, $6, $7, $8, $9, $10)",
        )
        .bind(&playground.id)
        .bind(&playground.name)
        .bind(playground.longitude)
        .bind(playground.latitude)
        .bind(&playground.capabilities)
        .bind(playground.min_age)
        .bind(playground.max_age)
        .bind(&playground.photo_urls)
        .bind(&playground.source_url)
        .bind(playground.source_updated_at)
        .execute(&mut *transaction)
        .await
        .with_context(|| format!("stage playground {}", playground.id))?;
    }

    sqlx::query("DELETE FROM playground_neighborhoods")
        .execute(&mut *transaction)
        .await?;
    sqlx::query("DELETE FROM playgrounds")
        .execute(&mut *transaction)
        .await?;
    sqlx::query("DELETE FROM neighborhoods")
        .execute(&mut *transaction)
        .await?;
    sqlx::query("INSERT INTO neighborhoods SELECT * FROM import_neighborhoods")
        .execute(&mut *transaction)
        .await?;
    sqlx::query("INSERT INTO playgrounds SELECT * FROM import_playgrounds")
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
    Ok(ImportCounts {
        neighborhoods: neighborhoods.len() as u64,
        playgrounds: playgrounds.len() as u64,
        memberships,
    })
}

fn normalize_root(element: &Element) -> Result<Root> {
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

    let mut capabilities = CAPABILITIES
        .iter()
        .filter(|capability| {
            tag(element, &format!("playground:{capability}")).is_some_and(is_truthy)
                || (**capability == "climbing_frame"
                    && tag(element, "playground:climbingframe").is_some_and(is_truthy))
        })
        .map(|capability| (*capability).to_owned())
        .collect::<Vec<_>>();
    if let Some(capability) = tag(element, "playground").and_then(normalize_capability) {
        capabilities.push(capability.to_owned());
    }

    let mut min_age = tag(element, "min_age").and_then(parse_age);
    let mut max_age = tag(element, "max_age").and_then(parse_age);
    if min_age.zip(max_age).is_some_and(|(min, max)| min > max) {
        min_age = None;
        max_age = None;
    }

    let photo_urls = tag(element, "image")
        .into_iter()
        .flat_map(|images| images.split(';'))
        .map(str::trim)
        .filter(|url| {
            reqwest::Url::parse(url).is_ok_and(|parsed| {
                matches!(parsed.scheme(), "http" | "https") && parsed.host().is_some()
            })
        })
        .map(str::to_owned)
        .collect();
    let timestamp = element
        .timestamp
        .as_deref()
        .or_else(|| tag(element, "timestamp"));

    Ok(Root {
        playground: Playground {
            id: format!("{}/{}", element.kind, element.id),
            name: tag(element, "name")
                .map(str::trim)
                .filter(|name| !name.is_empty())
                .map(str::to_owned),
            longitude: point.x(),
            latitude: point.y(),
            capabilities,
            min_age,
            max_age,
            photo_urls,
            source_url: format!(
                "https://www.openstreetmap.org/{}/{}",
                element.kind, element.id
            ),
            source_updated_at: timestamp.and_then(|value| {
                DateTime::parse_from_rfc3339(value)
                    .ok()
                    .map(|parsed| parsed.with_timezone(&Utc))
            }),
        },
        area,
    })
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
        "way" | "relation" => element
            .center
            .ok_or_else(|| anyhow!("{}/{} is missing center", element.kind, element.id))?,
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
                {"type":"way","id":20,"timestamp":"2026-09-20T12:00:00Z",
                 "center":{"lat":42.5,"lon":23.5},
                 "geometry":[{"lat":42.0,"lon":23.0},{"lat":42.0,"lon":24.0},{"lat":43.0,"lon":24.0},{"lat":43.0,"lon":23.0},{"lat":42.0,"lon":23.0}],
                 "tags":{"leisure":"playground","name":" Test ","playground:swing":"yes","playground:slide":"no","min_age":"-1","max_age":"12","image":"ftp://bad; https://example.test/photo.jpg"}},
                {"type":"node","id":21,"lat":42.25,"lon":23.25,"tags":{"playground":"slide"}},
                {"type":"node","id":22,"lat":41.0,"lon":23.25,"tags":{"playground":"seesaw"}}
              ]
            }"#,
        )
        .unwrap();

        assert_eq!(playgrounds.len(), 1);
        let playground = &playgrounds[0];
        assert_eq!(playground.id, "way/20");
        assert_eq!(playground.name.as_deref(), Some("Test"));
        assert_eq!(playground.capabilities, ["slide", "swing"]);
        assert_eq!(playground.min_age, None);
        assert_eq!(playground.max_age, Some(12));
        assert_eq!(playground.photo_urls, ["https://example.test/photo.jpg"]);
        assert_eq!((playground.longitude, playground.latitude), (23.5, 42.5));
        assert!(playground.source_updated_at.is_some());
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

        assert_eq!(playgrounds[0].id, "node/7");
        assert_eq!(
            (playgrounds[0].longitude, playgrounds[0].latitude),
            (23.3, 42.7)
        );
        assert_eq!(playgrounds[1].id, "relation/8");
        assert_eq!(playgrounds[1].capabilities, ["climbing_frame"]);
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
