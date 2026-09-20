# Design

## Context

The repository currently contains a static Vite, vanilla JavaScript, and Leaflet application. It loads curated neighborhood polygons from a local GeoJSON file and has no backend, database, authentication, or runtime playground data. See `proposal.md` for motivation and `specs/playground-discovery/spec.md` for required behavior.

The agreed first backend slice is read-only and local-development only. The agreed stack is Rust, Axum, async-graphql, SQLx, PostgreSQL with PostGIS, and Docker Compose. The browser will use native `fetch` when frontend integration is added.

## Goals / Non-Goals

**Goals:**

- Keep one small backend service and one database.
- Make geographic filtering accurate, indexed, and explicit about meters.
- Keep OpenStreetMap import separate from request-time API availability.
- Preserve incomplete source data instead of guessing missing attributes.
- Leave a reproducible local setup and focused automated checks.

**Non-Goals:**

- Production hosting, high availability, or deployment automation.
- Frontend playground rendering or a GraphQL client library.
- Mutations, accounts, reviews, uploads, moderation, or admin tools.
- A generic repository layer, service layer, plugin system, or ORM.

## Decisions

### Use one Rust package for the API and importer

Create one Cargo package under `backend/`. Its default binary serves HTTP. A second binary performs the OpenStreetMap import while reusing the same database types and queries. Keep modules organized by API, database, and import responsibilities only where needed; do not introduce interfaces with one implementation.

Axum owns HTTP routing and shared state. async-graphql owns schema validation and resolver execution. Tokio provides the async runtime required by both. SQLx provides the PostgreSQL pool, migrations, and direct SQL.

Alternative: separate services or crates for import and API. Rejected because the first slice has one deployment boundary and one database.

### Expose a query-only GraphQL schema

Serve GraphQL at `POST /graphql` and GraphiQL at `/graphiql` for local learning and inspection. The query root contains:

```graphql
type Query {
  playgrounds(filter: PlaygroundFilter, limit: Int = 200): [Playground!]!
  playground(id: ID!): Playground
}
```

`PlaygroundFilter` accepts an optional bounding box, optional center and radius, neighborhood identifier, child age, and required capabilities. All supplied filters combine with logical AND. The default limit is 200 and the maximum is 500. Radius results order by distance then stable identifier; other results order by stable identifier. `distanceMeters` is nullable and is populated only for radius searches.

Input validation rejects invalid coordinates, inverted bounds, incomplete center/radius pairs, non-positive radii, child ages outside `0..=18`, and limits outside `1..=500` before SQL runs. A single recorded age bound is open-ended; a playground with neither bound does not match an age filter. Return safe GraphQL errors without database or configuration details.

Alternative: REST. Rejected because GraphQL was selected as a learning goal. Alternative: Apollo Client in the browser. Rejected because two queries need only native `fetch`.

### Store a compact relational catalog with PostGIS geography

Use three tables:

```text
neighborhoods
  id text primary key
  name text not null
  boundary geometry(MultiPolygon, 4326) not null

playgrounds
  id text primary key                 # canonical OSM type/id, such as node/123
  name text null
  location geography(Point, 4326) not null
  capabilities text[] not null
  min_age smallint null
  max_age smallint null
  photo_urls text[] not null
  source_url text not null
  source_updated_at timestamptz null

playground_neighborhoods
  playground_id text references playgrounds(id)
  neighborhood_id text references neighborhoods(id)
  primary key (playground_id, neighborhood_id)
```

Add GiST location, membership lookup, and GIN capability indexes. Use `ST_DWithin` for radius filtering, converting the requested radius to meters through the `geography` type. Use PostGIS containment for map bounds and `ST_Covers` for neighborhood memberships, including nested and boundary matches. Always construct points as longitude then latitude.

Use SQLx migrations as the only schema history. Use direct parameterized SQL with SQLx row mapping; no ORM, prepared metadata workflow, or repository abstraction.

