# Proposal

## Why

Visitors can see South Park and Sofia Zoo on the base map but cannot select those real destination boundaries or see playgrounds inside them. This change is being documented retrospectively because implementation began before the required OpenSpec proposal was created.

## What Changes

- Add separate, selectable OpenStreetMap polygons for South Park and Sofia Zoo without treating them as neighborhoods.
- Apply one hover, selection, keyboard, and click-to-zoom interaction model to neighborhoods and named destinations.
- Replace the static map heading with a prominent live area name.
- Display accessible playground pins for the visible bounds above the area polygons, including basic details and source links.
- Add deterministic offset pagination so the frontend can retrieve every playground in a viewport.
- Route local frontend API requests through Vite and document the data refresh and runtime configuration.

## Capabilities

### New Capabilities

None.

### Modified Capabilities

- `sofia-neighborhood-map`: Extend selectable map areas to South Park and Sofia Zoo, add live hover naming and click-to-zoom, and display visible playground pins.
- `playground-discovery`: Add offset pagination to the bounded playground search API so viewport rendering is not truncated.

## Impact

The change affects the Leaflet frontend, its HTML and responsive styling, bundled OpenStreetMap GeoJSON preparation and attribution, GraphQL playground search, local Vite/API configuration, backend import requests, automated tests, and setup/test documentation. It adds no new runtime dependency and no breaking API change.
