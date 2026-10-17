# Research 23: the target design against the code

Ticket: [Compare the target design with the code](../issues/23-compare-target-design-with-code.md).
Design: [`target-design.md`](target-design.md). Spec: [`clean-spec.md`](clean-spec.md) version 3. Rulings: [`clean-spec-rulings.md`](clean-spec-rulings.md).

Labels: **known** = read in the code in this session. **inferred** = follows from the code, not run. **tested** = a named test pins it. "Research NN" names the asset that found a fact first.

## 0. Method

- **Tree.** `9cbf3112`. `git diff 72cfffdc HEAD -- src tests docs/specs` is empty, and `git status` shows no change under `src/` or `tests/` (known). The line counts of the turn files are the counts in research 12 §1a (known). So the facts of research 04–12 hold at this tree, unless a row below says otherwise.
- **Files read again in this session:** `message_service.rs`, `generation/{gate,slot,guard}.rs`, `pipeline/pipeline_run.rs`, `pipeline/narration_generation.rs`, `action_pipeline/{core,action,retry,retrigger,options}.rs`, `domain/model/state/game_state.rs` (`push_message` to `evaluate_triggers`), `domain/model/action.rs::Action::parse`, `games/catalogue.rs::{reset, persist_initial_state_with_swipes}`, `bootstrap/init_game.rs::spawn_arrival_task_if_needed`, `bootstrap/run.rs::prepare_state`, the HTTP handlers of `/action`, `/swipe/new`, `/retrigger`, the swipe switch, `/history/*`, `/status/*`, `http/error.rs`, `utils/error.rs::generation_failure_summary`, `games/view_query.rs::role_health`.
- **The oracle gap.** A `scout` agent listed 140 code behaviours against the spec. This session checked the list, corrected two rows, and triaged all 140 (§5). The list is the appendix [`research-23-oracle-gap-list.md`](research-23-oracle-gap-list.md).
- **Review.** A read-only `reviewer` agent checked §1 to §4 and §6 against `src/` at this tree. It found 9 problems; all 9 were checked in the code and fixed (D-14, D-17, §3.2, §3.4, §3.6 and §4 rows).
- **Rows.** §2 has one row per decision D-01 to D-26 (ticket 22, "For ticket 23"). §3 splits the design's composite parts (§2.3, §3.2, §4.2–§4.7, §5, §6) into finer rows.
- **Classes** (research 15 §5, Step 4):

| Class | Rule |
|---|---|
| forced | Both agree, and the spec states the constraint. |
| shared, not forced | Both agree, and the spec is silent. The design often copied the code here, so the agreement is not evidence. |
| code defect | They differ, and the code breaks a requirement. |
| spec gap | They differ, and the design listed an assumption at that point. |
| free choice | They differ, and both meet the spec. The code's shape at this point is history. |

A row can hold two facts. Then the class names the main one, and the note names the other. "In effect" marks a row where the mechanism differs and the observable behaviour agrees.

## 1. Result in short

| Class | D-rows | Count |
|---|---|---|
| forced | D-19, D-23 | 2 |
| shared, not forced | D-12, D-20, D-26 | 3 |
| code defect | D-01, D-02, D-03, D-04, D-06, D-07, D-08, D-09, D-10, D-13, D-14, D-15, D-21, D-25 | 14 |
| spec gap | — | 0 |
| free choice | D-05, D-11, D-16, D-17, D-18, D-22, D-24 | 7 |

- **No spec gap.** The design lists four assumptions (§7: storage growth, memory-only busy, cancel, window of 4). The code does the same at each of them. The design took D-26 from the code.
- **The defects have two main roots.** 5 of the 14 defect rows come from the writer rule: one latest Snapshot with many writers, one process-wide Game id, no guard around a save, no work id (D-03, D-07 to D-10). 4 come from where a Swipe's state is: a Swipe points at a copy that its generation saved before the analysis, and nothing updates it (D-01, D-02, D-04, D-13). 2 come from notices and the dock (D-06, D-15). 3 have local roots: the stored event prompt (D-14), the `500` checks (D-21) and arrival (D-25). §6 groups them into clusters of change.
- **New facts, not in research 04–12:**
  - An event Retry and a Retrigger send the prompt that was assembled when the Trigger fired. An edit or a preset change after that does not reach the event redo (§2, D-14; inferred).
  - A swipe switch to a Swipe whose generation fired a Trigger restores no trigger context of its own: none after an Action, the stale one of the Input after a main Retry. So Retrigger is not offered for the Trigger that this Swipe fired (§3.4; inferred).
  - When a Trigger matches but its prompt cannot be built, the event is dropped with no failure (§3.3; known mechanism). After an Action no trigger context is left. After a main Retry the stale one stays, so Retrigger is offered for an older Trigger (inferred). Research 04 saw the drop. It did not tie it to R-13 and R-17.
  - `POST /action` maps every error to `500`, also a missing Game (R-45 asks `400`) and `/options` with no story (§3.3; known).
- **The oracle gap.** Of 140 code behaviours, 18 must go into the spec, 11 need an owner ruling first, and 20 belong to specs outside the lifecycle (§5). The other 91 are defects, covered, moot, internal or inside a black box.

## 2. One row per decision

