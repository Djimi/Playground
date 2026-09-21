# Sofia Neighborhood Map Specification

## Purpose

Provides a mobile-friendly map for discovering and selecting everyday neighborhoods within Sofia city using readable English-transliterated names.

## Requirements

### Requirement: Display Sofia neighborhood map
The system SHALL open with an interactive map focused on Sofia and SHALL display polygon boundaries for supported everyday neighborhoods within Sofia city.

#### Scenario: Visitor opens the map
- **WHEN** a visitor opens the application
- **THEN** the system displays Sofia with supported neighborhood boundaries visible

#### Scenario: Visitor navigates the map
- **WHEN** a visitor pans or zooms using mouse, touch, or keyboard controls
- **THEN** the map updates while neighborhood boundaries remain aligned with their locations

### Requirement: Limit coverage to Sofia city
The system SHALL exclude surrounding towns and villages from the neighborhood dataset shown on the map.

#### Scenario: Map includes only Sofia city neighborhoods
- **WHEN** the neighborhood layer loads
- **THEN** no polygon representing a surrounding town or village is displayed

### Requirement: Use English-transliterated neighborhood names
The system SHALL present every displayed neighborhood name in a curated Latin-script form understandable to foreign visitors.

#### Scenario: Visitor views a neighborhood name
- **WHEN** the system displays a neighborhood name
- **THEN** the name uses Latin characters, such as `Lozenets` or `Mladost 1`

#### Scenario: Visitor views Mladost subdivisions
- **WHEN** the neighborhood layer loads
- **THEN** it includes `Lozenets`, `Mladost 1`, `Mladost 1A`, `Mladost 2`, `Mladost 3`, and `Mladost 4` as separately named neighborhoods

### Requirement: Select a neighborhood
The system SHALL let a visitor select one neighborhood by tapping or clicking its polygon, visibly distinguish that polygon, and display its English-transliterated name.

#### Scenario: Visitor selects a neighborhood
- **WHEN** a visitor taps or clicks a neighborhood polygon
- **THEN** the system highlights that polygon and displays its English-transliterated name

#### Scenario: Visitor changes selection
- **WHEN** a visitor selects a different neighborhood
- **THEN** the previous highlight clears and only the new neighborhood remains selected

### Requirement: Select named family destinations
The system SHALL display selectable vector boundaries for South Park and Sofia Zoo separately from similarly named neighborhoods.

#### Scenario: Visitor selects South Park or Sofia Zoo
- **WHEN** a visitor taps, clicks, or keyboard-selects either destination polygon
- **THEN** the polygon remains highlighted
- **AND** the map fits that polygon at a closer zoom

### Requirement: Preview area names
The system SHALL display the hovered area name in a prominent live heading and restore the selected area name when the pointer leaves.

#### Scenario: Visitor hovers an area
- **WHEN** a visitor points at a neighborhood, South Park, or Sofia Zoo polygon
- **THEN** that polygon receives preview styling
- **AND** its name appears in the map heading

### Requirement: Display playground pins
The system SHALL display every playground returned for the visible map bounds as an interactive pin and refresh pins after the visible bounds change.

#### Scenario: Visitor zooms into an area
- **WHEN** the map finishes moving or zooming
- **THEN** playground pins within the visible bounds are displayed above area polygons

#### Scenario: Visitor opens a playground pin
- **WHEN** a visitor clicks, taps, or keyboard-opens a playground pin
- **THEN** the system displays its recorded name or an unnamed fallback, capabilities, and OpenStreetMap source link

### Requirement: Support phone and tablet use
The system SHALL keep the map and neighborhood selection usable at phone, tablet, and desktop viewport sizes.

#### Scenario: Visitor uses a phone-sized viewport
- **WHEN** the map is viewed at 320 CSS pixels wide
- **THEN** map controls, attribution, and selected neighborhood name remain visible and usable without horizontal scrolling

### Requirement: Show attribution
The system SHALL display attribution required by the map tile and neighborhood geometry sources.

#### Scenario: Visitor views source attribution
- **WHEN** the map is visible
- **THEN** required attribution remains visible and accessible
