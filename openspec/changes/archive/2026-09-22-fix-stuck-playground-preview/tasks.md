# Tasks

## 1. Fix stuck preview styling

- [x] 1.1 In `src/main.js` `closePreview()`, stop treating the marker as the previewed one when its style is recomputed (clear `previewMarker` before, or pass the marker to, the reset call), and verify by reading the code that `markerStyle()` can no longer return `PREVIEW_STYLE` for a marker being reset.
- [x] 1.2 In a browser, hover a playground pin and move the pointer away; verify the compact preview closes and the pin returns to default neutral blue, per the new spec scenarios.
- [x] 1.3 In a browser, move keyboard focus to a playground pin and then away; verify the compact preview closes and the pin returns to default neutral blue.

## 2. Regression checks

- [x] 2.1 Verify preview then click still opens details with the pin in selected orange styling and no compact preview popup left open.
- [x] 2.2 Verify hovering a pin then clicking empty map space (without opening details) leaves every pin in default styling, so the previous manual workaround is no longer needed.
- [x] 2.3 Run `npm test` and confirm the existing test suite passes.
