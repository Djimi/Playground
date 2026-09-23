# Sofia Neighborhood Map Test Cases

Run automated checks first:

```bash
npm test
npm run build
npm audit
cargo fmt --manifest-path backend/Cargo.toml --check
cargo clippy --manifest-path backend/Cargo.toml --all-targets --all-features -- -D warnings
docker compose up -d db
DATABASE_URL=postgres://playground:playground@127.0.0.1:5432/playground \
  cargo test --manifest-path backend/Cargo.toml
```

Start the application with `npm run start:local`, then run the manual cases below. The command must report successful frontend and GraphQL smoke checks before browser testing begins. It does not import playground data automatically; run the documented import command only when pin data is needed. Test at 320 px phone, 768 px tablet, and desktop widths unless a case specifies otherwise.

## Local startup smoke checks

| ID | Priority | Test | Expected result | Coverage source |
| --- | --- | --- | --- | --- |
| START-001 | High | Run `npm run start:local` from the repository root with Docker and npm dependencies available. | PostGIS, API, and Vite start; the command reports frontend and GraphQL URLs only after both checks pass. | Startup script |
| START-002 | High | Run the command without a required local tool or dependency. | The command identifies the missing prerequisite and exits non-zero without claiming readiness. | Startup script |
| START-003 | High | Press `Ctrl+C` after startup succeeds, then run the command again. | Vite exits cleanly and the named Postgres volume remains reusable. | Startup script |
| START-004 | Medium | Start without running the Overpass importer. | Startup succeeds with an empty catalog; no external import is triggered. | Startup script and API smoke query |

## Data and Build

| ID | Priority | Test | Expected result | Coverage source |
| --- | --- | --- | --- | --- |
| DATA-001 | High | Load `public/data/sofia-neighborhoods.geojson`. | Valid `FeatureCollection`; scope is Sofia relation `4283101`; license is `ODbL-1.0`. | Automated test |
| DATA-002 | High | Validate every feature. | Each feature has a unique stable ID, unique name, and `Polygon` or `MultiPolygon` geometry. | Automated test |
| DATA-003 | High | Check all displayed names. | Names contain no Cyrillic characters. | Automated test |
| DATA-004 | High | Check curated neighborhood coverage. | Lozenets; Mladost 1, 1A, 2, 3, and 4; Raina Knyaginya; Lyulin 1–10 and Center; and Nadezhda 1–4 are separate Latin-script features. | Automated test and UI review |
| DATA-007 | High | Inspect Raina Knyaginya and Hadzhi Dimitar. | Raina Knyaginya carries its curated OSM-node attribution; Hadzhi Dimitar has one polygon; the aggregate Lyulin relation is absent. | Automated test |
| DATA-005 | High | Check excluded outlying settlements. | Benkovski, Chelopechene, Kremikovtsi, Seslavtsi, and Trebich are absent. | Reported regression and automated test |
| DATA-006 | High | Load `public/data/sofia-discovery-areas.geojson`. | Exact South Park relation `16878152` and Sofia Zoo way `157686292` polygons are present with ODbL metadata. | Automated test |
| BUILD-001 | High | Run `npm run build`. | Vite production build exits successfully. | Automated build |
| BUILD-002 | Medium | Run `npm audit`. | No known dependency vulnerabilities are reported. | Session verification |

## Map Rendering and Navigation

| ID | Priority | Test steps | Expected result | Coverage source |
| --- | --- | --- | --- | --- |
| MAP-001 | High | Open the application. | Sofia map, neighborhood polygons, zoom controls, and header render without console errors. | Main session and subagent review |
| MAP-002 | High | Wait for tiles and polygons to load. | Tiles are not broken; polygon boundaries align with map locations. | Main session and subagent review |
| MAP-003 | High | Pan the map, then zoom in and out. | Map moves and scales normally; polygon boundaries stay aligned. | Main session and subagent review |
| MAP-004 | High | Inspect the map corner. | Leaflet and OpenStreetMap attribution remain visible and usable. | Main session and subagent review |
| MAP-005 | Medium | Resize the browser after the map loads. | Map fills the new viewport without blank or stale tile regions. | Main session regression check |
| MAP-006 | High | Start the API with imported data, open the application, then pan and zoom. | Every imported playground is displayed as a pin above area polygons; pins stay visible after panning and zooming with no further playground requests. | All-pins spec and browser network check |
| MAP-007 | High | Start with a fresh database and skip the import (or clear the `playgrounds` table), then open the application. | No playground pins render; the map, neighborhood polygons, controls, and area previews stay usable with no console errors. | All-pins spec and browser check |

