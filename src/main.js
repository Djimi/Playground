import L from "leaflet";
import "leaflet/dist/leaflet.css";
import "./styles.css";

const SOFIA_CENTER = [42.6977, 23.3219];
const TILE_URL = "https://tile.openstreetmap.org/{z}/{x}/{y}.png";
const TILE_ATTRIBUTION =
  '&copy; <a href="https://www.openstreetmap.org/copyright">OpenStreetMap contributors</a>';
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

const map = L.map("map", { zoomControl: true }).setView(SOFIA_CENTER, 12);
new ResizeObserver(() => map.invalidateSize()).observe(document.querySelector("#map"));

L.tileLayer(TILE_URL, {
  attribution: TILE_ATTRIBUTION,
  maxZoom: 19,
}).addTo(map);

const selection = document.querySelector("#selected-neighborhood");
let selectedLayer;

function selectNeighborhood(feature, layer) {
  if (selectedLayer) neighborhoods.resetStyle(selectedLayer);
  selectedLayer = layer;
  layer.setStyle(SELECTED_STYLE).bringToFront();
  selection.textContent = feature.properties.name;
}

const neighborhoods = L.geoJSON(null, {
  style: DEFAULT_STYLE,
  onEachFeature(feature, layer) {
    layer.bindTooltip(feature.properties.name, { sticky: true });
    layer.on({
      add() {
        const element = layer.getElement();
        element?.setAttribute("tabindex", "0");
        element?.setAttribute("role", "button");
        element?.setAttribute("aria-label", `Select ${feature.properties.name}`);
      },
      mouseover() {
        if (selectedLayer !== layer) layer.setStyle(HOVER_STYLE).bringToFront();
      },
      mouseout() {
        if (selectedLayer !== layer) neighborhoods.resetStyle(layer);
      },
      click() {
        selectNeighborhood(feature, layer);
      },
      keydown({ originalEvent }) {
        if (originalEvent.key !== "Enter" && originalEvent.key !== " ") return;
        originalEvent.preventDefault();
        selectNeighborhood(feature, layer);
      },
    });
  },
}).addTo(map);

fetch("/data/sofia-neighborhoods.geojson")
  .then((response) => {
    if (!response.ok) throw new Error(`Neighborhood data failed: ${response.status}`);
    return response.json();
  })
  .then((data) => {
    neighborhoods.addData(data);
    map.fitBounds(neighborhoods.getBounds(), { padding: [16, 16] });
  })
  .catch((error) => {
    selection.textContent = "Neighborhoods unavailable";
    console.error(error);
  });
