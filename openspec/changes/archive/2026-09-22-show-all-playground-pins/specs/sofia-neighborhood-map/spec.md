# Spec Delta

## REMOVED Requirements

### Requirement: Display playground pins
**Reason**: Replaced by displaying the full Sofia playground catalog loaded once, so visible-bounds scoping and refresh no longer exist.
**Migration**: Superseded by the ADDED requirement "Display every playground pin". Pin interactions (preview, details, selected styling, and layering above area polygons) are unchanged.

## ADDED Requirements

### Requirement: Display every playground pin
The system SHALL display every playground in the Sofia catalog as an interactive pin, independent of the visible map bounds and zoom level, SHALL load the catalog once per page load, and SHALL keep the pins displayed while the visitor pans or zooms. If the catalog request fails, the system SHALL display no playground pins rather than stale or partial playground data.

#### Scenario: Visitor opens the map
- **WHEN** the map loads
- **THEN** every playground in the catalog is displayed as an interactive pin above the area polygons

#### Scenario: Visitor pans and zooms
- **WHEN** a visitor pans or zooms the map
- **THEN** every playground pin remains displayed without a new catalog request

#### Scenario: Visitor opens a playground pin
- **WHEN** a visitor clicks, taps, or keyboard-opens a playground pin
- **THEN** the system displays its recorded name or an unnamed fallback, capabilities, and OpenStreetMap source link

#### Scenario: Visitor distinguishes a playground pin
- **WHEN** a playground pin appears on the map
- **THEN** its default appearance is visually distinct from area preview and selected-playground styling

#### Scenario: Catalog request fails
- **WHEN** loading the playground catalog fails or returns an error
- **THEN** no playground pins are displayed
- **AND** the map does not present partial or stale playground results
