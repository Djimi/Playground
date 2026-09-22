# Proposal

## Why

Opening playground details repeatedly leaves every clicked pin orange instead of only the current playground. The same interaction layer also has fragile hover/focus behavior at map edges and on small screens, which makes previews and details easy to lose or obstruct.

## What Changes

- Ensure opening a new playground clears the previous pin's selected styling before applying the new selection.
- Keep pointer hover and keyboard focus preview state independent for areas and playground pins.
- Keep playground hover previews visible within the map viewport, including near its edges.
- Make the playground details view behave as a modal overlay with predictable keyboard focus while it is open.
- Expose concise loading, empty, and failure feedback when playground or area data is unavailable instead of leaving an apparently empty map.
- Use meaningful playground names for image alternative text and show an explicit fallback when a recorded photo cannot load.

The hover preview currently shows the available photo, name, age, equipment, and rating state. A description is not added here because the current database, importer, and GraphQL API do not provide a description field.

## Capabilities

### New Capabilities

None.

### Modified Capabilities

- `sofia-neighborhood-map`: refine playground selection, transient previews, responsive details focus, and data-load feedback.

## Impact

- Frontend interaction and status rendering in `src/main.js`, `index.html`, and `src/styles.css`.
- Existing Sofia neighborhood map requirements and test cases will gain regression coverage for repeated pin clicks, focus/hover transitions, popup boundaries, modal focus, and load failures.
- No backend schema, API, dependency, or deployment changes are required. Dense-marker clustering is deferred until measured rendering cost justifies a broader map interaction change.
