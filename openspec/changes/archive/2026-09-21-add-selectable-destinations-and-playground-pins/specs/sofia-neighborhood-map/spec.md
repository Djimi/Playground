# Spec Delta

## ADDED Requirements

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
