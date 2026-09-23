# Proposal

## Why

The catalog currently exposes OpenStreetMap-only metadata, so visitors see limited playground facts and the operator cannot retain or inspect complementary municipal data. A one-time, operator-run enrichment import can combine current OpenStreetMap and SofiaPlan records while preserving source evidence and the existing catalog when anything fails.

## What Changes

- Add one operator-run import that fetches OpenStreetMap and SofiaPlan, validates both responses, retains each current source record, conservatively merges only unambiguous pairs within 15 metres, and transactionally replaces source records, mappings, canonical records, and memberships.
- Exclude SofiaPlan records explicitly marked `не се показват на картата` from canonical results while retaining their source records for inspection.
- Merge newest non-missing field values by effective date, preserve provenance and conflicting source values, and keep explicit `false`/`0` values distinct from unknown values.
- Resolve only directly referenced reusable Wikimedia Commons files; retain structured photo attribution and show explicit missing or failed-photo states.
- Extend GraphQL additively with address, surface, fencing, ownership, access, fee, historical municipal status, Ordinance 1 compliance, repairs, notes, structured photos, all source metadata, and source-value history while keeping `photoUrls` and existing source compatibility.
- Update compact playground previews and full details to the approved mockup hierarchy, including textual historical-data warnings, source history, attribution, and the existing empty rating/review states.
- Preserve existing map interactions, accessibility, focus management, responsive behavior, and neighborhood data behavior.
- Do not add scheduler or deployment work, visitor editing or authentication, Google, Mapillary, social-media, or browser-side source calls.

## Capabilities

### New Capabilities

None.

### Modified Capabilities

- `playground-discovery`: enrich the operator import, canonical merge and replacement contract, and additive GraphQL catalog fields.
- `sofia-neighborhood-map`: update playground preview and details presentation while preserving existing map interaction, accessibility, focus, and responsive requirements.

## Impact

- Import and persistence code will gain current source-record storage, conservative cross-source matching, field-level provenance/history, licensed Commons photo metadata, and atomic catalog replacement.
- The existing GraphQL query model will gain nullable/additive enrichment fields and structured photos without removing `photoUrls` or existing source metadata.
- The existing popup and details UI will render the enriched catalog and explicit unknown, stale, conflicting, and photo-failure states using the approved mockup.
- Tests and fixtures will cover both source parsers, matching and field selection, rollback preservation, Commons licence checks, and the updated UI states.
