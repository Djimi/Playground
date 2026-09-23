# Playground Data Enrichment Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Populate the local Sofia playground catalog once from OpenStreetMap and SofiaPlan, retain both current source records, expose a conservatively merged catalog with licensed photos and provenance, and render the approved popup and details UI.

**Architecture:** Extend the existing Rust importer into a two-source batch pipeline. It validates and stores current source records, resolves direct Wikimedia Commons references, conservatively matches records within 15 metres, writes one canonical catalog transactionally, and serves additive fields through the existing GraphQL API. The browser continues to query only the local API.

**Tech Stack:** Rust 2024, Tokio, reqwest, serde/serde_json, geo, sqlx, PostgreSQL/PostGIS, async-graphql, vanilla JavaScript, Leaflet, CSS, Node test runner, OpenSpec.

**Spec:** `docs/superpowers/specs/2026-09-22-playground-data-enrichment-design.md`

## Global Constraints

- Read `AGENTS.md`, `lessons.md`, the design spec, the approved HTML mockup, and current OpenSpec specs before editing behavior.
- Keep the existing Rust, PostgreSQL/PostGIS, GraphQL, vanilla JavaScript, and Leaflet stack. Add no dependency.
- Import both primary datasets only through the operator command. Add no browser-side source calls, scheduler, deployment work, or location-permission flow.
- Keep only the current OSM and SofiaPlan source records. Do not add source-version history.
- Keep OSM-backed canonical IDs unchanged. Use `sofiaplan/<nobekt_new>` for SofiaPlan-only canonical IDs.
- Merge proximity candidates only when each record has exactly one opposite-source candidate within `15.0` metres. Keep every ambiguous candidate separate.
- Missing values never replace known values. Explicit `false` and numeric `0` remain known values.
- Use the field observation date when present, otherwise the source-record update date. Newest effective date wins; equal or absent dates use the design's field fallback.
- Treat SofiaPlan's `2019-04-18` date as a source-wide observation date. Do not present portal modification time or local import time as field verification.
- Store every record returned by the current SofiaPlan export (1,839 at design time), but exclude records whose `new_label` is `не се показват на картата` from the canonical catalog.
- Accept only direct Wikimedia Commons file references with verified public-domain, CC0, CC BY, or CC BY-SA metadata. Reject NC, ND, non-free, missing, and unknown licences.
- Preserve `photoUrls` and `source` GraphQL compatibility while adding structured photos, all sources, provenance, and enriched fields.
- Keep `No ratings yet` and `No reviews yet`. Never fabricate ratings or Google data.
- Assign imported strings with DOM `textContent`; never inject source data through `innerHTML`.
- A primary-source, validation, or database failure leaves the previously usable source and canonical catalogs unchanged.

## Review Focus

- SofiaPlan returns GeoJSON as `text/plain`; a valid single-coordinate `MultiPoint` must import, while missing IDs or unusable geometry must abort the batch. Covered in Task 3.
- SofiaPlan `0` means explicit absence for equipment and fencing, while null, empty, `―`, and `—` mean unknown. Covered in Tasks 3 and 4.
- One-to-many spatial candidates, including candidates exactly at the 15-metre boundary, must never create a false merge. Covered in Task 4.
- OSM record-edit dates and SofiaPlan observation dates must stay distinguishable in API and UI copy. Covered in Tasks 4, 8, and 9.
- Commons metadata may be missing, non-free, or HTML-formatted; unsafe metadata must never render as HTML or fail the primary import. Covered in Tasks 5 and 9.

---

### Task 1: Create the OpenSpec change

**Files:**
- Create: `openspec/changes/enrich-playground-data/.openspec.yaml`
- Create: `openspec/changes/enrich-playground-data/proposal.md`
- Create: `openspec/changes/enrich-playground-data/design.md`
- Create: `openspec/changes/enrich-playground-data/tasks.md`
- Create: `openspec/changes/enrich-playground-data/specs/playground-discovery/spec.md`
- Create: `openspec/changes/enrich-playground-data/specs/sofia-neighborhood-map/spec.md`

**Interfaces:**
- Consumes: approved design at `docs/superpowers/specs/2026-09-22-playground-data-enrichment-design.md`.
- Produces: validated active change `enrich-playground-data`; every later task checks off its matching OpenSpec task.

- [ ] **Step 1: Confirm current specs and active changes**

Run:

```bash
openspec list
openspec show playground-discovery --type spec
openspec show sofia-neighborhood-map --type spec
```

Expected: no active enrichment change; main specs still describe OSM-only source metadata and the existing popup/detail fields.

- [ ] **Step 2: Generate the change artifacts**

Invoke `$openspec-propose` with this exact change brief:

```text
Change name: enrich-playground-data

Add one operator-run import that fetches OpenStreetMap and SofiaPlan, keeps each current source record, excludes SofiaPlan records explicitly marked "не се показват на картата" from canonical results, conservatively merges only unambiguous pairs within 15 metres, retains provenance and conflicting source values, resolves only directly referenced reusable Wikimedia Commons files, and transactionally replaces the catalog. Preserve the old catalog on failure. Extend GraphQL additively with address, surface, fencing, ownership, access, fee, historical municipal status, Ordinance 1 compliance, repairs, notes, structured photos, all source metadata, and source-value history while keeping photoUrls and source compatible. Update the popup and details UI to match the committed mockup, keep rating/review empty states, keep accessibility and focus behavior, and make no scheduler, deployment, visitor-editing, Google, Mapillary, or browser-source-call changes.
```

Expected artifacts must include these requirements and scenarios:

