use std::collections::{BTreeMap, BTreeSet};

use anyhow::{Context, Result, anyhow, bail};
use chrono::{DateTime, Utc};
use geo::{Distance, Haversine, Point};
use serde::Serialize;
use serde_json::Value;

use crate::commons::LicensedPhoto;

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
    pub photos: Vec<LicensedPhoto>,
    pub excluded_from_catalog: bool,
}

pub fn normalize_sofiaplan(body: &str) -> Result<Vec<SourcePlayground>> {
    let dataset: Value =
        serde_json::from_str(body).context("SofiaPlan response is not valid JSON")?;
    if dataset.get("type").and_then(Value::as_str) != Some("FeatureCollection") {
        bail!("SofiaPlan response must be a GeoJSON FeatureCollection");
    }
    let features = dataset
        .get("features")
        .and_then(Value::as_array)
        .ok_or_else(|| anyhow!("SofiaPlan response must contain a features array"))?;
    if features.is_empty() {
        bail!("SofiaPlan response contains no playgrounds");
    }
    let observed_at = DateTime::parse_from_rfc3339(SOFIAPLAN_OBSERVED_AT)
        .expect("SofiaPlan observation timestamp is valid")
        .with_timezone(&Utc);

    let records = features
        .iter()
        .map(|feature| normalize_sofiaplan_feature(feature, observed_at))
        .collect::<Result<Vec<_>>>()?;
    let mut ids = std::collections::BTreeSet::new();
    if records
        .iter()
        .any(|record| !ids.insert(&record.external_id))
    {
        bail!("SofiaPlan response contains duplicate playground identifiers");
    }
    Ok(records)
}

