# Proposal

## Why

Independent UI investigations found stale playground pins after failed refreshes, lost keyboard focus after a pin refresh, and two phone-width layout collisions. These failures make the map show misleading data and make core controls harder to use on small screens.

## What Changes

- Clear previously rendered playground pins when the current bounds request fails, while preserving newer successful requests.
- Restore details focus to the refreshed selected pin when the original pin DOM element was replaced.
- Reserve enough mobile header space for long area names beside the Back control.
- Move mobile zoom controls away from the fixed header.

## Capabilities

### New Capabilities

None.

### Modified Capabilities

- `sofia-neighborhood-map`: define refresh-failure behavior, focus restoration after pin replacement, and collision-free phone layout for the header and zoom controls.

## Impact

The change affects `src/main.js`, `src/styles.css`, and the existing Sofia neighborhood map specification. It changes no API shape, dependencies, or persistence behavior.