```text
playground-discovery
- retain current records from both primary sources
- merge only one-to-one candidates within 15 metres
- preserve uncertain records separately
- select newest non-missing field by effective date and retain provenance
- keep explicit false/zero distinct from unknown
- accept only verified reusable Commons photos
- replace source records, mappings, canonical records, and memberships atomically
- return enriched fields, structured photos, all sources, and field history additively

sofia-neighborhood-map
- compact preview follows approved field hierarchy
- full details show enriched facts, warnings, source history, and attribution
- missing data and failed photos remain explicit
- warning meaning is textual
- existing hover, focus, selection, modal, and mobile behavior remains unchanged
```

- [ ] **Step 3: Validate the change**

Run:

```bash
openspec validate enrich-playground-data --strict --no-interactive
```

Expected: `enrich-playground-data` is valid with zero findings.

- [ ] **Step 4: Commit the planning artifacts**

```bash
git add openspec/changes/enrich-playground-data
git commit -m "docs: propose playground data enrichment"
```

---

### Task 2: Add source-retention and canonical enrichment schema

**Files:**
- Create: `backend/migrations/202609220003_data_enrichment.sql`
- Modify: `backend/tests/postgis.rs`

**Interfaces:**
- Consumes: existing `playgrounds`, `neighborhoods`, and `playground_neighborhoods` tables.
- Produces: `source_playgrounds`, `playground_source_links`, enriched canonical columns, and JSON constraints used by Tasks 6–8.

- [ ] **Step 1: Write the failing migration test**

Add a `#[sqlx::test(migrations = "./migrations")]` test in `backend/tests/postgis.rs` that inserts one current source record, one canonical playground, and one link, then asserts JSON defaults and foreign keys:

```rust
#[sqlx::test(migrations = "./migrations")]
async fn enrichment_schema_keeps_current_sources_and_safe_defaults(pool: PgPool) {
    sqlx::query(
        r#"INSERT INTO source_playgrounds
           (source, external_id, raw_data, location, source_date, date_meaning)
           VALUES ('sofiaplan', '06.129', '{}'::jsonb,
                   ST_SetSRID(ST_MakePoint(23.3444, 42.7070), 4326)::geography,
                   '2019-04-18T00:00:00Z', 'observation')"#,
    ).execute(&pool).await.unwrap();
    sqlx::query(
        "INSERT INTO playgrounds (id, location, source_url, primary_source) VALUES ('sofiaplan/06.129', ST_SetSRID(ST_MakePoint(23.3444, 42.7070), 4326)::geography, 'https://urbandata.sofia.bg/dataset/playgrounds', 'sofiaplan')",
    ).execute(&pool).await.unwrap();
    sqlx::query(
        "INSERT INTO playground_source_links (playground_id, source, external_id, match_method) VALUES ('sofiaplan/06.129', 'sofiaplan', '06.129', 'unmatched')",
    ).execute(&pool).await.unwrap();

    let (photos, values): (Value, Value) = sqlx::query_as(
        "SELECT photos, source_values FROM playgrounds WHERE id = 'sofiaplan/06.129'",
    ).fetch_one(&pool).await.unwrap();
    assert_eq!(photos, json!([]));
    assert_eq!(values, json!([]));
}
```

- [ ] **Step 2: Run the test to verify it fails**

Run:

```bash
cargo test --manifest-path backend/Cargo.toml --test postgis enrichment_schema_keeps_current_sources_and_safe_defaults -- --exact
```

Expected: FAIL because `source_playgrounds` does not exist.

- [ ] **Step 3: Add the migration**

Create `backend/migrations/202609220003_data_enrichment.sql` with these exact public structures:

```sql
CREATE TABLE source_playgrounds (
    source text NOT NULL CHECK (source IN ('openstreetmap', 'sofiaplan')),
    external_id text NOT NULL,
    raw_data jsonb NOT NULL,
    normalized_name text,
    location geography(Point, 4326) NOT NULL,
    source_date timestamptz,
    date_meaning text CHECK (date_meaning IN ('observation', 'source_update')),
    imported_at timestamptz NOT NULL DEFAULT now(),
    PRIMARY KEY (source, external_id)
);

CREATE INDEX source_playgrounds_location_gist_idx
    ON source_playgrounds USING gist (location);

ALTER TABLE playgrounds
    ADD COLUMN address text,
    ADD COLUMN surface text,
    ADD COLUMN fenced boolean,
    ADD COLUMN ownership text,
    ADD COLUMN access text,
    ADD COLUMN fee text,
    ADD COLUMN municipal_status text,
    ADD COLUMN ordinance_compliant boolean,
    ADD COLUMN repairs text,
    ADD COLUMN notes text,
    ADD COLUMN primary_source text NOT NULL DEFAULT 'openstreetmap'
        CHECK (primary_source IN ('openstreetmap', 'sofiaplan')),
    ADD COLUMN photos jsonb NOT NULL DEFAULT '[]'::jsonb,
    ADD COLUMN source_values jsonb NOT NULL DEFAULT '[]'::jsonb,
    ADD CONSTRAINT playgrounds_photos_array_check
        CHECK (jsonb_typeof(photos) = 'array'),
    ADD CONSTRAINT playgrounds_source_values_array_check
        CHECK (jsonb_typeof(source_values) = 'array');

CREATE TABLE playground_source_links (
    playground_id text NOT NULL REFERENCES playgrounds(id) ON DELETE CASCADE,
    source text NOT NULL,
    external_id text NOT NULL,
    match_method text NOT NULL CHECK (match_method IN ('unmatched', 'proximity')),
    match_distance_meters double precision
        CHECK (match_distance_meters IS NULL OR match_distance_meters >= 0),
    PRIMARY KEY (source, external_id),
    FOREIGN KEY (source, external_id)
        REFERENCES source_playgrounds(source, external_id) ON DELETE CASCADE
);

CREATE INDEX playground_source_links_playground_idx
    ON playground_source_links(playground_id);
```

