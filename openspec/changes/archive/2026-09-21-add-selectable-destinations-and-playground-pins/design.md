# Design

## Context

The current Leaflet frontend loads one curated neighborhood GeoJSON file and selects neighborhood polygons locally. The existing GraphQL backend already searches playgrounds by map bounds, but its bounded response cannot be paged and the frontend does not consume it. See `proposal.md` for motivation.

The implementation already exists because this change is retrospective. Apply must reconcile that implementation against these artifacts rather than treating its presence as proof of completion.

## Goals / Non-Goals

**Goals:**

- Keep neighborhoods, South Park, and Sofia Zoo visually and interactively consistent while preserving their different data roles.
- Load all playgrounds inside the current viewport without stale responses replacing newer results.
- Preserve mouse, touch, and keyboard access on phone, tablet, and desktop layouts.
- Keep local API routing configurable without adding a frontend dependency.

**Non-Goals:**

- Automatically discover every park, zoo, or attraction in Sofia.
- Cluster pins, add playground filtering controls, or redesign playground details.
- Change database storage or automate recurring OpenStreetMap imports.

## Decisions

### Keep named destinations in a separate GeoJSON dataset

Store the canonical South Park relation and Sofia Zoo way in a small discovery-area file generated beside the neighborhood file. Load it after neighborhoods so overlapping destination geometry receives pointer priority. This avoids misclassifying facilities as neighborhoods while reusing the existing Leaflet area interaction code.

Alternative: import every OpenStreetMap park and attraction. Rejected because selection scope and data-quality rules are not yet defined.

### Use shared Leaflet layers with explicit panes

Render neighborhoods and named destinations through one area layer and playground markers through a higher pane. Shared handlers provide hover preview, persistent selection, keyboard activation, and `fitBounds` click zoom. The selected heading remains stable during programmatic zoom until the pointer actually moves.

Alternative: separate interaction code per area type. Rejected because the behavior is identical and duplicated handlers previously caused inconsistent selection.

### Page viewport queries with an offset

Add an optional, non-negative GraphQL offset with a default of zero. The frontend requests deterministic pages up to the existing maximum page size, aborts an older viewport request when the map moves again, and only renders the newest completed request.

Alternative: raise the API limit. Rejected because a fixed larger ceiling can still truncate dense viewports and increases response size for every client.

### Use same-origin API requests in the browser

Use `/graphql` by default and let Vite proxy it to the configured local API port. Deployments can set `VITE_API_URL` or route the same path through their reverse proxy. This avoids constructing an HTTP URL with a fixed port in browser code.

Alternative: derive port `3000` from the current browser host. Rejected because it ignores configured ports and fails under HTTPS deployments.

## Risks / Trade-offs

- [Offset pages can shift during a concurrent import] -> Imports replace the catalog atomically and are manual; move to cursor pagination only if imports become frequent.
- [Overlapping polygons can steal hover after click zoom] -> Load destination geometry last and suppress synthetic hover transitions until real pointer movement.
- [Dense city views can become visually busy] -> Fetch only visible bounds; defer clustering until real usage shows it is needed.
- [Public OpenStreetMap services can rate-limit refreshes] -> Keep data bundled at runtime, send a descriptive user agent, validate responses, and preserve the previous catalog on failure.

## Migration Plan

1. Reconcile the retrospective artifacts with the existing implementation and tests.
2. Restore prematurely edited main requirements before applying the delta so the change remains the source of truth.
3. Validate frontend, backend, accessibility, responsive behavior, and OpenSpec artifacts.
4. Sync the validated delta specs into the main specs and archive the change.

No database migration is required. The GraphQL offset defaults to zero, so existing clients remain compatible. Rollback removes the new frontend layers and discovery dataset; the optional API argument can remain without affecting old clients.
