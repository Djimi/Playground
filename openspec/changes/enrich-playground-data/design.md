# Design

## Context

The existing backend is a Rust/SQLx/PostgreSQL service. `backend/src/importer.rs` parses an Overpass response into typed playgrounds, replaces the `playgrounds`, `neighborhoods`, and `playground_neighborhoods` snapshot in one transaction, and already preserves the catalog when validation or staging fails. `backend/src/graphql.rs` exposes a read-only schema backed directly by that catalog, while `src/main.js` requests the catalog once and renders compact Leaflet previews plus a focus-managed details dialog.

The approved design in `docs/superpowers/specs/2026-09-22-playground-data-enrichment-design.md` and the delta specs are binding. The browser continues to read the local GraphQL API; it does not call any source API.

## Goals / Non-Goals

**Goals:**

- Extend the existing operator import into one validated OpenStreetMap + SofiaPlan snapshot import.
- Keep current source records inspectable, merge only unambiguous spatial pairs, and expose field-level provenance and conflicts.
- Store only directly referenced, verified reusable Commons photos.
- Replace the existing catalog, mappings, canonical records, and neighborhood memberships atomically while retaining the prior usable snapshot on failure.
- Add GraphQL fields without removing `photoUrls` or existing source metadata, then render those fields in the existing popup and details flow.
- Reuse the existing Rust, SQLx/PostgreSQL, GraphQL, Vite, and Leaflet stack and its test conventions.

**Non-Goals:**

- No scheduler, deployment selection, recurring refresh, historical source-version store, visitor editing, authentication, ratings/reviews, Google, Mapillary, social-media, or browser-side source integration.
- No general audit/event-sourcing system; field history is limited to the current source values needed to explain a canonical field.

## Decisions

### 1. Keep one operator command and extend the current importer

The existing `import-playgrounds` command remains the entry point. It fetches both primary datasets, validates and normalizes them, resolves directly referenced Commons metadata, performs matching and field selection, then commits one replacement snapshot.

Alternative considered: separate commands or a background refresh service. That would create inconsistent intermediate catalogs and violates the one-time operator-run scope.

### 2. Separate current source rows from the canonical query model

Add current source-record storage keyed by `(source, external_id)`, source-to-canonical links carrying match method and distance, field-level source values/history, and structured photo metadata. Keep the existing `playgrounds` table as the canonical query model, adding the typed fields needed by GraphQL and the UI. Preserve existing OSM-backed canonical IDs unchanged; use the OSM `<type>/<id>` for matched and OSM-only records, and `sofiaplan/<nobekt_new>` for SofiaPlan-only records. Store original normalized source JSON rather than discarding fields that are not currently displayed.

Alternative considered: merge source JSON directly into `playgrounds` or retain every import version. The former loses provenance; the latter is a historical data lake explicitly outside scope.

### 3. Match in memory with a named 15-metre threshold

Normalize both sources to representative points, build opposite-source candidates within the single `MATCH_RADIUS_METERS = 15` constant, and merge only when both records have exactly one candidate. Unmatched records become independent canonical records; ambiguous candidates stay separate. Persist each accepted link and distance so an operator can inspect the decision.

Alternative considered: fuzzy names or nearest-neighbour-only matching. Names are inconsistent across languages and nearest-neighbour matching can silently combine adjacent playgrounds; the conservative one-to-one rule is safer.

### 4. Merge fields by effective date while retaining every value

Represent a missing value separately from explicit `false`, `0`, or `free`. For each field, choose the newest non-missing effective value; use the source-record update/observation date when a field date is absent, then use the approved source-specific fallback when dates tie or are absent. Persist all source values with source identity, date meaning, and selected status. SofiaPlan's dataset-wide 18 April 2019 date is presented as an observation date, while an OSM element timestamp is presented as a source update date.

Alternative considered: source-priority-only or last-import-wins. Both hide conflicts and make old municipal data appear current.

### 5. Treat Commons verification as a photo-level best effort

Only direct Commons file references are resolved. A Commons response must provide a public-domain, CC0, CC BY, or CC BY-SA licence, author, attribution, original file page, and usable URL; NC, ND, non-free, missing, and unknown licences are rejected. A failed lookup or unsupported licence omits only that photo and records an unavailable/rejected outcome; it does not abort an otherwise valid playground import. Nearby geotagged images are never inferred to depict a playground.

Alternative considered: fail the whole import on any photo error or attach nearby imagery. Either choice makes the catalog brittle or presents unverified content.

### 6. Stage and swap the whole catalog inside the existing transaction pattern

Validate complete primary responses and normalized required fields before opening the replacement transaction. Stage source records, links, canonical rows, photos, neighborhoods, and memberships in temporary tables or equivalent transaction-local rows. Delete and repopulate the current snapshot only after staging succeeds, then commit once. Any fetch, validation, Commons-independent database, or commit failure rolls back without changing the prior snapshot.

Alternative considered: upsert each source or canonical row independently. That would expose mixed-source snapshots and cannot guarantee catalog preservation on mid-import failure.

### 7. Extend GraphQL and the UI additively

Add nullable/empty GraphQL fields for enriched facts, structured photos, all linked sources, and field history. Keep the existing `source` and `photoUrls` fields available. Update the existing query documents and rendering helpers in `src/main.js`; retain current marker state, modal focus, empty ratings/reviews, photo fallback, mobile layout, and attribution behavior. Historical municipal data gets a visible text warning rather than color-only signaling.

Alternative considered: a versioned GraphQL endpoint or a second UI flow. Additive fields let existing clients continue working and avoid duplicating the established focus-management behavior.

## Risks / Trade-offs

- [Risk] SofiaPlan records have a source-wide 2019 observation date and may be stale. → Label the date accurately and show `May be outdated`; preserve source history so users can distinguish it from OSM updates.
- [Risk] A 15-metre threshold can leave true matches separate or merge a rare nearby pair. → Require mutual one-to-one candidates, persist match distance, and keep ambiguous records separate.
- [Risk] Source exports can be incomplete or malformed. → Reject missing required identifiers/locations and unexpected empty primary datasets before replacement; retain the prior snapshot.
- [Risk] Commons metadata or network access can fail. → Treat photo resolution as best effort and show an explicit unavailable-photo state.
- [Risk] Additive schema and migration changes can break existing fixtures if defaults are missing. → Use nullable/defaulted columns, preserve `photoUrls` and source fields, and extend existing Rust/PostGIS/GraphQL fixtures before changing UI assertions.
- [Risk] SofiaPlan reuse terms need clarification before a commercial launch. → Keep the source attribution visible and obtain written reuse clarification before that launch; this change does not add a licensing assumption.

## Migration Plan

1. Add additive PostgreSQL migrations for source records, canonical enrichment fields, source links/field values, and structured photos with safe defaults for the current catalog.
2. Keep the current OSM-only rows queryable while code and fixtures are updated; existing `photoUrls` and `source` remain populated for legacy rows.
3. Run the operator import once against both sources and verify counts, matches, rejected records/photos, memberships, and attribution.
4. Roll back by stopping before the replacement transaction or restoring the previous database snapshot; any failed run already leaves the previous usable catalog untouched.
5. Remove no existing fields or clients. A later cleanup of legacy-only storage requires a separately reviewed compatibility change.
