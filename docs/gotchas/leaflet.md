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

## Selection state must change before repaint

Symptom: opening another playground leaves more than one orange pin visible.

Cause: repainting the old marker while it is still the selected marker makes the style resolver return the selected style again.

Fix: replace the selected-marker reference first, then repaint the old and new markers through the shared style resolver.

Verify `PLAYGROUND-012`.

## Pointer and keyboard preview sources need reconciliation

Symptom: moving the pointer across an unrelated layer clears the keyboard-focused preview or leaves the unrelated layer highlighted.

Cause: pointer hover and keyboard focus are independent browser states, but Leaflet path and marker styles are imperative.

Fix: track both sources, give the focused layer precedence, and reconcile all affected layers after every enter, leave, focus, and blur event.

Verify `POINTER-009` and `PLAYGROUND-013`.

## Popup previews need usable-viewport padding

Symptom: a preview near the map edge or fixed header is clipped.

Cause: disabling Leaflet auto-pan leaves popup positioning unaware of the fixed UI.

Fix: use Leaflet auto-pan with top and bottom padding for the header and map edges, without adding custom collision math.

Verify `PLAYGROUND-014`.
