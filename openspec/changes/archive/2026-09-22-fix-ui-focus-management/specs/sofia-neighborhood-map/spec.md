# Spec Delta

## MODIFIED Requirements

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

### Requirement: Open responsive playground details
The system SHALL let a visitor open a selected playground's details from its map pin, SHALL keep that pin visibly selected while the details remain open, and SHALL manage keyboard focus for the details view.

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
