# Local setup

## Requirements

- Node.js 22 or newer
- npm
- Internet access for OpenStreetMap tiles and neighborhood data preparation

## Install

From repository root:

```bash
npm install
```

## Run locally

```bash
npm run dev
```

Open the URL printed by Vite, usually `http://127.0.0.1:5173/`.

The app is a static browser map. It has no local database or backend service.

## Verify changes

```bash
npm test
npm run build
npm audit
```

`npm test` runs the built-in Node test runner. `npm run build` writes the production bundle to `dist/`.

## Refresh neighborhood data

```bash
node scripts/prepare-neighborhoods.mjs
```

This calls Overpass and Nominatim, batches requests, and writes the local GeoJSON file under `public/data/`. Respect upstream rate limits. Read [public/data/README.md](public/data/README.md) before redistributing the data.

## Stop the dev server

Press `Ctrl+C` in the terminal running Vite.
