# Research 06: what each retry form restores and re-runs

Ticket: [Research: map what each retry form restores and re-runs](../issues/06-research-retry-tails.md).

Labels: **known** = read in the code at the time of writing. **inferred** = follows from the code, not run. **tested** = a test pins it, or a test run in this session shows it. Symbols, not line numbers.

The Snapshot names (Sc, S0, S1, S3, S5, S6, Sf, Sr) come from [research 04](research-04-generation-snapshots.md). This asset does not repeat its save sequences.

## How a Retry picks its form (known)

All Retry forms enter through `ActionPipeline::retry` (`POST /swipe/new`).

1. `before_claim`: `MessageHistory::last_turn_is_retryable`. True when *any* Input row exists in the history, or when the last Input-or-Narration Message is guided. Else 400 "No input to retry".
2. `pre_spawn`: `check_retry_anchor` loads the anchor's active-Swipe Snapshot. A missing pointer or row is a 500 and an Error.
3. In the task: `resolve_retry_target` takes the **target** = the last Message with `Message::is_input_or_narration` (System Messages are skipped). It reads `event_header`, `is_guided` and `impersonated` from the target's **active** Swipe:
   - event header → `ReNarrate`, `is_event`;
   - Narration → `ReNarrate`;
   - Input, impersonated → `ReImpersonate`;
   - Input, plain → `UserRegen`.
4. The **anchor** comes from `MessageService::find_retry_anchor_msg`. It also starts from the last `is_input_or_narration` Message, so the target and the anchor cannot disagree on event-ness. (Follow-up 4 of the phase-2 ticket 09 is fixed in the current code.)
   - target is an event → the last Message before it with no event header (a Narration, an Input **or a System Message**);
   - target is guided → the target itself;
   - otherwise → the **last Input row in the whole history** (`rposition(MessageType::Input)`).
5. `reconstruct_retry_state` loads the restored Snapshot, replaces the in-memory history with the loaded Messages cut at the anchor, and puts the target in `narrative.retry_target`. The cut is in memory only. No Message row is deleted (known: no pipeline code calls `delete_message` or `soft_delete_message`).

The redo's text lands as a new Swipe on the target through `GameState::push_message` (research 04, R4). New Messages that the redo creates (an event Narration, a System Message) go into the history after the target.

## Table: one row per form

Phases: **N** = narration LLM call (`NarrationGeneration::run`). **Q** = main Quantifier and engine commit (movement, scene, encounters, Trigger evaluation). **T** = Trigger continuation (LLM call and `commit_trigger_narration`). **R** = post-trigger reconciliation (a second Quantifier run, NPC presence only, no movement). **O** = turn-end Options rewrite.

