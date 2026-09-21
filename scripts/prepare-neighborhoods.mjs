import { mkdir, writeFile } from "node:fs/promises";

const OVERPASS_URL = "https://overpass-api.de/api/interpreter";
const NOMINATIM_URL = "https://nominatim.openstreetmap.org/lookup";
const OUTPUT = new URL("../public/data/sofia-neighborhoods.geojson", import.meta.url);
const DISCOVERY_OUTPUT = new URL("../public/data/sofia-discovery-areas.geojson", import.meta.url);
const USER_AGENT = "SofiaPlaygrounds/0.1 (local development)";
const DISCOVERY_AREAS = [
  { osmId: "R16878152", id: "osm-relation-16878152", name: "South Park", areaType: "park" },
  { osmId: "W157686292", id: "osm-way-157686292", name: "Sofia Zoo", areaType: "zoo" },
];
const REQUIRED_NAMES = new Map([
  [16863633, "Lozenets"],
  [16864393, "Mladost 1"],
  [16864735, "Mladost 1A"],
  [16889162, "Mladost 2"],
  [16889163, "Mladost 3"],
  [16889164, "Mladost 4"],
]);
const EXCLUDED_NAMES = new Set([
  "Benkovski",
  "Chelopechene",
  "Kremikovtsi",
  "Seslavtsi",
  "Trebich",
]);
const CYRILLIC = {
  А: "A", Б: "B", В: "V", Г: "G", Д: "D", Е: "E", Ж: "Zh", З: "Z",
  И: "I", Й: "Y", К: "K", Л: "L", М: "M", Н: "N", О: "O", П: "P",
  Р: "R", С: "S", Т: "T", У: "U", Ф: "F", Х: "H", Ц: "Ts", Ч: "Ch",
  Ш: "Sh", Щ: "Sht", Ъ: "A", Ь: "Y", Ю: "Yu", Я: "Ya",
  І: "I",
};

const BOUNDING_BOXES = [
  "42.55,23.15,42.68,23.28",
  "42.55,23.28,42.68,23.41",
  "42.55,23.41,42.68,23.55",
  "42.68,23.15,42.82,23.28",
  "42.68,23.28,42.82,23.41",
  "42.68,23.41,42.82,23.55",
];

function transliterate(value) {
  const clean = value.replace(/^(ж\.к\.|кв\.|м\.|в\.з\.|НПЗ|СПЗ)\s*/iu, "");
  const latin = [...clean]
    .map((character) => {
      const upper = character.toLocaleUpperCase("bg");
      const replacement = CYRILLIC[upper];
      if (!replacement) return character;
      return character === upper ? replacement : replacement.toLocaleLowerCase("en");
    })
    .join("");

  return latin.replace(/(^|[\s-])([a-z])/g, (_, separator, letter) =>
    `${separator}${letter.toUpperCase()}`,
  );
}

async function getJson(url, attempts = 5) {
  for (let attempt = 1; attempt <= attempts; attempt += 1) {
    const response = await fetch(url, { headers: { "User-Agent": USER_AGENT } });
    if (response.ok) return response.json();
    if (attempt === attempts) throw new Error(`${url.origin} returned ${response.status}`);
    const retryAfter = Number(response.headers.get("retry-after")) || attempt * 10;
    await new Promise((resolve) => setTimeout(resolve, retryAfter * 1000));
  }
}

const relations = new Map();
for (const boundingBox of BOUNDING_BOXES) {
  const overpass = new URL(OVERPASS_URL);
  overpass.searchParams.set(
    "data",
    `[out:json][timeout:45];relation["boundary"]["place"~"^(suburb|quarter|neighbourhood)$"](${boundingBox});out tags;`,
  );
  const { elements } = await getJson(overpass);
  for (const { id, tags } of elements) relations.set(id, tags);
  await new Promise((resolve) => setTimeout(resolve, 5000));
}

const relationIds = [...relations.keys()];
const features = [];

