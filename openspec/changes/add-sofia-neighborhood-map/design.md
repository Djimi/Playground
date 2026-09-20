# Design

## Context

No web application, package setup, framework, or test runner exists. Neighborhood geometry is external data with uncertain canonical boundaries, licensing, naming, and city-scope rules. See `proposal.md` for motivation and `specs/sofia-neighborhood-map/spec.md` for behavior.

## Goals / Non-Goals

**Goals:**

- Establish the smallest static web application that satisfies the map contract.
- Keep neighborhood geometry local, versioned, and deterministic at runtime.
- Make touch selection and narrow-screen layout first-class.
- Preserve source and attribution metadata beside the normalized data.

**Non-Goals:**

- Server-side rendering, backend APIs, database, accounts, analytics, or deployment automation.
- Runtime geocoding, automatic transliteration, or live boundary downloads.
- Search, location tracking, playground markers, or filters.

## Decisions

### Use a static vanilla JavaScript application with Leaflet

Use HTML, CSS, and JavaScript with Leaflet for tiles, GeoJSON rendering, controls, and pointer interaction. A small build tool may serve and bundle the application, but no UI framework is needed for one map screen.

Alternative: React or another component framework. Rejected because the current slice has no component or state complexity that justifies it.

### Ship normalized neighborhood GeoJSON as a local asset

Prepare one versioned GeoJSON file before runtime. Each feature contains a stable `id` and curated English `name`. Remove non-Sofia settlements during preparation instead of filtering by viewport in the browser. Record source URL, retrieval date, license, and attribution.

Municipal Sofia neighborhood data is a useful candidate, but its published license must permit reuse before bundling. If permission cannot be confirmed, use a suitably licensed source or stop before shipping geometry.

Alternative: fetch municipal or Overpass data in the browser. Rejected because availability, response shape, rate limits, and upstream edits would make the first screen unreliable.

### Curate English names once

Store final Latin-script display names in GeoJSON. Do not transliterate at runtime. This handles conventional spellings and numbered subdivisions consistently.

Alternative: automatic Cyrillic transliteration. Rejected because correct visitor-facing names need exceptions and stable spelling.

### Keep selection state in memory

Leaflet polygon events set one selected feature, update its style, restore the previous style, and update a compact name panel. No URL, browser storage, or backend persistence is needed.

### Use provider attribution and a replaceable tile URL constant

Start with a standards-compliant raster tile provider and visible attribution. Keep provider configuration in one place so production traffic can move away from a public development tile service without changing map behavior.

## Risks / Trade-offs

- **Everyday boundaries are not canonical:** describe boundaries as informational and pin the chosen dataset version.
- **Dataset license may be unclear:** verify reuse terms before adding geometry; unresolved rights block release.
- **Source may include villages or towns:** use an explicit Sofia-city inclusion set during data preparation and validate the result.
- **Dense polygons and labels can clutter phones:** always show the selected name; add persistent labels only where they remain readable without another dependency.
- **Public tile services have usage limits:** comply with provider policy and replace provider configuration before traffic exceeds allowed use.
- **Local GeoJSON can grow:** measure first; simplify geometry only if load or interaction performance is poor on a phone.

## Migration Plan

This is a greenfield application. Deploy the static build after the data source, license, attribution, and phone-width behavior pass validation. Roll back by restoring the prior static deployment; no user data or schema migration exists.