- [ ] **Step 4: Run the focused and existing database tests**

Run:

```bash
cargo test --manifest-path backend/Cargo.toml --test postgis
```

Expected: PASS.

- [ ] **Step 5: Commit**

```bash
git add backend/migrations/202609220003_data_enrichment.sql backend/tests/postgis.rs
git commit -m "feat(db): retain playground source records"
```

---

### Task 3: Normalize SofiaPlan and traceable OSM source records

**Files:**
- Create: `backend/src/enrichment.rs`
- Modify: `backend/src/lib.rs`
- Modify: `backend/src/importer.rs`
- Create: `backend/tests/fixtures/sofiaplan-playgrounds.geojson`

**Interfaces:**
- Consumes: raw OSM JSON and SofiaPlan GeoJSON.
- Produces: `SourcePlayground`, `SourceValue`, `SourceKind`, `DateMeaning`, `normalize_sofiaplan`, and traceable OSM source records for Tasks 4–7.

- [ ] **Step 1: Add failing SofiaPlan parser tests**

Create unit tests in `backend/src/enrichment.rs` for this public contract:

```rust
let records = normalize_sofiaplan(include_str!("../tests/fixtures/sofiaplan-playgrounds.geojson"))?;
assert_eq!(records.len(), 3);
assert_eq!(records[0].external_id, "06.129");
assert_eq!(records[0].source_date.unwrap().to_rfc3339(), "2019-04-18T00:00:00+00:00");
assert_eq!(records[0].date_meaning, Some(DateMeaning::Observation));
assert_eq!(records[0].values["fenced"], json!(false));
assert_eq!(records[0].equipment["swing"], Some(0));
assert_eq!(records[1].equipment["swing"], Some(2));
assert!(records[2].excluded_from_catalog);
```

Fixture records must cover:

```text
06.129: single-point MultiPoint, new_ograda="0", new_lulki="0", valid age "3 до 12"
06.130: single-point MultiPoint, new_ograda="1", new_lulki="2", combined age "0 до 3; 3 до 12"
06.131: new_label="не се показват на картата"
```

Add parser assertions that `null`, `""`, `"―"`, and `"—"` are unknown; malformed `"12 до 1"` is unknown; missing `nobekt_new` and multi-coordinate geometry return errors.

- [ ] **Step 2: Run tests to verify they fail**

Run:

```bash
cargo test --manifest-path backend/Cargo.toml enrichment::tests
```

Expected: FAIL because `enrichment` and `normalize_sofiaplan` do not exist.

- [ ] **Step 3: Define source types and SofiaPlan normalization**

Create `backend/src/enrichment.rs` with these interfaces:

```rust
pub const SOFIAPLAN_URL: &str = "https://api.sofiaplan.bg/datasets/5";
pub const SOFIAPLAN_DATASET_PAGE: &str = "https://urbandata.sofia.bg/dataset/playgrounds";
pub const SOFIAPLAN_OBSERVED_AT: &str = "2019-04-18T00:00:00Z";
pub const MATCH_RADIUS_METERS: f64 = 15.0;

#[derive(Clone, Copy, Debug, Eq, PartialEq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum SourceKind { OpenStreetMap, SofiaPlan }

#[derive(Clone, Copy, Debug, Eq, PartialEq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum DateMeaning { Observation, SourceUpdate }

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

pub fn normalize_sofiaplan(body: &str) -> Result<Vec<SourcePlayground>>;
```

Use `nobekt_new` as the stable external ID. Prefer `new_*` properties, then their matching old property. Apply these exact mappings:

```text
address              new_mestopolozh / mestopolozh_old
ownership            new_vids_kk / vids_kk_old
ownership_detail     new_sobstvenos / sobstvenost_old
min_age,max_age      conservative parse of new_vazrgrupi / vazr_old
fenced               new_ograda / ograda: 0=false, positive=true
ordinance_compliant  new_naredba1 / naredba1_old: да=true, не=false
municipal_status     new_label / new_meropr / meropr_old
repairs              new_meropr / meropr_old
notes                new_zabelezhka / zabelezhka
swing count          new_lulki
climbing_frame count new_katerushka
sandpit count        new_pyasachnik
seesaw count         sum known new_klatush_0_3 and new_klatushka_3_12
```

Keep unmapped fields in `raw_data`. Do not map `plost` to surface. Treat `new_label="не се показват на картата"` as `excluded_from_catalog=true` while retaining the source record.

For an invalid optional value, store no normalized value and emit `tracing::warn!` with source, external ID, field name, and rejected value. Never include the full raw record in the log.

- [ ] **Step 4: Refactor OSM normalization to retain raw provenance**

Modify `backend/src/importer.rs` so each OSM playground root produces a `SourcePlayground`. Store raw JSON as:

```json
{
  "root": { "type": "way", "id": 10, "tags": {} },
  "equipment": [
    { "type": "node", "id": 11, "tags": { "playground": "slide" } }
  ]
}
```

Parse `surface`, `access`, `fee`, ages, equipment, and only `wikimedia_commons=File:...` photo references. Preserve the current OSM ID format and source-update timestamp. Remove arbitrary `image=*` URLs from canonical photo candidates.

- [ ] **Step 5: Run focused tests**

Run:

```bash
cargo test --manifest-path backend/Cargo.toml enrichment::tests
cargo test --manifest-path backend/Cargo.toml importer::tests
```

