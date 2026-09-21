# Spec Delta

## ADDED Requirements

### Requirement: Return structured playground equipment inventory
The system SHALL return each recorded supported play-equipment type once with an optional positive count while preserving the existing capability list for compatible filtering and clients.

#### Scenario: Source records separate equipment features
- **WHEN** an imported playground contains one or more separately mapped supported equipment features of the same type
- **THEN** the equipment inventory returns that type with the number of mapped features as its count

#### Scenario: Source records equipment presence without quantity
- **WHEN** source data records a supported equipment type but provides no reliable quantity
- **THEN** the equipment inventory returns that type with a `null` count
- **AND** the system does not report zero or infer a quantity

#### Scenario: Source records no equipment information
- **WHEN** source data records no supported equipment for a playground
- **THEN** the equipment inventory is empty

#### Scenario: Client retrieves equipment inventory
- **WHEN** a client requests a playground through search or single-playground lookup
- **THEN** equipment entries are returned in deterministic type order
- **AND** every counted type is also present in the existing capability list

