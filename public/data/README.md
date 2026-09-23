# Sofia playground source data

## Neighborhood boundaries

Source: [OpenStreetMap](https://www.openstreetmap.org/) neighborhood boundary relations contained by [Sofia city relation 4283101](https://www.openstreetmap.org/relation/4283101).

- Retrieval query: `scripts/prepare-neighborhoods.mjs`
- License: [Open Data Commons Open Database License 1.0](https://opendatacommons.org/licenses/odbl/1-0/)
- Attribution: © OpenStreetMap contributors
- Attribution guidance: [OpenStreetMap copyright and license](https://www.openstreetmap.org/copyright)

The bundled GeoJSON is a normalized derivative database. It remains available under ODbL 1.0. Normalization keeps polygon relations tagged as Sofia suburbs, quarters, or neighborhoods; assigns stable relation IDs; and stores curated Latin-script display names. `EXCLUDED_NAMES` in the preparation script explicitly removes outlying settlements that fall outside the Sofia-town neighborhood scope.

`Raina Knyaginya` is a curated polygon because OpenStreetMap currently maps it as [node 13848488763](https://www.openstreetmap.org/node/13848488763), not a boundary relation. Its four anchors follow the published scope between Nadezhda overpass, the Central Station railway area, Kamenodelska Street, and Istoriya Slavyanobulgarska Boulevard. The script preserves this source attribution with the feature.

## Discovery-area boundaries

`sofia-discovery-areas.geojson` contains the complete South Park boundary from
[relation 16878152](https://www.openstreetmap.org/relation/16878152) and Sofia
Zoo from [way 157686292](https://www.openstreetmap.org/way/157686292). The same
preparation script refreshes this separate file so parks and attractions are
not imported as neighborhoods by the backend.

## Playground catalog

The backend importer fetches Sofia playgrounds from OpenStreetMap through
Overpass and the [SofiaPlan playground dataset](https://urbandata.sofia.bg/dataset/playgrounds).
It retains both source records, merges only clear nearby matches, and associates
canonical playgrounds with every covering bundled neighborhood polygon.
SofiaPlan's `2019-04-18` date is a source-wide observation, while OSM timestamps
are source updates. See the [SofiaPlan](../../docs/gotchas/sofiaplan.md) and
[OpenStreetMap](../../docs/gotchas/openstreetmap.md) notes for source quirks.

Run from repository root after starting PostGIS:

```bash
docker compose run --rm api import-playgrounds
```

The OpenStreetMap contribution remains subject to ODbL 1.0. Any UI or data
export that uses it must keep visible attribution:
`© OpenStreetMap contributors`, linked to
[OpenStreetMap copyright and license](https://www.openstreetmap.org/copyright).
Each GraphQL playground record exposes its linked source URLs, attribution, and
licence/terms status. SofiaPlan reuse metadata needs written clarification
before commercial launch; do not assume ODbL covers SofiaPlan. Only directly
referenced Commons photos with verified reusable metadata are attached. Missing
capabilities, ages, names, surface, current condition, or images are not inferred.