| D | Design | Code (symbol) | Spec | Class | Note |
|---|---|---|---|---|---|
| D-01 | Each Swipe holds a full Swipe state. The current state is derived. No live record. | The live state is the latest Snapshot row (`Storage::load_latest_snapshot` through `MessageService::load_expecting_valid_state`). A Swipe holds a pointer `Swipe::snapshot_id`, set once by `MessageService::save_message_and_snapshot`. `switch_swipe` copies the pointed Snapshot into a new latest row; `delete_last` saves the latest state with no change (known). | R-36, R-39, R-76 | code defect | After a switch, a Narration's Swipe gives back the state from before its analysis (research 04 R2; the seed §1). After a delete, the state stays as the deleted Message left it (`delete_last`, known). A live state with correct copies would meet the spec (GLM's design), so the shape alone is a free choice; the pointer timing is the defect. |
| D-02 | The start state is the Swipe state of the Message before the last Message. The opening starts from its first Swipe. | `MessageService::find_retry_anchor_msg`: event → the Message before it, with its own Swipe pointer restored (`retry_last_response`); guided → the target itself; else the **last Input row anywhere** (known). `last_turn_is_retryable` refuses a story with no Input and no guided last Message (known). | R-40, R-43, R-44 | code defect | A Retry of a continue Narration, or of a Narration after an Impersonate, starts from an earlier Action (research 06, O-2). A Retry of the opening, or of a continue in a story with no Input, answers `400` "No input to retry" (known). For a Narration after a typed Input, for a guided Narration, an Impersonate and a user regen, the restored Snapshot is the start state in effect (inferred). |
| D-03 | Only the work that makes a Swipe writes its Swipe state, at most twice. | A Snapshot row is never changed after it is written (known: no `UPDATE` of `game_state_snapshots` in `src/`). So a Swipe's pointed state has one writer. But the live state (the latest row) has 23 save sites in research 04's table, plus `edit_history` and `delete_last` (research 04, below the table; research 05 §2). | R-73 | code defect | The Swipe part agrees in effect. The live-state part breaks R-73 in nine concurrent pairs (research 05 §3; inferred, one pair tested). |
| D-04 | The text save holds the pre-analysis state; a state update then saves the analysed state on the same Swipe. | `NarrationGeneration::run` saves the Narration at "pre-quantifier narration" (S1), and its pointer is S1 (known). Nothing moves the pointer after that (`save_message_and_snapshot` sets it only while it is `None`, known). Only the room report is written later, as a row update (`MessageService::save_arriving_location`, known). | R-11a, R-36 | code defect | The first half is **forced** (R-11a) and the code agrees: the text shows after the narrator call. The second half is missing (S-1). For an event Narration, the pointer is S5, before `reconcile_post_trigger_npcs` (known). |
| D-05 | Busy, step name and cancel handle in memory; the failure line on the Game record. | Busy: the `GenerationGate` registry, in memory (known). Step name and failure line: `NarrativeState::input_buffer` inside every Snapshot (known). `generating_status_handler` shows a failure first, else the step if busy, else idle (known). | R-10, R-21a, R-71, R-72, R-92 | free choice | The code meets R-72 and R-92 through the registry and `heal_stale_status`. The status inside each Snapshot is why `switch_swipe` must force `Idle` (F-6). `delete_last` keeps a failure line, against R-21a; that is a delete-rule defect (§3.2). |
| D-06 | A dock record per Game: the offered set and one notice field. | The offered set is `NarrativeState::current_options` inside every Snapshot. Notices are `MessageType::System` Messages, from five sources (`phase_post_generation`, `finalize_options_failure`, `rewrite_options_after_turn` twice, `GameState::attempt_movement`) (known). | R-82, R-84, R-85, R-86 | code defect | The notices reach every prompt and own the last-Message controls (research 07 §3, §2). OR-34 and OR-35 drop two sources. OR-01 makes a migration of saved `System` rows unneeded. |
| D-07 | A per-Game guard for one check and one save unit, never across an LLM call. | `GenerationGate::try_claim` holds the registry lock for the check and the insert only (known). No save runs under a guard. `edit_history` and `delete_last` check nothing; `switch_swipe` checks `is_busy` and then writes (known). | R-70, R-73 | code defect | Research 05 pairs 6–9 (inferred). |
| D-08 | Each save unit of a work checks the work id; a stop and a reset remove the entry. | A `generation_id` exists, but only `release_owned_slot` reads it (known). No save checks it. `PipelineRun::check_game_unchanged` compares the Game id at three points: after the narrator call, around the event call, and once in `execute_options_refresh` (known). | R-74a, R-81 | code defect | After a stop, the task saves on (research 05 pair 5). |
| D-09 | A request reads the active-Game pointer once; every write names that Game. | `Storage::game_id()` is one process-wide atomic, read at each storage call. No writer passes a Game id. The spawned body reads `current_game_id()` again at its start (`run_from_input`, `retry_event_continuation`, `retry_user_regen`, `execute_options_refresh`) (known). | R-74, R-90 | code defect | Research 05 pairs 2–4. `check_game_unchanged` makes the work abort, where R-74a asks that it go on for its own Game. `handle_cancellation` writes `Idle` into the new Game (known; pinned by `test_retry_user_regen_cancelled`, tested). |
| D-10 | A stop drops only the step that runs; what the work saved stays. | `reset_generating_handler` frees the slot (`release_generation_slot_for_game`) and saves the latest state with `Idle` (`reset_persisted_status`). The task runs on and saves (known). The dashboard has no Stop control (known: no file under `assets/` names the route). | R-81 | code defect | A second claim can then start a second task on the same Game (research 05 pair 5; inferred). |
| D-11 | A stop cancels the LLM call when it can; the work id check makes a late answer harmless. | No call is cancelled by a stop. `ActionPipeline::shutdown_token` is read only before a claim and at the start of the task (known). | R-09, R-81 | free choice | The design does not need cancel. The defect is the missing id check (D-08). |
| D-12 | An event generation's analysis has its movement taken out, and a Trigger it reports is ignored. | `PipelineRun::reconcile_post_trigger_npcs` uses only `npc_ids` of the analysis, and calls no `evaluate_triggers` (known). | OR-15 (free point) | shared, not forced | All three clean designs chose this too. |
| D-13 | The Swipe state that each kind leaves (§4.3 table). | See §3.4. | R-16, R-17a, R-18, R-22, R-24, R-36, R-39 | code defect | Three of the six kinds in §3.4 (Narration, Impersonate Input, event Narration) leave a state that is wrong after a switch or a delete. |
| D-14 | The event Narration's Message keeps its event record (the Trigger, its name, its direction). | The event Swipe holds `event_header` (the name), and its Snapshot S5 holds `last_trigger`, a `StoredTriggerContext` with the **prompt already assembled** when the Trigger fired (`PipelineRun::build_trigger_request`, known). An event Retry and a Retrigger send that stored prompt (`phase_trigger_continuation_llm_call`, known). | R-54, R-56, R-75, §3.1 | code defect | R-54 and R-56 hold (known: the event branch of `retry_last_response` restores S5). R-75 breaks: after an edit of the Narration that fired the Trigger, the event redo still sends the old text ("Previous narration: …") and the old history (inferred). §3.1 breaks too: the stored prompt holds the preset and response length of the moment the Trigger fired, so a later preset change does not reach the event redo, while a Narration redo reads the current preset (`NarrationGeneration::resolve_preset_choice`, known). Where the record lives is a free choice. |
| D-15 | Every Narration save empties the offered set and clears the notice; an Input save leaves the dock. | `rewrite_options_after_turn` clears the set at the end of the work, not at the text save (known). `run_from_input` skips it for an Impersonate, also when an event Narration follows (known). A failed event returns before it (known). | R-30, R-31, R-32, R-81 | code defect | OR-36 and OR-37. After a restart between the text save and the end of the work, the old set stays (inferred). |
| D-16 | The plans and the steps are behaviour; the code shape is the plan's choice. | Five ordering functions serve ten entry paths (research 12 §2). | — | free choice | The order differences that break a requirement are rows of §3.3. |
| D-17 | What each LLM call reads (§4.6). | Narrator: every Message, cut to fit (`LayerRenderer::render_history_layer`, `trim_history_to_budget`). Options: the 4 newest Messages (research 07 §3, known). Analysis: `determine_npcs_in_room` takes the 4 newest Messages of `state.narrative.history()` and the new text; it does not read `AgentContext::player_input` (known). After an Action the new Narration is already in the history (`push_message` at S1), so the window holds the new text and 3 Messages before it. After a Retry the new Swipe is on `retry_target`, outside the history, so the window holds the 4 Messages before the new text (known). | §3.2, §3.3, §3.5 (a short window) | free choice | The window differs by path; both meet "a short window". The player's text reaches the analysis only through the window: a typed Input is in it, a guide text is not (inferred). Whether that meets §3.2 is §5.3 row 3.7. Differences inside prompt assembly (it can cut a Message inside its text) are in the black box. |
| D-18 | The failure lines of §4.7. | `generation_failure_summary`: 8 kinds (known). | R-19, R-20, R-21 | free choice | The words are a free point. One line breaks R-20: see §3.6. |
| D-19 | The room report is on the Swipe, written with the Swipe state. | `Swipe::location_header`. The engine commit queues `ArrivingLocationWrite`; the next `save_message_and_snapshot` writes it as a row update (known). | R-22 to R-25a | forced | The place agrees. The write is a separate storage call, not one unit with the state (R-90), and a `save_state` between drops it (research 08 §1b). |
| D-20 | The accept unit clears the failure line. | `GenerationGate::try_claim` sets `Generating` in the claim save (known). | R-21a | shared, not forced | R-21a forces the clearing. The time (at accept) is shared, not forced. |
| D-21 | A `500` check writes the failure line and makes no work entry. | `check_retry_anchor` (the `pre_spawn`) checks the anchor's pointer, after the claim save, and frees the slot on failure (known). The event pointer, the trigger context and the Input text are checked in the task, after the `200` (`retry_last_response`, `retry_event_continuation`, known). | R-46, R-70, R-91 | code defect | For an event Retry the check reads a pointer the redo does not use (research 07 open point 1). Missing data found in the task gives `200` and a failure line, not `500`. |
| D-22 | The terms Swipe state, start state, current state, pre-analysis state. | Snapshot, anchor, `retry_target` (known). | — | free choice | Vocabulary. `CONTEXT.md` changes with the plan. |
| D-23 | The header banner reads the LLM call log. | `GameViewQuery::role_health` reads the newest row per role (known). | R-87 | forced | OR-50 is today's code. |
| D-24 | No start-up pass and no heal. | `heal_stale_status` at boot (`bootstrap::wiring`) and at a Game switch; `heal_stale_if_changed` before each claim (known). | R-92 | free choice | The heal follows from the persisted step (D-05). The heal before a claim saves a stale copy (research 05 pair 7, inferred), which is a D-07 defect. |
| D-25 | One create unit for create and reset; no LLM opening. | `persist_initial_state_with_swipes` adds the opening Narration only when the opening text is not empty (`build_fresh_initial_state`, known). `spawn_arrival_task_if_needed` runs an LLM opening at each start for a World with no opening text, with no slot and no history check (known). | R-77, R-73, OR-06 | code defect | A Game made from such a World has no Message until the next start (inferred). Arrival race: research 05 pair 1. |
| D-26 | The active-Game pointer at start is the newest Game of the configured World. | `bootstrap::init_game::resolve_game_id` (known). | outside the spec | shared, not forced | The design copied the code. |