Expected: PASS, including existing geometry and equipment tests.

- [ ] **Step 6: Commit**

```bash
git add backend/src/enrichment.rs backend/src/lib.rs backend/src/importer.rs backend/tests/fixtures/sofiaplan-playgrounds.geojson
git commit -m "feat(import): normalize playground source records"
```

---

### Task 4: Match and merge canonical playgrounds

**Files:**
- Modify: `backend/src/enrichment.rs`

**Interfaces:**
- Consumes: `&[SourcePlayground]` for OSM and SofiaPlan.
- Produces: `match_sources`, `merge_catalog`, `CanonicalPlayground`, and `SourceLink` for persistence and GraphQL.

- [ ] **Step 1: Write failing matching tests**

Add tests with Sofia coordinates that assert:

```rust
let result = match_sources(&osm, &sofiaplan);
assert_eq!(result.clear_pairs.len(), 1);
assert_eq!(result.clear_pairs[0].osm_id, "node/1");
assert_eq!(result.clear_pairs[0].sofia_id, "06.129");
assert!((result.clear_pairs[0].distance_meters - 14.9).abs() < 0.05);
assert!(result.ambiguous_source_ids.contains(&"node/2".into()));
```

The fixtures must prove all five review cases:

```text
14.9 m, exactly one candidate each: merge
15.0 m boundary, exactly one candidate each: merge
15.1 m: keep separate
one OSM record with two SofiaPlan candidates inside 15 m: keep all separate
one SofiaPlan record with two OSM candidates inside 15 m: keep all separate
```

Add merge tests for newest effective date, equal-date fallback, missing values, explicit `false`, explicit equipment `0`, stable IDs, and excluded SofiaPlan records.

- [ ] **Step 2: Run tests to verify they fail**

Run:

```bash
cargo test --manifest-path backend/Cargo.toml enrichment::tests::matching
cargo test --manifest-path backend/Cargo.toml enrichment::tests::merging
```

Expected: FAIL because matching and canonical merge functions do not exist.

- [ ] **Step 3: Implement the minimal matching and merge interfaces**

Add:

```rust
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
    pub source_values: Vec<SourceValue>,
}

#[derive(Clone, Debug, Default)]
pub struct MergeResult {
    pub playgrounds: Vec<CanonicalPlayground>,
    pub source_links: Vec<SourceLink>,
}

pub fn match_sources(osm: &[SourcePlayground], sofia: &[SourcePlayground]) -> MatchResult;
pub fn merge_catalog(osm: &[SourcePlayground], sofia: &[SourcePlayground], matches: &MatchResult) -> MergeResult;
```

Use `geo::{Distance, Haversine, Point}`. Build every cross-source candidate at `distance <= MATCH_RADIUS_METERS`, count candidates on both sides, and merge only pairs whose two counts are both one. Do not invent a fuzzy name matcher.

The current exports expose no shared identifier. Do not add an unused direct-ID matching path; add one later only if both sources publish a trustworthy shared value.

For canonical IDs, retain the OSM ID for matched and OSM-only records; use `sofiaplan/<external_id>` for SofiaPlan-only records. Keep a selected equipment count of `0` in `source_values`, but omit it from positive `capabilities` and `equipment_counts`. Sort canonical records, links, values, capabilities, and photo references deterministically.

- [ ] **Step 4: Run tests**

Run:

```bash
cargo test --manifest-path backend/Cargo.toml enrichment::tests
```

Expected: PASS.

- [ ] **Step 5: Commit**

```bash
git add backend/src/enrichment.rs
git commit -m "feat(import): merge unambiguous playground records"
```

---

### Task 5: Resolve licensed Wikimedia Commons photos

**Files:**
- Create: `backend/src/commons.rs`
- Modify: `backend/src/enrichment.rs`
- Modify: `backend/src/lib.rs`
- Create: `backend/tests/fixtures/commons-imageinfo.json`

**Interfaces:**
- Consumes: direct canonical `File:` titles extracted from OSM.
- Produces: `LicensedPhoto` and `resolve_commons(client, api_url, titles)` for Task 7.

- [ ] **Step 1: Write failing metadata tests**

Create tests that parse a fixture containing one CC BY-SA file, one public-domain file, one CC BY-NC file, one missing licence, one `NonFree=true` file, and HTML-formatted attribution.

Assert:

```rust
let requested = vec![
    "File:CC-BY-SA.jpg".to_owned(),
    "File:Public-domain.jpg".to_owned(),
    "File:NC.jpg".to_owned(),
    "File:Missing-license.jpg".to_owned(),
    "File:Non-free.jpg".to_owned(),
];
let result = parse_imageinfo(
    include_str!("../tests/fixtures/commons-imageinfo.json"),
    &requested,
)?;
assert_eq!(result.accepted.len(), 2);
assert_eq!(result.rejected, 3);
assert_eq!(result.accepted[0].photo.license, "CC BY-SA 4.0");
assert!(!result.accepted[0].photo.attribution.contains('<'));
assert!(result.accepted[0].photo.source_url.starts_with("https://commons.wikimedia.org/"));
```

- [ ] **Step 2: Run tests to verify they fail**

Run:

```bash
cargo test --manifest-path backend/Cargo.toml commons::tests
```

Expected: FAIL because `commons` does not exist.

- [ ] **Step 3: Implement metadata parsing and batched fetch**

Define:

