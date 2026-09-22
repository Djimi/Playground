# Design

## Context

See proposal.md. Area paths and playground pins are SVG elements made keyboard-focusable by `src/main.js`; the details panel is a native dialog-shaped `<aside>` controlled with the `hidden` attribute.

## Goals / Non-Goals

**Goals:**

- Keep one small focus state per area so pointer hover and keyboard focus can coexist without clearing each other.
- Keep the activating playground element so details can move focus in and restore it on close.
- Preserve the existing map, API, rendering, and navigation behavior.

**Non-Goals:**

- No new focus-trap dependency or modal framework.
- No changes to backend data, URLs, or visual design beyond existing preview styles.

## Decisions

- Use the existing `focus`/`blur` events and `HOVER_STYLE` for area previews. This reuses the current interaction model and avoids a second keyboard-only style.
- Focus the existing details Back button when details open. It is always visible and already has the required semantics.
- Restore focus to the activating marker only if it is still connected to the document; otherwise leave focus on the normal document flow rather than targeting stale SVG.

## Risks / Trade-offs

- [A map refresh may replace the activating marker while details are open] → Check `isConnected` before restoring focus.
- [Pointer and keyboard preview events can overlap] → Clear styling only when neither hover nor focus remains active.
