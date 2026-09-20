# Proposal

## Why

The static neighborhood map cannot yet discover actual playgrounds or filter them by location and features. A read-only backend is needed to import Sofia playground data and expose it through a typed API before accounts, reviews, or public submissions are added.

## What Changes

- Add a Rust backend with a read-only GraphQL API for playground search and details.
- Store playgrounds in PostgreSQL with PostGIS location data and spatial indexes.
- Support filtering by map bounds, distance, neighborhood, age suitability, and required capabilities.
- Import Sofia playgrounds from OpenStreetMap and associate them with the existing curated neighborhood dataset.
- Provide local development with Docker Compose, database migrations, and repeatable import commands.
- Keep the existing neighborhood map behavior unchanged.
- Exclude authentication, reviews, mutations, uploads, moderation, and production deployment.

## Capabilities

### New Capabilities

- `playground-discovery`: Read-only GraphQL discovery of imported Sofia playgrounds, including geographic and attribute filters, details, and source metadata.

### Modified Capabilities

None.

## Impact

- Adds a Rust service using Axum, async-graphql, and SQLx.
- Adds PostgreSQL with PostGIS, schema migrations, and local Docker Compose services.
- Adds an OpenStreetMap playground importer that reuses existing neighborhood polygons for assignment.
- Introduces `/graphql` as the backend API endpoint and a development GraphiQL interface.
- Extends local setup, verification commands, test cases, and source-attribution documentation.
