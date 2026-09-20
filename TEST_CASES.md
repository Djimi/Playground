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

Start the application with `npm run dev`, then run the manual cases below. Test at 320 px phone, 768 px tablet, and desktop widths unless a case specifies otherwise.

## Data and Build

| ID | Priority | Test | Expected result | Coverage source |
| --- | --- | --- | --- | --- |
| DATA-001 | High | Load `public/data/sofia-neighborhoods.geojson`. | Valid `FeatureCollection`; scope is Sofia relation `4283101`; license is `ODbL-1.0`. | Automated test |
| DATA-002 | High | Validate every feature. | Each feature has a unique stable ID, unique name, and `Polygon` or `MultiPolygon` geometry. | Automated test |
| DATA-003 | High | Check all displayed names. | Names contain no Cyrillic characters. | Automated test |
| DATA-004 | High | Check required neighborhoods. | Lozenets and Mladost 1, 1A, 2, 3, and 4 are present. | Automated test and UI review |
| DATA-005 | High | Check excluded outlying settlements. | Benkovski, Chelopechene, Kremikovtsi, Seslavtsi, and Trebich are absent. | Reported regression and automated test |
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

## Pointer Selection and Hover

| ID | Priority | Test steps | Expected result | Coverage source |
| --- | --- | --- | --- | --- |
| POINTER-001 | High | Click Lozenets. | Lozenets receives selected styling and its English name appears in the header. | Main session and subagent review |
| POINTER-002 | High | Select Lozenets, then select Mladost 3. | Lozenets returns to default styling; only Mladost 3 remains selected. | Main session and subagent review |
| POINTER-003 | Medium | Hover an unselected neighborhood. | Hovered polygon receives a clearly visible boundary/fill highlight and tooltip. | User-reported regression |
| POINTER-004 | Medium | Move pointer away from an unselected neighborhood. | Polygon returns to default styling; no stale hover highlight remains. | User-reported regression |
| POINTER-005 | Medium | Select a polygon, then hover it and move pointer away. | Selected styling remains after hover ends. | Hover regression follow-up |
| POINTER-006 | Medium | Move pointer directly between adjacent polygons. | Highlight follows the pointer; previous polygon does not remain highlighted. | Hover regression follow-up |

## Keyboard and Accessibility

| ID | Priority | Test steps | Expected result | Coverage source |
| --- | --- | --- | --- | --- |
| KEYBOARD-001 | High | Press `Tab` until a neighborhood polygon receives focus. | Focus reaches polygons; each exposes a readable `Select <name>` label and button role. | Reported regression and Luna verification |
| KEYBOARD-002 | High | Focus Lozenets and press `Enter`. | Lozenets becomes selected/highlighted and its name appears in the header. | Reported regression and Luna verification |
| KEYBOARD-003 | High | Focus Mladost 1 and press `Space`. | Mladost 1 replaces the previous selection; page does not scroll. | Reported regression and Luna verification |
| KEYBOARD-004 | High | Select one polygon by keyboard, then another. | Only the latest polygon keeps selected styling. | Luna verification |
| ACCESS-001 | Medium | Change neighborhood selection. | Header name update is announced through the live region. | Source review |

## Responsive Layout

| ID | Priority | Test steps | Expected result | Coverage source |
| --- | --- | --- | --- | --- |
| RESPONSIVE-001 | High | Set viewport to 320 × 640 px and interact with the map. | No horizontal scrolling; header, controls, selected name, and attribution remain visible and usable. | Main session and builder subagent |
| RESPONSIVE-002 | High | Set viewport width to 768 px and interact with the map. | Map fills viewport; controls and header do not clip or overlap critical content. | Main session and builder subagent |
| RESPONSIVE-003 | High | Test at 1280 × 800 px or larger. | Map fills viewport; header stays centered; controls and attribution remain visible. | Luna and builder subagents |
| RESPONSIVE-004 | Medium | At each viewport, click or keyboard-select two neighborhoods. | Selection and replacement work consistently at every size. | Builder subagent |

## Console and Failure Checks

| ID | Priority | Test steps | Expected result | Coverage source |
| --- | --- | --- | --- | --- |
| CONSOLE-001 | High | Load, pan, zoom, hover, click, and use keyboard selection while watching console. | No uncaught errors or failed local asset requests. | Builder subagent |
| FAILURE-001 | Medium | Temporarily make the GeoJSON request fail in a test environment. | Header changes to `Neighborhoods unavailable`; error is logged once. | Future regression test |

## Session Findings Covered

- Keyboard selection did not work with `Enter` or `Space`; covered by `KEYBOARD-001` through `KEYBOARD-004`.
- Outlying settlements appeared as selectable neighborhoods; covered by `DATA-005`.
- Hover showed a tooltip but did not visibly highlight the neighborhood boundary; covered by `POINTER-003` through `POINTER-006`.

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
| GRAPHQL-005 | High | Omit limit, use maximum limit, then exceed maximum. | Default is 200; maximum 500 succeeds; larger values fail; ordering is deterministic. | GraphQL module and `postgis` test commands |
| GRAPHQL-006 | High | Fetch existing, incomplete, and unknown playground IDs. | Existing details include all recorded fields; missing optional data is `null` or empty; unknown ID returns `null`. | GraphQL module and `postgis` test commands |
| CORS-001 | High | Send allowed-origin preflight and blocked-origin requests. | Configured Vite origin receives CORS headers; other origins do not. | `cargo test --manifest-path backend/Cargo.toml graphql::tests` |
| SOURCE-001 | High | Query imported playground source fields. | Stable OSM ID, source URL, `© OpenStreetMap contributors`, and `ODbL-1.0` are returned. | GraphQL module and `postgis` test commands |
