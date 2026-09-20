# OpenStreetMap

## Attribution and ODbL are required

Symptom: a map works technically but cannot be redistributed safely.

Cause: OpenStreetMap data is licensed under ODbL and tile usage also requires visible attribution.

Fix: keep `© OpenStreetMap contributors` linked to the copyright page. Keep derived neighborhood data under ODbL and document the source.

Verify the attribution link in the map and [public/data/README.md](../../public/data/README.md).

## Public APIs are rate-limited

Symptom: Overpass or Nominatim requests return 429, 504, or intermittent failures.

Cause: public services limit request frequency and query size.

Fix: batch lookups, add delays and retries, send a descriptive `User-Agent`, and keep downloaded data local at runtime.

Verify with `node scripts/prepare-neighborhoods.mjs`; do not call these services from the browser.