## 3. Finer rows

### 3.1 Derived values (design §2.3)

| Value | Design rule | Code | Spec | Class |
|---|---|---|---|---|
| Current state | the active Swipe's state of the last Message | the latest Snapshot row | R-36, R-39, R-76 | code defect (D-01) |
| Start state | the state of the Message before | the anchor's pointer (§3.5) | R-43, R-55, R-58 | code defect for continue, post-Impersonate and opening Retries; forced (in effect) for the others |
| Busy | a work entry exists | a registry slot exists (`GenerationGate::is_busy`) | R-72, R-92 | forced |
| Status | busy → step; else failure line; else idle | failure → line; else busy → persisted phase; else idle (`generating_status_handler`) | R-10, R-21, R-71, R-72 | forced (in effect): the order differs, but a claim clears a failure before the work runs (known) |
| Retrigger allowed | last Message is a non-event Narration **or an Impersonate Input**, and the current state holds a trigger context | last Message is a non-event Narration, and the latest Snapshot holds `last_trigger` (`ActionPipeline::retrigger`; `NarrativeLogTemplate::new` `show_retrigger`) | R-17, R-62, R-65 | code defect (OR-27) |
| One-shot Trigger fired | the fired set of the start state | `NpcEncounterLog::is_trigger_fired` on the live state; marked in `GameState::commit_trigger_narration` (known) | R-16 | forced for the marking time (OR-05); code defect after a delete of the event Narration, because `delete_last` keeps the mark (OR-13; inferred) |
| Room shown | the room report of the active Swipe | `Message::location_header()` of the active Swipe | R-23, R-25 | forced |
| Event header | the name in the event record | `Swipe::event_header` of the active Swipe | R-13 | forced |

