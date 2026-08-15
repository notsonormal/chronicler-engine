# Research AI steering in SillyTavern core

Type: research
Status: resolved

## Question

How does SillyTavern core (https://github.com/SillyTavern/SillyTavern) implement each of the three AI-steering surfaces, and what is portable to chronicler_engine? Cover all three:

1. **Guided generation** — transient, per-generation steering instructions not saved to history. (Note: the GuidedGenerations-Extension is a SillyTavern extension — establish how core supports or exposes the hook the extension uses, and whether core has its own equivalent.) Where in the prompt are they injected, and in what format/block? How are they passed for a retry (swipe/regenerate) vs a new generation? How is transience enforced (not persisted)?
2. **Narrator / system-style injection** — permanent instructions added to history from an omniscient voice. What message type/role is used? How are they rendered in the history sent to the model? How are they persisted and displayed in the UI?
3. **Impersonate** — forcing the AI to write as a specific persona. How is the persona target selected? How does it interact with the system prompt / output-format instructions / persona card? Does it replace or augment the narrator voice?

Invoke the `/research` skill. Fetch and read the repo (use `fetch_content` on the GitHub repo, or clone into `tmp/` if needed). Capture findings as a markdown summary at `.scratch/steering-and-guided-generation/research/03-sillytavern-core.md` and link it from the resolution. Focus on mechanisms portable to chronicler_engine's `LayerRenderer` + `PromptContext` + `MessageType` model — quote the relevant code/format blocks verbatim.

## Answer

SillyTavern core traced at `tmp/sillytavern-core/` (HEAD `8172dcd0ee672d3cd9a5e5f7af134f91a45cd2b8`). Full findings + verbatim code quotes in `research/03-sillytavern-core.md`.

- **Guided generation** — ST core's native transient mechanism is `/inject ... ephemeral=true`. Injects live in `chat_metadata.script_injects` (not `chat[]`) and register an in-memory extension prompt. `ephemeral=true` registers a one-shot `GENERATION_ENDED`/`GENERATION_STOPPED` listener that deletes the metadata entry after one generation, enforcing transience. `position` (`before`/`after`/`chat`/`none`), `depth` (default 4; 0 = end of chat), `role` (system/user/assistant), and `scan` are all native parameters. The caller re-supplies the guide on each retry (no replay metadata). ST core has no *other* transient guided-gen mechanism; `/note` (author's note) and `/comment` are persistent/display-only alternatives.
- **Narrator** — real `chat[]` row created via `/sys` (`/nar`) or `/sysgen` (`sendNarratorMessage`), tagged `extra.type === system_message_types.NARRATOR`, displayed with the system avatar. OpenAI backend maps it to `role: 'system'` with the speaker prefix suppressed (`openai.js:580`); text-completion `formatMessageHistoryItem` omits the prefix too. `/comment` (`COMMENT`) and `GENERIC` are `is_system: true` rows filtered *out* of the prompt — display only. ST thus distinguishes three classes: NARRATOR (history row → system), COMMENT/GENERIC (history row → not sent), and extension prompts (not a row, prompt-time only).
- **Impersonate** — `/impersonate` → `Generate('impersonate')`. No persona picker: target is always the active user persona via `{{user}}` macro and `default_impersonation_prompt`. The instruction is appended as a trailing `system` control prompt *after* chat history (not a user message, not prepended). ST keeps full character/world/preset context (does NOT suppress it); only the group nudge is skipped and `force_name2` is forced false. The streamed result is written to the input textarea for review and `IMPERSONATE_READY` is emitted — it is NOT saved as a chat message.

**Key divergences surfaced for ticket 04 (all three repos now in hand):**

| Topic | Marinara | GG-Extension | ST core |
|---|---|---|---|
| Transient guide | final in-memory `system` message | `ephemeral=true` `/inject` in metadata, flushed | same as GG, native to core; + `before`/`after`/`chat` positions & role |
| Narrator | real `narrator` row → `system` | **none** | real chat row, `extra.type='narrator'` → `system`, prefix suppressed |
| Impersonate instruction | final `user` message | final `user` message (direct) / delegates to ST core | trailing `system` control prompt |
| Impersonate context | suppresses preset/char sections (unless impersonate preset) | keeps full context | keeps full context |
| Impersonate output | saved as `user` message | input box for review | input box for review (no save) |

ST core lands on a third impersonate option (trailing `system` + keep context + review-not-save) distinct from both Marinara (final `user` + suppress + save) and GG (final `user` + keep context + review). Ticket 04 must pick one and justify it. ST core also confirms the GG `ephemeral=true` mechanism is native to core (not extension-only), and offers the depth-keyed author's-note (`/note`) as a persistent-but-not-in-history alternative to a narrator row — a real design fork for the Narrator Action surface.
