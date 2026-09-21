# Proposal

## Why

Pointer users cannot keep playground details open because the pin click also reaches the map handler, which immediately closes the details. Keyboard activation works, so the same control behaves inconsistently across input methods.

## What Changes

- Keep pointer-opened playground details from closing on the same input.
- Preserve the existing pointer and keyboard preview behavior.
- Add a regression check that opens playground details after pointer preview.

## Capabilities

### New Capabilities

None.

### Modified Capabilities

- `sofia-neighborhood-map`: Clarify that pointer activation opens and keeps playground details visible while the preview is active.

## Impact

- `src/main.js`: Playground marker event propagation.
- `TEST_CASES.md`: Pointer activation regression coverage.
- No API, data model, dependency, or deployment changes.
