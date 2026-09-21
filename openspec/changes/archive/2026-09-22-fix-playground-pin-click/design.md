# Design

## Context

See `proposal.md` for motivation. Playground pins are Leaflet circle markers, whose mouse events bubble to the map by default. A pin click opens details, then the same Leaflet event reaches the map click handler and immediately closes them. Browser hit testing confirms that the pin, not the preview popup, receives the pointer input.

## Goals / Non-Goals

**Goals:**

- Keep the existing hover and keyboard preview.
- Keep details open after pointer activation of a playground marker.
- Preserve bare-map clicks as a way to close details.
- Use Leaflet's existing per-layer event ownership option.

**Non-Goals:**

- Redesign the preview or details panel.
- Change preview popup behavior or styling.
- Change playground data or API behavior.

## Decisions

### Stop playground marker mouse events from bubbling to the map

Set `bubblingMouseEvents: false` on each playground circle marker. Leaflet then handles the click at the marker without forwarding it to the map click handler. Clicks on bare map space still reach the map and close open details.

Alternatives considered:

- Disable pointer events on the preview popup. Browser proof showed that the marker already receives the pointer input, and this change did not keep details open.
- Add a guard to the map click handler. This couples map behavior to event origin instead of using Leaflet's per-layer propagation control.
- Mutate Leaflet's internal stopped-event flag. This depends on an implementation detail when a public option already expresses the required behavior.

## Risks / Trade-offs

- Playground marker mouse events will not reach future map-level mouse handlers. Mitigation: keep playground interaction in the marker handlers and reassess only if a map-level handler must also observe marker clicks.
- Leaflet event behavior could change in a future upgrade. Mitigation: verify marker activation and bare-map closing after Leaflet upgrades.
