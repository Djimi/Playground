# Tasks

## 1. Startup command

- [x] 1.1 Add `scripts/start-local.mjs` with prerequisite checks and existing Docker Compose startup for `db` and `api`; verify it fails with a clear non-zero result when a required tool or dependency is unavailable.
- [x] 1.2 Start Vite as the attached child process with predictable local binding, poll the existing GraphQL and frontend endpoints with a bounded timeout, and print recent Compose diagnostics on failure; verify a successful run reports both local URLs only after both smoke checks pass.
- [x] 1.3 Handle interruption and child-process failure without orphaning the frontend or deleting the Postgres volume; verify `Ctrl+C` exits cleanly and a subsequent start retains the local database volume.

## 2. Repository integration and documentation

- [x] 2.1 Add the `npm run start:local` entry point and verify npm invokes the new script from the repository root.
- [x] 2.2 Update `LOCAL_SETUP.md` so the one-command startup path is primary, while retaining explicit import and shutdown commands; verify every documented startup URL and command matches the implementation.
- [x] 2.3 Update `TEST_CASES.md` with the startup smoke-check procedure and failure expectations; verify the documented check does not require an automatic Overpass import.

## 3. Verification

- [x] 3.1 Run `npm test`, `npm run build`, and the startup command with Docker available; verify existing tests/builds pass and the frontend plus GraphQL smoke checks succeed with an empty or imported catalog.
- [x] 3.2 Run `openspec validate add-local-startup-command --type change --strict`; verify the change artifacts and requirement scenarios validate successfully.
