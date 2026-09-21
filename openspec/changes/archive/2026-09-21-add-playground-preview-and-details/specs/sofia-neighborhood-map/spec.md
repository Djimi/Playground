# Spec Delta

## ADDED Requirements

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
The system SHALL provide a visible Back control and keyboard Escape behavior that leave the current focused state before restoring an earlier map viewport.

#### Scenario: Visitor leaves playground details
- **WHEN** playground details are open and the visitor activates Back or presses `Escape`
- **THEN** the system closes the details and clears the selected playground
- **AND** the current neighborhood focus and map viewport remain unchanged

#### Scenario: Visitor leaves neighborhood focus
- **WHEN** no playground details are open, a neighborhood is selected, and the visitor activates Back or presses `Escape`
- **THEN** the system clears the neighborhood selection
- **AND** restores the map viewport from immediately before that neighborhood was selected

#### Scenario: Visitor uses standard map zoom controls
- **WHEN** a visitor zooms with the available mouse wheel, touch gesture, keyboard, or visible zoom controls
- **THEN** the map zooms without requiring a `Ctrl` modifier

