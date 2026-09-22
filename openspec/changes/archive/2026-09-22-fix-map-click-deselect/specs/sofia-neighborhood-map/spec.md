# Spec Delta

## MODIFIED Requirements

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