for (let offset = 0; offset < relationIds.length; offset += 25) {
  const batch = relationIds.slice(offset, offset + 25);
  const lookup = new URL(NOMINATIM_URL);
  lookup.searchParams.set("format", "geojson");
  lookup.searchParams.set("polygon_geojson", "1");
  lookup.searchParams.set("addressdetails", "1");
  lookup.searchParams.set("osm_ids", batch.map((id) => `R${id}`).join(","));

  const result = await getJson(lookup);
  for (const feature of result.features) {
    const relationId = Number(feature.properties.osm_id);
    const tags = relations.get(relationId);
    if (
      !tags ||
      feature.properties.address?.city !== "София" ||
      !["Polygon", "MultiPolygon"].includes(feature.geometry?.type)
    ) continue;

    const name = REQUIRED_NAMES.get(relationId) || tags["name:en"] || transliterate(tags.name);
    if (EXCLUDED_NAMES.has(name)) continue;

    features.push({
      type: "Feature",
      id: `relation/${relationId}`,
      properties: {
        id: `osm-relation-${relationId}`,
        name,
      },
      geometry: feature.geometry,
    });
  }

  if (offset + 25 < relationIds.length) await new Promise((resolve) => setTimeout(resolve, 1100));
}

const groupedFeatures = Object.values(Object.groupBy(features, (feature) => feature.properties.name))
  .map((matchingFeatures) => {
    if (matchingFeatures.length === 1) return matchingFeatures[0];

    const relationIds = matchingFeatures
      .map((feature) => Number(feature.properties.id.replace("osm-relation-", "")))
      .sort((a, b) => a - b);
    const coordinates = matchingFeatures.flatMap((feature) =>
      feature.geometry.type === "Polygon" ? [feature.geometry.coordinates] : feature.geometry.coordinates,
    );

    return {
      type: "Feature",
      id: relationIds.map((id) => `relation/${id}`).join("+"),
      properties: {
        id: `osm-relations-${relationIds.join("-")}`,
        name: matchingFeatures[0].properties.name,
      },
      geometry: { type: "MultiPolygon", coordinates },
    };
  })
  .sort((a, b) => a.properties.name.localeCompare(b.properties.name, "en"));

const discoveryLookup = new URL(NOMINATIM_URL);
discoveryLookup.searchParams.set("format", "geojson");
discoveryLookup.searchParams.set("polygon_geojson", "1");
discoveryLookup.searchParams.set("osm_ids", DISCOVERY_AREAS.map(({ osmId }) => osmId).join(","));
const discoveryResult = await getJson(discoveryLookup);
const discoveryById = new Map(
  discoveryResult.features.map((feature) => [
    `${feature.properties.osm_type[0].toUpperCase()}${feature.properties.osm_id}`,
    feature,
  ]),
);
const discoveryFeatures = DISCOVERY_AREAS.map((area) => {
  const feature = discoveryById.get(area.osmId);
  if (!feature || !["Polygon", "MultiPolygon"].includes(feature.geometry?.type)) {
    throw new Error(`Missing polygon geometry for ${area.name}`);
  }
  return {
    type: "Feature",
    id: `${feature.properties.osm_type}/${feature.properties.osm_id}`,
    properties: { id: area.id, name: area.name, areaType: area.areaType },
    geometry: feature.geometry,
  };
});

await mkdir(new URL("../public/data/", import.meta.url), { recursive: true });
await writeFile(
  OUTPUT,
  `${JSON.stringify({
    type: "FeatureCollection",
    source: "OpenStreetMap relation 4283101 and contained neighborhood relations",
    scope: { city: "Sofia", osmRelation: 4283101 },
    retrieved: new Date().toISOString().slice(0, 10),
    license: "ODbL-1.0",
    attribution: "© OpenStreetMap contributors",
    features: groupedFeatures,
  })}\n`,
);
await writeFile(
  DISCOVERY_OUTPUT,
  `${JSON.stringify({
    type: "FeatureCollection",
    source: "OpenStreetMap relation 16878152 and way 157686292",
    scope: { city: "Sofia" },
    retrieved: new Date().toISOString().slice(0, 10),
    license: "ODbL-1.0",
    attribution: "© OpenStreetMap contributors",
    features: discoveryFeatures,
  })}\n`,
);

console.log(`Wrote ${groupedFeatures.length} neighborhoods to ${OUTPUT.pathname}`);
console.log(`Wrote ${discoveryFeatures.length} discovery areas to ${DISCOVERY_OUTPUT.pathname}`);
