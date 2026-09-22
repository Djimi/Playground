# Tasks

## 1. Fix close behavior in the map UI

- [x] 1.1 Add a `restoreFocus` option (default `true`) to `closeDetails` and call it with `restoreFocus: false` from the map `click` handler and from `activateArea`; verify by code review that only `goBack` still restores focus.
- [x] 1.2 Suppress the focus-triggered compact preview during the programmatic focus restore after Back/`Escape` (synchronous flag around the `focus()` call, checked in the pin `focus` listener); verify by selecting a pin, pressing `Escape`, and confirming the pin stays focused with no preview popup, then `Tab` away and back to confirm preview still opens normally.
- [x] 1.3 Blur the activated polygon in the area `click` handler only (not in the `keydown` activation path); verify clicking an area zooms it, then hovering another area highlights only that one, and the clicked area returns to default styling once the pointer leaves.

## 2. Update manual test expectations

- [x] 2.1 Update `TEST_CASES.md` cases `NAV-001` through `NAV-003`, `POINTER-001`, `POINTER-002`, and `PLAYGROUND-010` for the new close behavior, and add a case for clicking the map background or an area polygon while details are open (details close, no pin focus ring, no preview popup); verify every updated case is executable step by step.

## 3. Verify

- [x] 3.1 Run `npm test` and `npm run build`; verify both succeed.
- [x] 3.2 In a browser with imported data: select a pin, then click empty map space and an area polygon; verify the details close, the pin returns to default blue with no focus ring and no preview popup; verify Back and `Escape` clear the selection without reopening the popup; repeat at 320 px, 768 px, and desktop widths.
