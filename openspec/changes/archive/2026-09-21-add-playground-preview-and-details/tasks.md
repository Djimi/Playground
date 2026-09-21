# Tasks

## 1. Equipment Data

- [x] 1.1 Add the additive `equipment_counts` migration with an empty-object default and JSON-object validation, and verify existing rows retain their capabilities with empty counts in a migration test.
- [x] 1.2 Preserve counts for separately mapped equipment during import while leaving boolean-only equipment counts unknown, and verify importer unit and PostGIS tests cover repeated features, mixed known and unknown quantities, and snapshot replacement.
- [x] 1.3 Add deterministic GraphQL `Equipment` output while retaining `capabilities`, and verify API tests cover positive counts, `null` counts, empty inventory, ordering, search, and single-playground lookup.

## 2. Playground Preview and Details

- [x] 2.1 Extend the visible-bounds query with preview fields and add the single-playground detail request, and verify stale detail requests are aborted and `npm run build` succeeds.
- [x] 2.2 Add compact pointer-hover and keyboard-focus previews with photo, name fallback, age, equipment, `No ratings yet`, and preview pin styling, and verify pointer exit, focus movement, and missing-data states in a browser.
- [x] 2.3 Add the semantic playground detail view with gallery, age, equipment inventory, empty platform-review state, source attribution, selected-pin styling, and an accessible close control; verify click, tap, Enter, and Space open the same details.
- [x] 2.4 Make playground details fill a 320 CSS pixel phone viewport without horizontal scrolling and remain usable beside or over the map at tablet and desktop sizes; verify at 320, 768, and 1280 CSS pixel widths.

## 3. Focused Map Navigation

- [x] 3.1 Save the viewport before area focus and add one Back action that closes playground details before clearing area focus and restoring that viewport; verify each state transition preserves the expected selection and view.
- [x] 3.2 Bind `Escape` to the same Back action without changing native Leaflet wheel, touch, keyboard, or zoom-control behavior; verify zoom requires no `Ctrl` modifier and Escape is inert in the default state.

## 4. Verification and Documentation

- [x] 4.1 Extend `TEST_CASES.md` with preview, detail, missing-data, equipment-count, responsive, Back, and Escape cases, and verify every added requirement scenario has a matching automated or manual check.
- [x] 4.2 Run backend formatting and tests plus frontend tests and production build, then complete browser checks for pointer, keyboard, touch-sized layout, selected styling, console errors, and source attribution.
