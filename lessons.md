# Lessons

- Interpret "neighborhoods" as everyday Sofia areas such as Lozenets and Mladost, not administrative districts, unless the user says otherwise.
- Show neighborhood names in English transliteration, such as "Mladost 1", so foreign visitors can read them.
- For UI changes, delegate an independent browser review when subagents are requested; report only findings at the requested severity threshold.
- Verify pointer-hover boundary highlighting explicitly; tooltip visibility alone does not prove the hovered neighborhood is visually highlighted.
- When explaining map selection, distinguish visible base-tile shapes from loaded interactive geometry; also distinguish a named quarter from the real park or facility with the same name.
- Default subagents to Luna with max reasoning; use a higher-end model only when the task clearly needs it.
- Interpret "things where kids can play" as physical play equipment such as swings and slides, with optional counts; do not reinterpret it as review criteria.
- Never suggest fake production ratings. Show an unrated state, or label hardcoded values clearly as demo data during UI prototyping.
- When an OpenSpec implementation is complete, sync its delta specifications and archive the change before committing when the user requests it.
- Do not call missing dialog focus management "lost focus" without reproducing it; distinguish focus not entering the dialog from focus left inside hidden content after close.
- For OpenSpec onboarding, prefer a small visible bug the user can reproduce before and after the fix; do not lead with an abstract quality issue.
- When a skill guardrail conflicts with the user's explicit end-to-end instruction, state the conflict and ask once; after the user confirms, treat the whole requested chain as authorized and do not re-ask per phase.
- Once the user says "go" on a multi-step chain (apply, archive, commit, PR, merge), execute every step in the same turn; a completed intermediate tool call is not a stopping point.
- For licence allowlists, tokenize and match known forms exactly; never use substring or prefix checks for policy decisions.
