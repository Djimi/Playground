# Sofia Neighborhood Map Specification

## Purpose

Provides a mobile-friendly map for discovering and selecting everyday neighborhoods within Sofia city using readable English-transliterated names.

## Requirements

### Requirement: Display Sofia neighborhood map
The system SHALL open with an interactive map focused on Sofia and SHALL display one interactive polygon boundary for each supported everyday neighborhood within Sofia city.

#### Scenario: Visitor opens the map
- **WHEN** a visitor opens the application
- **THEN** the system displays Sofia with supported neighborhood boundaries visible

#### Scenario: Visitor navigates the map
- **WHEN** a visitor pans or zooms using mouse, touch, or keyboard controls
- **THEN** the map updates while neighborhood boundaries remain aligned with their locations

#### Scenario: Neighborhood has duplicate source geometry
- **WHEN** multiple source geometries name the same supported neighborhood
- **THEN** the system renders one non-overlapping interactive boundary for that neighborhood

#### Scenario: Neighborhood has mapped subdivisions
- **WHEN** a broad neighborhood polygon overlaps its supported named subdivisions
- **THEN** the system displays the subdivisions as separate interactive boundaries
- **AND** does not display the broad polygon as an interactive area

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

#### Scenario: Visitor views Raina Knyaginya
- **WHEN** the neighborhood layer loads
- **THEN** it includes `Raina Knyaginya` as a separately named neighborhood

#### Scenario: Visitor views Lyulin microregions
- **WHEN** the neighborhood layer loads
- **THEN** it includes `Lyulin 1`, `Lyulin 2`, `Lyulin 3`, `Lyulin 4`, `Lyulin 5`, `Lyulin 6`, `Lyulin 7`, `Lyulin 8`, `Lyulin 9`, and `Lyulin 10` as separately named neighborhoods
- **AND** it includes `Lyulin Center` separately from those numbered microregions

#### Scenario: Visitor views Nadezhda microregions
- **WHEN** the neighborhood layer loads
- **THEN** it includes `Nadezhda 1`, `Nadezhda 2`, `Nadezhda 3`, and `Nadezhda 4` as separately named neighborhoods

### Requirement: Select a neighborhood
The system SHALL let a visitor activate a neighborhood by tapping, clicking, or keyboard activation, fit the map to its polygon, and keep no persistent neighborhood-selection styling after activation. After pointer activation, preview styling SHALL follow the pointer only.

#### Scenario: Visitor selects a neighborhood
- **WHEN** a visitor taps, clicks, or keyboard-activates a neighborhood polygon
- **THEN** the map fits that polygon at a closer zoom
- **AND** the polygon does not receive persistent selected styling

#### Scenario: Visitor changes selection
- **WHEN** a visitor activates one neighborhood and then another
- **THEN** each activation fits its own polygon
- **AND** neither polygon remains selected after pointer preview ends

#### Scenario: Visitor previews another area after pointer activation
- **WHEN** a visitor pointer-activates one area polygon and then points at another area
- **THEN** only the hovered area receives preview styling
- **AND** the activated polygon shows no preview styling once the pointer leaves it

### Requirement: Select named family destinations
The system SHALL display selectable vector boundaries for South Park and Sofia Zoo separately from similarly named neighborhoods.

#### Scenario: Visitor selects South Park or Sofia Zoo
- **WHEN** a visitor taps, clicks, or keyboard-activates either destination polygon
- **THEN** the map fits that polygon at a closer zoom
- **AND** the polygon does not receive persistent selected styling

### Requirement: Communicate map data state
The system SHALL communicate when playground or area data is loading, empty, unavailable, or partially unavailable without preventing independent map content from remaining usable.

#### Scenario: Playground catalog is loading
- **WHEN** the application is waiting for the playground catalog
- **THEN** the map exposes a concise loading state so an empty pin layer is not mistaken for a successful empty catalog

#### Scenario: Playground catalog is empty
- **WHEN** the catalog request succeeds with no playgrounds
- **THEN** the map communicates that no playgrounds are currently available

#### Scenario: Playground catalog fails
- **WHEN** the playground catalog request fails or returns an error
- **THEN** the map communicates that playgrounds are unavailable
- **AND** neighborhood and destination areas remain usable

#### Scenario: Area data fails
- **WHEN** a neighborhood or destination-area data request fails
- **THEN** the map communicates which area data is unavailable
- **AND** successfully loaded map content remains usable

### Requirement: Preview area names
The system SHALL display the hovered or keyboard-focused area name in a prominent live heading, apply preview styling while the area is previewed, and restore the default prompt only when neither pointer hover nor keyboard focus remains on an area.

#### Scenario: Visitor hovers an area
- **WHEN** a visitor points at a neighborhood, South Park, or Sofia Zoo polygon
- **THEN** that polygon receives preview styling
- **AND** its name appears in the map heading

#### Scenario: Keyboard visitor focuses an area
- **WHEN** a visitor moves keyboard focus to a neighborhood, South Park, or Sofia Zoo polygon
- **THEN** that polygon receives the same preview styling as pointer hover
- **AND** its name appears in the map heading

#### Scenario: Visitor leaves an area preview
- **WHEN** a visitor moves the pointer away from an area with no remaining keyboard focus
- **THEN** that polygon returns to its default styling
- **AND** the map heading restores its default prompt

#### Scenario: Visitor moves the pointer across areas while one remains focused
- **WHEN** keyboard focus remains on one area while the pointer briefly enters and leaves another area
- **THEN** the focused area keeps its preview styling and name
- **AND** the other area's transient preview does not remain active

