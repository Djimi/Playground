# Design

## Context

`src/main.js` currently loads pins in `loadPlaygrounds()`, called once at startup and on every `moveend`. Each call queries `playgrounds(filter: { bounds })` in 500-item pages and re-renders all pins.

Constraints:

- The GraphQL API exposes `playgrounds(filter: PlaygroundFilter, limit: Int! = 200, offset: Int! = 0)`, accepts 1-500 per request, allows the filter to be omitted, and returns results in deterministic order.
- Pins are focusable SVG `circleMarker` paths (`tabindex`, `role="button"`, `aria-label`) so keyboard users can preview and open them. Hover/focus/selected styling is per marker element.
- Scale from Overpass for Sofia: ~2,006 matching elements, so an imported catalog is expected to hold roughly 1,000-2,000 playgrounds. The local database currently holds one test playground (`node/9002`); acceptance needs the documented import.
- Behavior contract: `openspec/changes/show-all-playground-pins/specs/sofia-neighborhood-map/spec.md`.

## Goals / Non-Goals

**Goals:**

- Every playground pin is visible regardless of viewport and zoom.
- One catalog load per page load; no refetching while panning or zooming.
- Pin interactions, styling, and keyboard access behave exactly as before.

**Non-Goals:**

- Clustering, filtering, or lazy rendering of pins. Clustering would add a dependency and is a separate decision.
- Backend changes; the existing paged query is sufficient.
- Changing area polygon behavior or the details panel.

## Decisions

### 1. Load the filter-less catalog once at startup

Reuse the existing paging loop with `PAGE_SIZE = 500` and `playgrounds(limit, offset)` without a `bounds` filter, collecting all pages, then render once. The loading code no longer needs `AbortController` request bookkeeping tied to bounds refreshes, nor the `moveend` listener; `map.fitBounds` after loading neighborhoods simply no longer triggers a data request.

Alternatives considered:

- Keep the bounds query at low zoom and switch to full loading when zoomed out → two code paths and request churn for no user-visible gain.
- Raise the page size beyond 500 → API rejects limits above 500.
- Add a backend count/full-catalog endpoint → unnecessary; paging already covers the scale.

### 2. Keep SVG circle markers; do not switch to the canvas renderer

A canvas renderer handles thousands of points faster, but canvas markers have no per-marker DOM element, which breaks `tabindex`, `role="button"`, ARIA labels, and the focus-driven preview. Preserve accessibility first.

If real data shows stutter on phones, the follow-ups are, in order: measure with a trace, then either canvas plus an accessible alternative (for example, a sidebar list), or clustering. Clustering introduces a new dependency and must be discussed before adoption.

### 3. Failure clears the layer

If any page of the catalog request fails, the code clears the playground layer and shows no pins, matching the "no partial or stale data" scenario. Rendering happens only after every page resolves, so partial results never flash.

## Risks / Trade-offs

- ~2,000 SVG pins may make pan/zoom stutter on low-end phones → Validate with imported data using a browser trace; fallback options are documented above and deferred deliberately.
- Initial load makes several sequential requests (about 2-4 pages today) → Collect all pages before rendering; one visible load, no flicker. Acceptable for a read-only catalog.
- City-wide view has heavy pin overlap → The user asked for all pins; markers stay clickable and zooming separates them. Clustering remains a future option.
- Acceptance depends on imported data, which the local database lacks → Tasks include running the documented import before browser verification, and a browser test with a stubbed API response for the failure and empty-catalog cases.
- `renderPlaygrounds` currently preserves the selected pin across refreshes; with a single load that path is mostly dormant → Keep the logic only where it is still reachable (selection can only exist after the initial render), and remove dead request bookkeeping.

## Migration Plan

No schema, API, or dependency migration. Deploy the frontend change; rollback is reverting `src/main.js`. Operators run the playground import to populate pins; the app also behaves correctly with an empty catalog (no pins, no error).

## Open Questions

- Clustering or a canvas renderer if real-data performance is poor: deferred until measured; adding a dependency requires the user's agreement first.
