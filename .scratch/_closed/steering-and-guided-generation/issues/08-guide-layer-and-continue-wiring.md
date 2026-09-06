# Guide layer in LayerRenderer and continue-path wiring

Type: task
Status: resolved
Assignee: pi-agent

## Question

Add the transient `Guide` layer to the prompt assembler and wire it through the continue path.

Per the design synthesis (`../research/04-design-synthesis.md`, Q2 + Q4 + Q9 + Q12):

1. Add a `Guide` layer rendered **last** in `LayerRenderer::render_and_fit` (`src/application/prompting/assembler.rs`), after `<PlayerInput>`. Wrapper text verbatim from Marinara/GG: `Take the following into special consideration for your next message: {guide}`. Guide wins recency over output-format and player input (Marinara model, Q2=A).
2. Add a transient `guide: Option<String>` field to `PromptContext` (not persisted to history — transience, Q4). The guide is never a `MessageEntry`.
3. Wire the continue path: a `/guide <text>` command (from ticket 07) sets the guide on `PromptContext` and runs `continue_narration` (not `process_action` — Q12=A: the guide replaces the player input, so the turn is a continue-with-guide). Verified continue is implemented (`actions.rs:27-28` → `action.rs:55`).
4. Surface: new generation (continue path) + retry; not retrigger (Q9=A). On retry, the replay blob (ticket 06) re-applies the guide.
5. Guide and impersonate are mutually exclusive per turn (Q8=A) — the dispatch must reject/redirect a guide when impersonate is active.

Grounding: Marinara pushes the guide after fully-assembled `finalMessages` (`generate.routes.ts:7140`); GG uses depth-0 (after last message). Chronicler's existing layer order already prioritizes recency (`<PlayerInput>` after output-format), so the guide-last placement is the consistent extension.

Blocked by: 06 (replay blob for retry), 07 (slash parser for entry).

## Answer

Guide layer added as the final `LayerRenderer` layer (after `<PlayerInput>`), wrapped verbatim with the Marinara string; threaded through the continue path via a transient `pending_replay` blob that supplies both the prompt-layer guide and the replay blob on the generated narration swipe. `python build.py` green (244s, 2 LLM tests skipped as standard — no `narrative_prompt/` or `driven/llm/` files touched).

**Prompt layer (`src/application/prompting/types.rs`, `assembler.rs`).** New `PromptLayer::Guide` variant (variant 7, after `User`). `PromptContext` gains `guide: Option<String>` (defaults `None` in `PromptContext::new`; set via `with_guide`). `LayerRenderer` carries `guide: Option<&str>`; `render_guide_layer` emits `<Guide>\nTake the following into special consideration for your next message: {guide}\n</Guide>` and is appended **last** in `render_and_fit`'s layer array, after `render_user_layer`. Empty/whitespace guide renders nothing (the layer joins via the existing `!s.is_empty()` filter). The wrapper is the Marinara verbatim (`buildGenerationGuideInstruction`, `generate.routes.ts:7140`); GG's depth-0 is the same recency position expressed differently.

**Transience + replay staging (`src/domain/model/state/narrative_state.rs`, `game_state.rs`, `message.rs`).** The guide never becomes a `MessageEntry`. A new transient `NarrativeState::pending_replay: Option<GenerationReplay>` field (`#[serde(skip)]`, mirroring `retry_target` and the existing `pending_location`/`pending_event` pattern) stages the blob for the next appended narration swipe. `push_message` consumes `pending_replay` and calls `Message::set_replay` on the freshly-constructed `Message` before appending — so the generated narration swipe records the guide (and, later, impersonate fields) on its replay blob. `Message::set_replay` is a new public setter (mirrors `set_snapshot_id`/`set_event_header`). On retry-swipe appends (the existing `retry_target` branch), `push_message` already inherits `target.replay()`; `pending_replay` is only consumed on the new-message branch, so retry-appended swipes keep the original blob and are not double-stamped.

**Continue-path wiring (`src/application/pipeline/action_pipeline/action.rs`, `core.rs`, `pipeline_run.rs`).** `guide_narration(gate, guide)` now calls a new `process_action_with_guide(gate, input, guide)` with an empty input (continue path — Q12=A: the guide replaces the player input). `process_action` delegates to `process_action_with_guide(..., None)`, preserving the existing signature and all ~30 test call sites unchanged. The spawn task calls `execute_action_with_guide(input, guide)`, which stages `pending_replay = GenerationReplay { guide: Some(g), .. }` on the freshly-loaded in-flight state before `run_from_input`. `execute_action` delegates to `execute_action_with_guide(..., None)`. `PipelineInputs` gains `guide: Option<String>`; `run_from_input` populates it from `state.narrative.pending_replay` (consuming the same blob the swipe will record). `phase_narrate` passes `inputs.guide` into `PromptContext::with_guide` via a new `PipelineRun::resolve_guide(state, inputs_guide)` helper.

**Retry re-application (`src/application/pipeline/pipeline_run.rs`).** `resolve_guide` returns `inputs.guide` when present (new guide generation); otherwise falls back to `state.narrative.retry_target.replay().guide` (retry — the blob on the retry-target swipe, reconstructed from snapshot). On retry, `pending_replay` is `None` (snapshot-reconstructed state has no transient fields), so the guide is sourced solely from the replay blob. Re-trigger is untouched (Q9=A: guide is new-generation + retry only); it goes through `run_from_input` with `inputs.guide = None` and `pending_replay = None`.

**Mutual exclusivity (Q8=A).** Not enforced in this ticket. The design holds guide and impersonate mutually exclusive per turn, but enforcement needs the impersonate-active state that ticket 09 owns. `guide_narration` stages only the guide field of `GenerationReplay`; ticket 09 will add the impersonate fields and the exclusivity guard. Logged as a deferred item for 09, not a new ticket — it is part of 09's scope ("Mutual exclusivity with guide").

**Tests.** `assembler_tests.rs`: three new — guide renders last after `<PlayerInput>` with the verbatim wrapper; no guide omits the layer; whitespace-only guide omits the layer. `game_state_tests.rs`: two new — `pending_replay` is consumed and stamped onto a new narration swipe's replay; no `pending_replay` leaves swipe replay `None`. `types_tests.rs`: `PromptLayer::Guide` variant-index assertion updated. Existing 998 unit tests pass unchanged; `process_action`/`execute_action`/`run_from_input` signatures are unchanged so the ~30 existing pipeline test call sites need no edits.

**Verification.** `cargo fmt` clean; `cargo clippy --all-targets -- -D warnings` clean; `cargo test --lib` 1003 passed (+5 new); `cargo nextest run --test architecture --test guardrails` 120 passed; `python build.py` green (all 12 steps, full integration suite, 2 LLM tests skipped — no `narrative_prompt/` or `driven/llm/` files touched, so `--llm-only` not required).

**Deferred.** (a) Mutual-exclusivity enforcement with impersonate → ticket 09 (owns the impersonate-active state). (b) Spec + integration tests for the guide layer → tickets 11/14 (the spec/test plan is a research asset, not yet committed). (c) Slash auto-suggestion UI → ticket 12.
