# OpenStreetMap

## Attribution and ODbL are required

Symptom: a map works technically but cannot be redistributed safely.

Cause: OpenStreetMap data is licensed under ODbL and tile usage also requires visible attribution.

Fix: keep `© OpenStreetMap contributors` linked to the copyright page. Keep derived neighborhood data under ODbL and document the source.

Verify the attribution link in the map and [public/data/README.md](../../public/data/README.md).

## `out center geom` returns only the last geometry mode

Symptom: the importer fails with `way/<id> is missing center` although the
Overpass response contains `bounds` and full `geometry`.

Cause: `center`, `bounds`, `bb`, and `geom` are alternative geometry output
modes, not additive ones. Combined in a single `out` statement, only the last
mode is emitted. `out meta center geom;` therefore returns `geometry` and
`bounds` but no `center` (verified on Overpass 0.7.62.7 and 0.7.62.11).

Fix: parse `bounds` as well and derive the fallback point from its center.
For ways and relations, the Overpass `center` is the center of the bounding
box, so the two are equivalent. `backend/src/importer.rs` does this in
`element_point`. Requesting each geometry mode in a separate response also
works but duplicates elements.

Verify: `cargo test importer::tests` covers `falls_back_to_bounds_center...`;
then run `docker compose run --rm api import-playgrounds`.

## Public APIs are rate-limited

Symptom: Overpass or Nominatim requests return 406, 429, 504, or intermittent failures.

Cause: public services limit request frequency and query size. Overpass can
also reject a client without a descriptive `User-Agent` with HTTP 406.

Fix: batch lookups, add delays and retries, send a descriptive `User-Agent`,
and keep downloaded data local at runtime. For a failed manual import, retry
later or select another trusted instance through `OVERPASS_URL`; do not bypass
import validation or rapidly repeat a large query.

Verify with `node scripts/prepare-neighborhoods.mjs`; do not call these services from the browser.

## Photo and date metadata need their own evidence

Symptom: a nearby Commons image appears to depict a playground, or an OSM edit
timestamp is presented as a field observation date.

Cause: proximity does not establish what a photo shows. An OSM element
timestamp records a source update, not when each tag was checked on site.

Fix: import only direct `wikimedia_commons=File:...` references, and attach a
photo only when Commons returns complete reusable licence and attribution
metadata. Label OSM timestamps as source updates, distinct from SofiaPlan's
source-wide observation date.

Verify with `cargo test --manifest-path backend/Cargo.toml commons::tests` and
`cargo test --manifest-path backend/Cargo.toml importer::tests`.
