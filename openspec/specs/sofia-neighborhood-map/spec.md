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
The system SHALL display the hovered or keyboard-focused area name in a prominent live heading, apply preview styling while the area is previewed, and restore the default prompt when the preview ends.

#### Scenario: Visitor hovers an area
- **WHEN** a visitor points at a neighborhood, South Park, or Sofia Zoo polygon
- **THEN** that polygon receives preview styling
- **AND** its name appears in the map heading

#### Scenario: Keyboard visitor focuses an area
- **WHEN** a visitor moves keyboard focus to a neighborhood, South Park, or Sofia Zoo polygon
- **THEN** that polygon receives the same preview styling as pointer hover
- **AND** its name appears in the map heading

#### Scenario: Visitor leaves an area preview
- **WHEN** the pointer leaves an area and the area no longer has keyboard focus
- **THEN** that polygon returns to its default styling
- **AND** the map heading restores the default prompt

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

### Requirement: Preview playgrounds from the map
The system SHALL let a visitor preview a playground without first selecting or zooming into its neighborhood, and SHALL visibly distinguish the previewed playground pin.

#### Scenario: Pointer visitor previews a playground
- **WHEN** a visitor points at a playground pin on a hover-capable device
- **THEN** the system displays a compact preview with the recorded main photo when available, name or unnamed fallback, recommended age when available, equipment summary, and platform rating state
- **AND** the pin receives preview styling

#### Scenario: Keyboard visitor previews a playground
- **WHEN** a visitor moves keyboard focus to a playground pin
- **THEN** the system displays the same compact preview available to a pointer visitor

#### Scenario: Playground has no platform ratings
- **WHEN** the compact preview has no platform rating data
- **THEN** the system displays `No ratings yet`
- **AND** the system does not display a fabricated rating or Google review data

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

### Requirement: Leave focused map states predictably
The system SHALL provide a visible Back control and keyboard Escape behavior that leave the current focused state before restoring an earlier map viewport.

#### Scenario: Visitor leaves playground details
- **WHEN** playground details are open and the visitor activates Back or presses `Escape`
- **THEN** the system closes the details and clears the selected playground
- **AND** the current neighborhood focus and map viewport remain unchanged
- **AND** focus returns to the activating playground pin when it remains available

#### Scenario: Visitor leaves neighborhood focus
- **WHEN** no playground details are open, a neighborhood is selected, and the visitor activates Back or presses `Escape`
- **THEN** the system clears the neighborhood selection
- **AND** restores the map viewport from immediately before that neighborhood was selected

#### Scenario: Visitor uses standard map zoom controls
- **WHEN** a visitor zooms with the available mouse wheel, touch gesture, keyboard, or visible zoom controls
- **THEN** the map zooms without requiring a `Ctrl` modifier

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

### Requirement: Show attribution
The system SHALL display attribution required by the map tile and neighborhood geometry sources.

#### Scenario: Visitor views source attribution
- **WHEN** the map is visible
- **THEN** required attribution remains visible and accessible
