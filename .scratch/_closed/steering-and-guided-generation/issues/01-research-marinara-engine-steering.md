# Research AI steering in Marinara-Engine

Type: research
Status: resolved

## Question

How does Marinara-Engine (https://github.com/notsonormal/Marinara-Engine) implement each of the three AI-steering surfaces, and what is portable to chronicler_engine? Cover all three:

1. **Guided generation** — transient, per-generation steering instructions not saved to history. Where in the prompt are they injected, and in what format/block? How are they passed for a retry vs a new generation? How is transience enforced (not persisted)?
2. **Narrator / system-style injection** — permanent instructions added to history from an omniscient voice. What message type/role is used? How are they rendered in the history sent to the model? How are they persisted and displayed in the UI?
3. **Impersonate** — forcing the AI to write as a specific persona. How is the persona target selected? How does it interact with the system prompt / output-format instructions / persona card? Does it replace or augment the narrator voice?

Invoke the `/research` skill. Fetch and read the repo (use `fetch_content` on the GitHub repo, or clone into `tmp/` if needed). Capture findings as a markdown summary at `.scratch/steering-and-guided-generation/research/01-marinara-engine.md` and link it from the resolution. Focus on mechanisms portable to chronicler_engine's `LayerRenderer` + `PromptContext` + `MessageType` model — quote the relevant code/format blocks verbatim.

## Answer

Research complete. Marinara-Engine (TS/Node monorepo, own prompt assembler — not a SillyTavern fork) was downloaded to `tmp/marinara-engine/` (clone blocked by permissions; archive used). HEAD: `bf103aa9`. Full findings with verbatim code quotes at `.scratch/steering-and-guided-generation/research/01-marinara-engine.md`.

Gist per surface:

- **Guided generation** — per-request `generationGuide` string injected as a **final `system` message** after prompt assembly (`generate.routes.ts:7080-7091`, `7153-7165`), wrapped as `"Take the following into special consideration for your next message: …"` (`generate-route-utils.ts:93-115`). Transience enforced by never saving it as a `messages` row; only durable artifact is `extra.generationReplay` on the generated assistant message, replayed on regeneration (`generation-replay.ts:95-110`). Also folded into lorebook keyword scan as a synthetic `user` turn. Retry replays the stored guide; new generation passes it directly.
- **Narrator / system-style injection** — dedicated persistent `narrator` role in the DB schema (`chats.ts:39-50`). Mapped to `role: "system"` with `contextKind: "history"` when building the LLM prompt (`generate.routes.ts:1448-1456`). UI renders a distinct amber "Narrator" bubble (`ChatMessage.tsx`). Post-history system/narrator handling: history-positioned kept in place (converted to `user` if needed), prompt-block ones merged into the latest user turn (`generate-route-utils.ts:499-523`).
- **Impersonate** — forces the model to write the next turn as the **active user persona** (not a character). Appends a `user`-role instruction built from `DEFAULT_IMPERSONATE_PROMPT` (`impersonate.ts:1-14`, `impersonate-prompt.ts:80-111`) at `generate.routes.ts:5001-5007`. Suppresses normal preset sections unless an impersonate-specific preset is selected (`assembler.ts:329-334`), skips assistant prefill, saves the response as a `user` message (`generate.routes.ts:6832-6838`). Replaces the AI voice for one turn.

Portable to chronicler_engine (detail in the file's "What is portable" section): a transient `Guide`/`Steering` layer in `LayerRenderer` fed from `PromptContext`; regeneration-replay metadata on the generated message; a `Narrator` variant on `MessageType` (or a display flag on `System`) mapped to `System` at prompt time; a post-processor for history-safe system/narrator placement; an `impersonate` request flag that suppresses preset sections, appends a persona instruction as the last user turn, and saves as a `user`-role message.

Open gaps noted in the file: unreachable `buildNarratorInstructionMessage` helper, unstripped bracket wrappers sent verbatim, unconfirmed manual narrator-creation UI path, untraced impersonate-preset output-format content.
