# Apply the six reshape flags the comment pass offered

Type: task (AFK)
Status: resolved
Blocked by: —

## Question

The comment pass deleted six comments whose reason the code could carry, and offered a reshape for each. Apply the six reshapes, or drop each one with a reason.

## Context

- Source: the comment pass over the uncommitted dashboard comments, run 2026-10-11 with `.agents/skills/chronicler-comment-fixer/SKILL.md`. The pass left its report at `tmp/comment-pass-report.md` (transient), so each flag is restated below together with the comment it removes.
- The pass applied none of them: the skill says "Reshape flags are proposals." [Decide whether the comment pass renames or gates instead of delete-and-flag](../../retro-issue-bucket/issues/44-comment-pass-rename-or-gate.md) is the open ticket that decides whether the pass applies a flag itself. This ticket disposes of these six either way.
- The comment named in each item is the reason the item exists. Keep the shape, and the comment has to stay.
- Sizes are measured (occurrence counts), not estimated.
- This is not a re-review finding, so it is not wired into the `Blocked by:` line of [Re-review the dashboard after the phase-2 fixes](14-re-review-after-phase-2-fixes.md). **F1 alone is larger than that ticket's destination** (128 occurrences and a schema migration), and F2–F5 change no behaviour the re-review inspects.
- One ticket = one session = one commit (map Notes). F1 and F6 are each a session of their own; F2–F5 are a single small session. Split this ticket if that matters.

## Items

- **F1 — split `Swipe::steering_instruction` into two fields. LARGE.** Split into `guide_text: Option<String>` and `impersonate_direction: Option<String>`. Today one field holds either, and `impersonated` discriminates them, so the field carries this contract comment (`src/domain/model/message.rs:16`):

  ```rust
  /// Holds the guide text (plain generation) or the impersonate direction;
  /// `impersonated` is the discriminator — the two never coexist.
  ```

  Measured surface: 128 occurrences across 28 files; 10 non-test files hold ~53 of them (`domain/model/message.rs`, `domain/model/state/game_state.rs`, `action_pipeline/core.rs`, `action_pipeline/action.rs`, `narration_generation.rs`, `storage/swipes.rs`, `storage/models/swipe.rs`, `storage/mappers/message.rs`, `storage/utils/plumbing.rs`, `test_support/fixtures.rs`). `Swipe` persists twice — as the `message_swipes` column and inside snapshot JSON. `plumbing.rs:551` already carries a migration that created `steering_instruction` and back-filled it from `replay` (`$.impersonate_direction`, `$.guide`), so the split needs a second migration or a reworked one. `Message::is_guided()` and `Message::set_stored_inputs(impersonated, steering_instruction)` decide the discriminator today.

- **F2 — give `PresetCardTemplate::activate_buttons` a named struct. SMALL.** Replace `Vec<(String, String)>` with `Vec<ActivateButton { label, mode_query }>`. The tuple's second element is the mode query appended to the activate route, so the field carries this contract doc (`src/adapters/driving/http/builders/presets.rs:142`):

  ```rust
  /// Each entry pairs the button's label with the mode query appended to its
  /// activate route.
  pub activate_buttons: Vec<(String, String)>,
  ```

  Sites: the struct, its constructor, the inline Askama source (which loops `{% for (label, mode_query) in self.activate_buttons %}`), and `presets_tests.rs`.

- **F3 — return named structs from the browser-test readers. SMALL–MEDIUM.** Three readers in `tests/browser/stub/failure_display.rs` return tuples whose meaning lives in the doc comment: `read_inline_error_sizes` (five-tuple, the "Returns (send height, toggle height, send size, toggle size, message size)" doc), `read_status_state` (four-tuple, the "The returned tuple: …" doc) and `read_clamped_line` (seven-tuple). Sites: 3 definitions and 7 call sites in the one file. Test-only.

