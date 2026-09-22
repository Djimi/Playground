# Proposal

## Why

When a visitor hovers a playground pin, the pin turns green (preview styling) and stays green after the pointer leaves. The stale highlight makes the map look like many playgrounds are simultaneously previewed or selected. The state only clears accidentally, for example by opening details and then closing them with a map click.

Root cause: in `src/main.js`, `closePreview()` calls `updateMarkerStyle(previewMarker)` before clearing `previewMarker`, so `markerStyle()` still sees the marker as the previewed one and re-applies `PREVIEW_STYLE` instead of `DEFAULT_PIN_STYLE`.

## What Changes

- Reset a playground pin to its default styling when its compact preview ends (pointer leaves or focus moves away).
- Order the preview cleanup so the marker is no longer treated as previewed when its style is recomputed.

## Capabilities

### New Capabilities

None.

### Modified Capabilities

- `sofia-neighborhood-map`: The "Preview playgrounds from the map" requirement gains explicit scenarios that a preview ends when the visitor stops pointing at the pin or moves keyboard focus away, and that the pin returns to its default appearance.

## Impact

- `src/main.js` only: `closePreview()` and the shared style helper `markerStyle()`.
- No API, backend, or data changes.
- Behavior covered by `TEST_CASES.md` PLAYGROUND-002 and PLAYGROUND-010.
