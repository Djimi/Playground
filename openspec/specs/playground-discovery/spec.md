# Playground Discovery Specification

## Purpose

Provides a read-only catalog API for discovering Sofia playgrounds by location, neighborhood, age suitability, and available equipment.

## Requirements

### Requirement: Expose a read-only playground GraphQL API
The system SHALL expose a GraphQL query endpoint that supports playground search and single-playground lookup without exposing data-changing operations.

#### Scenario: Client inspects available playground queries
- **WHEN** a client requests the GraphQL schema
- **THEN** the schema includes playground search and single-playground lookup fields
- **AND** the schema exposes no playground mutation field

#### Scenario: Client requests an unknown playground
- **WHEN** a client requests a playground identifier that does not exist
- **THEN** the playground field returns `null` without failing the full GraphQL response

### Requirement: Search playgrounds by geographic area
The system SHALL let a client search for playgrounds inside a valid map bounding box or within a distance in meters of a valid coordinate.

#### Scenario: Search inside map bounds
- **WHEN** a client supplies valid south-west and north-east coordinates
- **THEN** the system returns only playgrounds whose locations lie inside those bounds

#### Scenario: Search by distance
- **WHEN** a client supplies a valid coordinate and radius in meters
- **THEN** the system returns only playgrounds within that radius
- **AND** each result includes its distance from the supplied coordinate

#### Scenario: Reject invalid geographic input
- **WHEN** a client supplies invalid coordinate ranges, inverted bounds, or a non-positive radius
- **THEN** the system returns a GraphQL input error and does not execute the search

### Requirement: Filter playgrounds by discovery attributes
The system SHALL support optional filters for neighborhood, age suitability, and required playground capabilities, and SHALL combine supplied filters using logical AND.

#### Scenario: Filter by neighborhood
- **WHEN** a client supplies a supported neighborhood identifier
- **THEN** the system returns only playgrounds assigned to that neighborhood

#### Scenario: Filter by multiple capabilities
- **WHEN** a client requests both `SWING` and `SLIDE`
- **THEN** every returned playground is recorded as having both capabilities

#### Scenario: Filter by age suitability
- **WHEN** a client supplies a child age from 0 through 18
- **THEN** the system returns only playgrounds whose recorded minimum age, maximum age, or both include that age as an inclusive bound
- **AND** a missing minimum or maximum is treated as an open bound

#### Scenario: Exclude unknown metadata from attribute matches
- **WHEN** a client supplies an age or capability filter
- **THEN** a playground with unknown matching metadata is not returned as a match

#### Scenario: Reject invalid child age
- **WHEN** a client supplies a child age below 0 or above 18
- **THEN** the system returns a GraphQL input error

### Requirement: Return stable playground details
The system SHALL return a stable identifier, optional name, representative location, containing neighborhoods, recorded capabilities, optional age bounds, photo URLs, and OpenStreetMap source metadata for each playground.

#### Scenario: Retrieve playground details
- **WHEN** a client requests an existing playground by identifier
- **THEN** the system returns all recorded discovery fields for that playground

#### Scenario: Return incomplete source data honestly
- **WHEN** OpenStreetMap lacks an optional field for a playground
- **THEN** the API returns that field as `null` or an empty list as defined by the GraphQL schema
- **AND** the system does not infer an unsupported value

### Requirement: Bound playground search results
The system SHALL accept a result limit, apply a documented default when omitted, reject limits above the documented maximum, and return results in deterministic order.

#### Scenario: Client omits result limit
- **WHEN** a client searches without a result limit
- **THEN** the system returns no more than the documented default number of playgrounds

#### Scenario: Client exceeds result limit
- **WHEN** a client supplies a limit above the documented maximum
- **THEN** the system returns a GraphQL input error

### Requirement: Import Sofia playgrounds from OpenStreetMap
The system SHALL provide a repeatable operator command that imports OpenStreetMap features tagged `leisure=playground` within Sofia city, preserves their source identifiers, and records required attribution metadata.

#### Scenario: Import playground source data
- **WHEN** an operator runs the import against a complete valid OpenStreetMap response with no error or remark marker
- **THEN** the catalog contains the normalized Sofia playground records from that response
- **AND** each record retains its OpenStreetMap identifier and source attribution

#### Scenario: Repeat the same import
- **WHEN** an operator imports the same source data more than once
- **THEN** the catalog contains no duplicate playground records

#### Scenario: Import fails before completion
- **WHEN** source retrieval fails, the response reports an error or remark, or validation fails
- **THEN** the previously usable playground catalog remains available
- **AND** the command exits unsuccessfully with a diagnostic message

### Requirement: Associate playgrounds with curated neighborhoods
The system SHALL use the existing curated Sofia neighborhood polygons and identifiers to associate an imported playground with every neighborhood polygon that covers its location.

#### Scenario: Playground lies inside one supported neighborhood
- **WHEN** an imported playground location lies inside one curated neighborhood polygon
- **THEN** the playground includes that neighborhood's stable identifier and English-transliterated name

#### Scenario: Playground lies inside nested supported neighborhoods
- **WHEN** an imported playground location lies inside an aggregate neighborhood and one or more subdivisions
- **THEN** the playground includes every covering neighborhood
- **AND** filtering by any included neighborhood returns the playground

#### Scenario: Playground lies on neighborhood boundaries
- **WHEN** an imported playground location lies on the boundary of one or more curated neighborhoods
- **THEN** each polygon that covers the boundary location is included

#### Scenario: Playground lies outside supported neighborhoods
- **WHEN** an imported playground location lies outside all supported neighborhood polygons
- **THEN** the playground remains discoverable with an empty neighborhood list