- **F4 — derive `AUTO_GROW_VIEWPORT_HEIGHT_RATIO`. SMALL.** `tests/browser/stub/layout.rs:261` mirrors the stylesheet's cap, which is the only reason for its comment:

  ```rust
  /// Mirrors the `max-height: 50vh` cap `assets/styles.css` puts on `#edit-textarea`.
  const AUTO_GROW_VIEWPORT_HEIGHT_RATIO: f64 = 0.5;
  ```

  Derive the value from the stylesheet, or assert the CSS cap in the browser check instead of restating it. Two sites, one file, test-only.

- **F5 — stop `icon_ids` returning ids with their prefix. TRIVIAL.** `tests/http/support/http_assertions.rs:112` documents the prefix as part of the return contract:

  ```rust
  /// Each returned id keeps the `i-` prefix.
  fn icon_ids(markup: &str, marker: &str) -> Vec<String> {
  ```

  Strip the prefix before returning, or rename to `icon_ids_with_prefix`. Three sites in the one file (the definition and two callers).

- **F6 — collapse the failed-redo policy into one helper. MEDIUM, behavioural.** The nine-line doc on `ActionPipeline::restore_click_time_state_after_failed_redo` (`src/application/pipeline/action_pipeline/retry.rs:166`) exists because the "which world is durable after a failed redo" decision is spread over the persist call sites. It ends with the deliberate-keep ruling:

  ```rust
  /// Between the pre-quantifier narration persist, which saves the new Swipe,
  /// and the post-engine Snapshot, a failure — an engine-commit error, or a
  /// write that fails once and then recovers — keeps the rewound world that
  /// Swipe was written from.
  ///
  /// That world is kept on purpose: the case is too rare to special-case.
  ```

  One helper that answers "which world is durable after a failed redo" lets the doc shrink to that invariant. One function, one caller (`retry.rs:163`), plus `redo_persisted_a_swipe`. This is the rollback path [Make event Retry restore the trigger context and never roll the game back](09-fix-event-retry-snapshot.md) built, so the refactor changes no behaviour and `retry_tests.rs` must stay green without edits.

## Done when

- Each of the six items is applied or dropped with a reason.
- `python build.py` is green for whatever was applied; a behavioural item (F6) names the test that pins it unchanged.
- Each item's comment is either gone because the code carries its reason, or still there and the item is dropped with a reason.

## Answer

All six items applied. Three agents worked them in parallel (F1, F2–F5, F6), so the ticket lands as one combined change with ticket 15.

| ID | Outcome |
| --- | --- |
| **F1** | **Applied.** `Swipe` has `guide_text` and `impersonate_direction`; the contract comment is gone. `is_guided()` is `guide_text().is_some()`. `impersonated` stays persisted: a bare `/impersonate` has no direction, so it is not derivable. Migration v26 (`plumbing.rs`) adds both columns, fills each from `steering_instruction` by the row's `impersonated` flag, and drops the old column; unit test `test_v26_splits_merged_steering_instruction_into_its_two_columns`. The Context claim that a Swipe persists inside snapshot JSON is wrong (inferred from code, and checked in a dev DB): `NarrativeSnapshot` has no history, so no JSON migration was needed. |
| **F2** | **Applied.** `activate_buttons: Vec<ActivateButton { label, mode_query }>`. Tier 1 `tests/http/prompt_presets.rs` already pins the mode query. |
| **F3** | **Applied.** `InlineErrorSizes`, `StatusState` and `ClampedLine` structs; the JS probes return objects. One field doc stays on `ClampedLine` (box height against needed height). Browser tier. |
| **F4** | **Applied.** The check reads the textarea's computed `max-height` and caps at it; the mirrored ratio constant is gone. Browser tier. |
| **F5** | **Applied.** `icon_ids` returns bare names; a tier-1 assertion in `tests/http/dashboard.rs` pins it. |
| **F6** | **Applied, no behaviour change.** `durable_world_after_failed_redo` returns `DurableWorldAfterFailedRedo { ClickTime, Attempt }`; the deliberate-keep ruling sits on the `Attempt` variant. F6's own commit (`28c194a5`) left `retry_tests.rs` unchanged and green; the combined change edits that file only for F1's renames and ticket 15's tests; tier 1 scenarios 11.9 and 11.10 pin the behaviour. |

**Gate.** Combined with ticket 15, after the review fixes: `logs/build_20261011_231121.log` in the main checkout, all steps OK (guardrails 163, integration 1634 passed / 2 skipped, browser 29). The code review folded `redo_persisted_a_swipe` into `durable_world_after_failed_redo`, so the decision is one function.

**New reshape flags from the after-plan comment pass (offered, not applied).** (1) Split `run_migrations` (`plumbing.rs`) into per-version step functions, so the step headers can go. (2) Named row structs for the 4-tuple fixtures in `plumbing_tests.rs`. (3) A named struct for the `(Router, AppState, u64)` that `app_with_an_event_that_meets_gabriella` (`tests/http/swipe_new.rs`) returns. (4) Rename `Message::ensure_valid_swipe_index` so the name carries its `bool`. None blocks a merge; each waits for [Decide whether the comment pass renames or gates instead of delete-and-flag](../../retro-issue-bucket/issues/44-comment-pass-rename-or-gate.md).
