# Tasks

## 1. Implement full-catalog pin loading

- [x] 1.1 Replace the bounds GraphQL query with a filter-less paged query (`playgrounds(limit, offset)`); verify the browser network payload contains only `limit` and `offset` variables.
- [x] 1.2 Rework `loadPlaygrounds` into a single startup load: collect all pages at 500 per page, render once after the last page resolves, and remove the bounds computation and the `moveend` listener; verify pins render once, stay visible while panning and zooming, and no further playground requests appear in the network panel.
- [x] 1.3 Remove the request bookkeeping that existed only for bounds refreshes while keeping one in-flight guard; verify `npm run build` succeeds and the browser console stays clean during load, pan, and zoom.
- [x] 1.4 Clear the playground layer and render no pins when any page fails; verify by blocking or stopping the API and reloading, confirming no pins and no console-crashing error.

## 2. Data and manual cases

- [x] 2.1 Import the real Sofia catalog with `docker compose run --rm api import-playgrounds` and verify the `playgrounds` table count is in the expected hundreds-to-thousands range.
- [x] 2.2 Update `TEST_CASES.md` `MAP-006` and related pin cases to the all-pins behavior, add a case for an empty catalog, and add a case for the failed catalog request; verify each case is executable step by step.
- [x] 2.3 Add a responsiveness check to `TEST_CASES.md` for panning and zooming with the real catalog at a phone viewport; verify the check names an observable pass/fail signal.

## 3. Verify

- [x] 3.1 Run `npm test` and `npm run build`; verify both succeed.
- [x] 3.2 In a browser with imported data: verify all pins are visible at city zoom and after panning and zooming, and that hover preview, click/keyboard details, selected styling, and Tab focus on pins still work at 320 px, 768 px, and desktop widths.
