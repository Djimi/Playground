# Tasks

## 1. Data and Application Setup

- [x] 1.1 Select a neighborhood geometry source whose license permits redistribution, record its URL, version or retrieval date, license, and required attribution, and verify the source covers Lozenets plus separate Mladost 1, 1A, 2, 3, and 4 polygons
- [x] 1.2 Create the minimal vanilla JavaScript application with Vite and Leaflet, add `dev`, `build`, and `test` scripts, and verify dependency installation and an empty production build succeed
- [x] 1.3 Normalize approved geometry into one local GeoJSON asset containing only Sofia city neighborhoods with stable unique `id` and curated Latin-script `name` properties, and verify surrounding towns and villages are absent

## 2. Interactive Map

- [x] 2.1 Render a Leaflet map focused on Sofia with the configured tile provider, local neighborhood polygons, and all required attribution, and verify the layer aligns while panning and zooming
- [x] 2.2 Add tap and click selection that highlights exactly one polygon and shows its English-transliterated name, and verify changing selection clears the previous highlight
- [x] 2.3 Add responsive styles for map controls, attribution, and the selected-name panel, and verify the page works without horizontal scrolling at 320 CSS pixels and remains usable at tablet and desktop widths

## 3. Verification

- [x] 3.1 Add one Node built-in test that validates GeoJSON structure, supported polygon geometry, unique IDs and names, Latin-script display names, required Lozenets and Mladost features, and the Sofia-city inclusion set; verify `npm test` passes
- [x] 3.2 Run the production build and manually verify mouse, keyboard, and touch-style interaction plus visible attribution at phone, tablet, and desktop viewport sizes
