import assert from "node:assert/strict";
import { readFile } from "node:fs/promises";
import test from "node:test";

const dataUrl = new URL("../public/data/sofia-discovery-areas.geojson", import.meta.url);

test("discovery areas contain the real South Park and Sofia Zoo polygons", async () => {
  const data = JSON.parse(await readFile(dataUrl, "utf8"));

  assert.equal(data.type, "FeatureCollection");
  assert.deepEqual(data.scope, { city: "Sofia" });
  assert.equal(data.license, "ODbL-1.0");
  assert.equal(data.features.length, 2);
  assert.deepEqual(
    data.features.map(({ properties }) => properties),
    [
      { id: "osm-relation-16878152", name: "South Park", areaType: "park" },
      { id: "osm-way-157686292", name: "Sofia Zoo", areaType: "zoo" },
    ],
  );
  for (const feature of data.features) {
    assert.ok(["Polygon", "MultiPolygon"].includes(feature.geometry?.type));
    assert.ok(feature.geometry.coordinates.length > 0);
  }
});
