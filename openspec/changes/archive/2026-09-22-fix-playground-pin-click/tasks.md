# Tasks

## 1. Playground Pointer Activation

- [x] 1.1 Stop playground marker mouse events from bubbling to the map in `src/main.js`; verify a mouse click opens details and leaves them visible while the preview is active.
- [x] 1.2 Add the pointer-preview activation regression case to `TEST_CASES.md`; verify the case documents expected detail-panel and selected-pin behavior.

## 2. Verification

- [x] 2.1 Run `npm test` and `npm run build`; verify both commands pass.
- [x] 2.2 Reproduce the interaction flow in a browser; verify hover preview still appears, clicking the pin opens and keeps details visible, clicking bare map space closes details, and keyboard activation still works.
