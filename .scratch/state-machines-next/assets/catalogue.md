# Implicit state machines of Chronicler Engine

Made on 2026-11-08 by five read-only scouts (DeepSeek Flash), one for each area. The parent session merged the five reports and checked the main claims against the code. The raw reports are next to this file (`scout-*.md`). They hold the full file:line lists, the transitions and the guards. [`scan-brief.md`](scan-brief.md) holds the definition and the output format that the scouts got.

The file:line references are from commit `8b23151c`. The code moves, so an apply ticket checks each reference again before it uses it.

**Correction (charting, 2026-11-08).** The scouts marked B1 and B2 "not modeled". The TLA+ model of `verify-gate` already has the failure line (`failLine`), the Options failure of the dock (`optFail`) and `YouEditAMessage`. Only the offered set of the dock and the failure line after a restart have no TLA+ model.

Explicitness:

- **Implicit**: flags, `Option`s, counters or DOM attributes. No enum names the states.
- **Scattered**: an enum names the states, but the transitions are in many functions or modules.
- **Explicit**: one enum and one place that makes the moves.

Risk status:

- **checked**: the parent session read the code path.
- **scout**: only a scout read it.
- **refuted**: the parent session read the code, and the claim is false.

## A. Already modeled in `.scratch/statechart-models/` or `verify-gate`

| # | Machine | Where | Explicitness |
|---|---|---|---|
| A1 | Busy registry: the holder of each Game (`Holder::Job`, `Holder::Autoplay { job: Option<JobEntry> }`) | `src/application/generation/gate.rs` | Explicit |
| A2 | The life of one Job: steps, failure line, Stop, Options run | `src/application/pipeline/job.rs` | Explicit |
| A3 | One Autoplay run: holds between Jobs, cycles, budget, end | `src/application/autoplay/driver.rs` | Explicit |
| A4 | Active-Game pointer (`Mutex<Option<GameId>>`, `PointerHold`) | `src/application/games/active.rs` | Explicit |
| A5 | The two writes of a new Swipe state (pre-analysis, analysed) | `gate.rs` `save_text`, `save_state_update` | Scattered |
| A6 | The plan of one request (`JobPlan::for_request`) | `src/domain/model/generation.rs` | Explicit |
| A7 | Part of B1 and B2: the failure line and the Options failure of each Game | `verify-gate` `failLine`, `optFail` | Scattered |

## B. Game state on the server (not modeled)