### 3.2 Save units (design §3.2)

| Unit | Code | Spec | Class | Note |
|---|---|---|---|---|
| Create, reset | `GameCatalogue::create_game`, `reset`, `persist_initial_state_with_swipes`: a Snapshot, then the opening Message, in separate calls (known) | R-25a, R-77, R-78, OR-43, OR-46 | code defect | Empty opening text gives no Message (R-77). Starting Characters met once and met now: `init_scenario_npcs` (known), forced. The World and the Persona stay on reset (known), forced. A reset does not check for running work, and its later saves land in the new Game (inferred). |
| Accept | `try_claim` and the claim save (Sc) (known) | R-21a | shared, not forced | The claim save is a full copy loaded before the claim (research 05 pair 6). |
| Accept refused with `500` | `check_retry_anchor` → `persist_generation_error` (known) | R-46, R-91 | code defect | D-21. |
| Input save | the Input is added in `before_claim` and saved by the claim save (known) | R-02, R-22, R-39 | code defect | The Input is saved before the room check, so R-20 (OR-19) breaks (§3.3). Its pointer Sc holds the previous trigger context and `Generating` (research 04). |
| Text save | `NarrationGeneration::run` → "pre-quantifier narration"; event: "post-trigger snapshot" (known) | R-11a, R-41, R-90 | code defect | Not one unit: up to five storage calls with no transaction (research 05 §0; known: the only transactions in `storage/` are in `worlds.rs` and the schema migration in `utils/plumbing.rs`). The dock is not changed here (D-15). |
| State update | none for the Swipe; the room report only (`save_arriving_location`) | R-11, R-11a | code defect | D-04. |
| Failure line | five writers (research 12 §2c) | R-17, R-19 to R-21 | free choice | The line lands in a Snapshot. |
| Options save | `rewrite_options_after_turn`, `execute_options_refresh` (known) | R-28, R-29, R-33, R-83, R-85 | code defect | A failure adds a `System` Message (D-06). The `/options` failure keeps the set (R-29), forced. |
| Swipe switch | `switch_swipe`: `update_active_swipe`, then the copy with `Idle` and no options (known) | R-35, R-37, R-38, R-21a | code defect | The status and the dock agree (forced: R-37, R-38). The restored world is wrong (D-01). The active index is written before the copy is read, so a missing pointer leaves half a switch (research 06; R-90, R-91). While busy, `switch_swipe` returns `ApplicationError::ConcurrentGeneration`, which `http/error.rs` maps to `503`; R-70 asks `200` "Still thinking..." (known). |
| Delete | `delete_last`: saves the latest state, then deletes the row (known) | R-76, R-21a, R-85 | code defect | Keeps the game state, the offered set (OR-25) and the failure line (OR-32). `MessageHistory::delete_last` deletes the only Message too; only the template hides the control (known). |
| Edit | `edit_history`: a fresh full Snapshot copy plus `update_swipe_text` (known) | R-75, R-70 | code defect | The text part agrees. Allowed while busy (OR-29), and the full copy can undo a running work's save (research 05 pair 8). |
| Stop | §3.7 | R-81 | code defect | D-10. |

### 3.3 The plans (design §4.1, §4.2)

| Part | Design | Code | Spec | Class |
|---|---|---|---|---|
| Order of checks | busy, then `400`, then `500` | `claim_and_spawn`: heal, `before_claim` (the `400` checks), `try_claim` (busy), `pre_spawn` (the `500` check) (known) | R-70, OR-33 | code defect |
| Missing Game | `400` | `POST /action`: `dispatch_action` maps every `Err` to `500` (known); `require_game` gives `GameNotFound` | R-45 | code defect (inferred for the route) |
| `/options` with no story | `400`-class refusal that names the reason | `EngineError::Validation` → `500` from `dispatch_action` (known). The check is "history is empty", not "no Message has text" (known) | R-27, OR-21 | free choice for the code; R-27 names no status. The empty-text case cannot occur today (no empty opening). |
| Free text | room check, Input save, Narration, event | Input save (claim), Narration with the room check inside `NarrationGeneration::run`, analysis, event (known) | R-02, R-20, OR-19 | code defect: the Input stays when the room is invalid |
| Continue, `/guide` | Narration, event, Options | same (known) | R-03, R-04, R-31 | forced |
| `/impersonate` | Impersonate, event, Options after an event | Impersonate, event, **no** Options (known: `if !impersonated`) | R-30, R-31, OR-37 | code defect |
| Options after a failed event | Options run | none: `run_from_input` returns at the failure (known) | R-31, OR-36 | code defect |
| Trigger matches, prompt cannot be built | event generation runs; on failure the line says so and the trigger context stays | `build_trigger_request` returns `None` (no room data, settings unreadable, preset missing, assembler error); no event, no failure, no trigger context (known) | R-13, R-17 | code defect |
| Offered option text starting with "/" | free text | read as a command (OR-40; `Action::parse` reads any text, known) | R-06 | code defect |
| Command parse | R-01 | `Action::parse`: trims leading spaces, lower-cases the word, `/options` with text is free text (known) | R-01, OR-42 | forced |
| Stop button | the send button becomes Stop while busy | none (known) | R-81 | code defect |

