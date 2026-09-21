# Spec Delta

## MODIFIED Requirements

### Requirement: Bound and page playground search results
The system SHALL accept a result limit and non-negative offset, apply documented defaults when omitted, reject invalid values, and return results in deterministic order.

#### Scenario: Client omits result limit
- **WHEN** a client searches without a result limit
- **THEN** the system returns no more than the documented default number of playgrounds

#### Scenario: Client exceeds result limit
- **WHEN** a client supplies a limit above the documented maximum
- **THEN** the system returns a GraphQL input error

#### Scenario: Client requests a later page
- **WHEN** a client supplies a non-negative offset
- **THEN** the system skips that many deterministically ordered playgrounds

#### Scenario: Client supplies a negative offset
- **WHEN** a client supplies a negative offset
- **THEN** the system returns a GraphQL input error

## RENAMED Requirements

- FROM: `### Requirement: Bound playground search results`
- TO: `### Requirement: Bound and page playground search results`
