# Spec Delta

## ADDED Requirements

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

## MODIFIED Requirements

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
- **AND** the selected pin is visually distinct from every other pin
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