### 3.4 The Swipe state that each kind leaves (design §4.3, D-13)

"Pointer" is the Snapshot that the Swipe points at (research 04).

| Kind | Design | Code pointer | What differs | Spec | Class |
|---|---|---|---|---|---|
| Narration | the analysed state; its own trigger context; room report if it moved | S1: the start state, `last_trigger` cleared on an Action; on a main Retry the stale one from Sc (research 04) | no analysis; no trigger context even when it fired; a stale one after a main Retry | R-17a, R-36, R-39 | code defect |
| Impersonate Input | the same | S1 of the Impersonate Action | the same | R-36, R-60 | code defect |
| Event Narration | the analysed state with no movement; no trigger context; fired mark | S5: after the fired mark, with `last_trigger` set, before `reconcile_post_trigger_npcs` | no event analysis; holds the trigger context, which an event Retry reads (`retry_event_continuation`, known), so in the code it plays the part of the design's event record; Retrigger refuses while an event Narration is last (known) | R-14, R-36, R-55 | code defect for the analysis; free choice for where the trigger context lives |
| User regen | the start state | S1' of `retry_user_regen`, from the Input's Sc | in effect the same (inferred) | R-18, R-39 | forced |
| Typed Input | the current state | Sc: the latest state at the claim, with `Generating` and the previous trigger context | in effect the same, except the status (forced `Idle` by `switch_swipe`) | R-39 | forced |
| Opening | the starting state; the starting room as its report | Si (`persist_initial_state_with_swipes`; `init_game::load_game_state`) | the same, when the opening text is not empty | R-25a, R-39 | forced |

Consequence (inferred, new): a Narration fires a Trigger, its event Narration fails, and the player Retries. The redo fires nothing. A switch back to Swipe 0 restores S1. After an Action, S1 holds no trigger context, so Retrigger is not offered on the Swipe whose generation fired. If Swipe 0 itself came from a main Retry, S1' holds the stale context of the Input's Sc, so Retrigger is offered for an older Trigger. R-36 and OR-22 ask that the Swipe gives back its own context.

### 3.5 Retry by the kind of the last Message (design §4.5)

| Last Message | Design start state | Code restore | Spec | Class |
|---|---|---|---|---|
| Opening Narration | its first Swipe's state | refused: `400` "No input to retry" (`last_turn_is_retryable`, known) | R-43, R-44 | code defect |
| Narration after a typed Input | the Input's state | the Input's Sc (known) | R-40, R-51, R-52 | forced (in effect); code defect for the stale trigger context (O-1) |
| Continue, or Narration after an Impersonate | the state of the Message before | the last Input row anywhere, and the history cut after it (known) | R-40 | code defect (O-2) |
| Guided Narration | the state of the Message before | the guided Narration's own S1 (known) | R-40, R-53 | forced (in effect) |
| Event Narration | the state of the Message that fired | the event Swipe's S5 (known) | R-54 to R-58 | forced (in effect): S5 is the fired Narration's analysed state plus the fired mark (inferred) |
| Impersonate Input | the state of the Message before | the Input's S1 (known) | R-59, R-60 | forced (in effect) |
| Typed Input | the state of the Message before | the Input's Sc (known) | R-61 | forced (in effect) |
| A failed redo | the current state as the request found it | `restore_click_time_state_after_failed_redo` puts the click-time copy back when no Swipe landed (known) | R-47, R-48, R-49 | forced (in effect), by a restore step the design does not need |

### 3.6 Failure lines (design §4.7)

| Kind | Design | Code line (`generation_failure_summary`) | Spec | Class |
|---|---|---|---|---|
| No answer | "could not be reached" | the same | R-21 | shared, not forced |
| Prompt too long | "The story was too long …" | "The last turn was too long …" | R-21 | free choice |
| Unusable answer | "could not be read" | the same | R-21 | shared, not forced |
| Empty text | "gave an empty answer" | the unusable-answer line; the raw text "LLM Error: empty response" is in the disclosure (`call_narrator`, known) | R-19 | free choice: R-19 holds through the disclosure only |
| Output budget spent | the same | the same | R-21 | forced |
| Preset missing | the same | the same | R-21 | forced |
| Invalid room | "The room <name> is not on the map." | "The current scene could not be found."; raw "Room not found" (`NarrationGeneration::run`, known) | R-20 | code defect: the room is not named |
| Contradiction after the text | its own line | the save-failure or other line, raw "Failed to save engine commit: …" (`finalize_phase_error`, known) | R-20, OR-45 | free choice |
| Event narration failed | its own line | the line of the narrator failure kind. A transport error adds the context "Trigger narration failed" to the raw text, and a failed commit adds "Trigger error". An empty event answer gives `UnreadableAnswer` with the raw text "LLM Error: empty response" and no context (`phase_trigger_continuation_llm_call`, known) | R-17 | code defect for an empty event answer: neither the line nor the disclosure says that the event failed; free choice for the other cases, where the disclosure says it |
| Missing data for a Retry | names what is missing | "The last turn failed to generate."; the raw text names the id (known) | R-46 | free choice: the disclosure names it |
| Save failed | "The new text could not be saved." | "The last turn could not be saved." | R-21 | free choice |
| Internal error | its own line | "The last turn failed to generate." (kind `Other`, shared with other failures) | R-21, OR-47 | free choice |

### 3.7 Stop, restart, switch and reset (design §5)