#### Scenario: Visitor previews a park after an area activation
- **WHEN** a visitor activates one area and then points at South Park or Sofia Zoo
- **THEN** the destination polygon receives preview styling
- **AND** its name appears in the map heading

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

### Requirement: Preview playgrounds from the map
The system SHALL let a visitor preview a playground without first selecting or zooming into its neighborhood, SHALL visibly distinguish the previewed playground pin, SHALL return that pin to its default appearance when the preview ends, SHALL preserve a still-focused preview when unrelated pointer hover ends, and SHALL allow pointer activation of the previewed pin to open and keep its details visible within the map viewport.

#### Scenario: Pointer visitor previews a playground
- **WHEN** a visitor points at a playground pin on a hover-capable device
- **THEN** the system displays a compact preview with the recorded main photo when available, name or unnamed fallback, recommended age when available, equipment summary, and platform rating state
- **AND** the pin receives preview styling

#### Scenario: Pointer visitor leaves a playground preview
- **WHEN** a pointer visitor stops pointing at a playground pin with a visible compact preview
- **THEN** the compact preview closes
- **AND** the pin returns to its default neutral styling

#### Scenario: Keyboard visitor leaves a playground preview
- **WHEN** keyboard focus moves away from a playground pin with a visible compact preview
- **THEN** the compact preview closes
- **AND** the pin returns to its default neutral styling

#### Scenario: Pointer visitor opens details after preview
- **WHEN** a pointer visitor activates a playground pin while its compact preview is visible
- **THEN** the system opens that playground's details
- **AND** the details remain open after the activation completes

#### Scenario: Keyboard visitor previews a playground
- **WHEN** a visitor moves keyboard focus to a playground pin
- **THEN** the system displays the same compact preview available to a pointer visitor

#### Scenario: Pointer hover crosses a focused playground pin
- **WHEN** keyboard focus remains on one playground pin while the pointer briefly enters and leaves another pin
- **THEN** the focused pin keeps its preview styling and compact preview
- **AND** the unrelated pin returns to its default styling when the pointer leaves

#### Scenario: Playground preview is near a map edge
- **WHEN** a visitor previews a playground pin near the map viewport edge or fixed header
- **THEN** the compact preview remains fully visible within the map viewport

#### Scenario: Playground has no platform ratings
- **WHEN** the compact preview has no platform rating data
- **THEN** the system displays `No ratings yet`
- **AND** the system does not display a fabricated rating or Google review data

### Requirement: Open responsive playground details
The system SHALL let a visitor open a selected playground's details from its map pin, SHALL keep exactly one pin visibly selected while the details remain open, SHALL keep keyboard focus within the details view while it is open, and SHALL manage focus when a refresh replaces the selected pin element.

#### Scenario: Visitor opens playground details
- **WHEN** a visitor clicks, taps, or keyboard-opens a playground pin
- **THEN** the system displays all recorded photos, recommended age, structured equipment inventory, platform review state, and source attribution
- **AND** the selected pin is visually distinct from other pins
- **AND** focus moves to the details view's Back control

#### Scenario: Visitor opens a second playground
- **WHEN** playground details are open for one pin and a visitor opens a different pin
- **THEN** the first pin returns to its default neutral styling
- **AND** only the newly opened pin uses selected styling

#### Scenario: Visitor navigates details with the keyboard
- **WHEN** playground details are open
- **THEN** keyboard focus remains within the details view until it is closed
- **AND** hidden map controls and pins are not reached through normal Tab navigation

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

#### Scenario: Visitor opens details with recorded photos
- **WHEN** a playground has one or more recorded photos
- **THEN** each photo has alternative text that identifies the playground

#### Scenario: A recorded photo cannot load
- **WHEN** a recorded playground photo fails to load
- **THEN** the detail view shows an explicit unavailable-photo state instead of silently leaving an empty gallery cell

#### Scenario: Visitor closes details after a pin refresh
- **WHEN** a visitor opens details, the selected playground pin is replaced during a visible-bounds refresh, and the visitor activates Back or presses `Escape`
- **THEN** focus returns to the refreshed pin for that playground when it remains in the current results

### Requirement: Leave focused map states predictably
The system SHALL provide a visible Back control and keyboard Escape behavior that leave open playground details without changing the map viewport. Closing details SHALL clear the selected playground without reopening its compact preview, and clicking or tapping the map SHALL not focus or preview the pin that was selected.

#### Scenario: Visitor leaves playground details
- **WHEN** playground details are open and the visitor activates Back or presses `Escape`
- **THEN** the system closes the details and clears the selected playground
- **AND** the map viewport remains unchanged
- **AND** focus returns to the activating playground pin when it remains available
- **AND** the compact playground preview does not reopen through the restored focus

#### Scenario: Visitor leaves neighborhood focus
- **WHEN** no playground details are open and the visitor activates Back or presses `Escape`
- **THEN** no area receives persistent selected styling
- **AND** the map viewport remains unchanged

#### Scenario: Visitor clicks the map while details are open
- **WHEN** playground details are open and the visitor clicks or taps any map position
- **THEN** the system closes the details and clears the selected playground
- **AND** that input does not activate an area polygon beneath it
- **AND** the previously selected playground pin does not receive focus, preview styling, or a compact preview

#### Scenario: Visitor activates an area after closing details
- **WHEN** playground details were closed by a map click or tap
- **AND** the visitor next activates an area polygon
- **THEN** the map fits that area polygon

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
