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
  "Raina Knyaginya",
  "Lyulin 1",
  "Lyulin 2",
  "Lyulin 3",
  "Lyulin 4",
  "Lyulin 5",
  "Lyulin 6",
  "Lyulin 7",
  "Lyulin 8",
  "Lyulin 9",
  "Lyulin 10",
  "Lyulin Center",
  "Nadezhda 1",
  "Nadezhda 2",
  "Nadezhda 3",
  "Nadezhda 4",
];
const excludedNames = ["Benkovski", "Chelopechene", "Kremikovtsi", "Seslavtsi", "Trebich", "zh.k. Lyulin"];

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
    assert.match(feature.properties.id, /^osm-(?:relations?|nodes?)-\d+(?:-\d+)*$/);
    assert.doesNotMatch(feature.properties.name, /[\u0400-\u04ff]/, feature.properties.name);
    assert.ok(["Polygon", "MultiPolygon"].includes(feature.geometry?.type));
  }

  for (const name of requiredNames) assert.ok(names.includes(name), `missing ${name}`);
  for (const name of excludedNames) assert.ok(!names.includes(name), `unexpected ${name}`);
  assert.ok(!ids.includes("osm-relation-16871937"), "aggregate Lyulin must be excluded");

  const raina = data.features.find((feature) => feature.properties.name === "Raina Knyaginya");
  assert.equal(raina.properties.id, "osm-node-13848488763");
  assert.match(raina.properties.source, /OpenStreetMap node 13848488763/);

  const hadzhiDimitar = data.features.find((feature) => feature.properties.name === "Hadzhi Dimitar");
  assert.equal(hadzhiDimitar.geometry.type, "Polygon");
  assert.equal(hadzhiDimitar.geometry.coordinates.length, 1);
});