| Event | Design | Code | Spec | Class |
|---|---|---|---|---|
| Stop, busy | the entry goes under the guard; later saves are refused; "reset" | the slot goes; `Idle` is saved; the task saves on (known) | R-81 | code defect |
| Stop, idle | clears the failure; "reset" | saves `Idle`, which clears a failure (known) | R-21a, R-81 | forced |
| Stop, no Game | "failed" | "failed" when the save fails (inferred) | R-81 | forced (inferred) |
| Restart | nothing busy; no heal | the registry is empty; `heal_stale_status` at boot; arrival runs (known) | R-92, R-93 | forced for R-92 and R-93; code defect for arrival (D-25) |
| Switch or create while busy | the work goes on for its own Game | the work's later saves land in the new Game, or it aborts at `check_game_unchanged` and writes `Idle` into the new Game (known) | R-74, R-74a | code defect |
| Reset while busy | the work's next save is refused | the Game is deleted with no check; the work's later saves land in the new Game (inferred) | R-74a | code defect |

### 3.8 Views (design §6)

| View | Design reads | Code reads | Spec | Class |
|---|---|---|---|---|
| Story log | Messages; room report; event header; Swipe count; Delete if more than one Message; Retrigger by §2.3 | the same fields from the latest load (`GameViewQuery::get_story_log_entries`); `System` entries render too (known) | §3.7, R-84 | code defect for the `System` entries (D-06) and Retrigger (§3.1) |
| Dock | the notice, else the set; off while busy | `current_options` of the latest Snapshot; off while busy (`AppState::render_options_dock`, `is_busy`) | R-85, §3.7 | code defect: no notice field |
| Status line | §2.3 | `generating_status_handler` | R-21, R-71 | §3.1 |
| Debug route | busy | the registry (research 10, scenario 39.3, tested) | R-72 | forced |
| Input box | Stop while busy | no Stop | R-81 | code defect |
| Header banner | the LLM call log | `role_health` | R-87 | forced |

## 4. The states that tickets 01–03 ask about

The ticket asks for the function that writes each such state today.

| Design state | Ticket | Written today by |
|---|---|---|
| Swipe state of a Narration or Impersonate Swipe (pre-analysis) | 01 | `NarrationGeneration::run` → `PipelineRun::persist_snapshot_or_err("pre-quantifier narration")` → `MessageService::save_message_and_snapshot` (sets `Swipe::snapshot_id`) |
| Swipe state after the analysis (the state update) | 01, 03 | no Swipe holds it. The live state gets it at "post-engine snapshot" (`ActionPipeline::run_from_input`), and later saves copy it |
| Swipe state of an event Swipe | 01 | `PipelineRun::phase_trigger_continuation_llm_call` ("post-trigger snapshot"); the reconciled scene lands in the live state only (`rewrite_options_after_turn`, `phase_finalize`) |
| Swipe state of a typed Input | 01 | `GenerationGate::try_claim` (the claim save) |
| Swipe state of a Retry Swipe | 01 | the same function as its kind; the pointer is set on the `retry_target` Swipe by `save_message_and_snapshot` |
| Opening state | 01 | `GameCatalogue::persist_initial_state_with_swipes`; `bootstrap::init_game::load_game_state` |
| Start state of a Retry | 01 | read, not written: `MessageService::find_retry_anchor_msg`, `ActionPipeline::retry_last_response`, `reconstruct_retry_state` |
| Current state | 01 | the newest of the Snapshot writers: the 23 save sites of research 04's table (among them `switch_swipe`, `reset_persisted_status`, `heal_stale_if_changed`), plus `edit_history` and `delete_last` |
| Busy | 01 | `GenerationGate::try_claim`; freed by `GenerationGuard::drop`, `release_generation_slot`, `release_generation_slot_for_game` |
| Step name | 01 | 33 hand-written lines in 7 files (research 12 §1c), saved in each Snapshot |
| Failure line | 01, 02 | `finalize_phase_error`, `persist_generation_error`, `persist_snapshot_or_err`, `restore_click_time_state_after_failed_redo` |
| Offered set | 02 | `rewrite_options_after_turn`, `execute_options_refresh`; cleared by `switch_swipe` |
| Notice | 02 | `GameState::add_message(…, MessageType::System)` at five sites (§2, D-06) |
| Room report | 03 | at row birth: `Storage::insert_swipe` from `pending_location` (the opening only); after a move: `GameState::log_movement_completion` → `MessageService::save_arriving_location` → `Storage::update_swipe_location_header` |
| Event record | 01 | `PipelineRun::build_trigger_request` (the context) and `GameState::commit_trigger_narration` (`last_trigger`, `pending_event` → `Swipe::event_header`) |

## 5. The oracle gap

The question: which behaviours does the code have that the spec does not state? A rewrite tests against the spec, so it can lose each of them without a red test. The row ids are those of the appendix [`research-23-oracle-gap-list.md`](research-23-oracle-gap-list.md). The scout's labels are absent 43, loose 61, black box 5, contradicts 31.

### 5.1 Triage of the 140 rows