fn normalize_sofiaplan_feature(
    feature: &Value,
    observed_at: DateTime<Utc>,
) -> Result<SourcePlayground> {
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

    insert_text(
        &mut values,
        SourceKind::SofiaPlan,
        &external_id,
        "address",
        preferred(properties, &["new_mestopolozh", "mestopolozh_old"]),
    );
    insert_text(
        &mut values,
        SourceKind::SofiaPlan,
        &external_id,
        "ownership",
        preferred(properties, &["new_vids_kk", "vids_kk_old"]),
    );
    insert_text(
        &mut values,
        SourceKind::SofiaPlan,
        &external_id,
        "ownership_detail",
        preferred(properties, &["new_sobstvenos", "sobstvenost_old"]),
    );
    insert_text(
        &mut values,
        SourceKind::SofiaPlan,
        &external_id,
        "municipal_status",
        preferred(properties, &["new_label", "new_meropr", "meropr_old"]),
    );
    insert_text(
        &mut values,
        SourceKind::SofiaPlan,
        &external_id,
        "repairs",
        preferred(properties, &["new_meropr", "meropr_old"]),
    );
    insert_text(
        &mut values,
        SourceKind::SofiaPlan,
        &external_id,
        "notes",
        preferred(properties, &["new_zabelezhka", "zabelezhka"]),
    );

    if let Some(value) = preferred(properties, &["new_vazrgrupi", "vazr_old"]) {
        match parse_age_range(value) {
            Some((min_age, max_age)) => {
                values.insert("min_age".into(), Value::from(min_age));
                values.insert("max_age".into(), Value::from(max_age));
            }
            None => rejected(SourceKind::SofiaPlan, &external_id, "age", value),
        }
    }
    if let Some(value) = preferred(properties, &["new_ograda", "ograda"]) {
        match parse_fenced(value) {
            Some(fenced) => {
                values.insert("fenced".into(), Value::Bool(fenced));
            }
            None => rejected(SourceKind::SofiaPlan, &external_id, "fenced", value),
        }
    }
    if let Some(value) = preferred(properties, &["new_naredba1", "naredba1_old"]) {
        match parse_bulgarian_bool(value) {
            Some(compliant) => {
                values.insert("ordinance_compliant".into(), Value::Bool(compliant));
            }
            None => rejected(
                SourceKind::SofiaPlan,
                &external_id,
                "ordinance_compliant",
                value,
            ),
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
                None => rejected(SourceKind::SofiaPlan, &external_id, field, value),
            }
        }
    }
    let seesaw_counts = ["new_klatush_0_3", "new_klatushka_3_12"]
        .into_iter()
        .filter_map(|property| {
            known(properties.get(property)).and_then(|value| match parse_count(value) {
                Some(count) => Some(count),
                None => {
                    rejected(SourceKind::SofiaPlan, &external_id, "seesaw", value);
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
        photos: Vec::new(),
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

fn preferred<'a>(
    properties: &'a serde_json::Map<String, Value>,
    names: &[&str],
) -> Option<&'a Value> {
    names.iter().find_map(|name| known(properties.get(*name)))
}

fn insert_text(
    values: &mut BTreeMap<String, Value>,
    source: SourceKind,
    external_id: &str,
    field: &str,
    value: Option<&Value>,
) {
    let Some(value) = value else {
        return;
    };
    if let Some(value) = value
        .as_str()
        .map(str::trim)
        .filter(|value| !value.is_empty())
    {
        values.insert(field.into(), Value::String(value.into()));
    } else {
        rejected(source, external_id, field, value);
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
    value
        .as_bool()
        .or_else(|| parse_count(value).map(|count| count > 0))
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
        .ok_or_else(|| {
            anyhow!("SofiaPlan feature {external_id} must contain exactly one coordinate")
        })?;
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

fn rejected(source: SourceKind, external_id: &str, field: &str, value: &Value) {
    let source = match source {
        SourceKind::OpenStreetMap => "openstreetmap",
        SourceKind::SofiaPlan => "sofiaplan",
    };
    tracing::warn!(source, external_id, field, rejected_value = %value, "reject invalid source value");
}

#[derive(Clone, Debug, PartialEq)]
pub struct SourceLink {
    pub playground_id: String,
    pub source: SourceKind,
    pub external_id: String,
    pub match_method: &'static str,
    pub match_distance_meters: Option<f64>,
}

#[derive(Clone, Debug, PartialEq)]
pub struct SourceMatch {
    pub osm_id: String,
    pub sofia_id: String,
    pub distance_meters: f64,
}

#[derive(Clone, Debug, Default, PartialEq)]
pub struct MatchResult {
    pub clear_pairs: Vec<SourceMatch>,
    pub ambiguous_source_ids: BTreeSet<String>,
}

#[derive(Clone, Debug)]
pub struct CanonicalPlayground {
    pub id: String,
    pub name: Option<String>,
    pub longitude: f64,
    pub latitude: f64,
    pub capabilities: Vec<String>,
    pub equipment_counts: BTreeMap<String, i32>,
    pub min_age: Option<i16>,
    pub max_age: Option<i16>,
    pub address: Option<String>,
    pub surface: Option<String>,
    pub fenced: Option<bool>,
    pub ownership: Option<String>,
    pub access: Option<String>,
    pub fee: Option<String>,
    pub municipal_status: Option<String>,
    pub ordinance_compliant: Option<bool>,
    pub repairs: Option<String>,
    pub notes: Option<String>,
    pub primary_source: SourceKind,
    pub source_url: String,
    pub source_updated_at: Option<DateTime<Utc>>,
    pub photo_urls: Vec<String>,
    pub photos: Vec<LicensedPhoto>,
    pub source_values: Vec<SourceValue>,
}

#[derive(Clone, Debug, Default)]
pub struct MergeResult {
    pub playgrounds: Vec<CanonicalPlayground>,
    pub source_links: Vec<SourceLink>,
}

pub fn match_sources(osm: &[SourcePlayground], sofia: &[SourcePlayground]) -> MatchResult {
    let mut candidates = Vec::new();
    let mut osm_counts = vec![0usize; osm.len()];
    let mut sofia_counts = vec![0usize; sofia.len()];

    for (osm_index, osm_record) in osm
        .iter()
        .enumerate()
        .filter(|(_, record)| !record.excluded_from_catalog)
    {
        let osm_point = Point::new(osm_record.longitude, osm_record.latitude);
        for (sofia_index, sofia_record) in sofia
            .iter()
            .enumerate()
            .filter(|(_, record)| !record.excluded_from_catalog)
        {
            let distance_meters = Haversine.distance(
                osm_point,
                Point::new(sofia_record.longitude, sofia_record.latitude),
            );
            if within_match_radius(distance_meters) {
                osm_counts[osm_index] += 1;
                sofia_counts[sofia_index] += 1;
                candidates.push((osm_index, sofia_index, distance_meters));
            }
        }
    }

    let mut result = MatchResult::default();
    for (osm_index, sofia_index, distance_meters) in candidates {
        if osm_counts[osm_index] == 1 && sofia_counts[sofia_index] == 1 {
            result.clear_pairs.push(SourceMatch {
                osm_id: osm[osm_index].external_id.clone(),
                sofia_id: sofia[sofia_index].external_id.clone(),
                distance_meters,
            });
        } else {
            result
                .ambiguous_source_ids
                .insert(osm[osm_index].external_id.clone());
            result
                .ambiguous_source_ids
                .insert(sofia[sofia_index].external_id.clone());
        }
    }
    result.clear_pairs.sort_by(|left, right| {
        left.osm_id
            .cmp(&right.osm_id)
            .then_with(|| left.sofia_id.cmp(&right.sofia_id))
    });
    result
}

fn within_match_radius(distance_meters: f64) -> bool {
    distance_meters <= MATCH_RADIUS_METERS
}

pub fn merge_catalog(
    osm: &[SourcePlayground],
    sofia: &[SourcePlayground],
    matches: &MatchResult,
) -> MergeResult {
    let osm_by_id = osm
        .iter()
        .filter(|record| !record.excluded_from_catalog)
        .map(|record| (record.external_id.as_str(), record))
        .collect::<BTreeMap<_, _>>();
    let sofia_by_id = sofia
        .iter()
        .filter(|record| !record.excluded_from_catalog)
        .map(|record| (record.external_id.as_str(), record))
        .collect::<BTreeMap<_, _>>();
    let mut matched_osm = BTreeSet::new();
    let mut matched_sofia = BTreeSet::new();
    let mut result = MergeResult::default();
    let mut clear_pairs = matches.clear_pairs.clone();
    clear_pairs.sort_by(|left, right| {
        left.osm_id
            .cmp(&right.osm_id)
            .then_with(|| left.sofia_id.cmp(&right.sofia_id))
    });

    for pair in clear_pairs {
        let (Some(osm_record), Some(sofia_record)) = (
            osm_by_id.get(pair.osm_id.as_str()),
            sofia_by_id.get(pair.sofia_id.as_str()),
        ) else {
            continue;
        };
        if !matched_osm.insert(pair.osm_id.clone()) || !matched_sofia.insert(pair.sofia_id.clone())
        {
            continue;
        }
        let id = osm_record.external_id.clone();
        result.playgrounds.push(canonical_playground(
            id.clone(),
            &[*osm_record, *sofia_record],
            osm_record,
        ));
        result.source_links.extend([
            SourceLink {
                playground_id: id.clone(),
                source: SourceKind::OpenStreetMap,
                external_id: osm_record.external_id.clone(),
                match_method: "proximity",
                match_distance_meters: Some(pair.distance_meters),
            },
            SourceLink {
                playground_id: id,
                source: SourceKind::SofiaPlan,
                external_id: sofia_record.external_id.clone(),
                match_method: "proximity",
                match_distance_meters: Some(pair.distance_meters),
            },
        ]);
    }

    for record in osm_by_id.values() {
        if !matched_osm.contains(&record.external_id) {
            result.playgrounds.push(canonical_playground(
                record.external_id.clone(),
                &[*record],
                record,
            ));
            result
                .source_links
                .push(unmatched_link(record.external_id.clone(), record));
        }
    }
    for record in sofia_by_id.values() {
        if !matched_sofia.contains(&record.external_id) {
            let id = format!("sofiaplan/{}", record.external_id);
            result
                .playgrounds
                .push(canonical_playground(id.clone(), &[*record], record));
            result.source_links.push(unmatched_link(id, record));
        }
    }

    result
        .playgrounds
        .sort_by(|left, right| left.id.cmp(&right.id));
    result.source_links.sort_by(|left, right| {
        left.playground_id
            .cmp(&right.playground_id)
            .then_with(|| source_order(left.source).cmp(&source_order(right.source)))
            .then_with(|| left.external_id.cmp(&right.external_id))
    });
    result
}

fn unmatched_link(playground_id: String, record: &SourcePlayground) -> SourceLink {
    SourceLink {
        playground_id,
        source: record.source,
        external_id: record.external_id.clone(),
        match_method: "unmatched",
        match_distance_meters: None,
    }
}

fn canonical_playground(
    id: String,
    sources: &[&SourcePlayground],
    primary: &SourcePlayground,
) -> CanonicalPlayground {
    let mut equipment_names = BTreeSet::new();
    for source in sources {
        equipment_names.extend(source.equipment.keys().cloned());
    }
    let mut capabilities = Vec::new();
    let mut equipment_counts = BTreeMap::new();
    for name in equipment_names {
        match selected_equipment(&name, sources) {
            Some((_, count)) if *count > 0 => {
                capabilities.push(name.clone());
                equipment_counts.insert(name, *count);
            }
            None => capabilities.push(name),
            _ => {}
        }
    }

    CanonicalPlayground {
        id,
        name: selected_name(sources).map(|(_, name)| name.to_owned()),
        longitude: primary.longitude,
        latitude: primary.latitude,
        capabilities,
        equipment_counts,
        min_age: selected_i16("min_age", sources),
        max_age: selected_i16("max_age", sources),
        address: selected_string("address", sources),
        surface: selected_string("surface", sources),
        fenced: selected_bool("fenced", sources),
        ownership: selected_string("ownership", sources),
        access: selected_string("access", sources),
        fee: selected_string("fee", sources),
        municipal_status: selected_string("municipal_status", sources),
        ordinance_compliant: selected_bool("ordinance_compliant", sources),
        repairs: selected_string("repairs", sources),
        notes: selected_string("notes", sources),
        primary_source: primary.source,
        source_url: source_url(primary),
        source_updated_at: (primary.date_meaning != Some(DateMeaning::Observation))
            .then_some(primary.source_date)
            .flatten(),
        photo_urls: sources
            .iter()
            .flat_map(|source| source.photos.iter().map(|photo| photo.url.clone()))
            .collect(),
        photos: sources
            .iter()
            .flat_map(|source| source.photos.iter().cloned())
            .collect(),
        source_values: source_values(sources),
    }
}

fn source_values(sources: &[&SourcePlayground]) -> Vec<SourceValue> {
    let mut values = Vec::new();
    for source in sources {
        if let Some(name) = &source.name {
            values.push(SourceValue {
                field: "name".into(),
                value: Value::String(name.clone()),
                source: source.source,
                source_id: source.external_id.clone(),
                date: source.source_date,
                date_meaning: source.date_meaning,
                selected: selected_name(sources).is_some_and(|(selected, _)| {
                    selected.source == source.source && selected.external_id == source.external_id
                }),
            });
        }
        for (field, value) in &source.values {
            values.push(SourceValue {
                field: field.clone(),
                value: value.clone(),
                source: source.source,
                source_id: source.external_id.clone(),
                date: source.source_date,
                date_meaning: source.date_meaning,
                selected: selected_value(field, sources).is_some_and(|(selected, _)| {
                    selected.source == source.source && selected.external_id == source.external_id
                }),
            });
        }
        for (name, count) in &source.equipment {
            values.push(SourceValue {
                field: format!("equipment.{name}"),
                value: count.map_or(Value::Null, Value::from),
                source: source.source,
                source_id: source.external_id.clone(),
                date: source.source_date,
                date_meaning: source.date_meaning,
                selected: selected_equipment(name, sources)
                    .map(|(selected, _)| selected)
                    .or_else(|| selected_equipment_presence(name, sources))
                    .is_some_and(|selected| {
                        selected.source == source.source
                            && selected.external_id == source.external_id
                    }),
            });
        }
    }
    values.sort_by(|left, right| {
        left.field
            .cmp(&right.field)
            .then_with(|| source_order(left.source).cmp(&source_order(right.source)))
            .then_with(|| left.source_id.cmp(&right.source_id))
    });
    values
}

fn selected_name<'a>(sources: &[&'a SourcePlayground]) -> Option<(&'a SourcePlayground, &'a str)> {
    sources
        .iter()
        .filter_map(|source| source.name.as_deref().map(|name| (*source, name)))
        .max_by(|left, right| compare_candidates("name", left.0, right.0))
}

fn selected_value<'a>(
    field: &str,
    sources: &[&'a SourcePlayground],
) -> Option<(&'a SourcePlayground, &'a Value)> {
    sources
        .iter()
        .filter_map(|source| source.values.get(field).map(|value| (*source, value)))
        .max_by(|left, right| compare_candidates(field, left.0, right.0))
}

fn selected_equipment<'a>(
    name: &str,
    sources: &[&'a SourcePlayground],
) -> Option<(&'a SourcePlayground, &'a i32)> {
    sources
        .iter()
        .filter_map(|source| {
            source
                .equipment
                .get(name)
                .and_then(|count| count.as_ref())
                .map(|count| (*source, count))
        })
        .max_by(|left, right| compare_candidates("equipment", left.0, right.0))
}

fn selected_equipment_presence<'a>(
    name: &str,
    sources: &[&'a SourcePlayground],
) -> Option<&'a SourcePlayground> {
    sources
        .iter()
        .filter(|source| source.equipment.contains_key(name))
        .copied()
        .max_by(|left, right| compare_candidates("equipment", left, right))
}

fn compare_candidates(
    field: &str,
    left: &SourcePlayground,
    right: &SourcePlayground,
) -> std::cmp::Ordering {
    left.source_date
        .cmp(&right.source_date)
        .then_with(|| fallback_rank(field, left.source).cmp(&fallback_rank(field, right.source)))
        .then_with(|| source_order(right.source).cmp(&source_order(left.source)))
        .then_with(|| right.external_id.cmp(&left.external_id))
}

fn fallback_rank(field: &str, source: SourceKind) -> u8 {
    match (field, source) {
        ("name" | "surface" | "access" | "fee", SourceKind::OpenStreetMap)
        | (
            "address"
            | "ownership"
            | "min_age"
            | "max_age"
            | "fenced"
            | "municipal_status"
            | "ordinance_compliant"
            | "repairs"
            | "notes"
            | "equipment",
            SourceKind::SofiaPlan,
        ) => 2,
        ("name" | "surface" | "access" | "fee", _)
        | (
            "address"
            | "ownership"
            | "min_age"
            | "max_age"
            | "fenced"
            | "municipal_status"
            | "ordinance_compliant"
            | "repairs"
            | "notes"
            | "equipment",
            _,
        ) => 1,
        (_, SourceKind::OpenStreetMap) => 2,
        (_, SourceKind::SofiaPlan) => 1,
    }
}

fn selected_string(field: &str, sources: &[&SourcePlayground]) -> Option<String> {
    selected_value(field, sources).and_then(|(_, value)| value.as_str().map(str::to_owned))
}

fn selected_i16(field: &str, sources: &[&SourcePlayground]) -> Option<i16> {
    selected_value(field, sources).and_then(|(_, value)| value.as_i64()?.try_into().ok())
}

fn selected_bool(field: &str, sources: &[&SourcePlayground]) -> Option<bool> {
    selected_value(field, sources).and_then(|(_, value)| value.as_bool())
}

fn source_url(source: &SourcePlayground) -> String {
    match source.source {
        SourceKind::OpenStreetMap => {
            format!("https://www.openstreetmap.org/{}", source.external_id)
        }
        SourceKind::SofiaPlan => SOFIAPLAN_DATASET_PAGE.into(),
    }
}

fn source_order(source: SourceKind) -> u8 {
    match source {
        SourceKind::OpenStreetMap => 0,
        SourceKind::SofiaPlan => 1,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;
    use std::{
        collections::BTreeSet,
        io::{Result as IoResult, Write},
        sync::{Arc, Mutex},
    };
    use tracing_subscriber::fmt::MakeWriter;

    #[derive(Clone)]
    struct TestWriter(Arc<Mutex<Vec<u8>>>);

    impl<'a> MakeWriter<'a> for TestWriter {
        type Writer = Self;

        fn make_writer(&'a self) -> Self::Writer {
            self.clone()
        }
    }

    impl Write for TestWriter {
        fn write(&mut self, bytes: &[u8]) -> IoResult<usize> {
            self.0.lock().unwrap().extend_from_slice(bytes);
            Ok(bytes.len())
        }

        fn flush(&mut self) -> IoResult<()> {
            Ok(())
        }
    }

    #[test]
    fn normalizes_sofiaplan_fixture_with_explicit_zeroes_and_exclusions() {
        let records = normalize_sofiaplan(include_str!(
            "../tests/fixtures/sofiaplan-playgrounds.geojson"
        ))
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

    #[test]
    fn rejects_empty_and_duplicate_sofiaplan_records() {
        assert!(normalize_sofiaplan(r#"{"type":"FeatureCollection","features":[]}"#).is_err());
        assert!(normalize_sofiaplan(
            r#"{"type":"FeatureCollection","features":[
              {"type":"Feature","properties":{"nobekt_new":"same"},"geometry":{"type":"MultiPoint","coordinates":[[23.3,42.7]]}},
              {"type":"Feature","properties":{"nobekt_new":"same"},"geometry":{"type":"MultiPoint","coordinates":[[23.4,42.8]]}}
            ]}"#
        )
        .is_err());
    }

    #[test]
    fn accepts_boolean_fencing_values() {
        let records = normalize_sofiaplan(
            r#"{"type":"FeatureCollection","features":[
              {"type":"Feature","properties":{"nobekt_new":"false","new_ograda":false},"geometry":{"type":"MultiPoint","coordinates":[[23.3,42.7]]}},
              {"type":"Feature","properties":{"nobekt_new":"true","new_ograda":true},"geometry":{"type":"MultiPoint","coordinates":[[23.4,42.8]]}}
            ]}"#,
        )
        .unwrap();

        assert_eq!(records[0].values["fenced"], json!(false));
        assert_eq!(records[1].values["fenced"], json!(true));
    }

    #[test]
    fn warns_and_drops_invalid_text_mapped_values() {
        let output = Arc::new(Mutex::new(Vec::new()));
        let subscriber = tracing_subscriber::fmt()
            .with_ansi(false)
            .without_time()
            .with_writer(TestWriter(output.clone()))
            .finish();
        let records = tracing::subscriber::with_default(subscriber, || {
            normalize_sofiaplan(
                r#"{"type":"FeatureCollection","features":[{"type":"Feature","properties":{"nobekt_new":"invalid-address","new_mestopolozh":42},"geometry":{"type":"MultiPoint","coordinates":[[23.3,42.7]]}}]}"#,
            )
            .unwrap()
        });

        assert!(!records[0].values.contains_key("address"));
        let output = String::from_utf8(output.lock().unwrap().clone()).unwrap();
        assert!(output.contains("source=\"sofiaplan\""), "{output}");
        assert!(
            output.contains("external_id=\"invalid-address\""),
            "{output}"
        );
        assert!(output.contains("field=\"address\""), "{output}");
        assert!(output.contains("rejected_value=42"), "{output}");
    }

    fn source(
        source: SourceKind,
        external_id: &str,
        longitude: f64,
        latitude: f64,
    ) -> SourcePlayground {
        SourcePlayground {
            source,
            external_id: external_id.into(),
            raw_data: Value::Null,
            name: None,
            longitude,
            latitude,
            source_date: None,
            date_meaning: None,
            values: BTreeMap::new(),
            equipment: BTreeMap::new(),
            commons_titles: Vec::new(),
            photos: Vec::new(),
            excluded_from_catalog: false,
        }
    }

    fn latitude_offset(meters: f64) -> f64 {
        // Keep the f64 fixture on the inclusive side of its mathematical boundary.
        meters / 111_195.08 - 1e-12
    }

    #[test]
    fn matching_merges_a_mutual_pair_at_14_9_meters() {
        let osm = vec![source(SourceKind::OpenStreetMap, "node/1", 23.32, 42.70)];
        let sofia = vec![source(
            SourceKind::SofiaPlan,
            "06.129",
            23.32,
            42.70 + latitude_offset(14.9),
        )];

        let result = match_sources(&osm, &sofia);

        assert_eq!(result.clear_pairs.len(), 1);
        assert_eq!(result.clear_pairs[0].osm_id, "node/1");
        assert_eq!(result.clear_pairs[0].sofia_id, "06.129");
        assert!((result.clear_pairs[0].distance_meters - 14.9).abs() < 0.05);
        assert!(result.ambiguous_source_ids.is_empty());
    }

    #[test]
    fn matching_includes_the_15_meter_boundary() {
        assert!(within_match_radius(MATCH_RADIUS_METERS));
        assert!(!within_match_radius(f64::from_bits(
            MATCH_RADIUS_METERS.to_bits() + 1
        )));
    }

    #[test]
    fn matching_keeps_records_separate_beyond_15_meters() {
        let osm = vec![source(SourceKind::OpenStreetMap, "node/1", 23.32, 42.70)];
        let sofia = vec![source(
            SourceKind::SofiaPlan,
            "06.129",
            23.32,
            42.70 + latitude_offset(15.1),
        )];

        assert_eq!(match_sources(&osm, &sofia), MatchResult::default());
    }

    #[test]
    fn matching_keeps_one_osm_record_with_two_candidates_separate() {
        let osm = vec![source(SourceKind::OpenStreetMap, "node/2", 23.32, 42.70)];
        let sofia = vec![
            source(
                SourceKind::SofiaPlan,
                "06.129",
                23.32,
                42.70 + latitude_offset(5.0),
            ),
            source(
                SourceKind::SofiaPlan,
                "06.130",
                23.32,
                42.70 + latitude_offset(10.0),
            ),
        ];

        let result = match_sources(&osm, &sofia);

        assert!(result.clear_pairs.is_empty());
        assert_eq!(
            result.ambiguous_source_ids,
            BTreeSet::from(["node/2".into(), "06.129".into(), "06.130".into()])
        );
    }

    #[test]
    fn matching_keeps_one_sofiaplan_record_with_two_candidates_separate() {
        let osm = vec![
            source(SourceKind::OpenStreetMap, "node/1", 23.32, 42.70),
            source(
                SourceKind::OpenStreetMap,
                "node/2",
                23.32,
                42.70 + latitude_offset(10.0),
            ),
        ];
        let sofia = vec![source(
            SourceKind::SofiaPlan,
            "06.129",
            23.32,
            42.70 + latitude_offset(5.0),
        )];

        let result = match_sources(&osm, &sofia);

        assert!(result.clear_pairs.is_empty());
        assert_eq!(
            result.ambiguous_source_ids,
            BTreeSet::from(["node/1".into(), "node/2".into(), "06.129".into()])
        );
    }

    #[test]
    fn merging_selects_newest_values_and_retains_false_and_zero_provenance() {
        let mut osm = source(SourceKind::OpenStreetMap, "node/1", 23.32, 42.70);
        osm.source_date = Some(
            DateTime::parse_from_rfc3339("2025-01-01T00:00:00Z")
                .unwrap()
                .into(),
        );
        osm.date_meaning = Some(DateMeaning::SourceUpdate);
        osm.values.insert("fenced".into(), json!(false));
        osm.equipment.insert("swing".into(), Some(0));

        let mut sofia = source(SourceKind::SofiaPlan, "06.129", 23.32, 42.70);
        sofia.source_date = Some(
            DateTime::parse_from_rfc3339("2019-04-18T00:00:00Z")
                .unwrap()
                .into(),
        );
        sofia.date_meaning = Some(DateMeaning::Observation);
        sofia.values.insert("fenced".into(), json!(true));
        sofia.equipment.insert("swing".into(), Some(2));

        let matches = match_sources(&[osm.clone()], &[sofia.clone()]);
        let merged = merge_catalog(&[osm], &[sofia], &matches);
        let playground = &merged.playgrounds[0];

        assert_eq!(playground.id, "node/1");
        assert_eq!(playground.fenced, Some(false));
        assert!(playground.capabilities.is_empty());
        assert!(playground.equipment_counts.is_empty());
        assert_eq!(
            playground
                .source_values
                .iter()
                .find(|value| value.field == "equipment.swing" && value.source_id == "node/1")
                .map(|value| (&value.value, value.selected)),
            Some((&json!(0), true))
        );
        assert_eq!(
            playground
                .source_values
                .iter()
                .find(|value| value.field == "fenced" && value.source_id == "06.129")
                .map(|value| value.selected),
            Some(false)
        );
    }

    #[test]
    fn merging_uses_source_fallbacks_and_keeps_stable_ids_and_exclusions() {
        let mut osm = source(SourceKind::OpenStreetMap, "node/1", 23.32, 42.70);
        osm.name = Some("OSM name".into());
        osm.source_date = Some(
            DateTime::parse_from_rfc3339("2020-01-01T00:00:00Z")
                .unwrap()
                .into(),
        );
        osm.values.insert("address".into(), json!("OSM address"));
        osm.values.insert("surface".into(), json!("rubber"));
        osm.values.insert("min_age".into(), json!(1));

        let mut sofia = source(SourceKind::SofiaPlan, "06.129", 23.32, 42.70);
        sofia.source_date = Some(
            DateTime::parse_from_rfc3339("2020-01-01T00:00:00Z")
                .unwrap()
                .into(),
        );
        sofia
            .values
            .insert("address".into(), json!("Sofia address"));
        sofia.values.insert("surface".into(), json!("sand"));
        sofia.values.insert("min_age".into(), json!(3));
        sofia
            .values
            .insert("notes".into(), json!("Retained despite OSM missing it"));

        let osm_only = source(SourceKind::OpenStreetMap, "node/2", 23.40, 42.70);
        let sofia_only = source(SourceKind::SofiaPlan, "06.130", 23.50, 42.70);
        let mut excluded = source(SourceKind::SofiaPlan, "06.131", 23.60, 42.70);
        excluded.excluded_from_catalog = true;

        let osm_records = vec![osm.clone(), osm_only];
        let sofia_records = vec![sofia.clone(), sofia_only, excluded];
        let merged = merge_catalog(
            &osm_records,
            &sofia_records,
            &match_sources(&osm_records, &sofia_records),
        );
        let playground = merged
            .playgrounds
            .iter()
            .find(|playground| playground.id == "node/1")
            .unwrap();

        assert_eq!(playground.name.as_deref(), Some("OSM name"));
        assert_eq!(playground.address.as_deref(), Some("Sofia address"));
        assert_eq!(playground.surface.as_deref(), Some("rubber"));
        assert_eq!(playground.min_age, Some(3));
        assert_eq!(
            playground.notes.as_deref(),
            Some("Retained despite OSM missing it")
        );
        assert_eq!(
            merged
                .playgrounds
                .iter()
                .map(|playground| playground.id.as_str())
                .collect::<Vec<_>>(),
            vec!["node/1", "node/2", "sofiaplan/06.130"]
        );
        assert!(
            merged
                .source_links
                .iter()
                .all(|link| link.external_id != "06.131")
        );
    }

    #[test]
    fn merging_selects_newest_name_and_retains_name_provenance() {
        let mut osm = source(SourceKind::OpenStreetMap, "node/1", 23.32, 42.70);
        osm.name = Some("Older OSM name".into());
        osm.source_date = Some(
            DateTime::parse_from_rfc3339("2020-01-01T00:00:00Z")
                .unwrap()
                .into(),
        );

        let mut sofia = source(SourceKind::SofiaPlan, "06.129", 23.32, 42.70);
        sofia.name = Some("Newer SofiaPlan name".into());
        sofia.source_date = Some(
            DateTime::parse_from_rfc3339("2025-01-01T00:00:00Z")
                .unwrap()
                .into(),
        );

        let matches = match_sources(&[osm.clone()], &[sofia.clone()]);
        let merged = merge_catalog(&[osm], &[sofia], &matches);
        let playground = &merged.playgrounds[0];

        assert_eq!(playground.name.as_deref(), Some("Newer SofiaPlan name"));
        assert_eq!(
            playground
                .source_values
                .iter()
                .filter(|value| value.field == "name")
                .map(|value| (value.source_id.as_str(), value.selected))
                .collect::<Vec<_>>(),
            vec![("node/1", false), ("06.129", true)]
        );
    }

    #[test]
    fn merging_keeps_observations_out_of_source_updated_at() {
        let mut sofia = source(SourceKind::SofiaPlan, "06.129", 23.32, 42.70);
        sofia.source_date = Some(
            DateTime::parse_from_rfc3339("2019-04-18T00:00:00Z")
                .unwrap()
                .into(),
        );
        sofia.date_meaning = Some(DateMeaning::Observation);
        sofia
            .values
            .insert("notes".into(), json!("Observed municipal note"));
        let observed_at = sofia.source_date;

        let merged = merge_catalog(&[], &[sofia], &MatchResult::default());
        let playground = &merged.playgrounds[0];

        assert_eq!(playground.source_updated_at, None);
        assert_eq!(
            playground
                .source_values
                .iter()
                .find(|value| value.field == "notes")
                .map(|value| (value.date, value.date_meaning)),
            Some((observed_at, Some(DateMeaning::Observation)))
        );
    }

    #[test]
    fn merging_retains_unknown_equipment_count_provenance() {
        let mut osm = source(SourceKind::OpenStreetMap, "node/1", 23.32, 42.70);
        osm.equipment.insert("slide".into(), None);

        let merged = merge_catalog(&[osm], &[], &MatchResult::default());
        let playground = &merged.playgrounds[0];

        assert_eq!(playground.capabilities, vec!["slide"]);
        assert!(playground.equipment_counts.is_empty());
        assert_eq!(
            playground
                .source_values
                .iter()
                .find(|value| value.field == "equipment.slide")
                .map(|value| (&value.value, value.selected)),
            Some((&Value::Null, true))
        );
    }
}
