use std::collections::BTreeMap;

use anyhow::{Context, Result, anyhow, bail};
use chrono::{DateTime, Utc};
use serde::Serialize;
use serde_json::Value;

pub const SOFIAPLAN_URL: &str = "https://api.sofiaplan.bg/datasets/5";
pub const SOFIAPLAN_DATASET_PAGE: &str = "https://urbandata.sofia.bg/dataset/playgrounds";
pub const SOFIAPLAN_OBSERVED_AT: &str = "2019-04-18T00:00:00Z";
pub const MATCH_RADIUS_METERS: f64 = 15.0;

#[derive(Clone, Copy, Debug, Eq, PartialEq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum SourceKind {
    OpenStreetMap,
    SofiaPlan,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum DateMeaning {
    Observation,
    SourceUpdate,
}

#[derive(Clone, Debug, PartialEq, Serialize)]
pub struct SourceValue {
    pub field: String,
    pub value: Value,
    pub source: SourceKind,
    pub source_id: String,
    pub date: Option<DateTime<Utc>>,
    pub date_meaning: Option<DateMeaning>,
    pub selected: bool,
}

#[derive(Clone, Debug)]
pub struct SourcePlayground {
    pub source: SourceKind,
    pub external_id: String,
    pub raw_data: Value,
    pub name: Option<String>,
    pub longitude: f64,
    pub latitude: f64,
    pub source_date: Option<DateTime<Utc>>,
    pub date_meaning: Option<DateMeaning>,
    pub values: BTreeMap<String, Value>,
    pub equipment: BTreeMap<String, Option<i32>>,
    pub commons_titles: Vec<String>,
    pub excluded_from_catalog: bool,
}

pub fn normalize_sofiaplan(body: &str) -> Result<Vec<SourcePlayground>> {
    let dataset: Value = serde_json::from_str(body).context("SofiaPlan response is not valid JSON")?;
    if dataset.get("type").and_then(Value::as_str) != Some("FeatureCollection") {
        bail!("SofiaPlan response must be a GeoJSON FeatureCollection");
    }
    let features = dataset
        .get("features")
        .and_then(Value::as_array)
        .ok_or_else(|| anyhow!("SofiaPlan response must contain a features array"))?;
    let observed_at = DateTime::parse_from_rfc3339(SOFIAPLAN_OBSERVED_AT)
        .expect("SofiaPlan observation timestamp is valid")
        .with_timezone(&Utc);

    features
        .iter()
        .map(|feature| normalize_sofiaplan_feature(feature, observed_at))
        .collect()
}

