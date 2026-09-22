# Spec Delta

## MODIFIED Requirements

### Requirement: Preview playgrounds from the map
The system SHALL let a visitor preview a playground without first selecting or zooming into its neighborhood, SHALL visibly distinguish the previewed playground pin, and SHALL allow pointer activation of the previewed pin to open and keep its details visible.

#### Scenario: Pointer visitor previews a playground
- **WHEN** a visitor points at a playground pin on a hover-capable device
- **THEN** the system displays a compact preview with the recorded main photo when available, name or unnamed fallback, recommended age when available, equipment summary, and platform rating state
- **AND** the pin receives preview styling

#### Scenario: Pointer visitor opens details after preview
- **WHEN** a pointer visitor activates a playground pin while its compact preview is visible
- **THEN** the system opens that playground's details
- **AND** the details remain open after the activation completes

#### Scenario: Keyboard visitor previews a playground
- **WHEN** a visitor moves keyboard focus to a playground pin
- **THEN** the system displays the same compact preview available to a pointer visitor

#### Scenario: Playground has no platform ratings
- **WHEN** the compact preview has no platform rating data
- **THEN** the system displays `No ratings yet`
- **AND** the system does not display a fabricated rating or Google review data
