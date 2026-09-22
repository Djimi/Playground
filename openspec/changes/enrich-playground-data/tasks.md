# Tasks

## 1. OpenSpec contract

- [x] 1.1 Create the `enrich-playground-data` proposal, design, delta specs, and implementation checklist from the approved design; verify all six planning artifacts exist.
- [x] 1.2 Confirm the current main specs and absence of an active enrichment change with `openspec list` and both `openspec show ... --type spec` commands; verify the planning change with `openspec status --change enrich-playground-data --json`.
- [x] 1.3 Validate the contract with `openspec validate enrich-playground-data --strict --no-interactive` and commit only the planning artifacts as `docs: propose playground data enrichment`.

## 2. Source-retention and canonical schema

- [ ] 2.1 Add the additive migration for current `source_playgrounds`, enriched canonical columns, structured photos/source values, and `playground_source_links`; verify JSON defaults, source constraints, and foreign keys with a focused `#[sqlx::test]`.
- [ ] 2.2 Run `cargo test --manifest-path backend/Cargo.toml --test postgis` and confirm existing catalog queries still pass alongside the new schema assertions.

## 3. Source normalization

- [ ] 3.1 Add typed source-record/value/date models and SofiaPlan normalization for the official GeoJSON export, including single-coordinate `MultiPoint`, source-wide `2019-04-18` observation date, explicit zero/false values, unknown markers, excluded labels, and required-field validation; verify with fixture unit tests.
- [ ] 3.2 Refactor OpenStreetMap normalization to retain traceable raw source data, source-update timestamps, supported enriched tags, and only direct `wikimedia_commons=File:...` references; verify existing importer geometry/equipment tests plus focused enrichment tests.

## 4. Conservative matching and merge

- [ ] 4.1 Implement deterministic one-to-one spatial matching with a named 15-metre threshold, preserving unmatched and ambiguous records separately and retaining match evidence; preserve existing OSM-backed canonical IDs, use OSM `<type>/<id>` for matched and OSM-only records, and use `sofiaplan/<nobekt_new>` for SofiaPlan-only records; verify 14.9 m, exactly 15.0 m, 15.1 m, and one-to-many cases.
- [ ] 4.2 Implement canonical field selection by newest non-missing effective date with source-specific fallback, provenance/history, stable IDs, and explicit false/zero handling; verify merge fixtures and deterministic output ordering.

## 5. Licensed Wikimedia Commons photos

- [ ] 5.1 Add direct Commons metadata resolution and parsing that accepts only public-domain, CC0, CC BY, and CC BY-SA licences, rejects NC, ND, non-free, missing, and unknown licences, and retains normalized redirects, author/attribution text, and the original file page; verify accepted/rejected fixture counts and HTML-free attribution.
- [ ] 5.2 Attach accepted structured photos to source and canonical records without failing the primary import on Commons errors; verify `cargo test --manifest-path backend/Cargo.toml commons::tests`.

## 6. Atomic catalog replacement

- [ ] 6.1 Extend the importer snapshot boundary to stage and replace source records, links, canonical records, neighborhoods, and memberships together, deriving compatible `photoUrls` from accepted structured photos; verify idempotent counts and retained raw/provenance data.
- [ ] 6.2 Force a database failure during replacement and verify the previous source and canonical catalogs remain unchanged with `cargo test --manifest-path backend/Cargo.toml --test importer_postgis`.

## 7. Two-source operator import

- [ ] 7.1 Fetch and validate OpenStreetMap and SofiaPlan through one operator command using existing HTTP/configuration conventions, resolve Commons as best-effort enrichment, run matching/merge, and print source/match/photo/membership counts; verify fixture-server success, malformed/empty-primary rollback, and failed-Commons continuation.
- [ ] 7.2 Wire `SOFIAPLAN_URL` and `COMMONS_API_URL` overrides without adding scheduling or browser source calls; verify importer unit/integration tests and command help/configuration output.

## 8. Additive GraphQL contract

- [ ] 8.1 Extend GraphQL row mapping and schema with enriched facts, structured photos, all source metadata, date meaning, and source-value history while preserving `photoUrls` and the existing `source` field; verify typed JSON errors remain generic and ordering is deterministic.
- [ ] 8.2 Run focused and full database GraphQL tests, including existing filters, pagination, unknown-playground behavior, legacy fields, explicit false values, selected/older values, and `OBSERVATION` versus `SOURCE_UPDATE` dates.

## 9. Popup and details UI

- [ ] 9.1 Add pure formatting helpers and focused Node tests for age, known/unknown values, equipment counts, source dates, and neighborhoods; verify the focused `npm test` pattern.
- [ ] 9.2 Extend existing GraphQL queries, popup hierarchy, details sections, source history/attribution, textual stale warnings, missing/failed-photo states, and safe `textContent` rendering without changing marker/modal/focus/mobile behavior; verify `npm test` and `npm run build`.
- [ ] 9.3 Manually check the approved flow at 320, 768, and 1280 CSS pixels, including hover/focus, popup action activation, Back/Escape, failed photos, unknown/false values, warnings, attribution, console errors, and absence of browser calls to source APIs.

## 10. Documentation, verification, and archive

- [ ] 10.1 Document the one-time operator command, environment overrides, count meanings, failure-preservation contract, SofiaPlan quirks, direct Commons references, and source-date semantics in setup/data/gotcha documentation; verify links and examples are consistent.
- [ ] 10.2 Run `cargo fmt --manifest-path backend/Cargo.toml --check`, `cargo clippy --manifest-path backend/Cargo.toml --all-targets --all-features -- -D warnings`, backend tests, `npm test`, `npm run build`, `npm audit`, `openspec validate enrich-playground-data --strict --no-interactive`, and `git diff --check`.
- [ ] 10.3 Run one real local import against the configured services, inspect source/link/canonical/photo counts and rollback behavior, then archive the completed change so its deltas sync into the main specs; verify `openspec validate --all --strict --no-interactive`.
