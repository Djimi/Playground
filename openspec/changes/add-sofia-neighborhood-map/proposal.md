# Proposal

## Why

The project needs a useful first screen where residents and visitors can understand Sofia by everyday neighborhood. Starting with the map proves the geographic data and mobile interaction before adding playgrounds, filters, accounts, or a backend.

## What Changes

- Add a responsive web map centered on Sofia.
- Display polygon boundaries for everyday Sofia city neighborhoods only.
- Show English transliterated neighborhood names, such as `Lozenets` and `Mladost 1`.
- Keep named subdivisions such as Mladost 1, 1A, 2, 3, and 4 separately selectable.
- Let users select a neighborhood by tapping or clicking its polygon and see the selected English name.
- Show required map and geographic-data attribution.
- Exclude surrounding towns, villages, playground data, filters, authentication, and backend services from this change.

## Capabilities

### New Capabilities

- `sofia-neighborhood-map`: Responsive Sofia map, everyday-neighborhood boundaries, English labels, and neighborhood selection.

### Modified Capabilities

None.

## Impact

- Creates the initial web application because no application code or framework exists yet.
- Adds a lightweight browser map dependency, map tiles, and a local GeoJSON neighborhood dataset.
- Requires a documented, reusable data source and attribution before neighborhood geometry is shipped.
- Adds one small automated data check for supported geometry and complete English names.