```rust
pub const COMMONS_API_URL: &str = "https://commons.wikimedia.org/w/api.php";

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct LicensedPhoto {
    pub url: String,
    pub author: String,
    pub license: String,
    pub license_url: String,
    pub attribution: String,
    pub source_url: String,
}

#[derive(Clone, Debug, PartialEq)]
pub struct ResolvedPhoto {
    pub requested_title: String,
    pub photo: LicensedPhoto,
}

#[derive(Clone, Debug, Default, PartialEq)]
pub struct PhotoResolution {
    pub accepted: Vec<ResolvedPhoto>,
    pub rejected: usize,
}

pub async fn resolve_commons(
    client: &reqwest::Client,
    api_url: &str,
    titles: &[String],
) -> Result<PhotoResolution>;

pub fn parse_imageinfo(body: &str, requested_titles: &[String]) -> Result<PhotoResolution>;
```

After defining `LicensedPhoto`, extend both source and canonical records in `enrichment.rs`:

```rust
// SourcePlayground
pub photos: Vec<LicensedPhoto>,

// CanonicalPlayground
pub photo_urls: Vec<String>,
pub photos: Vec<LicensedPhoto>,
```

Batch at most 50 titles per request. Request `url|canonicaltitle|mime|mediatype|size|timestamp|extmetadata`, `iiurlwidth=1200`, and only the approved metadata fields. Follow the API's `normalized` and `redirects` mappings so each accepted result retains the exact `requested_title` used by OSM. Accept public domain, CC0, CC BY, and CC BY-SA. Reject `NC`, `ND`, `NonFree=true`, missing URL, missing licence, or required attribution with no usable attribution text.

Convert metadata HTML to text before storage with a small state-machine that discards characters between `<` and `>`; never return the original HTML. The frontend still uses `textContent`.

Use this exact no-dependency helper, then trim and reject an empty result when attribution is required:

```rust
fn metadata_text(value: &str) -> String {
    let mut in_tag = false;
    value
        .chars()
        .filter(|character| match character {
            '<' => { in_tag = true; false }
            '>' => { in_tag = false; false }
            _ => !in_tag,
        })
        .collect::<String>()
        .trim()
        .to_owned()
}
```

- [ ] **Step 4: Run tests**

Run:

```bash
cargo test --manifest-path backend/Cargo.toml commons::tests
```

Expected: PASS.

- [ ] **Step 5: Commit**

```bash
git add backend/src/commons.rs backend/src/enrichment.rs backend/src/lib.rs backend/tests/fixtures/commons-imageinfo.json
git commit -m "feat(import): resolve licensed Commons photos"
```

---

### Task 6: Persist source, link, and canonical snapshots atomically

**Files:**
- Modify: `backend/src/importer.rs`
- Modify: `backend/tests/importer_postgis.rs`

**Interfaces:**
- Consumes: neighborhoods, `SourcePlayground`, `CanonicalPlayground`, and `SourceLink`.
- Produces: one atomic database snapshot and `SnapshotCounts` for Task 7.

- [ ] **Step 1: Extend failing PostGIS integration tests**

Add assertions after an import for:

```rust
assert_eq!(count(&pool, "source_playgrounds").await, 6);
assert_eq!(count(&pool, "playground_source_links").await, 5);
assert_eq!(count(&pool, "playgrounds").await, 4);
```

Query the stored raw SofiaPlan JSON, `primary_source`, `source_values`, match method/distance, and excluded raw record. Run the same import twice and assert identical counts.

Add a rollback test that seeds a previous source and canonical record, creates a temporary trigger that raises on canonical insert, runs replacement, and asserts both previous rows remain after failure.

- [ ] **Step 2: Run integration tests to verify they fail**

Run:

```bash
cargo test --manifest-path backend/Cargo.toml --test importer_postgis
```

Expected: FAIL because source and link snapshots are not persisted.

- [ ] **Step 3: Extend the transaction boundary**

Change the persistence interface to:

```rust
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct SnapshotCounts {
    pub source_records: u64,
    pub neighborhoods: u64,
    pub playgrounds: u64,
    pub memberships: u64,
}

pub async fn replace_snapshot(
    pool: &PgPool,
    neighborhoods: &[Neighborhood],
    source_records: &[SourcePlayground],
    playgrounds: &[CanonicalPlayground],
    source_links: &[SourceLink],
) -> Result<SnapshotCounts>;
```

Inside the existing transaction, create temporary staging tables with `LIKE ... INCLUDING ALL`, stage every input, then replace in this order:

```text
delete playground_neighborhoods
delete playground_source_links
delete playgrounds
delete source_playgrounds
delete neighborhoods
insert neighborhoods
insert source_playgrounds
insert playgrounds
insert playground_source_links
rebuild playground_neighborhoods with ST_Covers
commit
```

Serialize photos and source values through `serde_json::to_value`. Derive legacy `photo_urls` from accepted structured photos. Preserve a valid primary `source_url` for old clients.

- [ ] **Step 4: Run integration tests**

Run:

```bash
cargo test --manifest-path backend/Cargo.toml --test importer_postgis
```

Expected: PASS, including the forced database rollback case.

- [ ] **Step 5: Commit**

```bash
git add backend/src/importer.rs backend/tests/importer_postgis.rs
git commit -m "feat(import): replace enriched catalog atomically"
```

---

### Task 7: Wire one-time fetching and operator command

**Files:**
- Modify: `backend/src/importer.rs`
- Modify: `backend/src/bin/import-playgrounds.rs`
- Modify: `backend/tests/importer_postgis.rs`
- Modify: `compose.yaml`
- Modify: `.env.example`

**Interfaces:**
- Consumes: `OVERPASS_URL`, `SOFIAPLAN_URL`, `COMMONS_API_URL`, `NEIGHBORHOODS_PATH`.
- Produces: one repeatable command that fetches both primary sources, treats Commons as optional enrichment, merges, persists, and prints counts.

