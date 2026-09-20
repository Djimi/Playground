# Sofia neighborhood data

Source: [OpenStreetMap](https://www.openstreetmap.org/) neighborhood boundary relations contained by [Sofia city relation 4283101](https://www.openstreetmap.org/relation/4283101).

- Retrieval query: `scripts/prepare-neighborhoods.mjs`
- License: [Open Data Commons Open Database License 1.0](https://opendatacommons.org/licenses/odbl/1-0/)
- Attribution: © OpenStreetMap contributors
- Attribution guidance: [OpenStreetMap copyright and license](https://www.openstreetmap.org/copyright)

The bundled GeoJSON is a normalized derivative database. It remains available under ODbL 1.0. Normalization keeps polygon relations tagged as Sofia suburbs, quarters, or neighborhoods; assigns stable relation IDs; and stores curated Latin-script display names. `EXCLUDED_NAMES` in the preparation script explicitly removes outlying settlements that fall outside the Sofia-town neighborhood scope.