| # | Machine | State variables | Where | Explicitness | Risks |
|---|---|---|---|---|---|
| B1 | Game status: Idle, Busy, Failed (the failure line is in `verify-gate`, see A7) | memory holder + `games.failure` column; `Status` and `GenerationStatus` derive from them | `view_query.rs:198`, writers in six `gate.rs` methods | Scattered | checked: `GameWriter::edit` clears neither the failure line nor the dock, but `switch_swipe` and `delete_last` clear both. checked: busy and a failure line can be true together, in the window between `Job::end` and `Drop for JobWriter`. scout: the failure line stays after a restart; no boot step clears it. |
| B2 | Dock record: offered set + Options failure (the Options failure is in `verify-gate`, see A7) | `offered: Vec<String>`, `failure: Option<OptionsFailure>`, row present or not | `domain/model/dock.rs`, `gate.rs:879`, `transaction.rs:307` | Implicit | checked: a failed Options run keeps the old set behind the failure line. The doc comment of `save_options` says this is intended, so a Stop shows the old set again. scout: "no row" and "cleared row" are two values for one state. scout: a failed Narration after an Input leaves the Options of the scene before the Input. |
| B3 | Trigger fire-once | `SwipeState.event_record`, `Encounter.trigger_fired[idx]`, `MessageKind::EventNarration` | `game_rules.rs:70`, `generation.rs:118` | Implicit | checked: the one-shot mark goes in only when the event Swipe is written. A failed event generation leaves the Trigger unmarked, so the next Action can fire it again. No spec says whether that is intended. scout: the marks are keyed by the position of the Trigger in its list, so a World edit that reorders Triggers moves the marks. |
| B4 | Encounters: met, in the room, gone | `times_met: u32`, `currently_meeting: bool`, entry present or not | `trigger.rs:43`, three writers in `game_rules.rs` and `swipe_state.rs` | Implicit | checked: `currently_meeting` has three writers and no production reader (`is_currently_meeting` has test callers only). scout: the Scenario start sets `times_met = 1` directly, so the first-encounter rule has two homes. |
| B5 | Story end (`Tail`): awaits narration, Cycle ended, event last, retriggerable | `last`, `state_before_last: Option<…>`, `count`, `message_type`, `kind`, `event_record` | `domain/model/tail.rs` | Scattered | scout: a missing state row of the Message before the last gives `None`, the same value as a story with one Message. A Retry then starts from the wrong state and gives no error. |
| B6 | Message type × Message kind × Swipe flags | `MessageType`, `MessageKind`, `Swipe.impersonated`, `guide_text`, `event_header` | `state/message_types.rs`, `message.rs` | Implicit | scout: `Message::new` accepts any pair. `Tail::awaits_narration` reads only the type, and `JobPlan::redo` reads only the kind. `impersonated` repeats `MessageKind::ImpersonateInput`. |
| B7 | Active Swipe | `messages.active_swipe_index` + the count of `message_swipes` rows | `transaction.rs:158`, `stories.rs:189`, `stories.rs:243` | Implicit | checked: `Tx::set_active_swipe` has no bounds check (the gate checks). scout: the two readers disagree on a bad index. `ensure_valid_swipe_index` resets it to 0, and the window read fails the whole story window. |
| B8 | Swipe text against Swipe state | `Swipe.text`, `Swipe.state` | `message.rs`, `gate.rs:560` | Scattered | checked: `edit` writes the text only. The Swipe state keeps the facts of the old text, and no marker shows this. |
| B9 | Movement: map Room or added Room | `current_room_id: String`, `dynamic_rooms` | `state/movement.rs`, `game_rules.rs:104` | Implicit | checked: `Room::new_dynamic` makes its id from whole seconds, so two added Rooms in the same second get one id. scout: the consistency check runs only with the `diagnostics` feature, which is not a default feature. |
| B10 | Role health: no calls, succeeded, failed | newest `llm_messages` row for each Role; `RoleHealthState` exists only in the renderer | `view_query.rs:249`, `builders/headers.rs:55` | Scattered | checked: retention keeps the newest 50 rows of the whole table, not 50 rows for each Role. A Role that has not run in the last 50 calls reads as "no calls", and its failure leaves the banner. The comment at `view_query.rs:250` says the opposite. |
| B11 | Change counters and the server id | six counters, `revision`, `per_game`, `server` uuid | `storage/changes.rs`, `domain/model/change.rs` | Scattered | scout: after `forget_game`, a deleted Game reads as zeros, the same as a Game that never changed. Two layers must agree on the forget (storage counts, the gate forgets). |
| B12 | Game config state: narrator mode + the mode-bound preset slots (added by ticket 02, from scan 2) | `games.narrator_mode`, the four `active_*_prompt_preset_id` columns, `narrative_perspective`, `narrative_tense`; the rule "the preset allows the mode" (`PromptPreset.allowed_modes`) | `domain/model/game.rs` (`ConfigChange`), `application/games/catalogue.rs` (`switch_mode`, `set_preset_selection`, `save_config`), `transaction.rs:252-292` | Scattered (`NarratorMode` and `ConfigChange` name the states; the `allows` check is in the catalogue, outside the hold) | scan 2 inferred: `save_config` refuses only an Autoplay holder, so a plain Job runs across a config change and reads the Game row again at each step. scan 2 inferred: `set_preset_selection` checks the preset against a row read before the hold, so a `switch_mode` in between can store a preset that the new mode does not allow. |

## C. Agents and prompts

