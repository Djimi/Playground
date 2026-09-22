# Tasks

## 1. Refresh and focus behavior

- [x] 1.1 Clear playground pins when the current non-aborted bounds request fails, while leaving newer request results untouched; verify with the existing test suite and a controlled failed refresh.
- [x] 1.2 Restore focus to the refreshed selected marker when the original marker element was replaced; verify with keyboard-open, refresh, and Back/Escape behavior.

## 2. Phone layout

- [x] 2.1 Reserve mobile header space for long area names beside the Back control; verify at 320 CSS pixels with a long area name.
- [x] 2.2 Move mobile zoom controls away from the fixed header; verify the controls remain usable and do not overlap the header at 320 CSS pixels.

## 3. Verification

- [x] 3.1 Run `npm test`, `npm run build`, `openspec validate --specs --strict`, and `git diff --check`; verify all pass.