| Disposition | Meaning | Rows | Count |
|---|---|---|---|
| Code defect | The code breaks a requirement. §2 and §3 have it. | 1.1, 1.6, 1.8, 2.4, 2.6, 2.7, 3.6, 4.4, 4.12, 5.7, 6.2, 6.4, 7.1, 7.2, 8.1, 8.4, 8.5, 8.8, 9.1, 9.3, 9.7, 9.13, 10.3, 10.11, 10.15, 10.20, 11.1, 11.4, 12.1, 12.2 | 30 |
| Out of this map | The story-log cut of 50, ruled out of scope on the map. A spec-driven rewrite drops it, because §3.7 shows each Message. | 10.8 | 1 |
| **Add to the spec** | Player-observable, inside the lifecycle or its black-box contracts, and not stated. | 1.4, 2.1, 2.2, 2.3, 2.8, 2.11, 4.2, 8.2, 9.8, 9.9, 9.11, 9.12, 10.16, 10.17, 10.18, 10.19, 10.21, 10.23 | 18 |
| **Decide first** | The code differs from a black-box contract, or may be a defect. The owner rules keep or drop, then the spec states it. | 2.5, 2.10, 3.5, 3.7, 4.3, 4.5, 4.6, 5.1, 10.12, 10.26, 11.6 | 11 |
| **Outside the lifecycle** | Belongs to a spec that does not exist yet: the Agents, the world rules, the Games and presets, the LLM log view (platform-choice map, fog "The design beyond scope B"). | 3.1, 3.2, 3.3, 3.4, 3.8, 4.1, 4.7, 4.8, 4.9, 5.2, 5.3, 5.6, 8.3, 8.9, 9.5, 9.6, 9.14, 10.13, 10.25, 12.5 | 20 |
| Covered | The spec or the design states it. | 1.2, 1.5, 1.9, 1.10, 2.12, 2.13, 2.15, 4.10, 5.4, 5.10, 5.11, 5.12, 6.1, 6.3, 6.5, 6.6, 6.7, 6.8, 6.9, 6.10, 7.3, 7.4, 7.5, 8.6, 8.7, 9.10, 10.1, 10.2, 10.4, 10.9, 10.22, 10.24, 11.2, 11.8, 11.9 | 35 |
| Moot | The design has no such part (heal, Snapshot copies, `System` rows, arrival). | 1.3, 5.8, 8.10, 10.6, 10.7, 10.10, 11.3, 12.3, 12.4 | 9 |
| Drop | Internal, debug, dead, or storage robustness that E-08 makes moot. | 1.7, 3.10, 4.11, 5.5, 9.2, 9.4, 10.5, 11.5, 11.10, 11.11 | 10 |
| Black box | Inside a contract or a free point. Not a lifecycle gap. | 2.9, 2.14, 3.9, 5.9, 10.14, 11.7 | 6 |

A script checked that every id of the appendix is in exactly one row of this table (tested in this session).

### 5.2 Add to the spec (18 rows, 12 items)

| Item | Rows | What the code does | Where it goes |
|---|---|---|---|
| Output cleanup | 2.1, 2.2, 2.3 | Every LLM answer, of every role, loses its reasoning blocks and leading spaces and is trimmed, before the empty check and the parse (`LlmCallRecorder::complete` → `sanitize_llm_output`, known). An answer that holds only reasoning counts as empty. | §3.1 to §3.3 results |
| Response length | 2.8 | The narrator call also reads the response-length setting (`PipelineRun::load_preset_and_response_length`, known). | §3.1 inputs |
| Command trim | 2.11 | `/action` trims the whole command; a command of spaces only is a continue (`action_handler`, known). | R-01 |
| Added-room name | 4.2 | An added room takes the destination text as its name, which the room report shows (`Room::new_dynamic`, known). | §3.4 |
| Option count | 8.2 | At most 3 options (`DEFAULT_OPTION_COUNT`, known). | §3.3 |
| Opening text template | 9.8 | The opening text is rendered with the Persona's name for `{{user}}` (`build_fresh_initial_state`, known). | R-77 |
| Failed create | 9.9 | A create whose first save fails keeps the old active Game (`GameCatalogue::create_game`, known). | R-78 |
| Switch to a missing Game | 9.11 | `400` "Game not found" (known). | R-79 |
| Shutdown | 1.4 | During a planned stop, a request that starts work answers `503` (`claim_and_spawn`, known). | E-02 or §2 |
| Answer bodies | 9.12, 10.18, 10.19, 10.23 | "Thinking...", "Retrying...", "Retriggering...", "Edited", and the `HX-Refresh` header; `dashboard-action-area.js` keys on the `.status.wait` class (known). | §2 |
| Action routes | 10.16, 10.17 | The dashboard's input box (`assets/index.html`) and the dock's regenerate form post to `/action/check` (text check, skipped for commands), and the preview posts to `/action/confirm`. No dashboard form posts to `/action` (known). | §2 |
| Swipe index out of range | 10.21 | `500`, after the bad index is written (known). The spec needs a rule: `400` with no change. | §2, R-35 |

### 5.3 Decide first (11 rows)

| Rows | Code | Against |
|---|---|---|
| 2.5 | Player text loses each `{{…}}` block, which becomes `[FILTERED]` (`sanitize_for_prompt` in `LayerRenderer`, known). | §3.5: prompt assembly "removes nothing" |
| 2.10 | A bare `/guide` makes a guided Narration with an empty guide (`Action::parse`, known). | R-01, R-04 |
| 3.5 | A low-confidence analysis keeps the Characters of the scene and **keeps its movement** (`static_npc_result`, known; whether a low answer can hold a movement: inferred). | §3.2: a fallback has no movement |
| 3.7 | The analysis reads the new text and the 4-Message window, but not the player's text as its own input (`determine_npcs_in_room`, known). | §3.2 inputs |
| 4.3 | Two added rooms in the same second get the same id, and the second replaces the first (inferred). | §3.4 |
| 4.5 | The analysis does not see added rooms by name, so a return to one by name adds another room (known). | §3.4 |
| 4.6 | Trigger conditions read the encounter counts from before this generation's encounters (known; a test pins the order). | §3.4: "conditions hold in the new game state" |
| 5.1 | When several Triggers match, `HashMap` order picks one (`evaluate_triggers`, known). | §3.4: "at most one"; which one is open |
| 10.12 | The banner reads the newest 50 LLM call rows of all Games, so an old failed role line can drop out with no success (research 09). | R-87 |
| 10.26 | A reset that fails answers `500` with the message text (known). | R-77 |
| 11.6 | The history in memory keeps at most 1000 Messages (`MAX_MESSAGES`, known). Prompts and the story log read that history. | none: the spec has no story limit |