| # | Machine | State variables | Where | Explicitness | Risks |
|---|---|---|---|---|---|
| C1 | Quantifier reading: High, Medium, Low, retry, fallback | `QuantifierConfidence`, `attempt`, `last_error` | `agents/quantifier/utils/orchestration.rs:63`, `parser.rs` | Scattered | scout: `QuantifierResult` drops the confidence, so a failed Quantifier looks like "nothing changed" and leaves no failure line. The Options Agent reports `Unavailable`. checked: `MovementType` is parsed and then dropped. Only `destination` moves the Persona. checked: the call uses the literal `"quantifier"`, not `Role::Quantifier.agent_name()`. |
| C2 | Options attempt loop | `attempt`, `last_error`, `MAX_OPTIONS_ATTEMPTS = 2` | `agents/options/utils/orchestration.rs:42` | Implicit | scout: the loop retries every error, a bad key too, and it reports only the last error. |
| C3 | Prompt call mode: plain, guided, Impersonate | `PromptContext.guide: Option<String>`, `impersonate: bool` (both `pub`) | `prompting/assembler.rs:45` | Implicit | scout: the pair encodes `Generation` again. Only one setter keeps out "Impersonate with a guide". |
| C4 | History cut, then the overflow guard one time | `HistoryCut { kept_from, blocks }`, `excess_chars` | `history_cut.rs`, `assembler.rs:188`, `token_budget.rs:67` | Implicit | checked: the guard runs one time. If the fixed parts still overflow, `response_tokens` clamps to `.max(1)` and asks for a 1-token answer instead of `ContextOverflow`. |
| C5 | Agent registration for each Hook (built one time) | `after_narration`, `end_of_job: Option<Box<…>>`, `enabled` | `agents/runner.rs:20` | Implicit | checked: a non-empty config list replaces all defaults. A list with only one disabled Agent gives a runner with no Agents. A second config of the same type replaces the first, with a log warning only. This is a configuration, not a machine that changes over time. |

## D. Settings and forms

| # | Machine | State variables | Where | Explicitness | Risks |
|---|---|---|---|---|---|
| D1 | Connection slot: named, or "use the Narration Connection" | `impersonate_connection_id`, `options_connection_id: Option<String>` | `domain/model/settings.rs:458`, `:621` | Implicit | scout: "empty" and "the same id as Narration" act the same until the Narration slot changes. |
| D2 | Text check: mode + auto-check | `TextCheckMode`, `enable_auto_check` | `settings.rs:298` | Scattered (one normaliser) | scout: the pair that is not valid can be stored. The read repairs it. |
| D3 | Mode preset registry: stored bundle or computed default | `Vec<ModePresetBundle>` | `settings.rs:154`, `:580` | Scattered | scout: a mode with no stored bundle follows the code defaults when they change. |
| D4 | Preset Allowed Modes | two checkboxes → `Option<Vec<NarratorMode>>`, `is_default` | `prompt_presets/handlers/prompt_presets.rs:123`, `:173`, `:207` | Implicit | scout: both boxes clear means "both modes" on create and "no change" on update. |
| D5 | World form: create or edit, with two write routes | `is_edit`, `is_readonly`, `auto_save_key`, `form_action` | `worlds/templates/worlds.rs:300`, `worlds/handlers/worlds.rs:165`, `:236` | Implicit | scout: the posture route and the detail save each read and write the whole card, so one can lose the change of the other. |
| D6 | Connection form: add or edit; test surface | `editing: bool`, `ConnectionTestSurface` | `settings/templates/settings.rs:211`, `settings/handlers/settings.rs:281` | Scattered | scout: none of importance. |

## E. Storage and boot

