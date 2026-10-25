# Report a failed trigger continuation without a story-log Message

Type: task (AFK)
Status: resolved
Blocked by: —

## Question

A failed trigger continuation adds the raw engine error to the story as a `System` Message. How should it report the failure instead, so the story, the prompt history and the Retrigger control stay clean?

## Context

- Finding **S3 (P2)** of the [2026-10-11 re-review](../re-review-2026-10-11.md), shot `tmp/p2-07/19-retrigger-failed.png`.
- `phase_trigger_continuation_llm_call` (`src/application/pipeline/pipeline_run.rs:220`) adds "[Trigger narration failed: {e}]" as `MessageType::System`, and also sets the generation error. The status line and the banner already report the failure.
- The prompt assembler sends every `System` Message to the Narrator as `System: …` (`src/application/prompting/assembler.rs:370`).
- The Message becomes the last entry, so `show_retrigger` (`NarrativeLogTemplate::new`, `src/adapters/driving/http/templates.rs`) is false and the swipe controls are absent.
- Check the other `System` Messages the pipeline adds ("[System] NPC detection uncertain — using room defaults" in the same file, and `action_pipeline/options.rs`) against the same rule, and record what you decide for them.

## Done when

- A failed trigger continuation adds no story-log Message; the status line and the banner report it; Retrigger stays available on the main narration.
- A test pins it. Name the tier.
- `python build.py` is green. Commit after user approval.

## Answer

Built, then corrected against the phase-2 review. Two changes: the failing Message goes, and the retry anchor and the retry target now read the same Message.

**The failure no longer writes story text.** `phase_trigger_continuation_llm_call` keeps `tracing::error!` and keeps setting `GenerationStatus::Error(GenerationFailure::from_engine_error(&e).with_context("Trigger narration failed"))`; only `state.add_message("[Trigger narration failed: {e}]", MessageType::System)` is gone. No new failure channel, no new `GenerationFailureKind`. The two channels the ticket named already carry it: the status line and its Details disclosure (the persisted status), and the banner, which reads the LLM Messages row the recorder wrote for the failed trigger call (`role_failure_sentence` → "Trigger failed — skipping the trigger").

**Retrigger and the swipe controls come back.** `NarrativeLogTemplate::new` sets `show_retrigger` on the last rendered Message and the template renders the swipe-controls block only for a last Narration or Input. With no notice appended, the main Narration is last, so both are offered again. `POST /retrigger`'s guard (`messages.last()` must be a non-event Narration) no longer answers 400 after a failure either — it was the same notice that blocked it.

**One decider for the anchor and the target.** `Message::is_input_or_narration` is the predicate; `find_retry_anchor_msg` computes the last such Message once and branches three ways (its own event header → the last Message without one before it; guided → itself; otherwise → the last Input), and `resolve_retry_target` finds its target with the same predicate. `MessageHistory::last_turn_is_retryable` reuses it too. A `System` notice can no longer make the two functions disagree.

That is finding 4 of ticket 09, fixed: for `[Input, Narration, event Narration, System]` the anchor is now the main Narration (index 1), so the redo reconstructs `[Input, Narration]`; before, the anchor fell back to the Input and `truncate(anchor_idx + 1)` cut the main Narration out of the run. Every other shape is unchanged.

**Tests, by tier.**