| Form | Target | Anchor | Snapshot restored | History cut (in memory) | Phases re-run | Rows it adds | Failed redo leaves |
|---|---|---|---|---|---|---|---|
| **Main Retry** (`ReNarrate`, plain Narration) | last Input-or-Narration, a non-event, non-guided Narration | last Input row in the history | anchor's active Swipe: **Sc** of the Input's Action (pre-turn, previous `last_trigger` kept) | after the anchor Input (`truncate(anchor+1)`) | N, Q, T if a Trigger matches, R, O | Swipe on the target (S1'). A Trigger adds a **new event Message**. System Messages if any | see "Failure rollback" |
| **Guided Retry** (`ReNarrate`, guided Narration) | last Input-or-Narration with `guide_text` on its active Swipe | the target | target's active Swipe: **S1** of the guided Action (pre-turn, `last_trigger: None`) | before the target (`truncate(anchor)`) | N (empty input; `run_from_input` falls back to the stored `guide_text`), Q, T, R, O | as Main Retry | as Main Retry |
| **Event Retry** (`ReNarrate`, `is_event`) | last Input-or-Narration, carrying an event header | last non-event Message before it (can be a System Message) | **the target's** active Swipe: **S5** (post-trigger, before R). The anchor's Snapshot is only checked in `pre_spawn` | after the anchor | T (from the restored `last_trigger`; missing → `TriggerMissing`), R, O. No N, no Q | Swipe on the event Message (S5') | as Main Retry |
| **Re-impersonate** (`ReImpersonate`) | last Input-or-Narration, an impersonated Input | the target | target's active Swipe: **S1** of the impersonate Action (pre-turn) | before the target | N (impersonate preset, stored direction), **Q, T, R**. No O (`run_from_input` skips O when `impersonated`) | Swipe on the Input (S1'). A Trigger adds a new event Message | as Main Retry |
| **User regen** (`UserRegen`) | last Input-or-Narration, a plain Input | the target | target's active Swipe: **Sc** (pre-turn, previous `last_trigger` kept) | before the target | N only (system preset, the user-regen instruction as input). No Q, T, R, O | Swipe on the Input (S1') | as Main Retry |
| **Retrigger** (`POST /retrigger`) | none | none | none: `load_or_fresh` (latest) | none | T (from the latest `last_trigger`), R, O | a **new** event Message (S5') | Error on the in-flight state. No rollback; nothing was rewound |
| **Swipe switch** (`MessageService::switch_swipe`) | the chosen Swipe of the last Message (any type) | none | the chosen Swipe's pointer, copied as a new row with `Idle`, default phase, no Options | none | none | none | the active index is written **before** the Snapshot is read. A missing pointer or row returns 500 and leaves the new active index over the old world (known) |
| **On-demand Options** (`Action::Options`) | none | none | none: latest | none | O only | a System Message on failure | the prior Options set stays. Not a retry form (known) |

All rows: known from the code, except where a cell says otherwise.

### Notes on the rows

- **Main Retry, input text (known).** `retry_last_response` passes `MessageHistory::last_input_text` of the cut history: the anchor Input's active-Swipe text. An edited Input is honoured (spec 9.4).
- **Main Retry, `last_trigger` (inferred).** Sc is written in `GenerationGate::try_claim`, before `execute_action_with_inputs` clears `last_trigger`. A Retry calls `run_from_input` directly and never clears it. When the redo fires no Trigger, the previous Action's `last_trigger` survives, and `retrigger`'s guard (`last_trigger` set, last Message a non-event Narration) accepts it. Research 04 found this first. No test covers it (known: no retry or retrigger test asserts `last_trigger` after a main Retry).
- **User regen, `last_trigger` (inferred, harmless).** Sc also holds the previous `last_trigger`. The last Message stays an Input, so `retrigger` refuses until the next Action, which clears it.
- **Event Retry, input text (known).** The `ReNarrate` arm reads `last_input_text` before it branches on `is_event`. A history with no Input row fails with "Retry failed: no input to retry", although the event redo uses the text only as the Quantifier input of R. `before_claim` already refuses such a history unless the last Input-or-Narration Message is guided; an event Message is never guided. So an event that follows a guided Action in a game with no Input row cannot be retried (inferred).
- **Event Retry, location header (known).** Follow-up 1 of the phase-2 ticket 09: the redo's event Swipe has no location header. Spec 10.8 now pins "no room on the event Swipe" for both Swipes.
- **Re-impersonate, location header (known).** `GameState::arriving_message_mut` returns `retry_target`, so a movement found by the redo's Q writes the room onto the new Input Swipe.

## Failure rollback (known)

`ActionPipeline::restore_click_time_state_after_failed_redo` runs after every Retry form (not after Retrigger or swipe switch):

| Condition | Result |
|---|---|
| the current game changed during the redo (cancellation) | nothing. `handle_cancellation` already wrote `Idle` into the current game (research 04) |
| the latest status is not `Error` | nothing |
| `Error`, and the target has **no more Swipes** than at the click | **Sr**: the click-time Snapshot (Sc of the Retry) with the Error, via `save_state`. Spec I.9, scenarios 11.8, 11.9 |
| `Error`, and the redo persisted a Swipe | the attempt's world stays. For Main, Guided and Re-impersonate Retry the attempt's world is the rewound world plus whatever Q, T or R committed. Scenario 11.10 |
| Messages cannot be read | the attempt's world stays |

Per form, the last point at which the click-time world comes back:

| Form | A failure before this save restores the click-time world | After it, the attempt's world stays |
|---|---|---|
| Main, Guided, Re-impersonate | S1' (new Swipe) | engine-commit failure → `persist_generation_error` keeps the latest Snapshot (S2/S2b: rewound world, no movement). Follow-up 3 of ticket 09 |
| Event | S5' | R failure → event Swipe kept with the post-T world |
| User regen | S1' | nothing can fail after S1' except `phase_finalize` |

## Where the anchor rule breaks (known mechanism, inferred effect)

The main-Retry anchor rule takes the last Input row. It assumes that the target Narration answers that Input in the same Action, and that the Input's pointer is the Action's rewind point. Two common flows break both assumptions. No test covers either (known: no retry test seeds an empty-input Action).

1. **Retry of a continue Narration.** `Action::FreeAction("")` adds no Input row (spec 1.5). Its Narration's anchor is the Input of an *earlier* Action.
   - Known: the restored Snapshot is that earlier Input's Sc. Every change since then (movement, encounters, fired Trigger marks, an event in between) is rewound.
   - Known: the in-memory history is cut after the earlier Input, so the Messages between it and the target are absent from the redo's prompt.
   - Known: the redo's input is the earlier Input's text; the original continue had empty input.
   - Inferred: the redo answers the earlier Input again in the earlier world, and the new Swipe lands on the continue Narration. A non-repeating Trigger fired in between can fire again.
2. **Retry of a Narration that follows an impersonated Input** (impersonate, then continue). The anchor is the impersonated Input. Its pointer is S1 of the impersonate Action: before that Action's Q, T and R.
   - Known: the restored world is before the impersonate Action's movement and Trigger.
   - Inferred: the redo's Q sees the impersonated text as input and may move the player again, or may not. The impersonate Action's own tail is not re-run as such.

A plain Action with text (Input row, then its Narration) and a guided Action are correct under the rule.

## Seed §2: does re-impersonate re-run the Quantifier?

**Yes. Known and tested.**

- Code: `RetryMode::ReImpersonate` calls `retry_main_narration(state, String::new())` → `run_from_input`. `run_from_input` takes `impersonated`, `guide_text` and `impersonate_direction` from `retry_target` when the caller passes none, so the impersonate preset runs and Q, T and R follow. O is skipped.
- Decision: the closed map [guided generations architecture](../../_closed/guided-generations-architecture/map.md), ticket 01, records "Impersonate redo folds into the impersonate flow and gains the full tail (behavior change)". The code follows an owner decision; the spec was not updated.
- Test: `test_retry_flow_impersonate_mode_runs_full_tail` (`retry_tests.rs`) asserts "impersonate redo now runs the full post-narration tail" through `scene.quantifier_confidence`. It passed in this session (`python build.py test-pattern "impersonate"`: 25 passed, `logs/build_20261011_233218.log`).

**Does scenario 22.1 pass only because of its test double? Yes, and the test does not assert the claim at all. Known.**

- `test_retry_re_impersonate_generates_input_swipe_http` (`tests/http/swipe_new.rs`) builds its app with `app_with_narrator`, which uses `AgentRegistry::default()`: an empty registry, so no Quantifier agent runs in that app.
- The test asserts the message count, the Swipe count, the text, `impersonated` and Idle. It has no assertion for the room. Spec line "the player has not moved (the quantifier did not re-run)" has no test behind it. The same holds for 22.2.

## Disagreements

Each item names both sides. "Code" means the current code unless a test is named.

| # | Subject | One side | Other side | Label |
|---|---|---|---|---|
| D1 | Re-impersonate tail | `swipe_new.md` I.6 ("Re-impersonate and user-regen retries do not re-run the quantifier. They generate one swipe and stop") and 22.1 | Code, `test_retry_flow_impersonate_mode_runs_full_tail`, and the closed guided-generations map ticket 01 decision: full tail (Q, T, R), no O | known, tested |
| D2 | Event Retry and the Quantifier | `swipe_new.md` I.6 ("event retry does not [re-run the quantifier]") and 10.2 | Code runs R, a Quantifier pass for NPC presence (`reconcile_post_trigger_npcs`). Spec 10.5 depends on it. Only movement detection is skipped | known |
| D3 | A Retry adds Messages | `swipe_new.md` I.2 ("Retry never adds a new message to the message history") | Spec 9.5 expects a new event Narration after a main Retry. Code: a redo's Trigger adds an event Message; Q and O can add System Messages | known |
| D4 | Where the Swipe lands | `CONTEXT.md` Retry: "a new Swipe on the last Message" | Code: the last Input-or-Narration Message; System Messages after it are skipped and stay after it in the history | known |
| D5 | Swipes on player Input | `CONTEXT.md` Swipe: "An alternate version of an AI-generated Message"; Retry: "Redoing the last generation" | Code: user regen adds a Swipe to a player-typed Input, which no generation produced. The closed guided-generations map ticket 06 states "every Message has Swipes" | known |
| D6 | What a Swipe's pointer holds | `storage_design.md` §Per-swipe state binding: the state "its own narration produced", including the Quantifier's updates (owner ruling, seed §1) | `storage.md`: "the state captured *after* the swipe was created", which matches the code (S1, before Q). Swipe switch therefore restores the pre-Q world | known (seed §1, research 04) |
| D7 | Retry flow coverage in the Reference doc | `game_flow.md` §Retry Flow names main Retry, event Retry and Retrigger. §Phase Flow: "Normal Actions and main retries run the same success-path phases" | Code has five Retry forms. Guided Retry matches main Retry; re-impersonate skips O; user regen runs N only. The doc is silent on three forms and on the anchor rule | known |
| D8 | Main Retry rewind point | Seed §1 table: main Retry gets S0 | Code: the Input's pointer is Sc, which holds the previous `last_trigger` (research 04). Effect on Retrigger inferred | known / inferred |
| D9 | Event-Retry anchor | Seed §1 table and the phase-2 ticket 09 Answer: "the anchor Narration" | Code: the last non-event Message before the event, which can be a System Message ("NPC detection uncertain", "Entered unknown location"). It only sets the cut and the `pre_spawn` check | known |
| D10 | Continue and after-impersonate Retries | `CONTEXT.md` Retry: redoing the *last generation* | Code rewinds to an earlier Action's Input (see "Where the anchor rule breaks"). No spec scenario covers a Retry of a continue Narration | known mechanism, inferred effect |
| D11 | Trigger-failure System entry | Phase-2 ticket 09 Answer: "Messages the attempt wrote (the trigger-failure System entry) stay" | Code adds no System Message on Trigger failure; spec 11.8 asserts "gained no entry for the failure" (research 04) | known |
| D12 | Preset on a redo | Closed guided-generations map ticket 03: "Impersonate preset stays pinned at entry" | Code (`NarrationGeneration::resolve_preset_choice`) and spec 22.4: a redo resolves the game's current active preset | known; a later change, not a defect |

No disagreement found for: the guided Retry anchor and cut (`ai_steering.md`, closed map), the failure rollback (`swipe_new.md` I.9 matches `restore_click_time_state_after_failed_redo`), the Retrigger guard and `last_trigger` survival on failure (`retrigger.md` I.9).

## Open points for later tickets

- **Ticket 01 and the fog patch "Retry tails":** the anchor rule (last Input row) is the root of D10 and of the stale `last_trigger` (D8). A turn model with a rewind pointer per Action, not per Input, would remove both. Re-impersonate's tail (D1) is an owner decision; the spec needs the edit, not the code.
- **Ticket 02 (System Messages):** D4 and D9 exist because System Messages sit in the Message history between and after generations.
- **Ticket 10 (test coverage):** 22.1 and 22.2 assert no room and run with an empty agent registry. No test covers a Retry of a continue Narration, a Retry after impersonate, or `last_trigger` after a main Retry.
- **Ticket 11 (docs):** D2–D7, D11.
- **Swipe switch write order** (known): `update_active_swipe` runs before the Snapshot read, so a dangling pointer leaves a half-applied switch.
