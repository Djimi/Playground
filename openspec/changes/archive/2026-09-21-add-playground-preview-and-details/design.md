# Design

## Context

The browser client is a small Leaflet application with area selection, visible-bounds playground loading, and Leaflet popups. The read-only GraphQL API already returns names, age bounds, photo URLs, capabilities, and source metadata. The importer already loads separately mapped OpenStreetMap playground equipment and associates equipment inside a playground polygon, but it currently deduplicates equipment types and loses repeated-feature counts.

See `proposal.md` for motivation and the capability deltas for observable behavior.

## Goals / Non-Goals

**Goals:**

- Reuse current Leaflet and DOM code for preview and detail interactions.
- Keep map summary requests small and fetch full playground details only after selection.
- Preserve the existing capability list while adding honest optional equipment counts.
- Make Back and Escape follow one deterministic state order.

**Non-Goals:**

- Rating or review submission, persistence, authentication, and moderation.
- Google ratings, reviews, photos, or map services.
- A physical `last verified` claim.
- A new frontend framework, router, state library, or gallery dependency.

## Decisions

### Use two information surfaces

Use a compact Leaflet preview for pointer hover and keyboard focus. Use one semantic detail `<aside>` for click, tap, Enter, or Space. CSS makes the detail view fill a phone viewport and present beside or over the map on wider screens.

This keeps transient information small while giving touch users one direct action to reach complete details. Reusing Leaflet and native DOM avoids a component framework or popup package.

### Keep a minimal explicit interaction state

Track only the selected area, selected playground, transient preview, and one saved pre-area viewport. Back and Escape unwind state in this order:

```text
playground details --> selected area --> prior viewport
```

Selecting a playground does not require an area selection. Closing playground details therefore leaves any existing area selection and viewport intact. A single saved viewport is enough because only one area can be selected.

### Split summary and detail reads

Extend the visible-bounds query only with fields needed by the compact preview: age bounds, first available photo, and equipment inventory. Use the existing single-playground query after selection for the complete photo and source data. Abort stale requests with the existing request pattern.

The UI renders `No ratings yet` and an empty platform-review state because this change adds no feedback data model. A later feedback change can replace those states with real aggregates and reviews; this change adds no speculative rating fields.

### Add optional counts without replacing capabilities

Keep `capabilities text[]` as the source for existing filters and compatible GraphQL clients. Add an `equipment_counts jsonb NOT NULL DEFAULT '{}'` object whose keys are supported capability database values and whose values are positive integers. Add a database check that the value is a JSON object; importer validation owns allowed keys and positive counts.

During import, boolean playground tags establish presence with unknown count. Separately mapped equipment features inside a playground polygon establish a count equal to the number of mapped features of that type. The GraphQL API merges both sources into deterministic `Equipment { capability, count }` entries. Missing map entries become `null`, never zero.

A separate equipment table was considered but rejected: equipment has no independent product identity yet, and the JSON object is the smallest additive representation that preserves current filtering.

## Risks / Trade-offs

- **Mapped-feature count may differ from physical seats or components.** The UI describes counts as recorded equipment items and leaves quantity unknown when source data only states presence.
- **Hover previews can obscure dense markers.** Keep previews compact and close them on pointer exit unless details are open.
- **Full-screen phone details hide map context.** Keep a persistent, accessible Back control and restore the same viewport on close.
- **Capabilities and count keys can diverge.** Build both values in one importer normalization step and test that every counted type is present in capabilities.
- **Existing imported rows lack counts.** Default to an empty count object so all existing capabilities remain valid with unknown quantities.

## Migration Plan

1. Add `equipment_counts` with an empty-object default and validation.
2. Extend importer normalization and snapshot replacement to populate known mapped-feature counts.
3. Add the additive GraphQL equipment field while retaining `capabilities`.
4. Deploy the client preview, detail view, and navigation behavior after the API field is available.
5. Re-run the importer to populate counts from current source data.

Rollback removes client use of the additive field first, then removes the GraphQL field and database column. Existing capability filtering remains available throughout.
