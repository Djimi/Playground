# Playground Data Enrichment Design

## Status

Approved in conversation on 22 September 2026.

## Purpose

Populate the Sofia playground catalog once from public sources so the map can show useful details without manual catalog maintenance. Keep each source record for debugging, expose an honest merged view, and leave recurring refreshes and visitor contributions for later work.

## Goals

- Import current playground records from OpenStreetMap and SofiaPlan in one operator-run batch.
- Keep the current record from each source separately.
- Merge only clear cross-source matches.
- Select the newest non-missing value for each field while retaining provenance and the alternative source value.
- Show useful compact information in the map popup and full information in the existing details view.
- Show only photos with verified reusable licences.
- Preserve the last usable catalog if an import fails.

## Non-goals

- Scheduled or on-demand refreshes.
- Deployment or scheduler selection.
- Google Places, Google Photos, Google Street View, Mapillary, or social-media ingestion.
- Visitor photo uploads, corrections, authentication, ratings, or reviews.
- Historical versions of source datasets.
- A general audit, event-sourcing, or data-lake system.

The existing `No ratings yet` and `No reviews yet` states remain visible until visitor contributions are implemented.

## Sources

### OpenStreetMap

The existing Overpass importer remains the source for playground identity, geometry, names, equipment, age tags, surface and access tags, source timestamps, and direct Wikimedia Commons references.

- Data licence: ODbL 1.0.
- Required attribution: `© OpenStreetMap contributors` with a link to the OpenStreetMap copyright page.
- An OpenStreetMap element timestamp is a record-edit timestamp, not proof that every field was physically verified on that date. The UI must describe it as a source update date.

### SofiaPlan

The importer downloads the current playground export from the official SofiaPlan API. SofiaPlan supplies municipal address text, ownership, age groups, equipment counts, fencing, Ordinance 1 compliance, historical repair/status labels, and notes. The export has no surface or current-condition field, so the application does not infer either value from SofiaPlan.

- Dataset page: <https://urbandata.sofia.bg/dataset/playgrounds>
- API export: <https://api.sofiaplan.bg/datasets/5>
- The dataset catalog records one source-wide observation date: 18 April 2019. Individual records have no observation timestamp.
- The UI must identify SofiaPlan and show that date beside municipal status, repair, compliance, and note data.
- Before a public commercial launch, obtain written clarification of reuse terms because the dataset metadata and SofiaPlan reuse policy are not fully aligned.

### Wikimedia Commons

Photos are accepted only when a source record directly references a Wikimedia Commons file and the Commons API returns its author, licence, attribution, and original file page. Nearby geotagged photos are not attached automatically because proximity does not prove that a photo depicts the playground.

If licence metadata is absent or unsupported, the importer omits the photo. The UI shows `No photo yet` instead.

## Architecture

```text
OpenStreetMap export ───┐
                       ├─ validate and normalize source records
SofiaPlan export ──────┘
                                  │
                                  ▼
                         conservative matching
                                  │
                    ┌─────────────┴─────────────┐
                    │                           │
               clear match                uncertain match
                    │                           │
                  merge                    keep separate
                    └─────────────┬─────────────┘
                                  │
                                  ▼
                     canonical playground catalog
                                  │
                                  ▼
                     existing GraphQL API and UI
```

The browser never calls OpenStreetMap, SofiaPlan, or Wikimedia Commons. It reads the local catalog through the existing GraphQL API.

## Stored data

### Current source records

`source_playgrounds` keeps one current record per `(source, external_id)`:

- source name and external identifier;
- original source JSON;
- normalized name and representative location used for matching;
- source update or observation date, when supplied;
- local import time.

A successful rerun replaces the current source rows. It does not retain previous source versions.

### Canonical playgrounds

The existing `playgrounds` catalog remains the query model. It keeps typed values needed by search and the UI, including existing location, age, and equipment fields plus:

- address;
- surface;
- fencing;
- ownership;
- access and fee information;
- historical municipal status, repair information, Ordinance 1 compliance, and notes;
- structured licensed photos.

Each merged playground stores structured source values per field. Every source value contains the value, source record, applicable date, date meaning, and whether it won the merge. This preserves both sources without adding a general audit system.

`playground_source_links` records which source records formed each canonical playground, the match method, and match distance. This lets an operator inspect and rebuild a bad merge.

Photos store URL, author, licence, attribution text, and original Commons page. The existing `photoUrls` GraphQL field remains compatible while the UI moves to structured photo metadata.

## Matching

The current exports contain no shared cross-source identifier, so the importer uses spatial matching:

