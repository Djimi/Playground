# Design

## Context

`src/main.js` currently uses one Leaflet GeoJSON layer for neighborhoods and named destinations. Its shared click handler stores a selected layer, applies orange selected styling, and suppresses hover until pointer movement. Playground pins use that same orange default style. The neighborhood generator groups equal names into `MultiPolygon` features; Hadzhi Dimitar currently contains two nested source polygons. `Raina Knyaginya` is absent from bundled data. The bundled data already includes Lyulin 1–10, Lyulin Center, and Nadezhda 1–4, but also includes an aggregate Lyulin polygon that covers every numbered Lyulin microregion.

See `proposal.md` and delta spec for required behavior.

## Goals / Non-Goals

**Goals:**

- Make area preview transient and reliable for every area type.
- Make one input close playground details before any area can activate.
- Correct reported geometry and coverage without changing API or adding dependencies.
- Preserve separately selectable Lyulin and Nadezhda microregions.

**Non-Goals:**

- Add persistent area selection, an area details view, more destination types, or general automatic neighborhood-boundary inference.
- Change playground data, map provider, API contract, or database schema.

## Decisions

### Use transient area preview only

Remove persistent selected-area state, saved area viewport, and post-fit hover suppression. Area pointer/focus entry applies preview style and heading; exit resets both. Click, tap, Enter, and Space only fit polygon bounds.

Alternative: retain selection with a less prominent color. Rejected because user requires click-to-zoom only and selection causes stale state.

### Dismiss details before handling an area activation

Handle empty-map clicks by closing playground details. In each area activation handler, first close open details and return without fitting bounds. Therefore a click on an area while details are visible closes details only; next activation performs fit-to-bounds.

Alternative: rely on Leaflet event propagation from area to map. Rejected because layer and map event order can activate area before dismissal.

### Give playground pins their own neutral default style

Use a default pin style distinct from area preview and selected-playground styles. Keep selected and preview pin cues, details content, and pin interaction unchanged.

Alternative: keep orange default pin. Rejected because it is visually mistaken for area-selection state.

### Curate one canonical boundary per neighborhood

Keep generated OSM relation geometry as primary data. Add a version-controlled curated override for `Raina Knyaginya` with verified Sofia boundary geometry and Latin-script name. For duplicate named geometries, choose the one canonical geometry rather than merging nested duplicates; pin Hadzhi Dimitar to its non-nested canonical relation. Extend validation to require Raina Knyaginya and prove Hadzhi Dimitar has one polygon.

Alternative: union duplicate polygons dynamically or add a spatial dependency. Rejected because one known nested duplicate needs no new dependency, and dynamic unions conceal source-correction choices.

### Prefer supported subdivisions over aggregate geometry

Keep Lyulin 1–10, Lyulin Center, and Nadezhda 1–4 as distinct source features. Exclude the generic Lyulin polygon from bundled interactive data because it spatially covers all numbered Lyulin microregions and can intercept their interaction.

Alternative: show both generic and detailed Lyulin boundaries. Rejected because overlap prevents reliable microregion selection.

## Risks / Trade-offs

- [Curated Raina Knyaginya geometry can drift from map-source edits] Mitigation: store source identity, attribution, and a focused geometry test with bundled data.
- [Other equal-name relations may be valid disjoint neighborhoods] Mitigation: apply canonical selection only to explicitly curated duplicates; preserve unrelated multi-part geometry.
- [Future aggregate boundaries can mask detailed areas] Mitigation: add required and excluded data assertions for every curated subdivision family.
- [Leaflet click paths vary between touch and mouse] Mitigation: cover empty map, area, pin, keyboard, and touch-sized manual checks.

## Migration Plan

1. Update map interaction state and styles.
2. Correct curated geometry output and its data tests, including subdivision-family coverage.
3. Update focused manual cases, then run frontend checks and OpenSpec validation.

Rollback restores previous client-only interaction and GeoJSON files. No persisted state or API migration exists.
