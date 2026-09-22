# Spec Delta

## MODIFIED Requirements

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
The system SHALL let a visitor activate a neighborhood by tapping, clicking, or keyboard activation, fit the map to its polygon, and keep no persistent neighborhood-selection styling after activation.

#### Scenario: Visitor selects a neighborhood
- **WHEN** a visitor taps, clicks, or keyboard-activates a neighborhood polygon
- **THEN** the map fits that polygon at a closer zoom
- **AND** the polygon does not receive persistent selected styling

#### Scenario: Visitor changes selection
- **WHEN** a visitor activates one neighborhood and then another
- **THEN** each activation fits its own polygon
- **AND** neither polygon remains selected after pointer preview ends

### Requirement: Select named family destinations
The system SHALL display selectable vector boundaries for South Park and Sofia Zoo separately from similarly named neighborhoods.

#### Scenario: Visitor selects South Park or Sofia Zoo
- **WHEN** a visitor taps, clicks, or keyboard-activates either destination polygon
- **THEN** the map fits that polygon at a closer zoom
- **AND** the polygon does not receive persistent selected styling

### Requirement: Preview area names
The system SHALL display the hovered or keyboard-focused area name in a prominent live heading and restore the default prompt when preview ends.

#### Scenario: Visitor hovers an area
- **WHEN** a visitor points at a neighborhood, South Park, or Sofia Zoo polygon
- **THEN** that polygon receives preview styling
- **AND** its name appears in the map heading

#### Scenario: Visitor leaves an area preview
- **WHEN** a visitor moves the pointer away from an unselected area polygon
- **THEN** that polygon returns to its default styling
- **AND** the map heading restores its default prompt

#### Scenario: Visitor previews a park after an area activation
- **WHEN** a visitor activates one area and then points at South Park or Sofia Zoo
- **THEN** the destination polygon receives preview styling
- **AND** its name appears in the map heading

### Requirement: Display playground pins
The system SHALL display every playground returned for the visible map bounds as an interactive pin and refresh pins after the visible bounds change.

#### Scenario: Visitor zooms into an area
- **WHEN** the map finishes moving or zooming
- **THEN** playground pins within the visible bounds are displayed above area polygons

#### Scenario: Visitor opens a playground pin
- **WHEN** a visitor clicks, taps, or keyboard-opens a playground pin
- **THEN** the system displays its recorded name or an unnamed fallback, capabilities, and OpenStreetMap source link

#### Scenario: Visitor distinguishes a playground pin
- **WHEN** a playground pin appears on the map
- **THEN** its default appearance is visually distinct from area preview and selected-playground styling

### Requirement: Open responsive playground details
The system SHALL let a visitor open a selected playground's details from its map pin and SHALL keep that pin visibly selected while the details remain open.

#### Scenario: Visitor opens playground details
- **WHEN** a visitor clicks, taps, or keyboard-opens a playground pin
- **THEN** the system displays all recorded photos, recommended age, structured equipment inventory, platform review state, and source attribution
- **AND** the selected pin is visually distinct from other pins

#### Scenario: Visitor opens details on a phone
- **WHEN** a visitor opens playground details at 320 CSS pixels wide
- **THEN** the detail view uses the available viewport without horizontal scrolling
- **AND** a visible control closes the detail view

#### Scenario: Playground details are incomplete
- **WHEN** a playground lacks photos, age information, equipment counts, ratings, or reviews
- **THEN** the detail view shows an honest empty or unknown state for each missing category
- **AND** the system does not infer missing values

### Requirement: Leave focused map states predictably
The system SHALL provide a visible Back control and keyboard Escape behavior that leave open playground details without changing the map viewport.

#### Scenario: Visitor leaves playground details
- **WHEN** playground details are open and the visitor activates Back or presses `Escape`
- **THEN** the system closes the details and clears the selected playground
- **AND** the map viewport remains unchanged

#### Scenario: Visitor leaves neighborhood focus
- **WHEN** no playground details are open and the visitor activates Back or presses `Escape`
- **THEN** no area receives persistent selected styling
- **AND** the map viewport remains unchanged

#### Scenario: Visitor clicks the map while details are open
- **WHEN** playground details are open and the visitor clicks or taps any map position
- **THEN** the system closes the details and clears the selected playground
- **AND** that input does not activate an area polygon beneath it

#### Scenario: Visitor activates an area after closing details
- **WHEN** playground details were closed by a map click or tap
- **AND** the visitor next activates an area polygon
- **THEN** the map fits that area polygon

#### Scenario: Visitor uses standard map zoom controls
- **WHEN** a visitor zooms with the available mouse wheel, touch gesture, keyboard, or visible zoom controls
- **THEN** the map zooms without requiring a `Ctrl` modifier
