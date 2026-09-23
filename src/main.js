import L from "leaflet";
import "leaflet/dist/leaflet.css";
import "./styles.css";
import { displayedSources, formatAge, formatEquipment, formatKnown, formatMunicipalStatus, formatNeighborhoods, formatPhotoCredit, formatSourceDate, selectedSourceValue } from "./playground-format.js";

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
      neighborhoods { name }
      photos { url author license licenseUrl attribution sourceUrl }
      equipment { capability count }
      municipalStatus
      sourceValues { field value source sourceId date dateMeaning selected }
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
      neighborhoods { name }
      address surface fenced ownership access fee
      municipalStatus ordinanceCompliant repairs notes
      photos { url author license licenseUrl attribution sourceUrl }
      equipment { capability count }
      source { id kind url updatedAt dateMeaning attribution license }
      sources { id kind url updatedAt dateMeaning attribution license }
      sourceValues { field value source sourceId date dateMeaning selected }
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
let previewSyncTimer;
let previewAutoPanPending = false;
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
  previewAutoPanPending = false;
  if (marker) updateMarkerStyle(marker);
}

function schedulePreviewSync(delay = 0) {
  clearTimeout(previewSyncTimer);
  previewSyncTimer = setTimeout(() => {
    previewSyncTimer = undefined;
    syncPlaygroundPreview();
  }, delay);
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

  const popupElement = previewPopup?.getElement();
  if (popupElement?.contains(document.activeElement)) return;
  if (previewAutoPanPending) return;
  const active = focusedPlayground ?? hoveredPlayground;
  if (!active) {
    if (popupElement?.matches(":hover")) return;
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
  const previewContent = playgroundPreview(active.playground, active.marker);
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
    .setContent(previewContent)
    .openOn(map);
  const openedPopupElement = previewPopup.getElement();
  openedPopupElement.addEventListener("mouseenter", () => {
    previewAutoPanPending = false;
    clearTimeout(previewSyncTimer);
  });
  openedPopupElement.addEventListener("mouseleave", schedulePreviewSync);
  openedPopupElement.addEventListener("focusin", () => {
    previewAutoPanPending = false;
    clearTimeout(previewSyncTimer);
  });
  openedPopupElement.addEventListener("focusout", schedulePreviewSync);
}

function clearPlaygroundPreview() {
  clearTimeout(previewSyncTimer);
  previewSyncTimer = undefined;
  hoveredPlayground = undefined;
  focusedPlayground = undefined;
  closePreviewPopup();
}

map.on("autopanstart", () => {
  if (previewMarker && !focusedPlayground) previewAutoPanPending = true;
});
mapElement.addEventListener("pointermove", () => {
  if (!previewAutoPanPending) return;
  clearTimeout(previewSyncTimer);
  // ponytail: wait for pointer motion to settle; revisit if slow gap crossings still close the popup.
  previewSyncTimer = setTimeout(() => {
    previewAutoPanPending = false;
    syncPlaygroundPreview();
  }, 120);
});
mapElement.addEventListener("pointerleave", () => {
  if (!previewAutoPanPending) return;
  previewAutoPanPending = false;
  schedulePreviewSync();
});

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

function textElement(tag, value, className) {
  const element = document.createElement(tag);
  element.textContent = value;
  if (className) element.className = className;
  return element;
}

function safeUrl(value) {
  try {
    const url = new URL(value);
    return ["http:", "https:"].includes(url.protocol) ? url.href : null;
  } catch {
    return null;
  }
}

function externalLink(label, url) {
  const href = safeUrl(url);
  if (!href) return textElement("span", label);
  const link = textElement("a", label);
  link.href = href;
  link.target = "_blank";
  link.rel = "noopener noreferrer";
  return link;
}

function image(url, alt, className) {
  if (!safeUrl(url)) return textElement("div", "Photo unavailable", `${className} photo-unavailable`);
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

function photoFigure(photo, label, className, linked = true) {
  const figure = document.createElement("figure");
  figure.className = "playground-photo";
  figure.append(image(photo.url, `${label} photo`, className));
  const caption = document.createElement("figcaption");
  caption.append(linked ? externalLink(formatPhotoCredit(photo), photo.sourceUrl) : textElement("span", formatPhotoCredit(photo)));
  caption.append(" · ", linked ? externalLink(photo.license, photo.licenseUrl) : textElement("span", photo.license));
  figure.append(caption);
  return figure;
}

function nextMapTabStop(marker) {
  const popupElement = previewPopup?.getElement();
  const focusable = [...document.querySelectorAll("button:not([disabled]), a[href], [tabindex='0']")]
    .filter((element) => !element.closest("[hidden], [inert]") && !popupElement?.contains(element));
  return focusable[focusable.indexOf(marker.getElement()) + 1];
}

function sourceName(source) {
  return source === "SOFIA_PLAN" ? "SofiaPlan" : "OpenStreetMap";
}

function historicalWarning(playground) {
  if (![playground.municipalStatus, playground.repairs, playground.ordinanceCompliant, playground.notes].some((value) => value != null)) return null;
  const selected = selectedSourceValue(playground.sourceValues ?? [], "municipal_status");
  const source = playground.sources?.find((item) => item.kind === "SOFIA_PLAN");
  const date = selected?.date ?? source?.updatedAt;
  const meaning = selected?.dateMeaning ?? source?.dateMeaning;
  const warning = document.createElement("div");
  warning.className = "playground-warning";
  warning.append(textElement("strong", `Historical municipal record${playground.municipalStatus ? `: ${playground.municipalStatus}` : ""}`));
  warning.append(textElement("p", `${formatSourceDate(date, meaning)}. May be outdated; current conditions may differ.`));
  return warning;
}

function playgroundPreview(playground, marker) {
  const content = document.createElement("article");
  content.className = "playground-preview";
  const label = playground.name ?? "Unnamed playground";
  content.append(playground.photos?.[0]
    ? photoFigure(playground.photos[0], label, "preview-photo", false)
    : textElement("div", "No photo yet", "preview-photo photo-unavailable"));
  content.append(textElement("strong", label));
  content.append(textElement("p", formatNeighborhoods(playground.neighborhoods ?? []), "playground-muted"));
  const chips = document.createElement("div");
  chips.className = "playground-chips";
  chips.append(textElement("span", formatAge(playground.minAge, playground.maxAge), "playground-chip"));
  chips.append(textElement("span", formatEquipment(playground.equipment), "playground-chip"));
  content.append(chips);
  const warning = playground.municipalStatus != null ? historicalWarning(playground) : null;
  if (warning) content.append(warning);
  else content.append(textElement("p", formatMunicipalStatus(playground.municipalStatus), "playground-muted"));
  content.append(textElement("p", "No ratings yet", "playground-muted"));
  const button = textElement("button", "View details", "preview-action");
  button.type = "button";
  L.DomEvent.disableClickPropagation(button);
  button.addEventListener("click", (event) => {
    L.DomEvent.stopPropagation(event);
    openDetails(playground, marker);
  });
  button.addEventListener("keydown", (event) => {
    if (event.key !== "Tab") return;
    const target = event.shiftKey ? marker.getElement() : nextMapTabStop(marker);
    if (!target) return;
    event.preventDefault();
    target.focus();
  });
  content.append(button);
  return content;
}

function detailSection(title) {
  const section = document.createElement("section");
  section.append(textElement("h3", title));
  return section;
}

function detailRow(list, label, value) {
  const row = document.createElement("div");
  row.className = "detail-row";
  row.append(textElement("dt", label), textElement("dd", value ?? "Unknown"));
  list.append(row);
}

function sourceValueText(value) {
  try {
    const parsed = JSON.parse(value);
    return parsed == null ? "Unknown" : typeof parsed === "object" ? JSON.stringify(parsed) : String(parsed);
  } catch {
    return value;
  }
}

function renderDetails(playground) {
  detailsContent.replaceChildren();
  if (!playground) {
    detailsContent.append(textElement("p", "Playground details are unavailable."));
    return;
  }

  const label = playground.name ?? "Unnamed playground";
  detailsTitle.textContent = label;

  const gallery = document.createElement("div");
  gallery.className = "playground-gallery";
  if (playground.photos?.length) {
    for (const photo of playground.photos) gallery.append(photoFigure(photo, label, "detail-photo"));
  } else {
    gallery.append(textElement("p", "No photo yet", "photo-unavailable"));
  }
  detailsContent.append(gallery);
  detailsContent.append(textElement("p", playground.address ?? "Address unknown", "playground-muted"));
  detailsContent.append(textElement("p", formatNeighborhoods(playground.neighborhoods ?? []), "playground-muted"));
  const { latitude, longitude } = playground.location;
  detailsContent.append(textElement("p", `${latitude.toFixed(5)}, ${longitude.toFixed(5)}`, "playground-muted"));
  const actions = document.createElement("div");
  actions.className = "detail-actions";
  actions.append(externalLink("Directions in OpenStreetMap", `https://www.openstreetmap.org/directions?route=;${latitude}%2C${longitude}`));
  detailsContent.append(actions);

  const info = detailSection("Play information");
  const facts = document.createElement("dl");
  facts.className = "detail-facts";
  detailRow(facts, "Age", formatAge(playground.minAge, playground.maxAge));
  detailRow(facts, "Equipment", formatEquipment(playground.equipment));
  detailRow(facts, "Surface", playground.surface);
  detailRow(facts, "Fenced", formatKnown(playground.fenced, "Yes", "No"));
  detailRow(facts, "Ownership", playground.ownership);
  detailRow(facts, "Access", playground.access);
  detailRow(facts, "Fee", playground.fee);
  info.append(facts);
  detailsContent.append(info);

  const municipal = detailSection("Historical municipal record");
  const municipalFacts = document.createElement("dl");
  municipalFacts.className = "detail-facts";
  detailRow(municipalFacts, "Status", playground.municipalStatus);
  detailRow(municipalFacts, "Repairs", playground.repairs);
  detailRow(municipalFacts, "Ordinance 1", formatKnown(playground.ordinanceCompliant, "Compliant", "Not compliant"));
  detailRow(municipalFacts, "Notes", playground.notes);
  municipal.append(municipalFacts);
  const warning = historicalWarning(playground);
  if (warning) municipal.append(warning);
  detailsContent.append(municipal);

  const reviews = detailSection("Community");
  reviews.append(textElement("p", "No ratings yet"), textElement("p", "No reviews yet"));
  detailsContent.append(reviews);

  const history = detailSection("Source values");
  if (playground.sourceValues?.length) {
    const list = document.createElement("ul");
    list.className = "source-history";
    for (const item of playground.sourceValues) {
      const field = item.field.replaceAll("_", " ").replaceAll(".", " · ");
      list.append(textElement("li", `${item.selected ? "Selected" : "Older or conflicting"} ${field}: ${sourceValueText(item.value)} — ${sourceName(item.source)} ${item.sourceId}, ${formatSourceDate(item.date, item.dateMeaning)}`));
    }
    history.append(list);
  } else {
    history.append(textElement("p", "No source value history recorded"));
  }
  detailsContent.append(history);

  const sources = detailSection("Sources");
  for (const source of displayedSources(playground.sources, playground.source)) {
    const entry = document.createElement("p");
    entry.className = "source-entry";
    entry.append(externalLink(sourceName(source.kind), source.url));
    entry.append(` · ${formatSourceDate(source.updatedAt, source.dateMeaning)} · ${source.attribution} · ${source.license}`);
    sources.append(entry);
  }
  detailsContent.append(sources);
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
        clearTimeout(previewSyncTimer);
        hoveredPlayground = { playground, marker };
        syncPlaygroundPreview();
      },
      mouseout: () => {
        if (hoveredPlayground?.marker === marker) hoveredPlayground = undefined;
        // ponytail: bridge the popup's marker-to-tip gap; use a pointer hit area if it grows.
        schedulePreviewSync(200);
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
      clearTimeout(previewSyncTimer);
      focusedPlayground = { playground, marker };
      previewAutoPanPending = false;
      syncPlaygroundPreview();
    });
    element?.addEventListener("blur", () => {
      if (focusedPlayground?.marker === marker) focusedPlayground = undefined;
      schedulePreviewSync();
    });
    element?.addEventListener("keydown", (event) => {
      if (event.key === "Tab" && previewMarker === marker && previewPopup) {
        const button = previewPopup.getElement()?.querySelector(".preview-action");
        if (event.shiftKey) {
          // Let native reverse tab order skip the popup that precedes the marker pane.
          if (button) {
            button.tabIndex = -1;
            setTimeout(() => { if (button.isConnected) button.removeAttribute("tabindex"); }, 0);
          }
          return;
        }
        if (button) {
          event.preventDefault();
          button.focus();
          return;
        }
      }
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