- [ ] **Step 1: Add failing two-source fixture-server tests**

Extend the existing Axum fixture server with `/sofiaplan`, `/commons`, malformed/empty SofiaPlan routes, and a failing Commons route. Assert:

```rust
let counts = run_import(&pool, &ImportEndpoints {
    overpass_url: format!("{base_url}/valid"),
    sofiaplan_url: format!("{base_url}/sofiaplan"),
    commons_api_url: format!("{base_url}/commons"),
}, neighborhoods).await.unwrap();
assert_eq!(counts.sofia_source_records, 3);
assert_eq!(counts.rejected_photos, 0);
```

Assert failed/malformed/empty primary sources preserve the previous catalog. Assert failed Commons fetch still commits the primary catalog with zero photos and a rejected-photo/warning count.

- [ ] **Step 2: Run tests to verify they fail**

Run:

```bash
cargo test --manifest-path backend/Cargo.toml --test importer_postgis
```

Expected: FAIL because `ImportEndpoints` and two-source orchestration do not exist.

- [ ] **Step 3: Implement fetch orchestration**

Add:

```rust
pub struct ImportEndpoints {
    pub overpass_url: String,
    pub sofiaplan_url: String,
    pub commons_api_url: String,
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

pub async fn run_import(
    pool: &PgPool,
    endpoints: &ImportEndpoints,
    neighborhoods_path: &Path,
) -> Result<ImportCounts>;
```

Use one `reqwest::Client`. Fetch and validate OSM and SofiaPlan before database work. Resolve Commons titles after both primary sources normalize, index accepted photos by `ResolvedPhoto.requested_title`, and attach them to the referencing OSM `SourcePlayground` before merge. Catch Commons resolution errors, record warnings/rejected counts, and continue with no affected photos. Then match, merge, and call `replace_snapshot` once.

Populate `ImportCounts` from normalization, matching, photo resolution, and the returned `SnapshotCounts`. Count each ambiguous source record once even if it participates in multiple candidate edges.

- [ ] **Step 4: Wire environment variables and output**

In `backend/src/bin/import-playgrounds.rs`, default to the constants from `importer`, `enrichment`, and `commons`, then print one summary line containing every count. Add `SOFIAPLAN_URL` and `COMMONS_API_URL` pass-through entries to `compose.yaml` and `.env.example`; do not add scheduling.

- [ ] **Step 5: Run importer tests**

Run:

```bash
cargo test --manifest-path backend/Cargo.toml importer::tests
cargo test --manifest-path backend/Cargo.toml --test importer_postgis
```

Expected: PASS.

- [ ] **Step 6: Commit**

```bash
git add backend/src/importer.rs backend/src/bin/import-playgrounds.rs backend/tests/importer_postgis.rs compose.yaml .env.example
git commit -m "feat(import): fetch OSM and SofiaPlan once"
```

---

### Task 8: Expose enriched GraphQL details compatibly

**Files:**
- Modify: `backend/src/graphql.rs`
- Modify: `backend/tests/postgis.rs`

**Interfaces:**
- Consumes: enriched canonical columns, source links, retained source records, structured `photos`, and `source_values`.
- Produces: additive GraphQL contract used by Task 9; existing `photoUrls` and `source` remain available.

- [ ] **Step 1: Write the failing GraphQL integration query**

Extend the enriched seed in `backend/tests/postgis.rs`, then query:

```graphql
query {
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
}
```

Assert deterministic OSM-before-SofiaPlan source ordering, structured photo metadata, legacy `photoUrls`, selected and older values, explicit `false`, and correct `OBSERVATION` versus `SOURCE_UPDATE` enums.

- [ ] **Step 2: Run tests to verify they fail**

Run:

```bash
cargo test --manifest-path backend/Cargo.toml --test postgis graphql_queries_use_postgis_and_combine_catalog_filters -- --exact
```

Expected: FAIL with unknown enriched GraphQL fields.

- [ ] **Step 3: Add GraphQL types and row mapping**

Add these schema types near the existing `SourceMetadata`:

```rust
#[derive(Clone, Copy, Debug, Eq, Enum, PartialEq)]
pub enum DataSource { OpenStreetMap, SofiaPlan }

#[derive(Clone, Copy, Debug, Eq, Enum, PartialEq)]
pub enum SourceDateMeaning { Observation, SourceUpdate }

#[derive(SimpleObject)]
pub struct Photo {
    pub url: String,
    pub author: String,
    pub license: String,
    pub license_url: String,
    pub attribution: String,
    pub source_url: String,
}

#[derive(SimpleObject)]
pub struct FieldSourceValue {
    pub field: String,
    pub value: String,
    pub source: DataSource,
    pub source_id: ID,
    pub date: Option<String>,
    pub date_meaning: Option<SourceDateMeaning>,
    pub selected: bool,
}
```

Extend `SourceMetadata` with `kind` and `date_meaning`. Extend `Playground` with all query fields above. Parse JSON through typed serde structs, reject malformed stored provenance with the existing generic `playground data is temporarily unavailable` error, and never expose raw JSON.

Extend the shared SELECT with deterministic JSON aggregation from `playground_source_links` and `source_playgrounds`. Use OSM as primary source when linked; otherwise use SofiaPlan. Keep the existing `source` field and OSM attribution behavior for old records. Build source metadata with these exact values:

```text
OpenStreetMap
  attribution: © OpenStreetMap contributors
  license: ODbL-1.0
  URL: https://www.openstreetmap.org/<external_id>

SofiaPlan
  attribution: SofiaPlan
  license: Reuse terms need confirmation
  URL: https://urbandata.sofia.bg/dataset/playgrounds
```

