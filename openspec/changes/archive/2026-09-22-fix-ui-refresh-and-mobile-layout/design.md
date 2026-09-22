# Design

## Context

See `proposal.md` for motivation. Playground pins are rendered into one Leaflet layer after a bounds request completes. Details keep a reference to the selected marker element, while a later refresh recreates marker elements. On phones, the fixed header and Leaflet's default top-left control placement share the same space.

## Goals / Non-Goals

**Goals:**

- Make failed current refreshes fail closed by clearing stale pins.
- Preserve focus for the same logical playground after its marker element is recreated.
- Keep the existing mobile header and controls usable at 320 CSS pixels.

**Non-Goals:**

- No new loading or empty-state UI.
- No API, data-model, dependency, or map-library changes.
- No changes to unrelated photo, area-data, or catalog error handling.

## Decisions

- Clear the playground layer only for a non-aborted request that is still the current request. This prevents stale results and avoids an older request clearing results from a newer request. Retrying, caching, and a new status component are unnecessary for this bug fix.
- Before closing details, use the existing selected-marker reference as a fallback when the original return-focus element has been detached. This reuses the marker refresh path instead of adding IDs, a registry, or another focus state.
- Reserve additional right-side header padding on phones and move the mobile zoom group to the lower-left. CSS handles both collisions without viewport-specific JavaScript or a new layout abstraction.

## Risks / Trade-offs

- [Risk] A failed refresh leaves the map without playground pins until a later successful refresh. → Mitigation: this is preferable to displaying stale locations; the existing console error remains available for diagnosis.
- [Risk] The selected marker may also be gone after a failed refresh. → Mitigation: focus restoration already checks DOM connectivity and will not focus a detached element.
- [Risk] Moving zoom controls changes their phone location. → Mitigation: keep them visible and separated from the header while preserving the existing desktop placement.

## Migration Plan

No migration is required. Deploy the JavaScript and CSS together, then run the existing test/build and OpenSpec validation commands.