fn normalize_sofiaplan_feature(feature: &Value, observed_at: DateTime<Utc>) -> Result<SourcePlayground> {
    let properties = feature
        .get("properties")
        .and_then(Value::as_object)
        .ok_or_else(|| anyhow!("SofiaPlan feature is missing properties"))?;
    let external_id = known(properties.get("nobekt_new"))
        .and_then(Value::as_str)
        .map(str::trim)
        .filter(|id| !id.is_empty())
        .ok_or_else(|| anyhow!("SofiaPlan feature is missing nobekt_new"))?
        .to_owned();
    let (longitude, latitude) = sofiaplan_point(feature, &external_id)?;
    let mut values = BTreeMap::new();
    let mut equipment = BTreeMap::new();

    insert_text(&mut values, "address", preferred(properties, &["new_mestopolozh", "mestopolozh_old"]));
    insert_text(&mut values, "ownership", preferred(properties, &["new_vids_kk", "vids_kk_old"]));
    insert_text(&mut values, "ownership_detail", preferred(properties, &["new_sobstvenos", "sobstvenost_old"]));
    insert_text(
        &mut values,
        "municipal_status",
        preferred(properties, &["new_label", "new_meropr", "meropr_old"]),
    );
    insert_text(&mut values, "repairs", preferred(properties, &["new_meropr", "meropr_old"]));
    insert_text(&mut values, "notes", preferred(properties, &["new_zabelezhka", "zabelezhka"]));

    if let Some(value) = preferred(properties, &["new_vazrgrupi", "vazr_old"]) {
        match parse_age_range(value) {
            Some((min_age, max_age)) => {
                values.insert("min_age".into(), Value::from(min_age));
                values.insert("max_age".into(), Value::from(max_age));
            }
            None => rejected(&external_id, "age", value),
        }
    }
    if let Some(value) = preferred(properties, &["new_ograda", "ograda"]) {
        match parse_fenced(value) {
            Some(fenced) => {
                values.insert("fenced".into(), Value::Bool(fenced));
            }
            None => rejected(&external_id, "fenced", value),
        }
    }
    if let Some(value) = preferred(properties, &["new_naredba1", "naredba1_old"]) {
        match parse_bulgarian_bool(value) {
            Some(compliant) => {
                values.insert("ordinance_compliant".into(), Value::Bool(compliant));
            }
            None => rejected(&external_id, "ordinance_compliant", value),
        }
    }
    for (field, property, equipment_name) in [
        ("swing", "new_lulki", "swing"),
        ("climbing_frame", "new_katerushka", "climbing_frame"),
        ("sandpit", "new_pyasachnik", "sandpit"),
    ] {
        if let Some(value) = known(properties.get(property)) {
            match parse_count(value) {
                Some(count) => {
                    equipment.insert(equipment_name.into(), Some(count));
                }
                None => rejected(&external_id, field, value),
            }
        }
    }
    let seesaw_counts = ["new_klatush_0_3", "new_klatushka_3_12"]
        .into_iter()
        .filter_map(|property| {
            known(properties.get(property)).and_then(|value| match parse_count(value) {
                Some(count) => Some(count),
                None => {
                    rejected(&external_id, "seesaw", value);
                    None
                }
            })
        })
        .collect::<Vec<_>>();
    if !seesaw_counts.is_empty() {
        equipment.insert("seesaw".into(), Some(seesaw_counts.into_iter().sum()));
    }

    Ok(SourcePlayground {
        source: SourceKind::SofiaPlan,
        external_id,
        raw_data: feature.clone(),
        name: None,
        longitude,
        latitude,
        source_date: Some(observed_at),
        date_meaning: Some(DateMeaning::Observation),
        values,
        equipment,
        commons_titles: Vec::new(),
        excluded_from_catalog: known(properties.get("new_label"))
            .and_then(Value::as_str)
            .is_some_and(|label| label.trim() == "не се показват на картата"),
    })
}

fn known(value: Option<&Value>) -> Option<&Value> {
    value.filter(|value| {
        !value.is_null()
            && value
                .as_str()
                .is_none_or(|value| !matches!(value.trim(), "" | "―" | "—"))
    })
}

fn preferred<'a>(properties: &'a serde_json::Map<String, Value>, names: &[&str]) -> Option<&'a Value> {
    names.iter().find_map(|name| known(properties.get(*name)))
}

fn insert_text(values: &mut BTreeMap<String, Value>, field: &str, value: Option<&Value>) {
    if let Some(value) = value.and_then(Value::as_str).map(str::trim).filter(|value| !value.is_empty()) {
        values.insert(field.into(), Value::String(value.into()));
    }
}

fn parse_age_range(value: &Value) -> Option<(i16, i16)> {
    let ranges = value
        .as_str()?
        .split(';')
        .map(|range| {
            let (min, max) = range.trim().split_once(" до ")?;
            let min = min.trim().parse::<i16>().ok()?;
            let max = max.trim().parse::<i16>().ok()?;
            ((0..=18).contains(&min) && (0..=18).contains(&max) && min <= max).then_some((min, max))
        })
        .collect::<Option<Vec<_>>>()?;
    (!ranges.is_empty()).then(|| {
        (
            ranges.iter().map(|(min, _)| *min).min().unwrap(),
            ranges.iter().map(|(_, max)| *max).max().unwrap(),
        )
    })
}

fn parse_fenced(value: &Value) -> Option<bool> {
    parse_count(value).map(|count| count > 0)
}

fn parse_bulgarian_bool(value: &Value) -> Option<bool> {
    match value.as_str()?.trim().to_lowercase().as_str() {
        "да" => Some(true),
        "не" => Some(false),
        _ => None,
    }
}

fn parse_count(value: &Value) -> Option<i32> {
    match value {
        Value::Number(number) => number.as_i64().and_then(|count| count.try_into().ok()),
        Value::String(number) => number.trim().parse().ok(),
        _ => None,
    }
    .filter(|count| *count >= 0)
}

