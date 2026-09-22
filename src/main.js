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
  query AllPlaygrounds($limit: Int!, $offset: Int!) {
    playgrounds(limit: $limit, offset: $offset) {
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

const mapElement = document.querySelector("#map");
const map = L.map(mapElement, { zoomControl: true }).setView(SOFIA_CENTER, 12);
map.createPane("areas").style.zIndex = 400;
map.createPane("playgrounds").style.zIndex = 450;
new ResizeObserver(() => map.invalidateSize()).observe(mapElement);

L.tileLayer(TILE_URL, {
  attribution: TILE_ATTRIBUTION,
  maxZoom: 19,
}).addTo(map);

const areaName = document.querySelector("#area-name");
const mapHeader = document.querySelector(".map-header");
const mapStatus = document.querySelector("#map-status");
const backButton = document.querySelector("#back-button");
const details = document.querySelector("#playground-details");
const detailsClose = document.querySelector("#details-close");
const detailsTitle = document.querySelector("#playground-details-title");
const detailsContent = document.querySelector("#playground-details-content");
const playgrounds = L.layerGroup().addTo(map);
const layerStatus = {
  neighborhoods: "loading",
  destinations: "loading",
  playgrounds: "loading",
};
let selectedPlayground;
let selectedPlaygroundMarker;
let playgroundsLoading = false;
let detailRequest;
let previewPopup;
let previewMarker;
let detailsReturnFocus;
let suppressFocusPreview = false;
let hoveredArea;
let focusedArea;
let hoveredPlayground;
let focusedPlayground;

function showAreaName(name) {
  areaName.textContent = name ?? DEFAULT_LABEL;
}

function updateMapStatus() {
  const loading = [];
  if (layerStatus.playgrounds === "loading") loading.push("playgrounds");
  if (layerStatus.neighborhoods === "loading") loading.push("neighborhood areas");
  if (layerStatus.destinations === "loading") loading.push("destination areas");

  const messages = loading.length ? [`Loading ${loading.join(", ")}…`] : [];
  if (layerStatus.playgrounds === "empty") messages.push("No playgrounds are currently available.");
  if (layerStatus.playgrounds === "error") messages.push("Playgrounds are unavailable.");
  if (layerStatus.neighborhoods === "empty") messages.push("No neighborhood areas are currently available.");
  if (layerStatus.neighborhoods === "error") messages.push("Neighborhood areas are unavailable.");
  if (layerStatus.destinations === "empty") messages.push("No destination areas are currently available.");
  if (layerStatus.destinations === "error") messages.push("Destination areas are unavailable.");

  mapStatus.textContent = messages.join(" ");
  mapStatus.hidden = messages.length === 0;
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

function closePreviewPopup() {
  if (previewPopup) {
    map.closePopup(previewPopup);
    previewPopup = undefined;
  }
  const marker = previewMarker;
  previewMarker = undefined;
  if (marker) updateMarkerStyle(marker);
}

function syncAreaPreview() {
  const active = focusedArea ?? hoveredArea;
  areas.eachLayer((layer) => {
    if (layer === active?.layer) layer.setStyle(HOVER_STYLE);
    else areas.resetStyle(layer);
  });
  showAreaName(active?.name);
}

function syncPlaygroundPreview() {
  if (selectedPlayground) {
    closePreviewPopup();
    return;
  }

  const active = focusedPlayground ?? hoveredPlayground;
  if (!active) {
    closePreviewPopup();
    return;
  }
  if (previewMarker === active.marker && previewPopup) {
    updateMarkerStyle(active.marker);
    return;
  }

  closePreviewPopup();
  previewMarker = active.marker;
  updateMarkerStyle(active.marker);
  previewPopup = L.popup({
    autoPan: true,
    autoPanPaddingTopLeft: [16, 128],
    autoPanPaddingBottomRight: [16, 16],
    keepInView: true,
    animate: false,
    closeButton: false,
    className: "playground-preview-popup",
    offset: [0, -12],
  })
    .setLatLng(active.marker.getLatLng())
    .setContent(playgroundPreview(active.playground))
    .openOn(map);
}

function clearPlaygroundPreview() {
  hoveredPlayground = undefined;
  focusedPlayground = undefined;
  closePreviewPopup();
}

function setDetailsModalOpen(open) {
  details.hidden = !open;
  mapElement.inert = open;
  mapHeader.inert = open;
}

function closeDetails({ restoreFocus = true } = {}) {
  detailRequest?.abort();
  detailRequest = undefined;
  const focusTarget = detailsReturnFocus;
  const refreshedFocusTarget = selectedPlaygroundMarker?.getElement();
  clearPlaygroundPreview();
  detailsReturnFocus = undefined;
  selectedPlaygroundMarker = undefined;
  selectedPlayground = undefined;
  for (const marker of playgrounds.getLayers()) updateMarkerStyle(marker);
  setDetailsModalOpen(false);
  detailsContent.replaceChildren();
  updateBackButton();
  if (!restoreFocus) return;
  const target = focusTarget?.isConnected ? focusTarget : refreshedFocusTarget;
  if (!target?.isConnected) return;
  suppressFocusPreview = true;
  target.focus();
  suppressFocusPreview = false;
}

function goBack() {
  if (selectedPlayground) {
    closeDetails();
  }
}

function activateArea(layer) {
  if (selectedPlayground) {
    closeDetails({ restoreFocus: false });
    return;
  }
  map.fitBounds(layer.getBounds(), { maxZoom: 16, padding: [40, 40] });
}

function onEachArea(feature, layer) {
  const area = { layer, name: feature.properties.name };
  layer.on({
    add() {
      const element = layer.getElement();
      element?.setAttribute("tabindex", "0");
      element?.setAttribute("role", "button");
      element?.setAttribute("aria-label", `Select ${feature.properties.name}`);
      element?.addEventListener("focus", () => {
        focusedArea = area;
        syncAreaPreview();
      });
      element?.addEventListener("blur", () => {
        if (focusedArea?.layer === layer) focusedArea = undefined;
        syncAreaPreview();
      });
    },
    mouseover() {
      hoveredArea = area;
      syncAreaPreview();
    },
    mouseout() {
      if (hoveredArea?.layer === layer) hoveredArea = undefined;
      syncAreaPreview();
    },
    click() {
      activateArea(layer);
      layer.getElement()?.blur();
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

function addAreaData(url, statusKey, fitMap = false) {
  return fetch(url)
    .then((response) => {
      if (!response.ok) throw new Error(`${url} failed: ${response.status}`);
      return response.json();
    })
    .then((data) => {
      const featureCount = Array.isArray(data.features) ? data.features.length : 0;
      areas.addData(data);
      layerStatus[statusKey] = featureCount ? "ready" : "empty";
      updateMapStatus();
      if (fitMap && featureCount) map.fitBounds(areas.getBounds(), { padding: [16, 16] });
    })
    .catch((error) => {
      layerStatus[statusKey] = "error";
      updateMapStatus();
      console.error(error);
    });
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
  element.addEventListener("error", () => {
    const fallback = document.createElement("div");
    fallback.className = [className, "photo-unavailable"].filter(Boolean).join(" ");
    fallback.textContent = "Photo unavailable";
    element.replaceWith(fallback);
  });
  return element;
}

function playgroundPreview(playground) {
  const content = document.createElement("article");
  content.className = "playground-preview";
  const label = playground.name ?? "Unnamed playground";
  if (playground.photoUrls?.[0]) {
    content.append(image(playground.photoUrls[0], `${label} photo`, "preview-photo"));
  }
  const title = document.createElement("strong");
  title.textContent = label;
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

function renderDetails(playground) {
  detailsContent.replaceChildren();
  if (!playground) {
    const message = document.createElement("p");
    message.textContent = "Playground details are unavailable.";
    detailsContent.append(message);
    return;
  }

  const label = playground.name ?? "Unnamed playground";
  detailsTitle.textContent = label;

  const gallery = document.createElement("div");
  gallery.className = "playground-gallery";
  if (playground.photoUrls?.length) {
    for (const url of playground.photoUrls) gallery.append(image(url, `${label} photo`, "detail-photo"));
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
  const previousMarker = selectedPlaygroundMarker;
  clearPlaygroundPreview();
  selectedPlayground = summary;
  selectedPlaygroundMarker = marker;
  if (previousMarker && previousMarker !== marker) updateMarkerStyle(previousMarker);
  detailsReturnFocus = marker.getElement();
  updateMarkerStyle(marker);
  updateBackButton();
  setDetailsModalOpen(true);
  detailsClose.focus();
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
  clearPlaygroundPreview();
  playgrounds.clearLayers();
  for (const playground of items) {
    const label = playground.name ?? "Unnamed playground";
    const marker = L.circleMarker(
      [playground.location.latitude, playground.location.longitude],
      { ...DEFAULT_PIN_STYLE, pane: "playgrounds", bubblingMouseEvents: false },
    ).addTo(playgrounds);
    marker.on({
      mouseover: () => {
        hoveredPlayground = { playground, marker };
        syncPlaygroundPreview();
      },
      mouseout: () => {
        if (hoveredPlayground?.marker === marker) hoveredPlayground = undefined;
        syncPlaygroundPreview();
      },
      click: ({ originalEvent }) => {
        L.DomEvent.stopPropagation(originalEvent);
        openDetails(playground, marker);
      },
    });
    const element = marker.getElement();
    element?.setAttribute("tabindex", "0");
    element?.setAttribute("role", "button");
    element?.setAttribute("aria-label", `Open details for ${label}`);
    element?.addEventListener("focus", () => {
      if (suppressFocusPreview) return;
      focusedPlayground = { playground, marker };
      syncPlaygroundPreview();
    });
    element?.addEventListener("blur", () => {
      if (focusedPlayground?.marker === marker) focusedPlayground = undefined;
      syncPlaygroundPreview();
    });
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
  if (playgroundsLoading) return;
  playgroundsLoading = true;
  layerStatus.playgrounds = "loading";
  updateMapStatus();
  try {
    const items = [];
    for (let offset = 0; ; offset += PAGE_SIZE) {
      const response = await fetch(API_URL, {
        method: "POST",
        headers: { "content-type": "application/json" },
        body: JSON.stringify({
          query: PLAYGROUNDS_QUERY,
          variables: { limit: PAGE_SIZE, offset },
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
    renderPlaygrounds(items);
    layerStatus.playgrounds = items.length ? "ready" : "empty";
    updateMapStatus();
  } catch (error) {
    playgrounds.clearLayers();
    layerStatus.playgrounds = "error";
    updateMapStatus();
    console.error("Playgrounds unavailable", error);
  } finally {
    playgroundsLoading = false;
  }
}

backButton.addEventListener("click", goBack);
detailsClose.addEventListener("click", goBack);
details.addEventListener("keydown", (event) => {
  if (event.key !== "Tab") return;
  const focusable = [...details.querySelectorAll("button:not([disabled]), a[href], input:not([disabled]), select:not([disabled]), textarea:not([disabled])")];
  if (!focusable.length) {
    event.preventDefault();
    detailsClose.focus();
    return;
  }
  const first = focusable[0];
  const last = focusable.at(-1);
  if (event.shiftKey && document.activeElement === first) {
    event.preventDefault();
    last.focus();
  } else if (!event.shiftKey && document.activeElement === last) {
    event.preventDefault();
    first.focus();
  }
});
document.addEventListener("keydown", (event) => {
  if (event.key === "Escape" && selectedPlayground) {
    event.preventDefault();
    closeDetails();
  }
}, true);
document.addEventListener("click", (event) => {
  if (selectedPlayground && !details.contains(event.target)) closeDetails({ restoreFocus: false });
});
map.on("click", () => {
  if (selectedPlayground) closeDetails({ restoreFocus: false });
});
updateMapStatus();
addAreaData("/data/sofia-neighborhoods.geojson", "neighborhoods", true);
addAreaData("/data/sofia-discovery-areas.geojson", "destinations");
loadPlaygrounds();