## Pointer Selection and Hover

| ID | Priority | Test steps | Expected result | Coverage source |
| --- | --- | --- | --- | --- |
| POINTER-001 | High | Click or tap Lozenets. | Map fits Lozenets; the highlight follows the pointer only, and the polygon returns to default styling once the pointer leaves. | Main session review |
| POINTER-002 | High | Click Lozenets, then Mladost 3. | Each click fits its own bounds; after the second click only Mladost 3 shows hover styling while the pointer is over it, and Lozenets returns to default. | Main session review |
| POINTER-003 | High | Hover an unselected neighborhood, South Park, or Sofia Zoo. | Hovered polygon receives a clearly visible highlight and its name appears in the large header. | User request |
| POINTER-004 | High | Move pointer or keyboard focus away from an area. | Polygon returns to default styling and the header restores its default prompt. | User request |
| POINTER-005 | Medium | Click an area, then hover South Park or Sofia Zoo and move away. | The destination preview is visible and clears after preview ends. | Hover regression follow-up |
| POINTER-006 | Medium | Move pointer directly between adjacent polygons. | Highlight follows the pointer; previous polygon does not remain highlighted. | Hover regression follow-up |
| POINTER-007 | High | Click South Park, then Sofia Zoo. | Each real facility polygon fits the map without persistent styling; similarly named neighborhood polygons do not intercept the click. | User request |
| POINTER-008 | High | Preview named and unnamed playground pins. | Popup shows a safe name fallback, recorded equipment, and a View details action. | User request |
| POINTER-009 | High | Focus one area with `Tab`, then briefly move the pointer across another area. | The focused area keeps its highlight and heading; the unrelated area returns to default styling after the pointer leaves. | OpenSpec preview-state regression |

## Playground Preview and Details

