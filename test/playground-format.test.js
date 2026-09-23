import test from "node:test";
import assert from "node:assert/strict";
import {
  formatAge,
  formatKnown,
  formatEquipment,
  formatSourceDate,
  formatNeighborhoods,
  formatPhotoCredit,
  selectedSourceValue,
  displayedSources,
  formatMunicipalStatus,
} from "../src/playground-format.js";

test("playground formatting: age bounds stay explicit", () => {
  assert.equal(formatAge(3, 12), "3–12 years");
  assert.equal(formatAge(3, null), "3+ years");
  assert.equal(formatAge(null, 12), "Up to 12 years");
  assert.equal(formatAge(null, null), "Age not recorded");
});

test("playground formatting: false differs from unknown", () => {
  assert.equal(formatKnown(false, "Yes", "No"), "No");
  assert.equal(formatKnown(null, "Yes", "No"), "Unknown");
});

test("playground formatting: equipment counts and missing quantities remain clear", () => {
  assert.equal(formatEquipment([{ capability: "SWING", count: 2 }]), "2 swings");
  assert.equal(formatEquipment([{ capability: "SLIDE", count: null }]), "slide (quantity unknown)");
  assert.equal(formatEquipment([]), "No equipment recorded");
});

test("playground formatting: source dates distinguish observations from updates", () => {
  assert.equal(formatSourceDate("2019-04-18T00:00:00Z", "OBSERVATION"), "Observed 18 Apr 2019");
  assert.equal(formatSourceDate("2026-09-22T10:00:00Z", "SOURCE_UPDATE"), "Source updated 22 Sep 2026");
  assert.equal(formatSourceDate(null, null), "Date unknown");
});

test("playground formatting: every neighborhood is displayed", () => {
  assert.equal(formatNeighborhoods([{ name: "Lozenets" }, { name: "South Park" }]), "Lozenets · South Park");
  assert.equal(formatNeighborhoods([]), "Neighborhood unknown");
});

test("playground formatting: selected provenance is field-specific", () => {
  const values = [
    { field: "fenced", selected: false },
    { field: "fenced", selected: true },
    { field: "surface", selected: true },
  ];
  assert.equal(selectedSourceValue(values, "fenced"), values[1]);
  assert.equal(selectedSourceValue(values, "fee"), null);
});

test("playground formatting: photo author and credit stay visible without identical duplication", () => {
  assert.equal(formatPhotoCredit({ author: "Photographer", attribution: "Publisher credit" }), "Photographer · Publisher credit");
  assert.equal(formatPhotoCredit({ author: "Photographer", attribution: "Photographer" }), "Photographer");
});

test("playground formatting: an empty linked source list falls back to the populated legacy source", () => {
  const legacy = { id: "node/1", kind: "OPEN_STREET_MAP" };
  const linked = [{ id: "openstreetmap/node/1" }, { id: "sofiaplan/06.129" }];
  assert.deepEqual(displayedSources([], legacy), [legacy]);
  assert.deepEqual(displayedSources(undefined, legacy), [legacy]);
  assert.deepEqual(displayedSources(linked, legacy), linked);
  assert.deepEqual(displayedSources([], null), []);
});

test("playground formatting: missing municipal status keeps an explicit unknown state", () => {
  assert.equal(formatMunicipalStatus("Major repair"), "Major repair");
  assert.equal(formatMunicipalStatus(null), "Municipal status unknown");
  assert.equal(formatMunicipalStatus(undefined), "Municipal status unknown");
});
