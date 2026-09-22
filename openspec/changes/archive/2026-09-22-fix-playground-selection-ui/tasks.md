# Tasks

## 1. Map interaction state

- [x] 1.1 Fix playground selection state ordering so opening a second pin repaints the previous pin as neutral and leaves exactly one orange pin; verify by opening three different pins in the browser and counting selected styles.
- [x] 1.2 Separate pointer-hover and keyboard-focus preview state for area polygons and playground pins, including restoration after unrelated hover ends; verify with pointer transitions and Tab focus at desktop and phone widths.
- [x] 1.3 Configure the existing Leaflet preview popup to remain inside the usable map viewport near edges and the fixed header; verify with edge pins and no clipped preview content.

## 2. Details and data feedback

- [x] 2.1 Make the details overlay modal for keyboard users, including modal semantics, background inertness, Tab containment, Escape/Back close behavior, and focus restoration; verify at 320 px and 1280 px widths.
- [x] 2.2 Add an accessible status region for playground/area loading, empty, and failure states without hiding successfully loaded layers; verify with delayed, empty, and failed requests.
- [x] 2.3 Use playground names in image alternative text and replace failed photo elements with an explicit unavailable-photo state; verify with a named image and a blocked image URL.

## 3. Regression coverage and handoff

- [x] 3.1 Extend `TEST_CASES.md` and the relevant Leaflet gotcha note with repeated-pin, hover/focus, popup-edge, modal-focus, status, and photo-fallback checks; verify the documented cases match the modified specification.
- [x] 3.2 Run `npm test` and `npm run build`, then perform the focused browser checks with no uncaught console errors; verify the final diff contains only the scoped frontend, documentation, and OpenSpec changes.
