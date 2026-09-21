# Local Development Startup Specification

## Purpose

Provides a repeatable local entry point that starts the existing Sofia playground application stack and verifies that its frontend and API are usable before handing control to the developer.

## Requirements

### Requirement: Start the local application with one command

The project SHALL provide one documented command that, when run from the repository root, starts the existing local database/API services and frontend development server using the repository's local configuration.

#### Scenario: Start a clean local environment

- **WHEN** a developer runs the documented local-start command with the required local tools and dependencies available
- **THEN** the command starts the PostGIS database, GraphQL API, and Vite development server
- **AND** it keeps the development server attached to the invoking terminal

#### Scenario: Start an already initialized environment

- **WHEN** a developer runs the command after the local database/API containers or database volume already exist
- **THEN** the command reuses or starts those existing resources without requiring manual cleanup
- **AND** previously imported playground data remains available

#### Scenario: Start without importing external data

- **WHEN** a developer runs the local-start command
- **THEN** the command does not call the external OpenStreetMap/Overpass import operation automatically

### Requirement: Verify local service readiness

The local-start command SHALL wait for the started services and SHALL perform bounded smoke checks before reporting successful startup.

#### Scenario: Services become ready

- **WHEN** the database/API and frontend become reachable within the startup timeout
- **THEN** the command verifies that the frontend returns a successful HTTP response
- **AND** it verifies that the API accepts a valid GraphQL query and returns a successful response
- **AND** it reports the local URLs only after all checks pass

#### Scenario: API is reachable but unusable

- **WHEN** the API port accepts connections but migrations have failed or the GraphQL request returns an error
- **THEN** the command reports the API check as failed
- **AND** exits unsuccessfully

#### Scenario: A service does not become ready

- **WHEN** any required service remains unreachable until the startup timeout expires
- **THEN** the command reports which service failed and the relevant diagnostic or log location
- **AND** exits with a non-zero status

### Requirement: Handle failure and interruption clearly

The local-start command SHALL fail clearly when prerequisites, service startup, or smoke checks fail, and SHALL not leave an orphaned frontend process after interruption.

#### Scenario: A required local tool is missing

- **WHEN** Docker Compose, Node.js, npm dependencies, or another required prerequisite is unavailable
- **THEN** the command identifies the missing prerequisite
- **AND** exits before claiming that the application started

#### Scenario: Developer stops the local application

- **WHEN** the developer interrupts the attached local-start command
- **THEN** the frontend development process exits
- **AND** the command terminates cleanly without deleting the persistent database volume
