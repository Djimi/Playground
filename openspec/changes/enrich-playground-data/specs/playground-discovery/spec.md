# Spec Delta

## ADDED Requirements

### Requirement: Import current records from both primary sources

The system SHALL provide one repeatable operator-run import that fetches complete OpenStreetMap and SofiaPlan playground datasets, retains one current source record for every valid `(source, external identifier)`, and excludes SofiaPlan records explicitly marked `не се показват на картата` from canonical playground results while retaining those source records for inspection.

#### Scenario: Import current records from OpenStreetMap and SofiaPlan

- **WHEN** an operator runs the import against complete valid responses from both primary sources
- **THEN** the catalog contains the normalized visible records from both sources
- **AND** each source record retains its source name, external identifier, original source data, and applicable source date

#### Scenario: Retain records excluded from canonical results

- **WHEN** a SofiaPlan record is explicitly marked `не се показват на картата`
- **THEN** the record remains available as a retained SofiaPlan source record
- **AND** it does not create or contribute to a canonical playground result

#### Scenario: Repeat the enrichment import

- **WHEN** an operator imports the same OpenStreetMap and SofiaPlan responses more than once
- **THEN** the catalog contains one current source record per `(source, external identifier)`
- **AND** the canonical catalog contains no duplicate records caused by the rerun

### Requirement: Match source records conservatively

The system SHALL consider cross-source records as match candidates only when their representative locations are within 15 metres, SHALL merge only one-to-one candidates where each record has exactly one opposite-source candidate in that radius, and SHALL keep unmatched or uncertain records as separate canonical playgrounds.

#### Scenario: Merge one-to-one candidates within 15 metres

- **WHEN** one OpenStreetMap record and one visible SofiaPlan record are within 15 metres and neither has another opposite-source candidate in that radius
- **THEN** the system creates one canonical playground linked to both source records
- **AND** the link retains the match method and measured distance

#### Scenario: Preserve uncertain records separately

- **WHEN** either source record has multiple opposite-source candidates within 15 metres
- **THEN** the system does not merge those records
- **AND** each uncertain source record remains represented by a separate canonical playground

#### Scenario: Preserve unmatched records

- **WHEN** a source record has no opposite-source candidate within 15 metres
- **THEN** the system creates an independent canonical playground for that source record

### Requirement: Select field values with provenance

The system SHALL select the newest non-missing source value for each canonical field using the value's effective date, fall back deterministically when dates cannot decide, and retain every source value with its source record, date meaning, and whether it was selected.

#### Scenario: Select the newest non-missing value

- **WHEN** source records provide non-missing values for the same field with different effective dates
- **THEN** the canonical field uses the value with the newest effective date
- **AND** the field history retains the older value and identifies the winning source value

#### Scenario: Preserve conflicting source values

- **WHEN** matched sources provide different non-missing values for the same field
- **THEN** the canonical result exposes the selected value
- **AND** source-value history exposes the conflicting alternative and its provenance

#### Scenario: Keep explicit false and zero distinct from unknown

- **WHEN** a source explicitly supplies `false` or `0` for a supported field
- **THEN** the system retains that value as known
- **AND** it does not treat it as missing or replace it solely because it is false or zero

### Requirement: Accept only verified reusable Commons photos

The system SHALL accept a photo only when a source record directly references a Wikimedia Commons file and the Commons response supplies a public-domain, CC0, CC BY, or CC BY-SA licence, author, attribution, and original file page; NC, ND, non-free, missing, and unknown licences SHALL be rejected, and nearby or unverifiable photos SHALL NOT be attached automatically.

#### Scenario: Accept a directly referenced reusable photo

- **WHEN** a source record directly references a Commons file and the Commons response contains complete supported reuse metadata
- **THEN** the canonical playground includes structured photo metadata with its URL, author, licence, attribution, and original Commons page

#### Scenario: Omit an unverified or unsupported photo

- **WHEN** the Commons request fails, licence metadata is absent or unsupported, or the source has no direct Commons file reference
- **THEN** the playground import succeeds without that photo
- **AND** the catalog reports the photo as unavailable rather than attaching an unverified image

### Requirement: Replace the catalog atomically

The system SHALL validate both primary datasets before replacement and SHALL replace current source records, source-to-canonical mappings, canonical records, and neighborhood memberships in one transaction.

#### Scenario: Commit a complete replacement

- **WHEN** both datasets validate and the replacement transaction commits successfully
- **THEN** the new source records, mappings, canonical records, and memberships become visible together
- **AND** no partial catalog from the replacement is observable

#### Scenario: Preserve the previous catalog on failure

- **WHEN** source retrieval, validation, or any database operation fails
- **THEN** the replacement transaction is rolled back
- **AND** the previously usable source records, mappings, canonical records, and memberships remain unchanged
- **AND** the operator command exits unsuccessfully with a diagnostic

### Requirement: Return enriched playground data additively

The system SHALL extend the read-only GraphQL playground results with nullable or empty values for address, surface, fencing, ownership, access, fee, historical municipal status, repairs, Ordinance 1 compliance, notes, structured licensed photos, all source metadata for linked records, and source-value history while preserving existing `photoUrls` and source fields for compatible clients.

#### Scenario: Return enriched fields and evidence

- **WHEN** a client requests an existing canonical playground
- **THEN** the response includes the recorded enriched fields, structured photos, every linked source record, and field history
- **AND** each selected value identifies its source and applicable date meaning

#### Scenario: Return unknown optional values honestly

- **WHEN** a source does not provide an optional enriched field or a photo is unavailable
- **THEN** the corresponding GraphQL value is `null`, an empty list, or an explicit unavailable state as defined by the schema
- **AND** the system does not infer a value from unrelated fields

#### Scenario: Preserve compatible fields

- **WHEN** an existing client requests `photoUrls` or existing source metadata
- **THEN** those fields remain available with their existing meaning
- **AND** adding enriched fields does not require a breaking query change
