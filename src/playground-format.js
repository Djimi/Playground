const EQUIPMENT_NAMES = {
  CLIMBING_FRAME: ["climbing frame", "climbing frames"],
  PLAYHOUSE: ["playhouse", "playhouses"],
  ROUNDABOUT: ["roundabout", "roundabouts"],
  SANDPIT: ["sandpit", "sandpits"],
  SEESAW: ["seesaw", "seesaws"],
  SLIDE: ["slide", "slides"],
  SPRINGY: ["spring rider", "spring riders"],
  SWING: ["swing", "swings"],
};

export function formatAge(minAge, maxAge) {
  if (minAge != null && maxAge != null) return `${minAge}–${maxAge} years`;
  if (minAge != null) return `${minAge}+ years`;
  if (maxAge != null) return `Up to ${maxAge} years`;
  return "Age not recorded";
}

export function formatKnown(value, yes, no) {
  return value == null ? "Unknown" : value ? yes : no;
}

export function formatEquipment(items) {
  if (!items?.length) return "No equipment recorded";
  return items.map(({ capability, count }) => {
    const fallback = capability.toLowerCase().replaceAll("_", " ");
    const names = EQUIPMENT_NAMES[capability] ?? [fallback, fallback];
    if (count == null) return `${names[0]} (quantity unknown)`;
    return `${count} ${count === 1 ? names[0] : names[1]}`;
  }).join(", ");
}

export function formatSourceDate(value, meaning) {
  if (!value) return "Date unknown";
  const date = new Intl.DateTimeFormat("en-GB", {
    day: "numeric", month: "short", year: "numeric", timeZone: "UTC",
  }).format(new Date(value)).replace("Sept", "Sep");
  return `${meaning === "OBSERVATION" ? "Observed" : "Source updated"} ${date}`;
}

export function formatNeighborhoods(items) {
  return items.length ? items.map(({ name }) => name).join(" · ") : "Neighborhood unknown";
}

export function formatPhotoCredit({ author, attribution }) {
  return [author, attribution && attribution !== author ? attribution : null].filter(Boolean).join(" · ") || "Photo source";
}

export function selectedSourceValue(items, field) {
  return items.find((item) => item.field === field && item.selected) ?? null;
}

export function displayedSources(sources, legacySource) {
  if (sources?.length) return sources;
  return legacySource ? [legacySource] : [];
}

export function formatMunicipalStatus(value) {
  return value ?? "Municipal status unknown";
}
