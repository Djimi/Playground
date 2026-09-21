# Design

## Context

The repository already has the complete local runtime split across Docker Compose (`db` and `api`) and Vite (`npm run dev`). The API exposes `/graphql` and `/graphiql`, while the frontend is served on Vite's default local port. There is no dedicated health endpoint, so readiness must use existing HTTP behavior.

## Goals / Non-Goals

**Goals:**

- Make one npm command the normal local entry point.
- Start the existing services in the order they require: Compose database/API first, then the Vite server.
- Verify both HTTP surfaces before printing the ready message.
- Preserve existing `.env`, Compose, import, and shutdown behavior.
- Keep the command dependency-free and easy to debug.

**Non-Goals:**

- Add a production health endpoint or change the GraphQL schema.
- Put the frontend into Docker.
- Run the Overpass importer during startup.
- Replace the existing automated test suite with smoke checks.

## Decisions

### Use a Node startup script exposed through npm

Add `scripts/start-local.mjs` and expose it as `npm run start:local`. Node 22 already provides `fetch`, child-process management, and filesystem/process APIs, so no process runner or HTTP dependency is needed. A Node script also keeps the command usable anywhere the project already runs, without relying on shell-specific features.

Alternative considered: a shell script running background jobs. It would be shorter on one platform but would make signal forwarding, bounded HTTP checks, and useful diagnostics less portable.

### Reuse Docker Compose and Vite as the service owners

The script runs the existing Compose `db` and `api` services with the repository's configuration, then starts Vite as an attached child process with a fixed local host/port. It does not duplicate container configuration or add another supervisor.

Alternative considered: add a frontend Compose service. That would introduce a second development-server configuration and a container lifecycle that is unnecessary for the current Vite workflow.

### Check existing endpoints instead of adding a health route

Poll the API until a GraphQL request such as `playgrounds(limit: 1)` returns HTTP success with no GraphQL errors, then poll the frontend root until it returns HTTP success. The query is valid even when the catalog is empty, so startup checks do not require a fresh data import. Resolve the published API port through the existing Compose mapping so `.env` overrides continue to work.

Alternative considered: add `/health`. That would be a new API surface for a local-development concern and would not prove that migrations and GraphQL execution are working.

### Keep readiness bounded and diagnostics local

Use a short polling interval and a fixed startup timeout. On timeout or failed smoke check, return non-zero status and print the relevant service URLs plus recent Compose logs. Do not report the application as ready until both checks pass.

### Keep interruption safe

Forward `SIGINT`/`SIGTERM` to the Vite child and wait for it to exit. Do not remove the named Postgres volume; imported local data must survive a restart. Compose services may be stopped separately with the existing documented command.

## Risks / Trade-offs

- [Docker is unavailable or image build is slow] → Fail before claiming readiness and point to the prerequisite or recent Compose logs.
- [Port 5173 or the configured API port is already occupied] → Use strict Vite port binding and report the conflicting endpoint instead of silently selecting another port.
- [The API has no imported playground rows] → Treat an empty successful GraphQL result as healthy; importing remains an explicit operator action.
- [Smoke checks are weaker than browser interaction] → Keep them intentionally small and retain the existing build, unit, integration, and manual UI checks for deeper verification.

## Migration Plan

Add the script, npm entry point, and documentation in one change. Existing commands remain valid, so rollback is limited to removing the new entry point/script and reverting the documentation. No database or production migration is required.
