# Node.js and GeoJSON

## Use the built-in test runner

Symptom: a test command requires an extra test framework for simple data checks.

Cause: the project already runs on modern Node.js.

Fix: use `node:test`, `node:assert/strict`, and `npm test`.

Verify with `npm test`.

## Resolve local JSON with module URLs

Symptom: a test or script cannot find the GeoJSON file when run from another working directory.

Cause: relative filesystem paths depend on the current shell directory.

Fix: resolve paths from `import.meta.url`, then read the resulting URL.

Verify by running `npm test` from repository root and from a different working directory.

## Normalize before runtime

Symptom: the browser must handle source-specific names, scope filters, and geometry quirks.

Cause: raw OSM responses vary and upstream data can change.

Fix: prepare one local GeoJSON asset, assign stable IDs and Latin names, and validate it before serving.

Verify the data assertions in `test/neighborhoods.test.js`.
