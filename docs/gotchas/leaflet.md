# Leaflet

## Resize changes need `invalidateSize()`

Symptom: after a responsive viewport or container resize, tiles occupy the old area or a blank region appears.

Cause: Leaflet caches map dimensions and does not reliably detect every layout change.

Fix: observe the map container and call `map.invalidateSize()` after its size changes.

Verify with the 320 px, 768 px, and desktop cases in [TEST_CASES.md](../../TEST_CASES.md).

## Keyboard paths need explicit DOM semantics

Symptom: polygon tooltip appears, but `Enter` or `Space` does not select a neighborhood.

Cause: Leaflet path click handlers do not automatically provide keyboard interaction.

Fix: after the SVG path is added, set `tabindex`, `role="button"`, and an accessible label. Handle `keydown` and prevent the default Space scroll.

Verify `KEYBOARD-001` through `KEYBOARD-004`.

## Hover must preserve selection

Symptom: selected polygon loses its selected styling after hover ends, or a previous polygon stays highlighted.

Cause: `resetStyle()` applies the default style without checking whether the layer is selected.

Fix: apply hover style only to unselected layers. Reset hover style only when the layer is not the selected layer.

Verify `POINTER-003` through `POINTER-006`.
