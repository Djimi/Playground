# Spec Delta

## ADDED Requirements

### Requirement: Present enriched playground previews

The system SHALL make the compact playground preview follow the approved field hierarchy: a licensed photo or `No photo yet`, recorded name or `Unnamed playground`, neighborhood, distance only when visitor location is already available, age range, equipment and known counts, historical municipal status with source date and a textual `May be outdated` warning, the existing `No ratings yet` state, and a `View details` action.

#### Scenario: Compact preview follows the approved field hierarchy

- **WHEN** a visitor previews a playground pin
- **THEN** the compact preview presents the recorded photo or `No photo yet`, name fallback, neighborhood, available distance, age range, equipment summary, historical municipal status with date and warning, rating empty state, and `View details` in the approved hierarchy
- **AND** it does not display fabricated ratings or inferred optional values

#### Scenario: Compact preview has missing enrichment data

- **WHEN** a playground lacks a photo, name, age, equipment count, municipal status, or visitor ratings
- **THEN** the preview keeps the corresponding explicit empty or unknown state
- **AND** the remaining recorded fields remain usable

### Requirement: Show enriched playground details and source evidence

The system SHALL show a full playground details view with licensed photo gallery or explicit empty and failed-photo states, address, coordinates and directions, equipment and known counts, age range, surface, fencing, ownership, access, fee, historical municipal status, repairs, Ordinance 1 compliance, notes, warnings, source-value history, source dates, and required OpenStreetMap, SofiaPlan, and photo attribution when applicable.

#### Scenario: Full details show enriched facts, warnings, source history, and attribution

- **WHEN** a visitor opens playground details for a record with enriched source data
- **THEN** the details view shows the recorded facts, source dates, textual historical-data warning, conflicting or older source values when present, source history, and applicable attribution
- **AND** the selected playground remains visibly selected while details are open

#### Scenario: Missing data and failed photos remain explicit

- **WHEN** a playground lacks an optional fact or a recorded photo fails to load
- **THEN** the details view shows `Unknown`, an empty state, or an explicit unavailable-photo state for that item
- **AND** it does not infer a replacement value or silently leave a blank gallery cell

#### Scenario: Warning meaning is textual

- **WHEN** the details view presents historical municipal status, repair, compliance, or note data that may be outdated
- **THEN** the view includes a visible text warning explaining that the source data may be outdated
- **AND** warning meaning does not depend on color or an icon alone

#### Scenario: Ratings and reviews remain empty until implemented

- **WHEN** a playground has no platform ratings or visitor reviews
- **THEN** the details view displays `No ratings yet` and `No reviews yet`
- **AND** it does not display Google review data or fabricated values

### Requirement: Preserve existing map interaction and accessibility behavior

The system SHALL preserve existing hover, keyboard-focus, selection, modal, mobile-width, attribution, and focus-management behavior unchanged while adding enriched preview and details content.

#### Scenario: Existing map interaction remains unchanged

- **WHEN** a visitor hovers, focuses, activates, refreshes, or closes a playground pin or details view
- **THEN** the existing preview, selection, Back, Escape, viewport, and focus behavior remains as specified by the Sofia neighborhood map capability
- **AND** no enriched content requires a new scheduler, visitor-editing flow, authentication flow, or browser-side source request

#### Scenario: Enriched details remain usable on mobile

- **WHEN** a visitor opens the enriched details view at 320 CSS pixels wide
- **THEN** all detail content uses the available viewport without horizontal scrolling
- **AND** a visible close control and normal keyboard focus behavior remain available

## MODIFIED Requirements

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
- **THEN** the system displays its recorded name or an unnamed fallback, capabilities, and, when the playground has an OpenStreetMap source record, its OpenStreetMap source link

#### Scenario: Visitor distinguishes a playground pin

- **WHEN** a playground pin appears on the map
- **THEN** its default appearance is visually distinct from area preview and selected-playground styling

#### Scenario: Catalog request fails

- **WHEN** loading the playground catalog fails or returns an error
- **THEN** no playground pins are displayed
- **AND** the map does not present partial or stale playground results

### Requirement: Preview playgrounds from the map

The system SHALL let a visitor preview a playground without first selecting or zooming into its neighborhood, SHALL visibly distinguish the previewed playground pin, SHALL keep the compact preview open while the playground pin or the preview popup content owns pointer hover or keyboard focus, SHALL close the compact preview and return that pin to its default appearance only when neither the pin nor the popup content owns pointer hover or keyboard focus, SHALL preserve a still-focused preview when unrelated pointer hover ends, and SHALL allow pointer activation of the previewed pin to open and keep its details visible within the map viewport. The existing Back, Escape, modal focus, and viewport behavior remains unchanged.

#### Scenario: Pointer visitor previews a playground

- **WHEN** a visitor points at a playground pin on a hover-capable device
- **THEN** the system displays a compact preview with the recorded main photo when available, name or unnamed fallback, recommended age when available, equipment summary, and platform rating state
- **AND** the pin receives preview styling

#### Scenario: Pointer visitor leaves a playground preview

- **WHEN** a pointer visitor stops pointing at a playground pin with a visible compact preview
- **AND** the preview popup content does not own pointer hover or keyboard focus
- **THEN** the compact preview closes
- **AND** the pin returns to its default neutral styling

#### Scenario: Keyboard visitor leaves a playground preview

- **WHEN** keyboard focus moves away from a playground pin with a visible compact preview
- **AND** the preview popup content does not own pointer hover or keyboard focus
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