1. Source records become candidates when their representative points are within 15 metres.
2. The importer merges only pairs where each record has exactly one opposite-source candidate inside that radius.
3. If either record has multiple candidates inside the radius, the records remain separate.
4. Unmatched records remain independent canonical playgrounds.

If a trustworthy shared identifier appears in both sources later, it should take precedence over spatial matching in a separately reviewed change.

This deliberately prefers a possible duplicate over combining facts from two different nearby playgrounds. The 15-metre limit is one named importer constant so later evidence can tune it without changing the matching design.

## Field merge rules

For each field:

1. Use a field observation date when present; otherwise use the source-record update date as the value's effective date.
2. Prefer the non-missing value with the newest effective date.
3. If dates are equal or absent, apply the field fallback below.
4. Preserve every source value and identify the selected value.

Missing values never replace known values. Explicit values such as `false`, `0`, or `free` are not missing.

Fallbacks when dates cannot decide:

- OpenStreetMap: identity, name, representative location, mapped equipment, surface, access, fee, and Commons reference.
- SofiaPlan: municipal address, ownership, age group, equipment counts, fencing, historical municipal status, repairs, Ordinance 1 compliance, and notes.

The UI labels dates accurately as observations or source-record updates. It never presents an import time as a verification date.

## Import lifecycle

The operator runs one command during implementation:

```text
fetch both datasets
validate complete responses
normalize current source records
resolve licensed Commons metadata
match and merge
replace source records, links, and canonical records in one transaction
print import summary
```

The command reports source counts, clear matches, unmatched records, ambiguous candidates, rejected records, accepted photos, and rejected photos.

The importer validates both primary datasets before opening the replacement transaction. A successful transaction changes the source and canonical catalogs together.

## Failure handling

- A failed request, source error marker, malformed top-level response, or unexpected empty primary dataset aborts the import.
- A record missing a required identifier or usable location fails validation and aborts the import, matching the existing catalog-preservation contract.
- An invalid optional field becomes unknown and produces a diagnostic warning.
- A Wikimedia Commons failure or unusable licence omits the affected photo without failing the playground import.
- Any database failure rolls back source records, links, and canonical records together.
- When a usable catalog already exists, every failed import leaves it unchanged.

## User interface

The [approved popup and details mockup](assets/playground-popup-details-mockup.html) is the visual implementation reference. Its example values demonstrate hierarchy and states; imported source data supplies the real values.

### Compact pin popup

- Licensed photo or `No photo yet`.
- Recorded name or `Unnamed playground`.
- Neighborhood.
- Distance only when the application already has the visitor's location; this change does not add a location-permission flow.
- Age range.
- Main equipment and known counts.
- Historical municipal status with source date and `May be outdated` warning.
- Existing `No ratings yet` state.
- `View details` action.

### Full details view

- Licensed photo gallery or empty state.
- Address, coordinates, and directions link.
- Full equipment inventory and known counts.
- Age range, surface, fencing, ownership, access, and fee.
- Historical municipal status, repairs, compliance, and notes with source, date, and explicit warning.
- Source values and dates, including an older conflicting value when present.
- Existing `No ratings yet` and `No reviews yet` states.
- OpenStreetMap, SofiaPlan, and photo attribution as applicable.

Unknown optional values display as `Unknown`; the application never infers them.

## Accessibility

- Warning meaning appears in text and does not depend on an icon or color.
- Photo alternative text identifies the playground when known.
- Missing and failed photos retain the existing visible fallback.
- Popup and detail actions remain keyboard accessible.
- The existing mobile-width and focus-management requirements remain in force.

## Verification

Use the existing Rust and frontend test styles. Add focused fixtures and checks for:

- OpenStreetMap and SofiaPlan parsing;
- current source-record replacement without version history;
- clear, unmatched, and ambiguous spatial matches;
- mutual-nearest and 15-metre boundary behavior;
- newest-value selection and field fallback rules;
- explicit false values versus missing values;
- retained source values and match evidence;
- accepted and rejected Commons licences;
- complete rollback after validation or database failure;
- popup and detail rendering with complete, stale, conflicting, and missing data;
- visible textual warnings and required attribution.

## Acceptance criteria

- One operator command imports both primary sources and commits one consistent catalog.
- Both current source records remain inspectable after merging.
- Only clear matches merge; uncertain matches remain separate.
- Every displayed merged value can be traced to a stored source record and correctly described date.
- Only verified reusable photos appear.
- The UI remains useful when names, photos, ages, or other optional fields are absent.
- A failed import never replaces a previously usable catalog.
