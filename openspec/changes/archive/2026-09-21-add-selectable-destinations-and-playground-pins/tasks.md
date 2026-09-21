# Tasks

## 1. Repair the OpenSpec Baseline

- [x] 1.1 Restore the premature edits in the two main capability specs to their pre-change state and verify `git diff` shows the new requirements only in this change's delta specs.

## 2. Add Destination Geometry

- [x] 2.1 Generate and bundle a separate discovery-area GeoJSON containing exact South Park relation `16878152` and Sofia Zoo way `157686292` polygons, and verify the dataset test checks IDs, geometry, and ODbL metadata.
- [x] 2.2 Document discovery-area sources, refresh behavior, and public OpenStreetMap failure handling, and verify every bundled dataset has visible attribution guidance.

## 3. Page Playground Search

- [x] 3.1 Add an optional non-negative GraphQL offset, apply it after deterministic ordering, and verify unit and PostGIS integration tests cover later pages and negative offsets.
- [x] 3.2 Send a descriptive importer user agent and preserve atomic failure behavior, and verify the backend release build succeeds.

## 4. Integrate the Map UI

- [x] 4.1 Load neighborhoods and named destinations through shared area interactions with hover styling, a prominent live heading, persistent selection, keyboard activation, and click-to-fit zoom; verify pointer and keyboard selection for South Park and Sofia Zoo.
- [x] 4.2 Fetch every playground page for the visible bounds, cancel stale viewport requests, and render accessible pins above polygons with a fallback name, capabilities, and source link; verify pins refresh after map movement and have at least a 24 CSS pixel touch target.
- [x] 4.3 Use a same-origin `/graphql` browser default with a Vite proxy to `API_PORT`, align the documented direct CORS origin, and verify local requests succeed without a hard-coded browser port.

## 5. Verify the Retrospective Change

- [x] 5.1 Run `npm test`, `npm run build`, `npm audit`, backend tests with PostGIS, the backend release build, and `git diff --check`; verify every command succeeds.
- [x] 5.2 Perform desktop and 320 CSS pixel browser checks for South Park, Sofia Zoo, hover naming, zoom, pin popups, responsive layout, and console errors; verify the acceptance behavior in both viewports.
- [x] 5.3 Run `openspec validate add-selectable-destinations-and-playground-pins --type change --strict` and verify the retrospective artifacts are internally consistent before apply is declared complete.