- **Unit** (`src/**/*_tests.rs`): `core_tests.rs::test_pipeline_trigger_complete_failure` — no `System` Message mentioning the failure, and the history still ends in a Narration. `retry_tests.rs::test_retry_event_trigger_narration_fails` — no `System` Message at all. New `retry_tests.rs::test_retry_target_and_anchor_agree_when_a_system_notice_trails_an_event_narration` — drives `resolve_retry_target` then `reconstruct_retry_state` on that four-Message shape and asserts `is_event`, `anchor_idx == 1`, the event Narration as the target, and a reconstructed `[Input, Narration]`. New `message_service_tests.rs` cases for the trailing-notice shape after a plain Message and after an event continuation. `message_service_tests.rs` had no trailing-`System` case before.
- **Tier 1** (HTTP E2E): `tests/http/actions.rs` 2.4 — the history holds only the turn's own Input and Narration (the count pins that the failure added no entry of its own), the last row is the Narration, and `GET /fragment/story-log` still carries `.retrigger-btn` on that Message (the last rendered `id="entry-N"` matches it). `tests/http/retrigger.rs` 14.5 — the same fragment check on the `/retrigger` path, plus no new event Narration. `tests/http/swipe_new.rs` 11.8 — the message count is unchanged by the failure, which is what its scenario says.
- No browser tier: every outcome here is a response body, a fragment, or a read-seam value, so the tier-1 rule places it there, and the htmx swap that delivers the fragment is not the point.

**The other `System` Messages: verdicts.**

- **NPC detection** (`pipeline_run.rs:173`, "[System] NPC detection uncertain — using room defaults"): **kept, unchanged**, but with the reason corrected. The phase-1 report claimed a Narration always follows it. That is wrong. The main Narration is appended in `NarrationGeneration::run` (`narration_generation.rs:116`), which runs at `core.rs:253`, before `phase_post_generation` (`core.rs:275`) adds the notice; `phase_engine_commit` adds no Narration. `load_messages` orders rows by `id ASC`. So on a successful trigger-less turn whose quantifier is empty-or-not-high, the notice **is** the last Message: it hides Retrigger and the swipe controls and the Narrator reads it as `System: …`. It is kept because it is not an error channel, its text is plain prose with no engine internals, it is the only player-visible record of a silent NPC-detection degradation (the status line and the banner report failures only, not this fallback), and removing it would need a new channel plus spec and UI work. Its control-hiding harm is recorded for ticket 13, not fixed here.
- **Options** (`options.rs` on-demand `finalize_options_failure` and turn-end `rewrite_options_after_turn`): **kept, unchanged.** The turn-end variants run last in the turn (`core.rs:363-372`), after the Narration and after any event Narration, so they too hide Retrigger and the swipe controls until the next Message, and they too reach the Narrator. `"[System] Options agent is not available"` is prose; `"[System] Options generation failed: {e}"` carries raw engine text. The on-demand path is specified in `docs/specs/options.md` and unit-tested, and has no other player-visible channel. Recorded for ticket 13.
- `"[System] Entered unknown location"` (`game_state.rs:223`) is added during the engine commit, after the Narration, and shares the same harm. Recorded for ticket 13.

**`pending_location`.** No test or spec asserts a location header in these flows (`location_header` appears only in storage fixtures, the stub server and the quarantined fragment tests, all `None`). The header the movement set now stays pending for the following Message instead of landing on the error notice. Reported as an S6-family line, not changed.

**Specs and docs.** Scenarios actions.md 2.4, retrigger.md 14.5 and swipe_new.md 11.8 now state the new outcome; the lines this ticket wrote name no Rust identifier, and 2.4 no longer over-claims. The `message_service.load_messages()` lines ticket 09 left in 14.5 and 11.8 are untouched and recorded for ticket 13. Two code comments a review found stale were reworded, and `Message::is_turn` was renamed to `is_input_or_narration` because `CONTEXT.md` deprecates "Turn".

**Not changed, on purpose.** `retrigger.rs:23` and `MessageService::switch_swipe` still decide from `messages.last()`, so a trailing `System` notice still blocks them; the moved location header stays as is. Both are follow-ups for ticket 13.

**Gate.** `python build.py` green (`logs/build_20261011_171340.log`): every step OK; architecture `1 passed, 0 failed`; guardrails `162 passed, 0 failed`; integration `1619 passed, 0 failed, 2 skipped`; browser `27 passed, 0 failed`. This ticket file was written after that gate ran; the gate does not read `.scratch`.