fn sofiaplan_point(feature: &Value, external_id: &str) -> Result<(f64, f64)> {
    let geometry = feature
        .get("geometry")
        .and_then(Value::as_object)
        .ok_or_else(|| anyhow!("SofiaPlan feature {external_id} is missing geometry"))?;
    if geometry.get("type").and_then(Value::as_str) != Some("MultiPoint") {
        bail!("SofiaPlan feature {external_id} must be a MultiPoint");
    }
    let coordinates = geometry
        .get("coordinates")
        .and_then(Value::as_array)
        .filter(|coordinates| coordinates.len() == 1)
        .ok_or_else(|| anyhow!("SofiaPlan feature {external_id} must contain exactly one coordinate"))?;
    let point = coordinates[0]
        .as_array()
        .filter(|point| point.len() == 2)
        .ok_or_else(|| anyhow!("SofiaPlan feature {external_id} has invalid coordinates"))?;
    let longitude = point[0]
        .as_f64()
        .filter(|longitude| longitude.is_finite() && (-180.0..=180.0).contains(longitude))
        .ok_or_else(|| anyhow!("SofiaPlan feature {external_id} has invalid longitude"))?;
    let latitude = point[1]
        .as_f64()
        .filter(|latitude| latitude.is_finite() && (-90.0..=90.0).contains(latitude))
        .ok_or_else(|| anyhow!("SofiaPlan feature {external_id} has invalid latitude"))?;
    Ok((longitude, latitude))
}

fn rejected(external_id: &str, field: &str, value: &Value) {
    tracing::warn!(source = "sofiaplan", external_id, field, rejected_value = %value, "reject invalid SofiaPlan value");
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn normalizes_sofiaplan_fixture_with_explicit_zeroes_and_exclusions() {
        let records = normalize_sofiaplan(include_str!("../tests/fixtures/sofiaplan-playgrounds.geojson"))
            .unwrap();

        assert_eq!(records.len(), 3);
        assert_eq!(records[0].external_id, "06.129");
        assert_eq!(
            records[0].source_date.unwrap().to_rfc3339(),
            "2019-04-18T00:00:00+00:00"
        );
        assert_eq!(records[0].date_meaning, Some(DateMeaning::Observation));
        assert_eq!(records[0].values["fenced"], json!(false));
        assert_eq!(records[0].equipment["swing"], Some(0));
        assert_eq!(records[1].equipment["swing"], Some(2));
        assert!(records[2].excluded_from_catalog);
    }

    #[test]
    fn treats_unknown_markers_and_invalid_age_ranges_as_unknown() {
        let records = normalize_sofiaplan(
            r#"{"type":"FeatureCollection","features":[
              {"type":"Feature","properties":{"nobekt_new":"one","new_ograda":null,"new_vazrgrupi":"12 до 1"},"geometry":{"type":"MultiPoint","coordinates":[[23.3,42.7]]}},
              {"type":"Feature","properties":{"nobekt_new":"two","new_ograda":"","new_vazrgrupi":"―"},"geometry":{"type":"MultiPoint","coordinates":[[23.3,42.7]]}},
              {"type":"Feature","properties":{"nobekt_new":"three","new_ograda":"—"},"geometry":{"type":"MultiPoint","coordinates":[[23.3,42.7]]}}
            ]}"#,
        )
        .unwrap();

        for record in records {
            assert!(!record.values.contains_key("fenced"));
            assert!(!record.values.contains_key("min_age"));
            assert!(!record.values.contains_key("max_age"));
        }
    }

    #[test]
    fn rejects_missing_external_ids_and_non_single_point_geometries() {
        assert!(normalize_sofiaplan(
            r#"{"type":"FeatureCollection","features":[{"type":"Feature","properties":{},"geometry":{"type":"MultiPoint","coordinates":[[23.3,42.7]]}}]}"#
        )
        .is_err());
        assert!(normalize_sofiaplan(
            r#"{"type":"FeatureCollection","features":[{"type":"Feature","properties":{"nobekt_new":"one"},"geometry":{"type":"MultiPoint","coordinates":[[23.3,42.7],[23.4,42.8]]}}]}"#
        )
        .is_err());
    }
}