Do not claim a settled SofiaPlan licence. `updatedAt` carries the source date, and `dateMeaning` distinguishes `SOURCE_UPDATE` from `OBSERVATION`.

- [ ] **Step 4: Run GraphQL tests**

Run:

```bash
cargo test --manifest-path backend/Cargo.toml graphql::tests
cargo test --manifest-path backend/Cargo.toml --test postgis
```

Expected: PASS, including existing filters, pagination, error hiding, and legacy detail fields.

- [ ] **Step 5: Commit**

```bash
git add backend/src/graphql.rs backend/tests/postgis.rs
git commit -m "feat(api): expose sourced playground details"
```

---

### Task 9: Render the approved popup and details flow

**Files:**
- Create: `src/playground-format.js`
- Create: `test/playground-format.test.js`
- Modify: `src/main.js`
- Modify: `src/styles.css`
- Modify: `TEST_CASES.md`

**Interfaces:**
- Consumes: Task 8 GraphQL names exactly as defined.
- Produces: approved compact preview, full details, source warnings/history, licensed-photo attribution, and tested formatting helpers.

- [ ] **Step 1: Write failing pure-format tests**

Create `test/playground-format.test.js` with `node:test` and `node:assert/strict`. Cover:

```javascript
assert.equal(formatAge(3, 12), "3–12 years");
assert.equal(formatAge(null, null), "Age not recorded");
assert.equal(formatKnown(false, "Yes", "No"), "No");
assert.equal(formatKnown(null, "Yes", "No"), "Unknown");
assert.equal(formatEquipment([{ capability: "SWING", count: 2 }]), "2 swings");
assert.equal(formatSourceDate("2019-04-18T00:00:00Z", "OBSERVATION"), "Observed 18 Apr 2019");
assert.equal(formatSourceDate("2026-09-22T10:00:00Z", "SOURCE_UPDATE"), "Source updated 22 Sep 2026");
assert.equal(formatNeighborhoods([{ name: "Lozenets" }, { name: "South Park" }]), "Lozenets · South Park");
```

Prefix each `test(...)` name with `playground formatting:` so the focused command below selects the new tests.

- [ ] **Step 2: Run tests to verify they fail**

Run:

```bash
npm test -- --test-name-pattern="playground formatting"
```

Expected: FAIL because `src/playground-format.js` does not exist.

- [ ] **Step 3: Add the minimal formatting module**

Export only these helpers:

```javascript
const EQUIPMENT_NAMES = {
  CLIMBING_FRAME: ["climbing frame", "climbing frames"],
  PLAYHOUSE: ["playhouse", "playhouses"],
  ROUNDABOUT: ["roundabout", "roundabouts"],
  SANDPIT: ["sandpit", "sandpits"],
  SEESAW: ["seesaw", "seesaws"],
  SLIDE: ["slide", "slides"],
  SPRINGY: ["spring rider", "spring riders"],
  SWING: ["swing", "swings"],
};

export function formatAge(minAge, maxAge) {
  if (minAge != null && maxAge != null) return `${minAge}–${maxAge} years`;
  if (minAge != null) return `${minAge}+ years`;
  if (maxAge != null) return `Up to ${maxAge} years`;
  return "Age not recorded";
}

export function formatKnown(value, yes, no) { return value == null ? "Unknown" : value ? yes : no; }

export function formatEquipment(items) {
  if (!items?.length) return "No equipment recorded";
  return items.map(({ capability, count }) => {
    const names = EQUIPMENT_NAMES[capability] ?? [capability.toLowerCase().replaceAll("_", " "), capability.toLowerCase().replaceAll("_", " ")];
    if (count == null) return `${names[0]} (quantity unknown)`;
    return `${count} ${count === 1 ? names[0] : names[1]}`;
  }).join(", ");
}

export function formatSourceDate(value, meaning) {
  if (!value) return "Date unknown";
  const date = new Intl.DateTimeFormat("en-GB", {
    day: "numeric", month: "short", year: "numeric", timeZone: "UTC",
  }).format(new Date(value));
  return `${meaning === "OBSERVATION" ? "Observed" : "Source updated"} ${date}`;
}

export function formatNeighborhoods(items) { return items.length ? items.map(({ name }) => name).join(" · ") : "Neighborhood unknown"; }
export function selectedSourceValue(items, field) { return items.find((item) => item.field === field && item.selected) ?? null; }
```

Keep functions pure. Move existing age/equipment formatting from `main.js` rather than duplicating it.

- [ ] **Step 4: Extend GraphQL queries and DOM rendering**

Update the summary query with neighborhoods, structured photos, municipal status, and the selected municipal-status provenance. Update the detail query with every Task 8 field.

In `playgroundPreview`:

```text
licensed photo or visible "No photo yet"
name fallback
all neighborhood names
age and equipment chips
historical municipal status plus accurate source date and "May be outdated"
existing "No ratings yet"
keyboard-accessible "View details" button
```

Pass the marker to `playgroundPreview`. Keep the popup open while its content has pointer hover or keyboard focus. Defer marker `mouseout`/`blur` synchronization with one zero-delay task, then close only when neither marker nor popup content owns preview. Clear that state and timer in `clearPlaygroundPreview`. The button stops map-event propagation and calls the existing `openDetails(playground, marker)`.

In `renderDetails`, use DOM creation plus `textContent` for:

