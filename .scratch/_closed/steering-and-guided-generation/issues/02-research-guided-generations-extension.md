# Research AI steering in the Guided Generations Extension

Type: research
Status: resolved
Assignee: agent (GLM-5.2 session 2026-08-15)

## Question

How does the GuidedGenerations-Extension (https://github.com/Samueras/GuidedGenerations-Extension) implement each of the three AI-steering surfaces, and what is portable to chronicler_engine? Cover all three:

1. **Guided generation** — transient, per-generation steering instructions not saved to history. Where in the prompt are they injected, and in what format/block? How are they passed for a retry vs a new generation? How is transience enforced (not persisted)?
2. **Narrator / system-style injection** — permanent instructions added to history from an omniscient voice. What message type/role is used? How are they rendered in the history sent to the model? How are they persisted and displayed in the UI?
3. **Impersonate** — forcing the AI to write as a specific persona. How is the persona target selected? How does it interact with the system prompt / output-format instructions / persona card? Does it replace or augment the narrator voice?

Invoke the `/research` skill. Fetch and read the repo (use `fetch_content` on the GitHub repo, or clone into `tmp/` if needed). Capture findings as a markdown summary at `.scratch/steering-and-guided-generation/research/02-guided-generations-extension.md` and link it from the resolution. Focus on mechanisms portable to chronicler_engine's `LayerRenderer` + `PromptContext` + `MessageType` model — quote the relevant code/format blocks verbatim.

## Answer

Research complete. GuidedGenerations-Extension (v1.7.8, SillyTavern v3 client-side extension) was downloaded as a `main` archive to `tmp/guided-generations-extension/GuidedGenerations-Extension-main/` (clone blocked by permissions). Full findings with verbatim code/format quotes at `.scratch/steering-and-guided-generation/research/02-guided-generations-extension.md`.

Gist per surface:

- **Guided generation** — a transient `/inject id=instruct position=chat ephemeral=true scan=true depth=N role=R {prompt}` STScript command. Two surfaces share the inject-then-generate core: Guided Response triggers `/trigger await=true` (new generation); Guided Swipe calls `context.swipe.right()` (new swipe on last AI message). Wrapper is `[Take the following into special consideration for your next message: {{input}}]` (identical for response and swipe). Transience enforced by `ephemeral=true` + explicit `/flushinject instruct` in `finally`; the injection lives in `chatMetadata.script_injects.instruct`, never as a chat row. Configurable role (system/assistant/user, default system) and per-surface depth (default 0 = end of history). No replay metadata — caller re-supplies the guide each press (simpler than Marinara's `generationReplay`). Auto-trigger path saves/flushes/re-injects the ephemeral guide around the auto-guide cycle (`index.js:1804–1872`).

- **Narrator / system-style injection — NOT PRESENT (key negative finding).** GG has no omniscient-voice narrator message type and no permanent instruction saved into chat history. Its "Persistent Guides" (clothes/state/thinking/situational/rules/customAuto) are the closest analogue but are a **different mechanism class**: non-ephemeral depth-keyed `/inject` (no `ephemeral=true`) stored in `chatMetadata.script_injects`, wrapped `[Current Situation: {{pipe}}]`-style, updated via `move` (accumulate) or `flush` (replace), never stored as `chat[]` rows and never distinctly rendered. For the destination's Narrator Action, GG offers nothing directly portable — only an *alternative design* (history-clean depth injection vs a persisted narrator message) that ticket 04 should weigh explicitly against Marinara's `MessageType::Narrator` approach.

- **Impersonate** — forces the model to write the next turn as `{{user}}` (the active user persona, not a character). Two paths: a direct LLM call (`requestCompletion` with `includeChatHistory: true, includeIdentityContext: true`, prompt prepended as the newest `user` message) or ST's `/impersonate await=true {prompt}` slash fallback. Prompts: `Write in first/second/third Person perspective from {{user}}. {{input}}` — perspective is a single enum parameter, not three code paths. Identity context is **included**, not suppressed (contrast Marinara, which suppresses preset sections). Output is written to the input textarea for review, **not** auto-saved as a message; a toggle-restore guard undoes the last impersonate on a second press. Replaces the AI voice for one turn.

Portable to chronicler_engine (detail in the file's "What is portable" section): a transient `Guide`/`Steering` layer in `LayerRenderer` fed from `PromptContext`, removed after one generation, with the bracketed wrapper portable verbatim and retry/new-gen sharing one inject-then-generate path; a perspective-enum impersonate flag that includes history + identity, appends the perspective prompt as the final user turn, and writes to the input buffer for review. Two divergences from Marinara to decide at ticket 04: (1) impersonate output — review-before-send (GG) vs save-as-user-message (Marinara); (2) impersonate preset suppression — none (GG) vs suppress preset sections (Marinara). For Narrator Action, GG contributes only the alternative-design trade-off, not a mechanism.

Open gaps in the file: ST `/impersonate` internals untraced (ticket 03 scope); `scan=true` keyword-scan behaviour ST-internal and untraced; `position=chat` vs other positions not enumerated; direct-call impersonate preset content user-configured and uninspected.