| # | Machine | State variables | Where | Explicitness | Risks |
|---|---|---|---|---|---|
| E1 | Schema migration ladder, versions 0 to 34 | `PRAGMA user_version`, `due(step)` | `storage/utils/plumbing.rs:17`, `:25` | Scattered | checked: the block `due(10)` sets version 11, and no block `due(11)` exists. `git log -S` finds no commit that wrote version 10, so no database can be at 10. scout: no check that the last block is `LATEST_VERSION`. |
| E2 | Transaction (`Tx`): open, committed, dropped | live `rusqlite::Transaction`, `WriteGuard.changed` | `storage/transaction.rs:52`, `:369` | Implicit (`Drop`) | scout: the rule "no `Storage` call inside the closure" is only a comment. The backend `Mutex` is not reentrant, so a nested call deadlocks. |
| E3 | Boot seeding precedence | row present or not, `has_content` of a preset | `bootstrap/run.rs:44`, `bootstrap/load.rs` | Implicit | refuted by scan 2: `seed_worlds` (`bootstrap/load.rs:115-125`) calls `seed_world` (which upserts) only when no World row exists. Characters and Personas use `INSERT OR IGNORE`. So after the first boot the database wins for every kind, and an edit of a file in `data/` never reaches a stored World, Character or Persona. A preset row with no text is filled again (`run.rs:246`). scout: a malformed `data/settings.json` stops the boot also when the settings row exists. |
| E4 | Boot Game resolution: resume or create | `Option<GameId>` of the newest Game of the World | `bootstrap/init_game.rs`, `run.rs:97` | Implicit | refuted: "the boot touches `updated_at` two times". `resolve_boot_game` gives `None` as `initial_game`. scout: the boot continues with an id that has no row, with a warning only. |
| E5 | LLM message retention window | row count, limit 50 | `storage/llm_messages.rs:11` | Implicit | See B10. scout: the prune orders by `created_at` text, which has no tie-break. |

## F. Test machinery that is compiled into `src/`

| # | Machine | Where | Risks |
|---|---|---|---|
| F1 | Storage backend Direct or Test, with injected failures | `storage/core.rs:78`, `write_guard.rs` | scout: `with_overrides` on a storage that is already Test drops the old overrides in a release build (the guard is a `debug_assert!`). |
| F2 | `MockBackend` scripted state | `llm/providers/mock.rs` | checked: the narration list and the prompt-response list share one `call_index`. scout: `fail_first_n` counts down on narration calls only. The provider is rebuilt for each call, so the counter never reaches zero in production wiring. |
| F3 | `AutoplayPause` hold | `autoplay/pause.rs` | checked: a `release` before the loop reaches `point` makes `wait_paused` report a pause that did not block. |
| F4 | `JobTasks` watch | `generation/job_tasks.rs` | scout: a Job that panics is only logged, and `wait` does not report it. |
| F5 | Harper dictionary cache | `text_check/harper_text_checker.rs:21` | none. |

## G. Browser (dashboard)