| ID | Priority | Test steps | Expected result | Coverage source |
| --- | --- | --- | --- | --- |
| PLAYGROUND-001 | High | Hover a playground pin with a photo, age, and equipment. | Compact preview shows the photo, name, age, equipment summary, `No ratings yet`, and preview pin styling. | Browser check and `src/main.js` |
| PLAYGROUND-002 | High | Focus a playground pin with `Tab`, then move focus away. | The same preview and styling appear on focus, then close and reset on focus exit. | Browser check and `src/main.js` |
| PLAYGROUND-003 | High | Preview a playground with no name, photo, age, equipment, or ratings. | The preview uses an unnamed fallback and honest empty or unknown states; no fabricated rating appears. | Browser check and `src/main.js` |
| PLAYGROUND-004 | High | Click or tap a playground pin. | Its neutral-blue default appearance differs from area preview and selected-pin orange; details retain all photos, age, equipment, review, source, and selected-pin cues. | Browser check and `src/main.js` |
| PLAYGROUND-005 | High | Focus a playground pin and press `Enter`, then repeat with `Space`. | Both keys open the same detail view without scrolling the page. | Browser check and `src/main.js` |
| PLAYGROUND-006 | High | Open one detail view, then quickly open another pin. | The stale detail request is aborted and only the latest playground details are rendered. | Source review and browser network check |
| PLAYGROUND-007 | High | Open an incomplete playground detail. | Missing photos, age, equipment counts, ratings, and reviews are labeled as empty or unknown; no values are inferred. | Browser check and `src/main.js` |
| PLAYGROUND-008 | High | Open details at 320 × 640 px. | Details fill the phone viewport, the Back control remains visible, and the page has no horizontal scrolling. | Browser check and responsive CSS |
| PLAYGROUND-009 | Medium | Open details at 768 px and 1280 px widths. | Details remain usable beside or over the map; gallery, source link, and selected styling do not clip. | Browser check and responsive CSS |
| PLAYGROUND-010 | High | Hover a playground pin, then click it while the compact preview is visible. | The details panel opens and remains visible; the selected pin uses orange styling and no compact preview popup remains. | Pointer activation regression check |
| PLAYGROUND-011 | High | Hover a playground pin, then move the pointer to empty map space without clicking. | The compact preview closes and the pin returns to default neutral blue; no pin stays in preview styling. | User report and `src/main.js` |
| PLAYGROUND-012 | High | Open three different playground pins one after another. | The previous pins return to neutral blue and exactly one pin—the current playground—uses orange selected styling. | OpenSpec selection-state regression |
| PLAYGROUND-013 | High | Focus one playground pin with `Tab`, then briefly move the pointer across another pin. | The focused pin keeps its preview and the unrelated pin returns to neutral styling after the pointer leaves. | OpenSpec preview-state regression |
| PLAYGROUND-014 | High | Preview pins at the map edge and below the fixed header. | The compact preview auto-pans into the usable map viewport without clipped content or disruptive animation. | OpenSpec popup-boundary regression |
| PLAYGROUND-015 | High | Open details for a named playground with one valid and one blocked structured photo. | Each image uses the playground name in its alternative text; the blocked image becomes an explicit `Photo unavailable` state while attribution remains. | OpenSpec photo-fallback regression |
| PLAYGROUND-016 | High | Preview and open a fully enriched playground. | Popup shows licensed photo, all neighborhoods, age/equipment chips, dated municipal warning, rating empty state, and View details; details show address, coordinates, directions, every recorded fact, history, and all sources. | Enrichment flow browser check |
| PLAYGROUND-017 | High | Preview and open a playground with no photo and missing optional facts. | `No photo yet`, `Age not recorded`, `No equipment recorded`, and `Unknown` are visible where appropriate; ratings and reviews remain empty. | Enrichment flow browser check |
| PLAYGROUND-018 | High | Open records with `false` fencing/compliance and a dated municipal status. | `No` and `Not compliant` remain distinct from `Unknown`; warning says `May be outdated` and labels the 2019 SofiaPlan date as an observation. | Formatting tests and browser check |
| PLAYGROUND-019 | High | Open a record with conflicting source values. | Selected and older/conflicting values each show source identity and correctly labeled date; all source links, attribution, and licences are visible. | Enrichment flow browser check |
| PLAYGROUND-020 | High | Open a record with a Commons photo, then block the image response. | Attribution and licence link remain visible; failed image becomes `Photo unavailable`. | Enrichment flow browser check |
| PLAYGROUND-021 | High | Activate Directions. | OpenStreetMap directions opens for the playground coordinates without prompting for visitor location. | Enrichment flow browser check |
| PLAYGROUND-022 | High | Move pointer from pin to popup and click View details. Focus a pin, Tab to View details, Shift+Tab back, Tab forward again, then Tab past the button; also open details with Enter/Space directly on the pin. | Popup stays open across pointer and keyboard transitions; reverse Tab returns to the pin, forward Tab exits to the next map control, and button/direct activation opens details without a map click closing them. | Enrichment interaction regression |
| PLAYGROUND-023 | High | Keep keyboard focus on View details and hover another pin. | The focused popup stays visible until focus leaves it; unrelated hover does not replace it. | Enrichment interaction regression |

## Focused Map Navigation

| ID | Priority | Test steps | Expected result | Coverage source |
| --- | --- | --- | --- | --- |
| NAV-001 | High | Open playground details, then press Back. | Details close, pin selection clears, the viewport does not move, and the compact preview does not reopen through the restored pin focus. | Browser check and `src/main.js` |
| NAV-002 | High | Open playground details, click an area, then click that area again. | First input closes details only without focusing or previewing the pin; second fits the area bounds without persistent styling. | Browser check and `src/main.js` |
| NAV-003 | High | Open playground details, then press Escape. | Details close, pin selection clears, the viewport does not move, and the compact preview does not reopen through the restored pin focus. | Browser check and `src/main.js` |
| NAV-004 | High | Press `Escape` in the default state. | Nothing changes; native Leaflet wheel, touch, keyboard, and zoom-control behavior remains available without `Ctrl`. | Browser check and Leaflet behavior |
| NAV-005 | High | Open playground details, then click or tap empty map space. | Details close and the pin selection clears; the pin returns to default blue with no focus ring and no compact preview. | User report and `src/main.js` |
| NAV-006 | High | Open playground details, then click or tap a neighborhood polygon. | Details close and the pin selection clears with no pin focus or preview; the first polygon input does not fit bounds, and the polygon shows no persistent styling once the pointer leaves. | User report and `src/main.js` |
| NAV-007 | High | Open details at 320 px and 1280 px, press `Tab` and `Shift+Tab`, then press `Escape`. | The dialog is modal; focus cycles only through its controls, the map/header are inert, and focus returns to the refreshed/current pin when available. | OpenSpec modal-focus regression |

