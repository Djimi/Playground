# Proposal

## Why

Clicking an empty map spot while playground details are open closes the details and clears the selected pin, but the closing code restores keyboard focus to that pin. The pin's focus handler then reopens the compact preview popup, draws a browser focus ring around the pin, and repaints it from selected orange to preview teal. The user sees a square and a popup after clicking on the side and expects nothing to stay selected.

Clicking a neighborhood or destination polygon with the pointer also leaves that polygon in focus-styling after the pointer leaves, so the highlight does not follow the pointer to the next hovered area.

## What Changes

- Clicking or tapping the map background or an area polygon while details are open closes the details, clears the selected playground, and does not restore focus to the pin, so no preview popup, focus ring, or preview styling appears.
- Pressing `Escape` or activating the Back control still closes the details and returns keyboard focus to the pin when it is available, but the compact preview no longer reopens automatically from that restored focus. Selection is removed entirely.
- Pointer activation of a neighborhood, South Park, or Sofia Zoo polygon only zooms; the polygon no longer keeps preview styling after the pointer leaves. Hover highlighting follows the pointer instead. Keyboard activation keeps preview styling while the polygon has focus.
- Existing detail open/close behavior, keyboard focus return, and polygon zoom fitting stay intact.

## Capabilities

### New Capabilities

(none)

### Modified Capabilities

- `sofia-neighborhood-map`: "Leave focused map states predictably" (clicking the map while details are open must not focus or preview the pin), "Open responsive playground details" (Back/`Escape` must not reopen the compact preview through restored focus), and "Select a neighborhood" / "Preview area names" (pointer activation leaves no persistent polygon highlight).

## Impact

- `src/main.js`: `closeDetails`, `activateArea`, the map `click` handler, and the area `click` handler in `onEachArea`.
- `TEST_CASES.md`: manual cases `NAV-001` through `NAV-003`, `POINTER-001`, `POINTER-002`, and `PLAYGROUND-010` need updated expectations.
- No API, database, or dependency changes.
