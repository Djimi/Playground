# Proposal

## Why

Keyboard users can reach area polygons, but focus does not provide the same name and visual preview as pointer hover. Opening a playground detail view also leaves focus on the map pin, and closing the view loses focus entirely, making the next keyboard action unpredictable.

## What Changes

- Show area preview styling and the area name while an area polygon has keyboard focus; restore the default preview state on blur.
- Move focus to the details Back control when playground details open.
- Restore focus to the activating playground pin when details close, when that pin is still connected.
- Preserve existing pointer, selection, navigation, and responsive behavior.

## Capabilities

### New Capabilities

None.

### Modified Capabilities

- `sofia-neighborhood-map`: define keyboard-focused area previews and predictable focus entry/return for playground details.

## Impact

- `src/main.js`: add focus state handling for area previews and the playground details lifecycle.
- `openspec/specs/sofia-neighborhood-map/spec.md`: merge the new keyboard and dialog-focus behavior.
- No API, dependency, persistence, or deployment changes.