## Keyboard and Accessibility

| ID | Priority | Test steps | Expected result | Coverage source |
| --- | --- | --- | --- | --- |
| KEYBOARD-001 | High | Press `Tab` until a neighborhood polygon receives focus. | Focus reaches polygons; each exposes a readable `Select <name>` label and button role. | Reported regression and Luna verification |
| KEYBOARD-002 | High | Focus Lozenets and press `Enter`. | Map fits Lozenets without selected styling and the page does not scroll. | Source and browser review |
| KEYBOARD-003 | High | Focus Mladost 1 and press `Space`. | Map fits Mladost 1 without page scrolling or persistent area styling. | Source and browser review |
| KEYBOARD-004 | High | Keyboard-activate two polygons, then blur focus. | Each activation fits its polygon; previews clear on blur. | Source and browser review |
| KEYBOARD-005 | High | Focus a playground pin and press `Enter` or `Space`. | Its details open without scrolling the page. | Source and browser review |
| ACCESS-001 | Medium | Change neighborhood selection. | Header name update is announced through the live region. | Source review |

## Responsive Layout

| ID | Priority | Test steps | Expected result | Coverage source |
| --- | --- | --- | --- | --- |
| RESPONSIVE-001 | High | Set viewport to 320 × 640 px and interact with the map. | No horizontal scrolling; header, controls, selected name, and attribution remain visible and usable. | Main session and builder subagent |
| RESPONSIVE-002 | High | Set viewport width to 768 px and interact with the map. | Map fills viewport; controls and header do not clip or overlap critical content. | Main session and builder subagent |
| RESPONSIVE-003 | High | Test at 1280 × 800 px or larger. | Map fills viewport; header stays centered; controls and attribution remain visible. | Luna and builder subagents |
| RESPONSIVE-004 | Medium | At each viewport, click or keyboard-activate two neighborhoods and preview a park. | Bounds fitting and transient previews work consistently at every size. | Main session review |
| RESPONSIVE-005 | High | With the real imported catalog, pan and zoom at 320 × 640 px and watch the map frame. | The map follows the gesture and repaints pins and polygons after each gesture; no blank or frozen frame, no "page unresponsive" prompt, and no horizontal scrolling. | Performance follow-up and browser check |

## Console and Failure Checks

| ID | Priority | Test steps | Expected result | Coverage source |
| --- | --- | --- | --- | --- |
| CONSOLE-001 | High | Load, pan, zoom, hover, click, and use keyboard selection while watching console. | No uncaught errors or failed local asset requests. | Builder subagent |
| FAILURE-001 | Medium | Temporarily make one GeoJSON request fail in a test environment. | Other polygon data remains usable and the failed request is logged. | Source review |
| FAILURE-002 | Medium | Stop the API, then load and use the map. | Area polygons remain usable and no playground pins are displayed; the failure is logged without an uncaught error. | All-pins failure scenario and browser review |
| FAILURE-003 | Medium | Block the `/graphql` catalog request in browser DevTools (or return an error for it), then reload the map. | No playground pins are displayed, no partial or stale pins remain from earlier renders, and the failure is logged without an uncaught error. | All-pins failure scenario and browser check |
| FAILURE-004 | Medium | Delay neighborhood, destination-area, and playground requests, then return empty or failed responses independently. | The live status region communicates loading, empty, and unavailable states while successfully loaded map layers remain usable. | OpenSpec status-region regression |

## Session Findings Covered

