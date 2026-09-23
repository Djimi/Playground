# Local setup

## Requirements

- Node.js 22 or newer
- npm
- Docker with Docker Compose
- Rust 1.90 with Cargo for backend checks
- Internet access for OpenStreetMap tiles and data imports

## Install

From repository root:

```bash
npm install
cp .env.example .env
```

The example environment values are for local development only. Change
`POSTGRES_PASSWORD` if the database port is reachable by other machines.

## Start everything locally

From the repository root:

```bash
npm run start:local
```

This starts the PostGIS database, GraphQL API, and Vite frontend, then waits
for a successful frontend request and a real GraphQL query before reporting
that the application is ready. It does not import playground data from
Overpass automatically. Press `Ctrl+C` to stop the frontend; the Docker
services and named database volume remain available for the next start.

The usual URLs are `http://127.0.0.1:5173/` and
`http://127.0.0.1:3000/graphiql`.

## Run services separately

Use these commands when debugging one part of the local stack.

### Frontend

```bash
npm run dev
```

Open the URL printed by Vite, usually `http://localhost:5173/`.

The frontend loads selectable neighborhood, South Park, and Sofia Zoo polygons.
It also queries the backend for playground pins inside the visible map bounds,
so run the backend and import once to see pins.

### Backend

From repository root:

```bash
docker compose up --build -d db api
docker compose logs -f api
```

The API waits for PostGIS, connects to it, and applies embedded SQLx migrations
before listening. A migration failure stops the API instead of serving against
an outdated schema.

Open GraphiQL at `http://127.0.0.1:3000/graphiql`. Try:

```graphql
query NearbyPlaygrounds {
  playgrounds(
    filter: {
      center: { longitude: 23.3219, latitude: 42.6977 }
      radiusMeters: 1000
      requiredCapabilities: [SWING, SLIDE]
    }
    limit: 20
  ) {
    id
    name
    location { longitude latitude }
    distanceMeters
    capabilities
    neighborhoods { id name }
    source { url attribution license }
  }
}
```

GraphQL clients send the same query with `POST http://127.0.0.1:3000/graphql`.

## Import playgrounds

With PostGIS running, fetch one complete Sofia playground snapshot from
OpenStreetMap and SofiaPlan:

```bash
docker compose run --rm api import-playgrounds
```

This is a one-time/manual operator action; there is no scheduled import. The
command reports OSM and SofiaPlan source records, canonical playgrounds, clear
matches, ambiguous source records, excluded source records, accepted/rejected
Commons photos, neighborhoods, and memberships. Source counts include retained
records excluded from the canonical catalog. Ambiguous records remain separate;
clear matches merge two source records into one canonical playground. Photo
counts describe directly referenced Commons files accepted or rejected by
licence/metadata checks, not all playgrounds with photos.

The importer validates both primary sources before replacing the current
catalog in one transaction. A failed fetch, validation, or database write exits
with an error and preserves the prior catalog. Commons photo failures omit those
photos without aborting the import. Overpass is a shared public service; run
imports manually and respect its usage policy.

## Environment variables

| Variable | Default | Purpose |
| --- | --- | --- |
| `POSTGRES_PASSWORD` | `playground` | Local database password and API connection password |
| `POSTGRES_PORT` | `5432` | Host port for PostGIS |
| `API_PORT` | `3000` | Host port for API and GraphiQL |
| `FRONTEND_ORIGIN` | `http://localhost:5173` | Only direct browser origin allowed by API CORS |
| `VITE_API_URL` | `/graphql` | Browser API URL; the Vite dev server proxies this to `API_PORT` |
| `OVERPASS_URL` | `https://overpass-api.de/api/interpreter` | Import source endpoint |
| `SOFIAPLAN_URL` | `https://api.sofiaplan.bg/datasets/5` | SofiaPlan GeoJSON endpoint |
| `COMMONS_API_URL` | `https://commons.wikimedia.org/w/api.php` | Directly referenced Commons photo metadata endpoint |
| `DATABASE_URL` | Set by Compose | Required PostgreSQL connection for direct Rust runs |
| `API_ADDR` | `0.0.0.0:3000` | API bind address inside its runtime |
| `NEIGHBORHOODS_PATH` | Set by Compose | Curated GeoJSON read by the importer |

Inside Compose, `DATABASE_URL`, `API_ADDR`, and `NEIGHBORHOODS_PATH` are set for
the containers. When running Rust commands directly, set `DATABASE_URL`; the API
also accepts `API_ADDR` and `FRONTEND_ORIGIN`, while the importer accepts
`OVERPASS_URL`, `SOFIAPLAN_URL`, `COMMONS_API_URL`, and `NEIGHBORHOODS_PATH`.

## Verify changes

```bash
npm test
npm run build
npm audit
cargo fmt --manifest-path backend/Cargo.toml --check
cargo clippy --manifest-path backend/Cargo.toml --all-targets --all-features -- -D warnings
```

`npm test` runs the built-in Node test runner. `npm run build` writes the production bundle to `dist/`.

Run PostGIS integration tests against the Compose database with the example
credentials:

```bash
docker compose up -d db
DATABASE_URL=postgres://playground:playground@127.0.0.1:5432/playground \
  cargo test --manifest-path backend/Cargo.toml
```

Each PostGIS integration test creates and drops its own temporary database.

## Refresh neighborhood data

```bash
node scripts/prepare-neighborhoods.mjs
```

This calls Overpass and Nominatim, batches requests, and writes the neighborhood
and discovery-area GeoJSON files under `public/data/`. Respect upstream rate
limits. Read [public/data/README.md](public/data/README.md) before redistributing
the data.

## Stop the dev server

Press `Ctrl+C` in the terminal running `npm run start:local` or Vite.
The one-command startup keeps the Docker services running for reuse.

Stop backend containers while preserving imported data:

```bash
docker compose down
```

The named Postgres volume remains. Delete it only when local imported data is no
longer needed:

```bash
docker compose down --volumes
```
