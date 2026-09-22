# Design

## Context

Single-file Leaflet UI in `src/main.js`. Relevant state: `selectedPlayground`, `selectedPlaygroundMarker`, `detailsReturnFocus`, `previewPopup`, `previewMarker`.

Current mechanics, verified with a headless Chromium repro against the running app:

- Playground pins are SVG `circleMarker` paths with `tabindex="0"`; their `focus` listener opens the compact preview popup.
- `closeDetails()` always restores focus to the pin that opened the details. A pointer click on the map background or an area polygon closes details and then focuses that pin, so the pin shows a browser focus ring, turns from selected orange to preview teal, and the compact preview reopens above it.
- Area polygons are focusable (`tabindex="0"`). A pointer click focuses the polygon, and the `focused` flag in `onEachArea` keeps preview styling and the map heading after the pointer leaves. Hovering another area then highlights two areas at once.

Behavior contract: `openspec/changes/fix-map-click-deselect/specs/sofia-neighborhood-map/spec.md`.

## Goals / Non-Goals

**Goals:**

- A click or tap on the map background or an area polygon clears the selected playground without focusing, previewing, or restyling the pin.
- Back and `Escape` clear the selection without the compact preview reopening through restored focus.
- Pointer activation of an area only zooms; preview highlighting follows the pointer afterwards.

**Non-Goals:**

- Changing preview-on-hover or preview-on-focus behavior for normal interaction.
- Changing the details panel, its focus entry (`Back` control), or its layout.
- Blur/hover behavior of pins and area styling otherwise.

## Decisions

### 1. The close reason controls focus restoration

`closeDetails` accepts a `restoreFocus` option (default `true`).

- Map `click` handler and `activateArea` close with `restoreFocus: false`, because a pointer click elsewhere already places focus on the clicked target.
- `goBack` (Back control, `Escape`) keeps the default `true` so keyboard users keep their position.

Alternative considered: always restore focus and immediately close the preview. Rejected because the pin still receives a focus ring after a mouse click, which the user reads as "still selected", and because it races the focus handler.

### 2. Programmatic focus restore does not open the compact preview

A module-level suppression flag is set while restoring focus after Back/`Escape`; the pin `focus` listener skips `showPreview` for that one focus call, then the flag is cleared synchronously. Later hover or `Tab` focus previews the pin normally.

Alternative considered: call `showPreview` then `closePreview` (opens and hides a popup, possible flash, extra state churn).

Note: a keyboard-originated close can still draw the browser focus ring on the pin. That is focus indication, not selection styling, and is required for keyboard users.

### 3. Pointer area activation releases polygon focus

The area `click` handler blurs the polygon after activation, so the `focused` flag clears. The polygon returns to its default styling once the pointer leaves; while the pointer is still over it, normal hover styling applies. The `keydown` activation path does not blur.

Alternative considered: clear the `focused` flag without blurring. Rejected because the element would still be `document.activeElement` while the internal state says otherwise, breaking later `Tab` behavior and `:focus-visible` correctness.

## Risks / Trade-offs

- Keyboard users lose the automatic compact preview after closing details → The pin keeps focus and its focus ring; `Tab` away and back, or hover, previews it again. The spec scenario is updated accordingly.
- Suppression flag leaking if `focus()` never fires → Suppression is set and cleared synchronously around `focus()` on an element checked with `isConnected`; no async gap.
- Touch devices have no hover; blurring the polygon could look like nothing stayed highlighted after a tap → Intended. Zoom fitting is unaffected. Verify at a phone viewport.
- Multiple nearby code paths (area click, map click, Back, `Escape`, pin refresh) can regress separately → Cover with manual cases for each path, including the pin-replaced-after-refresh close path.

## Open Questions

None.
