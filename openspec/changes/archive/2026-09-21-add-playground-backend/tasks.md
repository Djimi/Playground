# Tasks

## 1. Backend and Local Runtime

- [x] 1.1 Create one Rust package under `backend/` with Axum, async-graphql, SQLx, Tokio, tracing, HTTP, and serialization dependencies; verify `cargo check` and `cargo fmt --check` pass.
- [x] 1.2 Add a minimal API container, PostGIS container, named database volume, environment example, and ignored local secrets file to Docker Compose; verify `docker compose config` succeeds and both services start.

## 2. PostgreSQL and PostGIS Storage

- [x] 2.1 Add a SQLx migration that enables PostGIS and creates `neighborhoods`, `playgrounds`, and `playground_neighborhoods` with required constraints and GiST, GIN, and membership indexes; verify migration succeeds against an empty local database and creates the expected constraints and indexes.
- [x] 2.2 Add database configuration, connection pooling, and embedded startup migrations; verify the API starts with a valid database and exits clearly when `DATABASE_URL` is invalid.

## 3. OpenStreetMap Import

- [x] 3.1 Implement the Sofia Overpass request and normalize playground nodes and areas into stable identifiers, valid representative points, optional names, ages, images, source metadata, and the documented capability set; verify fixture tests cover inline tags, contained equipment, missing fields, invalid fields, and longitude-latitude order.
- [x] 3.2 Load the existing neighborhood GeoJSON and store every polygon membership that covers each playground; verify tests cover one neighborhood, nested aggregate and subdivision polygons, shared boundaries, and locations outside all polygons.
- [x] 3.3 Replace the OpenStreetMap snapshot through staging and one database transaction; verify integration tests prove repeated imports create no duplicates and retrieval, Overpass error or remark, validation, and database failures preserve the prior catalog.
- [x] 3.4 Expose the repeatable importer binary with configurable Overpass URL and clear counts/errors; verify it imports a fixture endpoint successfully and exits non-zero for invalid, errored, and remarked responses.

## 4. GraphQL Discovery API

- [x] 4.1 Define query-only GraphQL types for `Playground`, its neighborhood list, `PlaygroundFilter`, bounds, coordinates, capabilities, source metadata, and the `playgrounds` and `playground` fields; verify schema inspection exposes both queries and no playground mutations.
- [x] 4.2 Validate coordinates, bounds, center-radius pairs, child ages in `0..=18`, and the `1..=500` limit before database access; verify focused tests cover inclusive and one-sided age bounds plus every rejection case.
- [x] 4.3 Implement parameterized SQLx detail and search queries using PostGIS for bounds, `ST_DWithin` radius, distance ordering, any matching neighborhood membership, age, and all-capability filtering; verify PostGIS integration tests cover each filter, combined filters, nested neighborhoods, unknown metadata exclusion, deterministic ordering, default limit, and missing identifiers.
- [x] 4.4 Serve `POST /graphql` and local `/graphiql` and restrict CORS to the configured Vite origin; verify HTTP tests cover existing-playground details, missing optional fields as `null` or empty lists, a missing identifier, safe internal errors, blocked origins, and allowed preflight.

## 5. Documentation and Verification

- [x] 5.1 Update local setup and data-source documentation with Docker Compose, migrations, import usage, GraphiQL examples, environment variables, OpenStreetMap attribution, and shutdown steps; verify every documented command works from a clean checkout.
- [x] 5.2 Add backend cases to `TEST_CASES.md`, including import safety, schema shape, filters, limits, CORS, and source attribution; verify each automated case maps to a runnable Rust or existing Node test.
- [x] 5.3 Run `cargo fmt --check`, `cargo clippy --all-targets --all-features -- -D warnings`, backend tests against PostGIS, `npm test`, `npm run build`, and `npm audit`; verify every gate passes without changing existing neighborhood-map behavior.
