# Spec Delta

## MODIFIED Requirements

### Requirement: Display playground pins
The system SHALL display every playground returned for the visible map bounds as an interactive pin and refresh pins after the visible bounds change. If the current visible-bounds request fails, the system SHALL remove pins from the previous bounds rather than leave stale playground data displayed.

#### Scenario: Visitor zooms into an area
- **WHEN** the map finishes moving or zooming
- **THEN** playground pins within the visible bounds are displayed above area polygons

#### Scenario: Visitor opens a playground pin
- **WHEN** a visitor clicks, taps, or keyboard-opens a playground pin
- **THEN** the system displays its recorded name or an unnamed fallback, capabilities, and OpenStreetMap source link

#### Scenario: Visible-bounds refresh fails
- **WHEN** the current playground request fails after pins from an earlier bounds request were displayed
- **THEN** the earlier pins are removed
- **AND** the map does not present those pins as current results

### Requirement: Open responsive playground details
The system SHALL let a visitor open a selected playground's details from its map pin, SHALL keep that pin visibly selected while the details remain open, and SHALL manage keyboard focus for the details view, including when a refresh replaces the selected pin element.

#### Scenario: Visitor opens playground details
- **WHEN** a visitor clicks, taps, or keyboard-opens a playground pin
- **THEN** the system displays all recorded photos, recommended age, structured equipment inventory, platform review state, and source attribution
- **AND** the selected pin is visually distinct from other pins
- **AND** focus moves to the details view's Back control

#### Scenario: Visitor closes details with the Back control
- **WHEN** a visitor activates the details view's Back control
- **THEN** the details view closes
- **AND** focus returns to the playground pin that opened it when that pin remains available

#### Scenario: Visitor closes details with Escape
- **WHEN** a visitor presses `Escape` while playground details are open
- **THEN** the details view closes
- **AND** focus returns to the playground pin that opened it when that pin remains available

#### Scenario: Visitor opens details on a phone
- **WHEN** a visitor opens playground details at 320 CSS pixels wide
- **THEN** the detail view uses the available viewport without horizontal scrolling
- **AND** a visible control closes the detail view

#### Scenario: Playground details are incomplete
- **WHEN** a playground lacks photos, age information, equipment counts, ratings, or reviews
- **THEN** the detail view shows an honest empty or unknown state for each missing category
- **AND** the system does not infer missing values

#### Scenario: Visitor closes details after a pin refresh
- **WHEN** a visitor opens details, the selected playground pin is replaced during a visible-bounds refresh, and the visitor activates Back or presses `Escape`
- **THEN** focus returns to the refreshed pin for that playground when it remains in the current results

### Requirement: Support phone and tablet use
The system SHALL keep the map and neighborhood selection usable at phone, tablet, and desktop viewport sizes, including without overlap between the phone map header, long selected area names, the Back control, and zoom controls.

#### Scenario: Visitor uses a phone-sized viewport
- **WHEN** the map is viewed at 320 CSS pixels wide
- **THEN** map controls, attribution, and selected neighborhood name remain visible and usable without horizontal scrolling

#### Scenario: Visitor views a long area name on a phone
- **WHEN** a selected area name is long enough to wrap at 320 CSS pixels wide
- **THEN** the name remains separate from the map Back control

#### Scenario: Visitor uses zoom controls on a phone
- **WHEN** a visitor views the map at 320 CSS pixels wide
- **THEN** the zoom controls remain usable without covering the fixed map header
