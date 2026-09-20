# Vite

## `public/` assets use root URLs

Symptom: a local asset exists but the browser returns 404.

Cause: files under `public/` are served from the site root, not from `/public/`.

Fix: fetch `/data/sofia-neighborhoods.geojson`, not `/public/data/sofia-neighborhoods.geojson`.

Verify with `npm run dev` and a browser network check.

## `dist/` is generated

Symptom: generated build files appear as noisy source changes.

Cause: `npm run build` writes compiled assets to `dist/`.

Fix: keep `dist/` in `.gitignore`; commit source files and the lockfile instead.

Verify with `git status` after `npm run build`.