```text
structured gallery with attribution and failed-image fallback
address, coordinates, and OpenStreetMap directions link
semantic definition list for age, equipment, surface, fence, ownership, access, and fee
historical municipal status, repairs, Ordinance 1 compliance, and notes
selected and older conflicting source values
existing rating/review empty states
all source links, date meanings, attribution, and licences
```

Do not add the mockup's Share control or a geolocation prompt. Keep `openDetails` abort handling, marker selection, modal focus trap, Escape/Back, and map-click behavior unchanged.

- [ ] **Step 5: Add focused styles and manual cases**

Extend existing popup/detail CSS with chips, warnings, definition-list rows, actions, source history, photo attribution, and the visible empty-photo state. Warning meaning must remain readable without color. Preserve the 320-pixel full-screen detail rule.

Add `TEST_CASES.md` cases for complete enrichment, no photo, unknown/false values, stale municipal status wording, conflicting sources, Commons attribution, directions, multiple neighborhoods, and a clickable popup action through pointer and keyboard transitions.

- [ ] **Step 6: Run frontend checks**

Run:

```bash
npm test
npm run build
```

Expected: all Node tests pass and Vite build exits zero.

- [ ] **Step 7: Browser-check the approved flow**

Run `npm run start:local`. At 320, 768, and 1280 CSS pixels, verify hover, focus, popup-button click, Enter, Space, Back, Escape, failed photo, missing values, warnings, attribution, console, and network calls. Confirm no browser request targets SofiaPlan, Overpass, or Commons.

- [ ] **Step 8: Commit**

```bash
git add src/playground-format.js test/playground-format.test.js src/main.js src/styles.css TEST_CASES.md
git commit -m "feat(map): show sourced playground details"
```

---

### Task 10: Document source quirks, verify a real import, and archive OpenSpec

**Files:**
- Create: `docs/gotchas/sofiaplan.md`
- Modify: `docs/gotchas/README.md`
- Modify: `docs/gotchas/openstreetmap.md`
- Modify: `LOCAL_SETUP.md`
- Modify: `public/data/README.md`
- Modify: `openspec/changes/enrich-playground-data/tasks.md`
- Modify through archive: `openspec/specs/playground-discovery/spec.md`
- Modify through archive: `openspec/specs/sofia-neighborhood-map/spec.md`

**Interfaces:**
- Consumes: completed Tasks 1–9.
- Produces: operator documentation, verified local catalog, synchronized main specs, archived change, and final review evidence.

- [ ] **Step 1: Document exact operational behavior**

Document in `LOCAL_SETUP.md`:

```text
docker compose run --rm api import-playgrounds
OVERPASS_URL, SOFIAPLAN_URL, COMMONS_API_URL, NEIGHBORHOODS_PATH overrides
one-time/manual operation only
summary count meanings
failure preserves current catalog
```

Create `docs/gotchas/sofiaplan.md` with symptom, cause, fix, and verification for:

```text
GeoJSON served as text/plain
single-coordinate MultiPoint geometry
2019-04-18 source-wide observation date
0 versus unknown markers
records marked "не се показват на картата" retained as source but excluded canonically
no surface or current-condition field
free-text malformed ages
licence metadata mismatch requiring clarification before commercial launch
```

Update `docs/gotchas/openstreetmap.md` for direct Commons references and source-update date semantics. Update both gotcha indexes/data docs.

- [ ] **Step 2: Run all automated gates**

Run:

```bash
cargo fmt --manifest-path backend/Cargo.toml --check
cargo clippy --manifest-path backend/Cargo.toml --all-targets --all-features -- -D warnings
cargo test --manifest-path backend/Cargo.toml
npm test
npm run build
npm audit
openspec validate enrich-playground-data --strict --no-interactive
git diff --check
```

Expected: every command exits zero; `npm audit` reports no unresolved vulnerability.

- [ ] **Step 3: Run one real import**

Run:

```bash
docker compose up -d db
docker compose run --rm api import-playgrounds
```

Expected: non-zero OSM, SofiaPlan, and canonical counts; source count includes every current SofiaPlan record; excluded/ambiguous counts are reported; command exits zero.

Query Postgres and verify:

```sql
SELECT source, count(*) FROM source_playgrounds GROUP BY source ORDER BY source;
SELECT count(*) FROM playground_source_links;
SELECT count(*) FROM playgrounds;
SELECT count(*) FROM playgrounds WHERE jsonb_array_length(photos) > 0;
```

- [ ] **Step 4: Request whole-branch review**

Invoke `superpowers:requesting-code-review`. Require review against the design, this plan, current OpenSpec change, source/licence handling, matching ambiguity, transaction rollback, GraphQL compatibility, accessibility, and the approved mockup. Fix all valid findings and rerun the affected focused tests.

- [ ] **Step 5: Complete and archive OpenSpec**

Mark every task in `openspec/changes/enrich-playground-data/tasks.md` complete. Invoke `$openspec-archive-change` for `enrich-playground-data`; it must sync the delta specs into the main specs before moving the change to the archive. Then run:

```bash
openspec validate --all --strict --no-interactive
```

Expected: change moves under `openspec/changes/archive/`, main specs contain the approved behavior, and all specs validate.

- [ ] **Step 6: Run final verification after archive**

Invoke `superpowers:verification-before-completion`, then rerun:

```bash
cargo test --manifest-path backend/Cargo.toml
npm test
npm run build
git diff --check
git status --short
```

Expected: tests/build pass; only intended implementation, documentation, and archived OpenSpec files are changed.

- [ ] **Step 7: Commit final documentation and archive**

```bash
git add LOCAL_SETUP.md TEST_CASES.md public/data/README.md docs/gotchas openspec
git commit -m "docs: archive playground data enrichment"
```