- Keyboard selection did not work with `Enter` or `Space`; covered by `KEYBOARD-001` through `KEYBOARD-004`.
- Outlying settlements appeared as selectable neighborhoods; covered by `DATA-005`.
- Hover showed a tooltip but did not visibly highlight the neighborhood boundary; covered by `POINTER-003` through `POINTER-006`.
- South Park and Sofia Zoo were visible only in raster tiles; covered by `DATA-006` and `POINTER-007`.
- Playground pins stayed in green preview styling after the pointer left; covered by `PLAYGROUND-011`.

## Backend API and Import

Run all cases below with:

```bash
cargo test --manifest-path backend/Cargo.toml
```

Tests that exercise SQL or PostGIS require `DATABASE_URL`. Each integration test
creates and drops its own temporary database. See `LOCAL_SETUP.md` for setup.
Targeted commands referenced below are:

```bash
export DATABASE_URL=postgres://playground:playground@127.0.0.1:5432/playground
cargo test --manifest-path backend/Cargo.toml importer::tests
cargo test --manifest-path backend/Cargo.toml graphql::tests
cargo test --manifest-path backend/Cargo.toml --test postgis
cargo test --manifest-path backend/Cargo.toml --test importer_postgis
```

| ID | Priority | Test | Expected result | Automated coverage |
| --- | --- | --- | --- | --- |
| IMPORT-001 | High | Normalize valid playground nodes, areas, inline equipment, and contained equipment. | Stable OSM IDs, longitude-latitude points, known capability values, and valid optional fields are stored. | `cargo test --manifest-path backend/Cargo.toml importer::tests` |
| IMPORT-002 | High | Normalize missing or invalid optional OSM tags. | Unknown names, ages, images, and capabilities remain `null` or empty; no values are inferred. | `cargo test --manifest-path backend/Cargo.toml importer::tests` |
| IMPORT-003 | High | Import the same complete snapshot twice. | Second import succeeds without duplicate playgrounds or memberships. | `cargo test --manifest-path backend/Cargo.toml --test importer_postgis` |
| IMPORT-004 | High | Fail retrieval, malformed JSON, validation, an Overpass error/remark response, or database replacement. | Command exits unsuccessfully and prior catalog remains usable. | Importer module and `importer_postgis` test commands |
| IMPORT-005 | High | Associate points inside, outside, on boundaries, and inside overlapping neighborhood polygons. | Every covering polygon is stored; outside playground has empty neighborhood list. | Importer module and `importer_postgis` test commands |
| GRAPHQL-001 | High | Inspect GraphQL schema. | `playgrounds` and `playground` queries exist; no playground mutation exists. | `cargo test --manifest-path backend/Cargo.toml graphql::tests` |
| GRAPHQL-002 | High | Search by bounds, radius, neighborhood, age, and multiple required capabilities. | Each filter works; combined filters use AND; radius results include meters. | `cargo test --manifest-path backend/Cargo.toml --test postgis` |
| GRAPHQL-003 | High | Search with unknown age/capability metadata. | Unknown metadata never matches requested attribute filters. | `cargo test --manifest-path backend/Cargo.toml --test postgis` |
| GRAPHQL-004 | High | Submit invalid coordinates, inverted bounds, incomplete center/radius, non-positive radius, age outside `0..=18`, or limit outside `1..=500`. | GraphQL returns a safe input error before database access. | `cargo test --manifest-path backend/Cargo.toml graphql::tests` |
| GRAPHQL-005 | High | Omit pagination, page with a non-negative offset, then submit invalid limit or offset values. | Defaults are limit 200 and offset 0; maximum limit 500 succeeds; ordering is deterministic; invalid values fail. | GraphQL module and `postgis` test commands |
| GRAPHQL-006 | High | Fetch existing, incomplete, and unknown playground IDs. | Existing details include all recorded fields; missing optional data is `null` or empty; unknown ID returns `null`. | GraphQL module and `postgis` test commands |
| CORS-001 | High | Send allowed-origin preflight and blocked-origin requests. | Configured Vite origin receives CORS headers; other origins do not. | `cargo test --manifest-path backend/Cargo.toml graphql::tests` |
| SOURCE-001 | High | Query imported playground source fields. | Stable OSM ID, source URL, `© OpenStreetMap contributors`, and `ODbL-1.0` are returned. | GraphQL module and `postgis` test commands |
