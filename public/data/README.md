# Sofia OpenStreetMap data

## Neighborhood boundaries

Source: [OpenStreetMap](https://www.openstreetmap.org/) neighborhood boundary relations contained by [Sofia city relation 4283101](https://www.openstreetmap.org/relation/4283101).

- Retrieval query: `scripts/prepare-neighborhoods.mjs`
- License: [Open Data Commons Open Database License 1.0](https://opendatacommons.org/licenses/odbl/1-0/)
- Attribution: © OpenStreetMap contributors
- Attribution guidance: [OpenStreetMap copyright and license](https://www.openstreetmap.org/copyright)

The bundled GeoJSON is a normalized derivative database. It remains available under ODbL 1.0. Normalization keeps polygon relations tagged as Sofia suburbs, quarters, or neighborhoods; assigns stable relation IDs; and stores curated Latin-script display names. `EXCLUDED_NAMES` in the preparation script explicitly removes outlying settlements that fall outside the Sofia-town neighborhood scope.

## Playground catalog

The backend importer fetches Sofia features tagged `leisure=playground` from
OpenStreetMap through Overpass. It normalizes locations, equipment, age tags,
images, and source metadata, then associates each playground with every bundled
neighborhood polygon that covers its location.

Run from repository root after starting PostGIS:

```bash
docker compose run --rm api import-playgrounds
```

The database catalog is a derived OpenStreetMap database and remains subject to
ODbL 1.0. Any UI or data export that uses it must keep visible attribution:
`© OpenStreetMap contributors`, linked to
[OpenStreetMap copyright and license](https://www.openstreetmap.org/copyright).
Each GraphQL playground record also exposes its source URL, attribution, and
license. Optional OpenStreetMap tags are incomplete; missing capabilities, ages,
names, or images must not be inferred.
