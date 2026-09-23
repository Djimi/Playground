# SofiaPlan

## GeoJSON response and geometry

Symptom: a strict HTTP client rejects the dataset as non-JSON, or a GeoJSON
reader expects a point and misses all playgrounds.

Cause: the endpoint serves GeoJSON as `text/plain`; each playground uses a
`MultiPoint` with exactly one coordinate.

Fix: read the response body as text and parse JSON; validate the feature
collection, IDs, and single-coordinate geometry before replacing the catalog.

Verify with `cargo test --manifest-path backend/Cargo.toml enrichment::tests`.

## Historical and missing values

Symptom: municipal facts look freshly verified, `0` disappears, or malformed
age text becomes a precise age range.

Cause: `2019-04-18` is one source-wide observation date, not an import date or
field-specific update. Empty/null/dash markers mean unknown, while numeric
`0` is known. Age ranges are free text and can be malformed.

Fix: label the date as an observation and warn that facts may be outdated;
preserve explicit zero and false; leave unknown or invalid ages unset while
retaining the raw source value. SofiaPlan supplies no surface or current
condition field, so leave those unknown unless another source provides them.

Verify with `cargo test --manifest-path backend/Cargo.toml enrichment::tests`.

## Excluded records and reuse terms

Symptom: the source count exceeds the number of SofiaPlan-backed canonical
playgrounds, or the portal and dataset licence metadata seem inconsistent.

Cause: records labelled `не се показват на картата` are retained as source
records but excluded from canonical discovery. The available reuse metadata
does not establish one unambiguous commercial licence.

Fix: retain excluded records for inspection but do not create canonical
playgrounds from them. Keep SofiaPlan attribution and obtain written reuse
clarification before a commercial launch; do not label the dataset ODbL.

Verify with `cargo test --manifest-path backend/Cargo.toml enrichment::tests`;
after an import, compare `source_playgrounds` counts with `playgrounds` and the
reported excluded count.
