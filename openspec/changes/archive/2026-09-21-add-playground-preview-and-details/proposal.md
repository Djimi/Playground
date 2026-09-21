# Proposal

## Why

The map can locate playgrounds, but its current popup exposes too little information to judge whether a playground suits a family. Visitors need a quick preview, a fuller detail view, and a predictable way to return from focused map states on both pointer and touch devices.

## What Changes

- Add a compact playground preview with a main photo, name, platform rating state, recommended age, and equipment summary.
- Open a responsive playground detail view from a selected pin with available photos, age suitability, equipment inventory, platform review state, and source attribution.
- Represent supported play equipment with an optional count so unknown quantities remain distinct from zero.
- Visibly distinguish the selected playground pin without requiring neighborhood selection first.
- Let visitors restore the previous map view with a visible Back control or `Escape`; closing details happens before leaving neighborhood focus.
- Keep standard Leaflet wheel, touch, pinch, keyboard, and zoom-control behavior rather than requiring `Ctrl+scroll`.
- Show `No ratings yet` when no platform rating exists; never substitute a hardcoded production rating.
- Exclude Google reviews, rating or review submission, authentication, moderation, and physical-verification claims from this change.

## Capabilities

### New Capabilities

None.

### Modified Capabilities

- `sofia-neighborhood-map`: Add playground preview, selection, responsive details, and reversible focused-map navigation.
- `playground-discovery`: Return structured equipment inventory with honest unknown counts alongside existing playground detail fields.

## Impact

- Extends the browser map UI, responsive styling, keyboard behavior, and map interaction tests.
- Extends the read-only GraphQL playground shape and PostgreSQL/import representation for optional equipment counts.
- Reuses existing Leaflet, GraphQL, PostgreSQL/PostGIS, photo, age, and capability infrastructure without adding dependencies.
- Preserves the read-only API and existing OpenStreetMap attribution requirements.
