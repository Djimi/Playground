import L from "leaflet";
import "leaflet/dist/leaflet.css";
import "./styles.css";

const SOFIA_CENTER = [42.6977, 23.3219];
const API_URL = import.meta.env.VITE_API_URL ?? "/graphql";
const TILE_URL = "https://tile.openstreetmap.org/{z}/{x}/{y}.png";
const TILE_ATTRIBUTION =
  '&copy; <a href="https://www.openstreetmap.org/copyright">OpenStreetMap contributors</a>';
const DEFAULT_LABEL = "Hover over an area";
const PAGE_SIZE = 500;
const PLAYGROUNDS_QUERY = `
  query VisiblePlaygrounds($bounds: BoundsInput!, $limit: Int!, $offset: Int!) {
    playgrounds(filter: { bounds: $bounds }, limit: $limit, offset: $offset) {
      id
      name
      location { longitude latitude }
      capabilities
      source { url }
    }
  }
`;
const DEFAULT_STYLE = {
  color: "#25766f",
  fillColor: "#83c9bf",
  fillOpacity: 0.22,
  weight: 1.5,
};
const HOVER_STYLE = {
  color: "#155e59",
  fillColor: "#5db8aa",
  fillOpacity: 0.38,
  weight: 2.5,
};
const SELECTED_STYLE = {
  color: "#9d341f",
  fillColor: "#ef8354",
  fillOpacity: 0.5,
  weight: 3,
};
const PLAYGROUND_STYLE = {
  color: "#7c2d12",
  fillColor: "#fb923c",
  fillOpacity: 0.95,
  radius: 12,
  weight: 2,
};

const map = L.map("map", { zoomControl: true }).setView(SOFIA_CENTER, 12);
map.createPane("areas").style.zIndex = 400;
map.createPane("playgrounds").style.zIndex = 450;
new ResizeObserver(() => map.invalidateSize()).observe(document.querySelector("#map"));

L.tileLayer(TILE_URL, {
  attribution: TILE_ATTRIBUTION,
  maxZoom: 19,
}).addTo(map);

const areaName = document.querySelector("#area-name");
const playgrounds = L.layerGroup().addTo(map);
let selectedLayer;
let playgroundRequest;
let suppressHover;

function showAreaName(name) {
  areaName.textContent = name ?? selectedLayer?.feature.properties.name ?? DEFAULT_LABEL;
}

function selectArea(feature, layer) {
  if (selectedLayer) areas.resetStyle(selectedLayer);
  selectedLayer = layer;
  suppressHover = true;
  layer.setStyle(SELECTED_STYLE);
  showAreaName(feature.properties.name);
  map.fitBounds(layer.getBounds(), { maxZoom: 16, padding: [40, 40] });
}

function onEachArea(feature, layer) {
  layer.on({
    add() {
      const element = layer.getElement();
      element?.setAttribute("tabindex", "0");
      element?.setAttribute("role", "button");
      element?.setAttribute("aria-label", `Select ${feature.properties.name}`);
    },
    mouseover() {
      if (suppressHover) return;
      if (selectedLayer !== layer) layer.setStyle(HOVER_STYLE);
      showAreaName(feature.properties.name);
    },
    mousemove({ originalEvent }) {
      if (!suppressHover || (!originalEvent.movementX && !originalEvent.movementY)) return;
      suppressHover = false;
      if (selectedLayer !== layer) layer.setStyle(HOVER_STYLE);
      showAreaName(feature.properties.name);
    },
    mouseout() {
      if (suppressHover) return;
      if (selectedLayer !== layer) areas.resetStyle(layer);
      showAreaName();
    },
    click() {
      selectArea(feature, layer);
    },
    keydown({ originalEvent }) {
      if (originalEvent.key !== "Enter" && originalEvent.key !== " ") return;
      originalEvent.preventDefault();
      selectArea(feature, layer);
    },
  });
}

const areas = L.geoJSON(null, {
  pane: "areas",
  style: DEFAULT_STYLE,
  onEachFeature: onEachArea,
}).addTo(map);

function addAreaData(url, fitMap = false) {
  return fetch(url)
    .then((response) => {
      if (!response.ok) throw new Error(`${url} failed: ${response.status}`);
      return response.json();
    })
    .then((data) => {
      areas.addData(data);
      if (fitMap) map.fitBounds(areas.getBounds(), { padding: [16, 16] });
    })
    .catch((error) => console.error(error));
}

function playgroundPopup(playground) {
  const content = document.createElement("div");
  const title = document.createElement("strong");
  title.textContent = playground.name ?? "Playground";
  content.append(title);

  if (playground.capabilities.length) {
    const capabilities = document.createElement("div");
    capabilities.textContent = playground.capabilities
      .map((value) => value.toLowerCase().replaceAll("_", " "))
      .join(", ");
    content.append(capabilities);
  }

  const source = document.createElement("a");
  source.href = playground.source.url;
  source.target = "_blank";
  source.rel = "noreferrer";
  source.textContent = "OpenStreetMap";
  content.append(source);
  return content;
}

function renderPlaygrounds(items) {
  playgrounds.clearLayers();
  for (const playground of items) {
    const label = playground.name ?? "Playground";
    const marker = L.circleMarker(
      [playground.location.latitude, playground.location.longitude],
      { ...PLAYGROUND_STYLE, pane: "playgrounds" },
    )
      .bindPopup(playgroundPopup(playground))
      .addTo(playgrounds);
    const element = marker.getElement();
    element?.setAttribute("tabindex", "0");
    element?.setAttribute("role", "button");
    element?.setAttribute("aria-label", label);
    element?.addEventListener("keydown", (event) => {
      if (event.key !== "Enter" && event.key !== " ") return;
      event.preventDefault();
      marker.openPopup();
    });
  }
}

async function loadPlaygrounds() {
  playgroundRequest?.abort();
  const request = new AbortController();
  playgroundRequest = request;
  const bounds = map.getBounds();
  const graphqlBounds = {
    southWest: { longitude: bounds.getWest(), latitude: bounds.getSouth() },
    northEast: { longitude: bounds.getEast(), latitude: bounds.getNorth() },
  };

  try {
    const items = [];
    for (let offset = 0; ; offset += PAGE_SIZE) {
      const response = await fetch(API_URL, {
        method: "POST",
        headers: { "content-type": "application/json" },
        signal: request.signal,
        body: JSON.stringify({
          query: PLAYGROUNDS_QUERY,
          variables: { bounds: graphqlBounds, limit: PAGE_SIZE, offset },
        }),
      });
      const result = await response.json();
      if (!response.ok || result.errors) {
        throw new Error(result.errors?.[0]?.message ?? `Playground API failed: ${response.status}`);
      }
      const page = result.data.playgrounds;
      items.push(...page);
      if (page.length < PAGE_SIZE) break;
      // ponytail: offset pages can shift during an import; use cursors if imports become frequent.
    }
    if (playgroundRequest === request) renderPlaygrounds(items);
  } catch (error) {
    if (error.name !== "AbortError") console.error("Playgrounds unavailable", error);
  }
}

map.on("moveend", loadPlaygrounds);
addAreaData("/data/sofia-neighborhoods.geojson", true).then(() =>
  addAreaData("/data/sofia-discovery-areas.geojson"),
);
loadPlaygrounds();
