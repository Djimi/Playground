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
