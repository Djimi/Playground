# Proposal

## Why

Starting the product currently requires the developer or an LLM to rediscover several commands, start the database/API separately, and manually verify readiness. A single repository command can make local startup repeatable, faster, and cheaper in tokens while catching common startup failures immediately.

## What Changes

- Add one documented local-start command that starts the existing PostGIS/API services and Vite development server.
- Wait for the services to become reachable before reporting success.
- Run lightweight smoke checks against the frontend and GraphQL API, with actionable failure output and a non-zero exit status on failure.
- Keep the existing manual playground import workflow unchanged; startup must not call the external Overpass service automatically.
- Update local setup and test-case documentation to make the new command the primary local entry point.

## Capabilities

### New Capabilities

- `local-development-startup`: Start the local application stack through one command and verify that its frontend and API are reachable.

### Modified Capabilities

- None.

## Impact

- Adds a small repository startup/check script and an npm script entry point.
- Reuses the existing Docker Compose services, Vite dev server, GraphQL endpoint, Node runtime, and standard library; no new dependency is required.
- Updates `LOCAL_SETUP.md` and the relevant test instructions.
- No production API, database schema, or user-facing application behavior changes.
