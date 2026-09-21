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
      minAge
      maxAge
      photoUrls
      capabilities
      equipment { capability count }
      source { url }
    }
  }
`;
const PLAYGROUND_DETAIL_QUERY = `
  query PlaygroundDetail($id: ID!) {
    playground(id: $id) {
      id
      name
      location { longitude latitude }
      minAge
      maxAge
      photoUrls
      capabilities
      equipment { capability count }
      source { id url updatedAt attribution license }
    }
  }
`;
const DEFAULT_PIN_STYLE = {
  color: "#1d4ed8",
  fillColor: "#60a5fa",
  fillOpacity: 0.95,
  radius: 12,
  weight: 2,
};
const HOVER_STYLE = {
  color: "#155e59",
  fillColor: "#5db8aa",
  fillOpacity: 0.38,
  weight: 2.5,
};
const PREVIEW_STYLE = {
  color: "#155e59",
  fillColor: "#5db8aa",
  fillOpacity: 1,
  radius: 15,
  weight: 3,
};
const SELECTED_STYLE = {
  color: "#9d341f",
  fillColor: "#ef5b32",
  fillOpacity: 1,
  radius: 16,
  weight: 4,
};
const CAPABILITY_LABELS = {
  CLIMBING_FRAME: "Climbing frame",
  PLAYHOUSE: "Playhouse",
  ROUNDABOUT: "Roundabout",
  SANDPIT: "Sandpit",
  SEESAW: "Seesaw",
  SLIDE: "Slide",
  SPRINGY: "Springy",
  SWING: "Swing",
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
const backButton = document.querySelector("#back-button");
const details = document.querySelector("#playground-details");
const detailsClose = document.querySelector("#details-close");
const detailsTitle = document.querySelector("#playground-details-title");
const detailsContent = document.querySelector("#playground-details-content");
const playgrounds = L.layerGroup().addTo(map);
let selectedPlayground;
let selectedPlaygroundMarker;
let playgroundRequest;
let detailRequest;
let previewPopup;
let previewMarker;

function showAreaName(name) {
  areaName.textContent = name ?? DEFAULT_LABEL;
}

function updateBackButton() {
  backButton.hidden = !selectedPlayground;
}

function markerStyle(marker) {
  if (marker === selectedPlaygroundMarker) return SELECTED_STYLE;
  if (marker === previewMarker) return PREVIEW_STYLE;
  return DEFAULT_PIN_STYLE;
}

function updateMarkerStyle(marker) {
  marker.setStyle(markerStyle(marker));
}

function closePreview() {
  if (previewPopup) {
    map.closePopup(previewPopup);
    previewPopup = undefined;
  }
  if (previewMarker) {
    updateMarkerStyle(previewMarker);
    previewMarker = undefined;
  }
}

function closeDetails() {
  detailRequest?.abort();
  detailRequest = undefined;
  selectedPlaygroundMarker = undefined;
  selectedPlayground = undefined;
  for (const marker of playgrounds.getLayers()) updateMarkerStyle(marker);
  details.hidden = true;
  detailsContent.replaceChildren();
  updateBackButton();
}

function goBack() {
  if (selectedPlayground) {
    closeDetails();
  }
}

function activateArea(layer) {
  if (selectedPlayground) {
    closeDetails();
    return;
  }
  map.fitBounds(layer.getBounds(), { maxZoom: 16, padding: [40, 40] });
}

function onEachArea(feature, layer) {
  let focused = false;
  let hovered = false;
  const preview = () => {
    layer.setStyle(HOVER_STYLE);
    showAreaName(feature.properties.name);
  };
  const endPreview = () => {
    if (hovered || focused) return;
    areas.resetStyle(layer);
    showAreaName();
  };
  layer.on({
    add() {
      const element = layer.getElement();
      element?.setAttribute("tabindex", "0");
      element?.setAttribute("role", "button");
      element?.setAttribute("aria-label", `Select ${feature.properties.name}`);
      element?.addEventListener("focus", () => {
        focused = true;
        preview();
      });
      element?.addEventListener("blur", () => {
        focused = false;
        endPreview();
      });
    },
    mouseover() {
      hovered = true;
      preview();
    },
    mouseout() {
      hovered = false;
      endPreview();
    },
    click() {
      activateArea(layer);
    },
    keydown({ originalEvent }) {
      if (originalEvent.key !== "Enter" && originalEvent.key !== " ") return;
      originalEvent.preventDefault();
      activateArea(layer);
    },
  });
}

const areas = L.geoJSON(null, {
  pane: "areas",
  style: {
    color: "#25766f",
    fillColor: "#83c9bf",
    fillOpacity: 0.22,
    weight: 1.5,
  },
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

function capabilityLabel(value) {
  return CAPABILITY_LABELS[value] ?? value.toLowerCase().replaceAll("_", " ");
}

function formatAge(minAge, maxAge) {
  if (minAge != null && maxAge != null) return `${minAge}–${maxAge} years`;
  if (minAge != null) return `${minAge}+ years`;
  if (maxAge != null) return `Up to ${maxAge} years`;
  return "Age not recorded";
}

function equipmentItems(playground) {
  return playground.equipment ?? playground.capabilities?.map((capability) => ({ capability, count: null })) ?? [];
}

function equipmentSummary(playground) {
  const items = equipmentItems(playground);
  if (!items.length) return "No equipment recorded";
  return items
    .map(({ capability, count }) => `${capabilityLabel(capability)}${count == null ? "" : ` (${count})`}`)
    .join(", ");
}

function image(url, alt, className) {
  const element = document.createElement("img");
  element.src = url;
  element.alt = alt;
  element.loading = "lazy";
  if (className) element.className = className;
  element.addEventListener("error", () => element.remove());
  return element;
}

function playgroundPreview(playground) {
  const content = document.createElement("article");
  content.className = "playground-preview";
  if (playground.photoUrls?.[0]) {
    content.append(image(playground.photoUrls[0], "", "preview-photo"));
  }
  const title = document.createElement("strong");
  title.textContent = playground.name ?? "Unnamed playground";
  content.append(title);

  const age = document.createElement("p");
  age.textContent = `Recommended age: ${formatAge(playground.minAge, playground.maxAge)}`;
  content.append(age);

  const equipment = document.createElement("p");
  equipment.textContent = equipmentSummary(playground);
  content.append(equipment);

  const rating = document.createElement("p");
  rating.textContent = "No ratings yet";
  content.append(rating);
  return content;
}

function showPreview(playground, marker) {
  if (selectedPlayground) return;
  closePreview();
  previewMarker = marker;
  updateMarkerStyle(marker);
  previewPopup = L.popup({
    autoPan: false,
    closeButton: false,
    className: "playground-preview-popup",
    offset: [0, -12],
  })
    .setLatLng(marker.getLatLng())
    .setContent(playgroundPreview(playground))
    .openOn(map);
}

function renderDetails(playground) {
  detailsContent.replaceChildren();
  if (!playground) {
    const message = document.createElement("p");
    message.textContent = "Playground details are unavailable.";
    detailsContent.append(message);
    return;
  }

  detailsTitle.textContent = playground.name ?? "Unnamed playground";

  const gallery = document.createElement("div");
  gallery.className = "playground-gallery";
  if (playground.photoUrls?.length) {
    for (const url of playground.photoUrls) gallery.append(image(url, "Playground", "detail-photo"));
  } else {
    const empty = document.createElement("p");
    empty.textContent = "No photos recorded";
    gallery.append(empty);
  }
  detailsContent.append(gallery);

  const ageSection = document.createElement("section");
  ageSection.innerHTML = "<h3>Recommended age</h3>";
  const age = document.createElement("p");
  age.textContent = formatAge(playground.minAge, playground.maxAge);
  ageSection.append(age);
  detailsContent.append(ageSection);

  const equipmentSection = document.createElement("section");
  equipmentSection.innerHTML = "<h3>Equipment</h3>";
  const equipment = equipmentItems(playground);
  if (equipment.length) {
    const list = document.createElement("ul");
    for (const item of equipment) {
      const entry = document.createElement("li");
      entry.textContent = `${capabilityLabel(item.capability)} — ${item.count == null ? "quantity unknown" : item.count}`;
      list.append(entry);
    }
    equipmentSection.append(list);
  } else {
    const empty = document.createElement("p");
    empty.textContent = "No equipment recorded";
    equipmentSection.append(empty);
  }
  detailsContent.append(equipmentSection);

  const reviews = document.createElement("section");
  reviews.innerHTML = "<h3>Platform reviews</h3>";
  const reviewState = document.createElement("p");
  reviewState.textContent = "No ratings yet. No reviews yet.";
  reviews.append(reviewState);
  detailsContent.append(reviews);

  const source = document.createElement("section");
  source.innerHTML = "<h3>Source</h3>";
  const sourceLink = document.createElement("a");
  sourceLink.href = playground.source.url;
  sourceLink.target = "_blank";
  sourceLink.rel = "noreferrer";
  sourceLink.textContent = "OpenStreetMap";
  source.append(sourceLink);
  const attribution = document.createElement("p");
  attribution.textContent = `${playground.source.attribution} · ${playground.source.license}`;
  source.append(attribution);
  detailsContent.append(source);
}

async function openDetails(summary, marker) {
  closePreview();
  if (selectedPlaygroundMarker && selectedPlaygroundMarker !== marker) {
    updateMarkerStyle(selectedPlaygroundMarker);
  }
  selectedPlayground = summary;
  selectedPlaygroundMarker = marker;
  updateMarkerStyle(marker);
  updateBackButton();
  details.hidden = false;
  detailsTitle.textContent = summary.name ?? "Playground details";
  detailsContent.replaceChildren();
  const loading = document.createElement("p");
  loading.textContent = "Loading playground details…";
  detailsContent.append(loading);

  detailRequest?.abort();
  const request = new AbortController();
  detailRequest = request;
  try {
    const response = await fetch(API_URL, {
      method: "POST",
      headers: { "content-type": "application/json" },
      signal: request.signal,
      body: JSON.stringify({
        query: PLAYGROUND_DETAIL_QUERY,
        variables: { id: summary.id },
      }),
    });
    const result = await response.json();
    if (!response.ok || result.errors) {
      throw new Error(result.errors?.[0]?.message ?? `Playground API failed: ${response.status}`);
    }
    if (detailRequest === request) renderDetails(result.data.playground);
  } catch (error) {
    if (error.name !== "AbortError") {
      console.error("Playground details unavailable", error);
      if (detailRequest === request) renderDetails(null);
    }
  }
}

function renderPlaygrounds(items) {
  closePreview();
  playgrounds.clearLayers();
  for (const playground of items) {
    const label = playground.name ?? "Unnamed playground";
    const marker = L.circleMarker(
      [playground.location.latitude, playground.location.longitude],
      { ...DEFAULT_PIN_STYLE, pane: "playgrounds", bubblingMouseEvents: false },
    ).addTo(playgrounds);
    marker.on({
      mouseover: () => showPreview(playground, marker),
      mouseout: closePreview,
      click: ({ originalEvent }) => {
        L.DomEvent.stopPropagation(originalEvent);
        openDetails(playground, marker);
      },
    });
    const element = marker.getElement();
    element?.setAttribute("tabindex", "0");
    element?.setAttribute("role", "button");
    element?.setAttribute("aria-label", `Open details for ${label}`);
    element?.addEventListener("focus", () => showPreview(playground, marker));
    element?.addEventListener("blur", closePreview);
    element?.addEventListener("keydown", (event) => {
      if (event.key !== "Enter" && event.key !== " ") return;
      event.preventDefault();
      openDetails(playground, marker);
    });
    if (selectedPlayground?.id === playground.id) {
      selectedPlaygroundMarker = marker;
      updateMarkerStyle(marker);
    }
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

backButton.addEventListener("click", goBack);
detailsClose.addEventListener("click", goBack);
document.addEventListener("keydown", (event) => {
  if (event.key === "Escape") goBack();
});
map.on("moveend", loadPlaygrounds);
map.on("click", () => {
  if (selectedPlayground) closeDetails();
});
addAreaData("/data/sofia-neighborhoods.geojson", true).then(() =>
  addAreaData("/data/sofia-discovery-areas.geojson"),
);
loadPlaygrounds();
