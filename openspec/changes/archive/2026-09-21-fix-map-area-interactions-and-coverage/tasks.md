# Tasks

## 1. Correct neighborhood data

- [x] 1.1 Add verified, attributed curated geometry for `Raina Knyaginya` to the neighborhood preparation flow and bundled GeoJSON; verify `test/neighborhoods.test.js` requires its unique Latin-script feature.
- [x] 1.2 Select Hadzhi Dimitar's canonical source geometry instead of merging its nested duplicate; verify its bundled feature has one interactive polygon and `npm test` passes.
- [x] 1.3 Keep `Lyulin 1`–`Lyulin 10`, `Lyulin Center`, and `Nadezhda 1`–`Nadezhda 4` as separate interactive features, while excluding aggregate Lyulin geometry; verify required-name and excluded-aggregate assertions in `test/neighborhoods.test.js`.

## 2. Correct map interactions

- [x] 2.1 Replace persistent area selection and suppressed-hover state with transient hover/focus preview plus click, tap, Enter, and Space fit-to-bounds; verify no area remains highlighted after preview ends.
- [x] 2.2 Give default playground pins a style distinct from area preview and selected-pin styles; verify pin hover, focus, and detail selection still show their existing cues.
- [x] 2.3 Close playground details on an empty map click and consume an area click while details are open; verify next area click fits bounds, while Back and Escape still close details without moving map.

## 3. Verify map behavior

- [x] 3.1 Update focused manual cases for Raina Knyaginya, Lyulin and Nadezhda microregions, single Hadzhi Dimitar geometry, transient neighborhood and park previews, pin distinction, and two-click detail dismissal; verify each behavior at desktop and 320 CSS pixels.
- [x] 3.2 Run `npm test`, `npm run build`, `git diff --check`, and `openspec validate fix-map-area-interactions-and-coverage --type change --strict`; verify all commands pass.
