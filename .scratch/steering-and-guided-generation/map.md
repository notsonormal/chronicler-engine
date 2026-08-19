# Map: AI Steering & Guided Generation

Labels: wayfinder:map

## Destination

AI steering implemented and build-green in chronicler_engine across three features, each wired through the prompt assembler and exposed in the HTMX UI:

1. **Guided Generation** — transient, per-generation steering instructions injected into the prompt but not saved to history (typically a retry-time nudge).
2. **Narrator Action** — permanent instructions added to history from an omniscient voice, persisted and rendered distinctly.
3. **Impersonate** — forcing the AI to write the next response as a specific persona.

The way is clear when the three features are designed, implemented, tested, and `python build.py` is green — not merely decided. This map carries implementation into itself (Notes override).

## Notes

- **Domain — re-grounded against current code (the old plan's paths are all stale):**
  - Prompt assembly: `src/application/prompting/` — `assembler.rs` (`LayerRenderer` with an ordered layer set; `PromptAssembler::assemble`), `types.rs` (`PromptContext`, `PromptLayer`), `builders/sections.rs` (`build_system_prompt`, `build_post_history_prompt`, render helpers).
  - Message model: `src/domain/model/state/message_types.rs` — `MessageType` (variants `Narration`/`Dialogue`/`System`/`Input`; no `Narrator` yet), `MessageEntry`.
  - HTTP driving adapter: `src/adapters/driving/http/action/handlers/actions.rs` (`ActionForm { command }`, `action_handler`), `src/adapters/driving/http/chat_window/handlers/chat_window.rs` (`retry_handler`, `retrigger_handler`, `switch_swipe_handler`).
  - Pipeline: `src/application/pipeline/pipeline_run.rs` (prompt invocation via `PromptContext::new` + `assemble`), `src/application/pipeline/action_pipeline/` (`action.rs`, `retry.rs`, `retrigger.rs` entry paths).
  - The old plan `docs/plans/steering-and-guided-generation.md` (2026-05-08) is stale on every path but remains the source intent. Do not trust its specifics — its locked choices (`<Consideration>` recency block, new `MessageType::Narrator`, `[Narrator: …]` format) are all open and may be overturned by research.
- **Skills every session should consult:** `/grilling`, `/domain-modeling`, `/prototype`, `/research`.
- **Standing preferences:**
  - This map carries implementation into itself — implementation tickets follow the research and design tickets; do not stop at decisions.
  - Re-verify every file path against the current tree before editing; the old plan's paths are wrong and the tree has moved since 2026-05-08.
  - Keep the three features' designs coherent at the one prompt-layer integration point (`LayerRenderer` order); do not let three independent design sessions pick conflicting placements.
- **Repos under research (each ticket covers all three steering surfaces):**
  - https://github.com/notsonormal/Marinara-Engine
  - https://github.com/Samueras/GuidedGenerations-Extension
  - https://github.com/SillyTavern/SillyTavern

## Decisions so far

<!-- empty — map charted 2026-08-15 -->

- [Research AI steering in Marinara-Engine](issues/01-research-marinara-engine-steering.md) — guided-gen = final `system` message + `generationReplay` for transience; narrator = dedicated persisted `narrator` role mapped to `system` at prompt time; impersonate = user-role persona instruction that suppresses preset sections and saves as a `user` message. Full findings + verbatim quotes in `research/01-marinara-engine.md`.
- [Research AI steering in GuidedGenerations-Extension](issues/02-research-guided-generations-extension.md) — guided-gen = transient `ephemeral` `/inject` of a bracketed instruction, flushed after one generation, no replay metadata; narrator = **not present** (persistent guides are non-ephemeral depth injections in chat metadata, not history rows — an alternative design, not a mechanism); impersonate = perspective-enum prompt prepended to full history + identity, written to the input box for review, no preset suppression. Two Marinara-vs-GG divergences deferred to 04: impersonate output (review vs save-as-user-message) and preset suppression (none vs suppress). Full findings + verbatim quotes in `research/02-guided-generations-extension.md`.
- [Research AI steering in SillyTavern core](issues/03-research-sillytavern-core-steering.md) — guided-gen = native `/inject ... ephemeral=true` in `chat_metadata.script_injects`, one-shot `GENERATION_ENDED` flush, supports `position`/`depth`/`role`/`scan`, caller re-supplies per retry; narrator = real `chat[]` row via `/sys`/`/sysgen`, `extra.type='narrator'` → `role: system` with prefix suppressed (distinct from display-only `COMMENT`/`GENERIC` and prompt-time-only `/note` author's note); impersonate = trailing `system` control prompt after history, keeps full context, no persona picker (always `{{user}}`), output to input box for review, **not saved**. ST core is a third impersonate option vs Marinara (final `user` + suppress + save) and GG (final `user` + keep + review); `/note` offers a persistent-but-not-in-history narrator alternative. Full findings + verbatim quotes in `research/03-sillytavern-core.md`.
- [Design the AI steering feature synthesis](issues/04-design-steering-feature-synthesis.md) — 14 grilling questions resolved into a coherent design for all three surfaces. New `MessageType::Narrator` variant (bare rendering, no prefix); guide as final `LayerRenderer` layer after `<PlayerInput>` (Marinara recency model); per-swipe replay blob on `Swipe` for guide+impersonate retry (snapshot stays pure); separate `PresetType::Impersonate` preset replacing narrator voice apparatus, persona injected into instruction; guide/impersonate mutually exclusive per turn; slash-command parser entry for all three; narrator generate-then-add via continue; guide replaces player input (continue path); slash auto-suggestion menu. Full design in `research/04-design-synthesis.md`. Nine implementation tickets graduated (05–13).
- [Research: existing specs and requires_migration audit, then spec + integration-test authoring](issues/11-research-specs-and-integration-tests.md) — audited `docs/specs/` and `tests/http/requires_migration/`: no existing spec or pending-migration test covers narrator/guide/impersonate. Proposed `docs/specs/steering.md` and `tests/http/steering.rs` authored as a research asset (not committed as repo files from a research ticket). Full audit + proposed spec + proposed tests in `research/11-specs-and-integration-tests.md`.
- [Narrator message type and history rendering](issues/05-narrator-type-and-rendering.md) — added `MessageType::Narrator` (last variant); `render_history_layer` renders it bare with no `{sender}: ` prefix while other types keep the existing format; updated the single exhaustive match in `view_models.rs`. Storage serde-round-trips the variant unchanged; `push_message` correctly treats it as a plain append (not a swipe). Two unit tests added. `python build.py` green.
- [Replay blob on Swipe for guide and impersonate retry](issues/06-replay-blob-on-swipe.md) — added `GenerationReplay { guide, impersonate, impersonate_direction, impersonate_preset_id }` + `Swipe.replay` field; storage migration v15 (`message_swipes.replay TEXT` JSON column) with full insert/load/mapper round-trip; `push_message` inherits the blob onto retry-appended swipes so re-retry preserves steering. In-memory backend rides along by clone. Prompt-layer consumption (guide layer / impersonate preset selection off `state.narrative.retry_target.replay()`) deferred to 08/09. `python build.py` green.
- [Slash-command parser for steering entry](issues/07-slash-command-parser.md) — replaced the dead `Action::FreeAction` stub with a typed enum `{ FreeAction, Guide, Narrator, Impersonate }` + case-insensitive `/guide`/`/narrator`/`/impersonate` parser (unknown slash and plain input fall through to `FreeAction` verbatim). Wired the single `dispatch_action` to route by variant. Added three decoupled pipeline seam methods (`guide_narration`/`narrator_action`/`impersonate`) that delegate to `continue_narration` until 08/09/10 fill their bodies — each feature ticket owns one method, so they land independently. Surfaced one new open question (text-check on recognized slash commands) → ticket 14 Q7. `python build.py` green.

## Not yet specified

<!-- fog toward the destination — graduates as the frontier advances -->

- **Spec/test plan review** — ticket 14 (`issues/14-grill-spec-test-plan.md`) grills the research asset from ticket 11 and resolves six open questions (scenario ID allocation, impersonate preset activation, unknown-slash behavior, mutual-exclusivity enforcement shape, narrator message editability, impersonate output type) before the spec/tests are committed. It is a standalone later ticket, not a gate on implementation — many of its questions resolve *during* implementation. Work it after the implementation tickets unless explicitly chosen earlier.

Implementation tickets 05–13 are the live frontier; see their `Blocked by:` lines for the dependency graph. Ticket 14 no longer blocks them.

## Out of scope

<!-- work ruled beyond the destination; closed, never graduates -->

- Other SillyTavern/Marinara steering-adjacent features not named in the Destination — lorebook/world-info injection as a standalone feature, author's-note-as-permanent-lore, group chats, persona registries beyond the impersonate target. These may *inform* the three features during research but are not *delivered* by this map.
- **Composition of guide + impersonate** — guide and impersonate are mutually exclusive per turn (ticket 04, Q8=A). Composing them ("write as the player, steering toward X") is net-new design with no ported reference across ST/Marinara/GG. Deferrable to a fresh effort if the use case emerges.
- **NPC impersonation** — impersonate targets the player persona only (ticket 04, Q3=A). Writing as an arbitrary NPC is net-new (no researched repo supports it). Deferrable to a fresh effort.