| # | Machine | State variables | Where | Explicitness | Risks |
|---|---|---|---|---|---|
| G1 | Action area: idle, checking, generating, autoplay, preview | `checkInFlight`, `sendPending`, `#status-display[data-status]`, child of `#action-preview` → `#action-area[data-state]` | `dashboard-action-area.js:5`, `:64` (`deriveActionState`) | Scattered (one derivation, nine setters) | checked: one `sendPending` serves every sender. `endPendingAfterRequest` ends it on any answer that is not `done`, also when another send is pending. checked: `afterScreenShows(null)` resolves at once, so a `done` answer with no `X-Revision` ends the pending state early. scout: the doc does not give the priority order (checking > autoplay > generating > preview > idle). |
| G2 | Screen stamp and its waiters ("pending until the screen shows it") | `<html data-revision>`, `screenWaiters[]`, `loadsStarted` | `dashboard-screen.js:170` to `:211` | Implicit | checked: only a successful `morphParts` resolves waiters. While the engine is unreachable, "Thinking…", the edit lock of the story log, the "Saving…" of the dialog and the "Saved" flag all wait. They end when a load succeeds again. |
| G3 | Ask and load loop | `askChain`, `asksPending`, `queuedAsk`, `data-screen-busy` | `dashboard-screen.js:342` | Implicit | checked: no shipped script or CSS reads `data-screen-busy`. Only the test harness reads it. |
| G4 | What a load keeps | `requestsInFlight: Map<Node, int>`, `data-editing`, `data-keep` | `dashboard-screen.js:72` | Implicit | scout: the count is keyed by the DOM node, not by the control. A replaced node is not held. |
| G5 | The Game that a request names | `data-draft-game`, `<html data-game>`, `X-Chronicler-Game` header, `409` | `dashboard-screen.js:415`, `dashboard-action-area.js:229`, `request_layers.rs:39` | Implicit | scout: dialog forms have no `data-draft-game`, so they name the Game of the last load. The record check is the backstop. |
| G6 | Story-log edit | `editState`, entry `data-editing`, root `data-editing-entry` | `dashboard-story-log.js:17` | Implicit | scout: `showEditForm` sets `editState` before it finds the entry. A missing entry leaves `editState` set, and each later Edit is refused until a tab change. |
| G7 | Story-log scroll follow | `storyLogSwapStart`, the animation-frame retry | `dashboard-story-log.js:184` | Implicit | none of importance. |
| G8 | Edit dialog session | `dialog.open`, `data-saving`, `dialogSession`, `dialogOpenerId` | `dashboard-dialog.js:29` | Scattered | scout: only `close` clears `data-saving`. refuted by scan 2 (checked by ticket 02): "any swap into the body becomes the opener, also the Test button". The Test button of the Connection form posts into its own `next .connection-test-slot` (`settings/templates/settings.rs:199-202`), not into the dialog body. The opener `connection-test-open-{id}` (`settings.rs:60`) is a real opener. |
| G9 | Notice place: none, request report, screen report | `#notice.hidden`, `dataset.source` | `dashboard-errors.js:113` | Implicit | scout: the identity of a report is its first line, so a second failure with the same first line keeps the old details. |
| G10 | Failure banner: degraded (server) + unreachable (client) | two slots, `.unreachable`, `role` | `dashboard-errors.js:49` | Implicit | scout: each successful response clears "unreachable", so the 1 s ask can clear it while a POST still fails. |
| G11 | Saved-on-change control | live value against `defaultValue` / `defaultChecked`, `data-saved` | `dashboard-saves.js` | Implicit | scout: "Saved" has no expiry. |
| G12 | Tab, Settings sub-tab, saved-Games page | `.active`, `aria-selected`, `<html data-games-page>`, `sessionStorage` | `dashboard-panels.js`, `dashboard-screen.js:31` | Scattered | checked: the server clamps the page (`clamp_page`), but the client keeps the page number that it asked for. The page shows "Page 1 of 1" while `data-games-page` still holds the old number. |
| G13 | Slash menu | `slashMenu`, `slashMatches`, `slashActiveIndex` | `dashboard-slash-menu.js` | Implicit | scout: no effect today. |
| G14 | Error detail popover | `popover.hidden` + `aria-expanded` | `dashboard-errors.js:188` | Implicit | scout: no effect today. |
| G15 | Live-region announcements: not primed, primed | `announcedStatusLabel`, `announcedEntryIds`, `announcedOptions` | `dashboard-action-area.js:110` | Implicit | scout: after a failure line, a return to "Ready" is not announced. |

## Not counted

- The status display (`data-status`) and the action controls (`action_controls.rs`) only show the busy registry (A1). They are views, not machines of their own.
- The error-answer classification and the route-word table (`utils/answer_words.rs`) are mappings with no state that changes over time.

## Doc mismatches that the frontend scout reported (scout only)

`docs/diataxis/reference/frontend/dashboard.md`:

- It says "the end of every request that is not a read forgets the stamp **and asks**". The code asks after every request, reads too (`dashboard-screen.js:93`).
- It says "the CSS reads `data-state`". The CSS reads only `generating` (`styles.css:695`).
- It names five action-area states but not their priority.
- It lists `data-screen-busy` as page state. Only tests read it.