### 5.4 What the comparison adds to the oracle

- The **current feature specs** (`docs/specs/*.md`) are more detailed than the clean spec in places, and today's tests are tagged to them (appendix §0, caveat 1). A rewrite oracle needs both: the clean spec for the lifecycle, and the feature-spec scenarios that the clean spec does not replace.
- The 20 "outside" rows show where the clean spec stops. Four black boxes (the Agents, the world rules, prompt assembly, the LLM transport) have no spec, as the platform-choice map records.

## 6. Input to the fix-or-rewrite decision

This section gives facts, not the decision. Each cluster is a group of defect rows with one root. "Change in today's shape" says what a targeted fix touches. Research 12 gives the module map.

| Cluster | Defect rows | Spec | Change in today's shape | Depends on |
|---|---|---|---|---|
| A. Per-Swipe state | D-01, D-02, D-04, D-13, §3.2 delete, §3.4, §3.5 | R-36, R-39, R-40, R-43, R-44, R-76, R-11a | A second write onto each Swipe after its analysis (a result pointer, or a state update), plus switch, delete and Retry that read it. Research 12 §3b: the pointer timing is set by phase order in five ordering functions, so each of them changes. The 2026-10-04 decision against a post-turn pointer (ticket 43, research 12 §4d) must be reversed. | — |
| B. Status out of the Snapshot | D-05 consequences, D-24 | R-21a, R-72 | Move the step name and the failure line out of every Snapshot: 33 hand-written status lines in 7 files (research 12 §1c), the heal, and the forced `Idle` in `switch_swipe`. | — (helps A: a Swipe state then holds no status) |
| C. Notices to a dock record | D-06, D-15, §3.8 | R-82 to R-86, R-31, R-32 | A notice field next to the offered set; five sources; four `match` sites; two templates and CSS rules; tests that assert `System` rows (research 09 §4). No migration (OR-01). | — |
| D. Writer rule | D-03, D-07, D-08, D-09, D-10, §3.7 | R-70, R-73, R-74, R-74a, R-81 | A Game id on every storage call, in place of the process-wide `Storage::game_id()`; a guard and a work id check at 14 save sites that can set a pointer and 10 Snapshot writers that cannot, 3 of them the player edits (research 12 §3a); a stop that ends the work. | — |
| E. One save unit | §3.2 text save, D-19 | R-90 | A storage transaction around `save_message_and_snapshot` (5 calls). | D (the unit must also name its Game) |
| F. Opening and arrival | D-25 | R-73, R-77, R-43 | Delete `arrival_service.rs` and the boot spawn; an empty opening Narration at create; Retry of the opening. | A (the opening Retry needs the start state rule) |
| G. Local rules | §3.3 order of checks, missing Game, Input before room check, Options after an Impersonate event and a failed event, dropped Trigger, option text, Retrigger on an Impersonate Input, swipe switch `503`, edit and delete while busy, delete of the only Message, delete keeps the set and the line, D-14 stored prompt, D-21 `500` checks, invalid-room line, empty event answer line | R-06, R-13, R-17, R-20, R-27, R-45, R-46, R-62, R-70, R-75, R-76, R-85 | Each is one function or a few. Some need D first to be safe (edit and delete under a guard). | partly D |

Facts that bear on the choice (from this asset and research 10 and 12):

- Clusters A and D touch every ordering function and most save sites. Clusters B, C and E touch many files but one concern each. Cluster G is local.
- No test reads back what a Swipe pointer restores (research 10 §4). So cluster A has no existing red test. Research 10 says what a new test must assert.
- The tests that seed the save order by hand (`seed_main_narration_flow`, `seed_event_flow`, the `retry_tests.rs` helpers) assume today's pointer timing (research 12 §5b). Cluster A changes that timing, so these fixtures change with it.
- The design leaves the code shape open (D-16). It names no module.

## 7. Spec observations

1. **R-76: the opening cannot be deleted** (design §8). The code agrees in effect only because the template hides the control: the route deletes the only Message (§3.2). Not a spec change.
2. **R-19 and R-46 can hold through the disclosure.** The code keeps the words "empty response" and the missing id in the raw text only. The spec does not say whether "the failure says" means the short line. The design chose the short line. A plan that keeps the code's lines should state that the disclosure is enough.
3. **R-27 names no answer code.** The code answers `500` for `/options` with no story. The design says `400` in §4.1 step 3. The spec says only "fails at once".
4. **Which Trigger fires when several match.** See §5.

## 8. Tickets 01–03

| Ticket | What the design and this comparison give it | Recommendation |
|---|---|---|
| [Model a generation's states, its Snapshot pointers and the game-state writers](../issues/01-model-a-generations-states.md) | The design is the model (T-01 settled; D-01 to D-13). §4 cites the function that writes each state today, which is the ticket's "done when". The pointer questions (rewind and result pointer) are implementation by the owner's ruling (ticket 17), and the design has no pointer. | Close. Its fog patches go to the fix-or-rewrite decision (Answer). |
| [Replace `System` Messages with an error channel outside the Message history](../issues/02-replace-system-messages.md) | The dock record (D-06), R-82 to R-86, OR-34 and OR-35 decide the channel and drop two sources. OR-01 removes the migration question. The ticket's open points are all answered. | Close. The work is cluster C of §6. |
| [Decide where the room a generation arrives in lives](../issues/03-decide-where-the-room-lives.md) | The room report is on the Swipe with the Swipe state (D-19), and a switch gives back the room of the active Swipe (T-03 settled). That is the ticket's option A without pointers. Its doc follow-ups (`storage_design.md` §Per-swipe state binding, `game_flow.md` §Movement Branch) go to the plan. | Close. The work is cluster A of §6. |
