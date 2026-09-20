import assert from "node:assert/strict";
import { readFile } from "node:fs/promises";
import test from "node:test";

const dataUrl = new URL("../public/data/sofia-neighborhoods.geojson", import.meta.url);
const requiredNames = [
  "Lozenets",
  "Mladost 1",
  "Mladost 1A",
  "Mladost 2",
  "Mladost 3",
  "Mladost 4",
];
const excludedNames = ["Benkovski", "Chelopechene", "Kremikovtsi", "Seslavtsi", "Trebich"];

test("Sofia neighborhood data is valid and English-readable", async () => {
  const data = JSON.parse(await readFile(dataUrl, "utf8"));

  assert.equal(data.type, "FeatureCollection");
  assert.deepEqual(data.scope, { city: "Sofia", osmRelation: 4283101 });
  assert.equal(data.license, "ODbL-1.0");
  assert.ok(data.features.length > requiredNames.length);

  const ids = data.features.map((feature) => feature.properties?.id);
  const names = data.features.map((feature) => feature.properties?.name);

  assert.equal(new Set(ids).size, ids.length, "IDs must be unique");
  assert.equal(new Set(names).size, names.length, "names must be unique");

  for (const feature of data.features) {
    assert.match(feature.properties.id, /^osm-relations?-\d+(?:-\d+)*$/);
    assert.doesNotMatch(feature.properties.name, /[\u0400-\u04ff]/, feature.properties.name);
    assert.ok(["Polygon", "MultiPolygon"].includes(feature.geometry?.type));
  }

  for (const name of requiredNames) assert.ok(names.includes(name), `missing ${name}`);
  for (const name of excludedNames) assert.ok(!names.includes(name), `unexpected ${name}`);
});
