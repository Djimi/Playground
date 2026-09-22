# Proposal

## Why

The map currently asks the API only for playgrounds inside the visible bounds and reloads pins on every pan or zoom. The user wants every Sofia playground pin visible at all times, independent of the viewport, so the map reads as a complete playground catalog.

## What Changes

- Load the full playground catalog once at startup using the existing paged `playgrounds(limit, offset)` query and render every returned playground as a pin.
- Remove visible-bounds querying and the `moveend` reload; pins stay on the map while panning and zooming.
- Replace the "refresh pins after the visible bounds change" behavior and its stale-pins failure scenario with a single-load failure behavior: if the catalog request fails, no pins are displayed and no stale data is presented.
- Keep existing pin interactions unchanged: hover/focus preview, click/keyboard details, selected pin styling, and pin layering above area polygons.
- No backend change is required: the GraphQL API already supports filter-less paged queries with a documented 500 maximum limit.

## Capabilities

### New Capabilities

(none)

### Modified Capabilities

- `sofia-neighborhood-map`: "Display playground pins" changes from visible-bounds refresh to displaying the full Sofia catalog loaded once, including the failure scenario.

## Impact

- `src/main.js`: the playground GraphQL query, `loadPlaygrounds`, the `moveend` listener, and `renderPlaygrounds`.
- `TEST_CASES.md`: `MAP-006` and related pin cases need updated expectations.
- Scale: a full import is expected to produce roughly 1,000-2,000 Sofia playgrounds (Overpass currently reports ~2,000 matching elements). The local database holds one test playground (`node/9002`), so validating this change requires running the documented import.
- Performance risk: all pins render as focusable SVG markers so keyboard access keeps working. If thousands of markers degrade panning on phones, clustering is a follow-up option that must be discussed before adding a dependency.
- No API, database, or dependency changes.
