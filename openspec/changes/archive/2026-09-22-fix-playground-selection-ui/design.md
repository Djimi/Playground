# Design

## Context

See `proposal.md` for the motivation and scope. The frontend is a small Leaflet application in `src/main.js` with DOM/CSS in `index.html` and `src/styles.css`. Playground pins and area polygons are rendered directly as Leaflet SVG layers; the backend already supplies the fields used by the current preview and details view.

The current selected-pin reset runs before the selected-marker reference changes, so the style resolver still sees the old marker as selected. Preview state is also shared between pointer hover and keyboard focus, while the details view overlays the map without making the background inert.

## Goals / Non-Goals

**Goals:**

- Make selected-marker state single-valued and order-safe.
- Preserve keyboard-focused area and playground previews when unrelated pointer hover starts or ends.
- Keep transient previews inside the usable map viewport.
- Give the details overlay modal keyboard behavior without adding a UI framework.
- Make loading, empty, and failure states visible while preserving independent map layers.
- Keep missing or failed photos honest and accessible.

**Non-Goals:**

- Adding playground descriptions or changing the backend/import data model.
- Adding marker clustering or another map dependency before rendering cost is measured.
- Changing map selection, catalog, or attribution APIs.

## Decisions

### Update selection state before repainting the previous marker

When a new pin opens details, clear or replace the selected-marker reference before repainting the previous marker. Reuse the existing `markerStyle` function so the previous marker naturally resolves to neutral styling and the new marker resolves to selected styling.

Alternative: add a one-off neutral-style assignment that bypasses the style resolver. Rejected because it duplicates the state precedence rules and can leave preview styling inconsistent.

### Track hover and focus as separate preview sources

Keep pointer-hover state and keyboard-focus state separate for both area polygons and playground pins. A small state reconciliation path will choose the currently visible preview, restore the focused preview after unrelated hover ends, and clear a preview only when neither source remains.

Alternative: rely on DOM `:hover`/`:focus` styling or reopen previews from blur events. Rejected because Leaflet SVG layers already have imperative styles and popup content; CSS alone cannot coordinate the heading, popup, and marker state.

### Use Leaflet's existing popup auto-pan

Allow Leaflet to reposition preview popups and configure padding for the fixed header and map edges. Do not add a custom collision or positioning library.

Alternative: calculate popup coordinates and clamp them manually. Rejected because it duplicates Leaflet viewport math and is more fragile across zoom and resize.

### Use native modal semantics and focus containment

Mark the details overlay as modal, move focus to its close control, keep Tab navigation within its focusable descendants, and make the map/header background unavailable to keyboard navigation while details are open. Restore the previous map focus when closing, using the existing refreshed-marker fallback.

Alternative: add a dialog/focus-trap dependency. Rejected because the overlay has one close control and native DOM focus handling is enough for this app.

### Add one lightweight status surface

Use a single accessible status region for catalog and area-layer states, updating it as requests start, finish empty, or fail. Keep status text concise and do not block map interaction. The status logic will reuse the existing independent fetch promises instead of introducing a store or event bus.

Alternative: hide failures in the console or add separate toasts for each request. Rejected because console-only failures are invisible to visitors and toasts add transient UI state without helping map recovery.

### Preserve honest photo fallbacks

Pass the playground name into image alternative text. When an image fails, replace it with a short unavailable-photo message rather than removing the element and leaving an unexplained gap.

Alternative: keep removing failed images. Rejected because it hides source-data failures and can make a populated gallery look empty.

## Risks / Trade-offs

- [Risk] A focused preview can compete with a pointer preview when both are active. → Mitigation: define one precedence rule and cover pointer-enter, pointer-leave, focus, and blur sequences in the browser checks.
- [Risk] Making the background inert can hide the header Back control while details are open. → Mitigation: keep the details close control as the only active close path and verify Back/Escape/focus restoration at phone and desktop widths.
- [Risk] Auto-pan can move the map when a visitor only hovers a pin. → Mitigation: keep popup auto-pan padding small, disable animation for hover previews, and verify edge pins do not create disruptive jumps.
- [Risk] A single status region can overwrite one failure with another request's success message. → Mitigation: track independent layer status and render the highest-priority active message, retaining failure text until that layer succeeds or is retried.

## Migration Plan

No data or deployment migration is required. Apply the frontend-only change, run the existing automated checks and focused browser cases, then deploy the rebuilt static assets. Rollback is a normal frontend rollback to the previous commit; no database or API rollback is needed.