Alternative: PostgreSQL latitude and longitude columns. Rejected because indexed distance and containment queries would require custom calculations. Alternative: MongoDB. Rejected because upcoming users and reviews favor relational constraints, while PostGIS covers current geographic needs.

### Import an atomic OpenStreetMap snapshot outside request handling

The importer reads playgrounds tagged `leisure=playground` within Sofia from Overpass. It also reads equipment mapped either on the playground through `playground:<device>=yes` or as contained `playground=*` features. Normalize an initial explicit set including swing, slide, climbing frame, sandpit, seesaw, springy, playhouse, and roundabout. Unknown equipment remains unreported until deliberately mapped.

For playground areas, use a representative point guaranteed to lie on the area rather than a simple centroid. Parse `min_age`, `max_age`, direct HTTP(S) image values, names, and source timestamps only when valid. Missing or invalid optional tags remain unknown. The API supplies the static ODbL license and OpenStreetMap attribution beside each record's source identifier and URL without duplicating those constants in every database row.

The existing `public/data/sofia-neighborhoods.geojson` remains the curated neighborhood source of truth. Each import refreshes its rows in `neighborhoods`, stages normalized playgrounds, records every polygon that covers each playground in `playground_neighborhoods`, and replaces the prior OpenStreetMap snapshot in one database transaction. This preserves aggregate and subdivision filters where polygons overlap. Fetching and validating remote data happens before that transaction. Reject malformed JSON and any Overpass error or remark marker, so a failed or partial response leaves the prior catalog usable. Stable playground identifiers combine OpenStreetMap element type and identifier.

Alternative: call Overpass from GraphQL resolvers. Rejected because upstream outages, latency, and rate limits must not affect normal reads. Alternative: infer missing equipment or ages. Rejected because unknown data must not become a false claim.

### Run the API and PostGIS with Docker Compose locally

Add one Compose service for the Rust API and one for a pinned PostGIS image with a named database volume. The API runs embedded SQLx migrations before serving and receives configuration through environment variables. Keep credentials in an ignored local environment file and provide non-secret example values.

The Vite development server remains separate. Allow only its configured local origin through CORS; do not use a wildcard origin. Production topology and secrets management remain deferred.

Alternative: require locally installed PostgreSQL and PostGIS. Rejected because Compose gives one repeatable setup across developer machines.

### Verify behavior at the GraphQL and database boundaries

Use small Rust tests for input validation and OpenStreetMap normalization. Use integration tests against PostGIS for migrations, geographic filters, combined capability filters, deterministic limits, missing records, idempotent import, and failed-import preservation. Keep the existing Node tests and static map behavior unchanged.

## Risks / Trade-offs

- **Sparse OpenStreetMap metadata:** Many playgrounds may lack equipment, age, name, or photo tags. Expose unknowns and document coverage instead of inferring values.
- **Equipment may be mapped as child features:** Import both inline `playground:*` tags and contained equipment features; retain fixtures for both forms.
- **Overpass can rate-limit or fail:** Import only through an operator command, validate before changing the database, and keep the previous snapshot on failure.
- **Geographic coordinate order is easy to reverse:** Name longitude and latitude explicitly and cover Sofia coordinates with an integration test.
- **Nested neighborhood polygons overlap:** Store every covering membership so aggregate and subdivision filters both work.
- **Local cross-origin requests can be misconfigured:** Permit one configured Vite origin and test the preflight response.
- **A full snapshot replacement removes vanished OSM features:** Treat OpenStreetMap as the sole playground source in this slice and log import counts before commit.

## Migration Plan

1. Add the Rust package, Compose services, and initial PostGIS migration without changing the current frontend runtime path.
2. Start the local services and apply migrations automatically.
3. Run the importer against fixtures, then against OpenStreetMap when network access is available.
4. Verify GraphQL schema, detail lookup, filters, attribution fields, and existing frontend checks.

Rollback stops the API and database services and restores the prior local workflow. The existing static frontend and neighborhood GeoJSON remain usable. Keep the named database volume unless the operator explicitly chooses to delete local imported data.
