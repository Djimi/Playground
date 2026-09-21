# Proposal

## Why

Map currently treats area clicks as persistent orange selections. This makes a playground pin look like an unexplained selection, leaves stale hover state after a click, and renders duplicate Hadzhi Dimitar geometry. Coverage also omits the everyday Raina Knyaginya neighborhood and lets an aggregate Lyulin polygon hide its microregions.

## What Changes

- Make every neighborhood, South Park, and Sofia Zoo polygon preview-only: hover or focus emphasizes it; click or keyboard activation only fits its bounds.
- Keep playground pins interactive, but use a distinct default pin appearance so a pin is not confused with an area selection.
- Make a map click close open playground details and consume that click. A second click may activate an area under it.
- Remove stale area-preview state after area fitting and ensure parks receive same preview treatment as neighborhoods.
- Add `Raina Knyaginya` as a curated Latin-script Sofia neighborhood.
- Remove nested duplicate Hadzhi Dimitar geometry so it renders as one boundary.
- Show `Lyulin 1` through `Lyulin 10`, `Lyulin Center`, and `Nadezhda 1` through `Nadezhda 4` as separately selectable neighborhoods; remove the aggregate Lyulin polygon from the interactive map.

## Capabilities

### New Capabilities

None.

### Modified Capabilities

- `sofia-neighborhood-map`: Change area activation from persistent selection to click-to-zoom, define dismissible playground details, require consistent area previews, and require corrected neighborhood coverage and geometry.

## Impact

Affected Leaflet area and map-click handlers, pin styles, neighborhood preparation data, bundled GeoJSON, automated data/UI tests, and manual map cases. No API, dependency, or database change.
