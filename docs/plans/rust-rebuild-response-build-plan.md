# Build plan: the response to the reviews of the Rust rebuild

## How to use this plan

This plan builds the response to the reviews of the whole Rust rebuild, on `wf/rebuild-response`, and then merges that branch to `main`. The rebuild itself (`wf/rebuild-core`) merged to `main` before the response, on 2026-10-20 (`22a8f090`; owner, which changes ticket 26 Q6). It has nine slices (R1 to R9) and the merge (§13). The slices run in order.

**The implementer of one slice reads §1, §2 and the section of that slice.** The reviewer reads the same sections.

Each item cites the ticket that decides it. A doubt about an item goes to that ticket, not to a new choice. A gap that needs a decision stops the slice and goes to the owner as a new ticket (map Notes).

Facts with no label are known: read in a ticket, or in the code at commit `5d3cfe5f`. **Inferred** and **guessed** facts are labelled; check them before you rely on them. Line numbers are of `5d3cfe5f`.

Source: [Write the build plan of the response to the reviews](../../.scratch/rust-rebuild-build/issues/27-write-the-build-plan-of-the-response.md), in the map [Build the Rust rebuild](../../.scratch/rust-rebuild-build/map.md). Ticket numbers below are tickets of that map.

## 1. The design sources

In the order of their authority:

1. The tickets of the response. Where two tickets conflict, the later ruling wins.
   - [19, the shape of a generation](../../.scratch/rust-rebuild-build/issues/19-shape-of-a-generation.md), as amended by ticket 20 Q13 and by ticket 25. Read its interface text with its two amendment notes.
   - [20, the shape of the Job](../../.scratch/rust-rebuild-build/issues/20-shape-of-the-job.md).
   - [21, the storage seam](../../.scratch/rust-rebuild-build/issues/21-storage-seam.md).
   - [23, the story log past 50 Messages](../../.scratch/rust-rebuild-build/issues/23-story-log-past-50-messages.md).
   - [25, Messages and rooms](../../.scratch/rust-rebuild-build/issues/25-messages-rooms-and-mapless-worlds.md).
   - [26, the small items and the merge branch](../../.scratch/rust-rebuild-build/issues/26-triage-the-small-items-and-the-merge-branch.md).
2. [Target design](../../.scratch/turn-state-model/assets/target-design.md) version 2. The slices edit it in place, and its label stays "version 2" (owner, 2026-10-20). There is no `target-design-v2.md` copy.
3. [The build plan of the rebuild](rust-rebuild-build-plan.md): its §2 rules and its guardrail rows, as §2 below adapts them.
4. The review inputs: [FINDINGS.md](../../.scratch/rust-rebuild-build/assets/whole-rebuild-review/FINDINGS.md) (findings by number), [synthesis.md](../../.scratch/rust-rebuild-build/assets/architecture-review/synthesis.md) (candidates M1 to M6, S1 to S5), [the UI findings](../../.scratch/rust-rebuild-build/assets/ui-review/findings.md) (F1 to F5).

The order of the slices follows synthesis §6, with these changes:

| Synthesis §6 | This plan | Reason |
|---|---|---|
| M2 storage is 7th, after the owner call | R1, first | Ticket 21 made the call. The later slices then change one backend, not two, and R7 needs the `StorageCall` key (C-4) |
| S3 story window is 5th | R2 | Synthesis §6: it does not depend on 1 to 4. R3 shows the room on the oldest Message of the window, so the window comes first |
| — | R3, rooms | Ticket 25 deletes the stored room report. The Game rules of R4 then have no `room_report` to keep |
| M4, M1 | R4 | One slice: M4 fixes the input type of M1 (synthesis §6) |
| S1 with finding 3 | R5 | As synthesis §6 |
| S2 preset module is 6th | R6, before the Job | The Job slices read the preset through the new module, so the reads change once |
| M3 with S5 | R7, R8 | Ticket 20 replaces them: the failure lines and the loop (R7), then the step rule and the narrator (R8) |
| M5 | R1 | Ticket 21 Q3 puts the one story entry in the storage seam |
| M6, S4 | — | Closed: ticket 26 Q2 (M6), ticket 22 (S4) |

## 2. Rules for every slice

The rules of [build plan §2](rust-rebuild-build-plan.md#2-rules-for-every-slice) hold: the one rule, the tests in a slice, the guardrails, the docs, the dev loop, the commit gate, and Done. These changes apply:

- **One branch.** Every slice commits on `wf/rebuild-response`, made from `main` at `22a8f090` (owner, 2026-10-20). No slice makes its own branch, and no slice merges to `main`. §13 holds the one merge.
- **One implementer at a time.** The slices run in order. No other work runs on `wf/rebuild-response` during the build.
- **Standing approval** (map Notes, owner 2026-10-19): the owner approves the commits of the slices in advance. The lead commits after a green commit gate.
- **The commit gate.** Run `/simple-after-plan` with the scope of the one commit, with build plan §2's two adaptations (an empty-input step is skipped). Every commit runs the full gate (`python build.py`), and all tests pass. The retro runs once per slice, at the last commit of the slice. No commit of this plan is red on purpose.
- **CASE ids.** A test in a seam-contract file (the Agents, the `Narrator`, `tests/storage/`) carries its id as its `// CASE <id>: <description>` tag (ticket 08 on the map). A test in any other file takes the id as its function name, in snake case (inferred: ticket 19 and ticket 20 give ids to plain unit tests and tier-1 tests too).
- **The one rule, for the new unit tests.** A unit test of new code lives in a seam-contract file or under `domain/model/` (build plan §2). `job.rs` and `gate.rs` have no unit tests; tier 1 covers them (build plan §9.4). §12, gap G2, places the two `job_tests.rs` cases of ticket 20.
- **Scenarios with their tests** (amends build plan §2, "before their tests"). A slice that changes a behaviour that a player can see writes its scenario in `docs/specs/` in the same commit as its test: `validate_feature_spec.py` fails the gate on a scenario with no test (ticket 23 Q3).
- **Doc searches.** Done item 4 of build plan §2 holds: a search of `docs/diataxis/` for each name that a slice removes finds nothing.

## 3. R1: the storage seam

**Content** (ticket 21; ticket 20 Q4; ticket 26 Q4 findings 11, 26 and 29).

1. **Delete the in-memory adapter** (ticket 21 Q1). `Backend` has one variant, SQLite. `Storage::new_in_memory()` keeps its name and opens SQLite `:memory:` through `DbPool::new(":memory:")`, so its 159 calls in 32 files do not change. Delete `in_memory_data.rs`, every `Backend::InMemory` arm, `no_in_memory` (in `utils/records.rs`), the no-op branch of `touch_game_updated_at` (`core.rs:225`), the placeholder backend in `add_failure` (`core.rs:105`), and the in-memory branch of `seed_game_row`.
2. **A typed read-failure key** (ticket 21 Q2). `StorageCall`: one variant per `Storage` method that the failure layer can fail (38 today), next to `TxWrite`. `with_failure`, `add_failure` and the shared handle (`set`, `clear`) take a `StorageCall` in place of a `&'static str`. The test `test_typoed_override_key_panics_on_assert` goes: a typo does not compile.
3. **`EngineError::Database { context: &'static str, source: rusqlite::Error }`** (ticket 20 Q4). It replaces `Database(#[from] rusqlite::Error)` and every hand wrap of a rusqlite error under `storage/` (96 `EngineError::Config` sites in non-test storage files, not all of them rusqlite errors). A serde wrap takes `Serde`, a datetime wrap takes `Parse`. `Config` stays for a setting with no valid value. With no `#[from]`, a `?` on a rusqlite error does not compile, so every site names its context. `GenerationFailure::from_engine_error` keeps its `Database | Io` arm until R7. Two effects from R1 to R7 (known): a hand wrap that was `Config` (the `Other` line) and reaches `from_engine_error` now gives the save line (known callers: `narrator.rs:87`, `:98`; inferred: low exposure); and `refusal_message_without_layer_prefix_for_validation_and_config` (`utils/error.rs:53`) treats `Database` as it treats `Config`, so an HTTP refusal does not gain the prefix "Database error:" (inferred: keeps today's texts).
4. **One story entry** (ticket 21 Q3). `StoryRow` (`message_types.rs:32`) gets `Serialize`. It serves the story log view models, `DebugStateView`, the prompt assembler history and the Agents. Delete `MessageEntry`, `StoryEntry` (`agent.rs:91`) and their three `From` impls. The story entries of `/debug/state` gain the key `kind` (inferred: no spec lists the keys of an entry; check 39.6 and 39.7). `tests/test_utils/stub_server.rs:75` builds `MessageEntry` values with `Default`; it builds `StoryRow` values.
5. **Small duplication** (finding 26). One body for `Storage::transaction` and `Storage::transaction_new_game`. One helper for the pair `put_dock(default)` + `set_game_failure(None)` (3 sites). Serde derives in place of the hand-written `serde_json` in `docks.rs`. `ScriptedModel` uses `Role` in place of `ModelRole`.
6. **A Message with no Swipe is a load error** (finding 29; `mappers/message.rs:72`).

**Spec scenarios.** None. R1 changes no behaviour that a player can see.

**Tests.**

| Tier | Id | Asserts |
|---|---|---|
| storage seam contract | `db-error-context` (ticket 20, ticket 21 Q2) | a failed read gives `EngineError::Database { context, .. }` with the context of its method. It is the storage read-failure case of finding 28 |
| storage seam contract | `message-without-swipe` | a Message row with no Swipe row gives a load error |
| storage seam contract | `storage-newest-game-updated-at` (finding 11), changed | the test data has a newer id with an older `updated_at`, and the newest `updated_at` wins |
| unit, `core_tests.rs` (legacy) | rekeyed in place | a `StorageCall` key fails only the call it names; `clear_all` removes it |
| existing | — | every storage, unit and tier-1 test stays green on SQLite `:memory:` |
| storage seam contract | deleted with their subject | `transaction_storage.rs:190` (`storage-tx-no-in-memory-arm`), `transaction_storage.rs:233` (`storage-newest-game-in-memory`), `story_storage.rs:669`, `world_storage.rs:428` |
| unit, legacy | renamed in place | the `*_in_memory*` tests of `characters_tests.rs`, `personas_tests.rs`, `worlds_tests.rs`, `games_tests.rs` and `llm_messages_tests.rs` stay valid on SQLite; they lose "in_memory" from their names |

**The SQLite count** (ticket 21, its risk). A probe on `5d3cfe5f` opened `new_in_memory()` on SQLite `:memory:` and ran `python build.py integration`: 1331 passed, 9 failed (log kept at `tmp/r1-sqlite-probe.log`, git-ignored). Each one is fixed in the slice:

| Test | Cause | Fate |
|---|---|---|
| `story_storage::the_new_records_refuse_the_in_memory_backend`, `transaction_storage::transaction_refuses_the_in_memory_backend` | their subject is the in-memory refusal | go with it |
| `core_tests::with_failure_adds_does_not_nest`, `add_failure_reuses_map_does_not_nest` | they assert the backend name `InMemory` | change in place (mechanical) |
| `characters_tests::test_get_character_found_in_memory`, `test_list_characters_in_memory_seeded`, `test_seed_character_idempotent_in_memory` | `FOREIGN KEY constraint failed`: SQLite enforces a key that the in-memory maps did not | the test arranges the parent row; rename the tests (no "in_memory") |
| `world_storage::test_update_world_nonexistent_noop` | an update of a missing World is an error on SQLite, a no-op in memory | the test asserts the SQLite behaviour (inferred: no caller relies on the no-op; check the World edit handler) |
| `preset_storage::test_list_presets_keeps_the_insertion_order` | SQLite lists the presets in the reverse order | production runs on SQLite, so the SQLite order is today's behaviour; the test asserts it and its name changes (inferred: check the SQL `ORDER BY` of the preset list) |

**Guardrail rules.** None new.

**Docs.**

- `tests/STRATEGY.md`: the unit-tier row says "SQLite `:memory:` `Storage`" in place of "in-memory `Storage`".
- `docs/diataxis/reference/storage.md`: remove "the in-memory maps" and every other in-memory statement.
- `storage_design.md`, `architecture.md`, `architecture_system.md`, `rust_idioms.md` and `integration_test_standards.md`, where they name the in-memory backend (they match a search for "in-memory" today).

**Done when** (§2, plus): a search of `src/` and `tests/` for `InMemory`, `in_memory_data`, `no_in_memory`, `MessageEntry` and `StoryEntry` finds nothing; a search of `src/adapters/driven/storage/` for `EngineError::Config` finds only the wraps of a setting with no valid value (known today: `utils/plumbing.rs:13`, `"Invalid {field}"`) and `test_support.rs:116`. The multi-line `format!(` wraps (`docks.rs:34`, `stories.rs:119`, `stories.rs:292`, `games.rs:133`) and the `"Migration failed"` wraps of `plumbing.rs` count.

## 4. R2: the story window

**Content** (ticket 23).

- `GameViewQuery::story_window(game)` owns the size (50). In one storage read, it returns the newest 50 entries (`StoryRow`), oldest first, and the `Tail` facts: the id of the last Message and `retrigger_allowed`. The one read closes the gap between the two reads of today (`entries`, then `tail`).
- `AppState::render_story_log` (`app_state.rs:99`) loses `MAX_LOG_DISPLAY` and `take`, and only renders. The template keeps `loop.last` for the last-entry controls.
- The shape of the value that `story_window` returns is the implementer's choice. Ticket 23 rejected a domain `StoryWindow` type that holds only the size; R3 adds the room names to the same value.

**Spec scenarios.** `story_log.md` scenarios 8.11 and 8.12, with the text of ticket 23 Q3.

**Tests.** `tests/http/story_log.rs`: one test for 8.11, one for 8.12. No new browser scenario: 30.14 and 30.23 cover the browser side (ticket 23 Q3).

**Guardrail rules.** None.

**Docs.** None: `CONTEXT.md` has **Story window** already. Check `docs/diataxis/reference/frontend/dashboard.md` and edit only a statement about the story log length that you find.

**Done when** (§2, plus): a search of `src/` for `MAX_LOG_DISPLAY` finds nothing; the story log handler calls one read.

## 5. R3: the room is one fact

**Content** (ticket 25; ticket 19 interface 4, the room lookup).

- **The one room lookup.** `SwipeState::room(&self, map: &MapDef) -> Option<&Room>`: the map room, else the added room, else none. It replaces the room lookups of `job.rs` (`room_of`), `view_query.rs` (inline) and `gate.rs` (`opening_room_report`, with its `.ok()`). Two of the old copies ignore `dynamic_rooms` (finding 4).
- **The derived change marker.** The story log shows the room name on a Message when the room name of its active Swipe state differs from the room name of the Message before it. The oldest Message of the story window always shows its room name. Compare by name, not by room id (ticket 25 Q1 to Q3).
- **Where the marker is made** (inferred). `story_window` (R2) also reads the active Swipe state of each entry, in the same storage read as the entries and the `Tail` (the reason of ticket 23 Q2), and reads the World map, and gives the room name to show for each entry. An entry whose room lookup gives none counts as "no room name": it shows nothing, and the next entry with a name shows its name. Cost (inferred): each 2 s poll parses up to 50 Swipe states. A pure function in `domain/model/` takes the room names of the window, oldest first, and returns the marker of each entry, so it has plain unit tests. The World read is a second read; it does not change while a Job runs, so the gap of ticket 23 Q2 does not apply (inferred).
- **Delete the stored room report.** Schema migration v29 drops `message_swipes.room_report`. Delete `Swipe.room_report`, `StoryRow.room_report`, its writers in `gate.rs` and `job.rs` (`job.rs:502`), the room-report part of `opening.rs`, and the reader of `GameRulesOutcome::arrived_room_name`. The three "Retry of the opening?" checks go, for the room: `opening.rs`, the `tail.count == 1` check of the writer, and `job.rs` `is_the_opening` (`job.rs:650`). Keep a check only if a rule other than the room still reads it (inferred: none does).
- **No change** to the Game rules for "named the room the player is in" (ticket 25: moot). The narrator prompt, the Quantifier prompt, the Trigger `room_id` condition and the sidebar read the Swipe state already, and do not change.

**Spec scenarios** (ticket 25, its spec scenarios).

- Keep the results of 1.14 (`actions.md`), 9.2, 9.8, 10.8, 22.5 (`swipe_new.md`) and the room scenario of `swipe_switch.md` (its line 40). A step on a Swipe that is not active ("its first swipe reports no room") becomes "after a switch to that Swipe, the Message shows …".
- New scenarios, one for each behaviour change: (1) a Narration that names the room the player is in shows no room; (2) two added rooms with the same name show one room name; (3) a move between two map rooms with the same name shows no room; (4) in a long story, the oldest entry of the story log shows its room. Put (4) in `story_log.md` next to 8.11, and (1) to (3) in `actions.md` next to 1.14 (inferred).

**Tests.**

| Tier | Id | Asserts |
|---|---|---|
| unit, `domain/model/` | `room-added-room` (ticket 19) | `SwipeState::room`: a map room, else an added room, else none |
| unit, `domain/model/` | `room-marker-by-name` (inferred) | the marker rule: a changed name shows; the same name does not; the oldest entry always shows |
| tier 1 | the four new scenarios | as written |
| tier 1 | the kept scenarios | rewritten steps, same results |

**Guardrail rules.** None.

**Docs.**

- Target design: D-19 is replaced by the derived marker, with its rejected option; the §4.3 rows "Room shown on a Message", "Create, reset", "Input save", "Text save" and "State update", and the room-report column of §4.4, lose the room report. T-03 and F-10 in §9 follow (inferred).
- [Clean spec](../../.scratch/turn-state-model/assets/clean-spec.md): R-22 to R-25a become two rules: the derived change marker, and the oldest Message of the story window.
- `storage.md`, `game_flow.md`, `integration_test_standards.md` and `architecture_system.md`, where they name the room report.

**Done when** (§2, plus): a search of `src/`, `tests/` and `docs/diataxis/` for `room_report` finds it only in the migration history of `storage/utils/plumbing.rs`; a search of `docs/diataxis/` for "room report" finds nothing.

## 6. R4: the Quantifier result and the Game rules

**Content** (ticket 19 Q3 to Q8, Q11, Q12, interfaces 3 to 5, with the amendment of ticket 20 Q13; ticket 26 finding 13).

- **The Quantifier result** (interface 3). `QuantifierResult { present: Vec<String>, destination: Option<String> }`. The Agent at the after-narration hook returns it, and the Game rules take it. Confidence and movement type stay private to the Quantifier, with one confidence enum. Delete `StatePatch`, `AnalysisOutput` (`agent.rs:224`, `231`), `StatePatch::merge`, the second confidence enum and its two `From` impls, the public `QuantifierParseResult` and `MovementParseResult` (they become parser-private), `SceneState.quantifier_confidence` and `DebugStateView.quantifier_confidence`.
- **Each Agent kind has its own result type, with one landing** (Q6). No merge of several outputs.
- **Hooks named by position** (Q7, Q8). `Hook` (`agent.rs:46`) has the after-narration hook and the end-of-Job hook. The before-narration hook comes with its first Agent (D-34). `AgentRunner` names its methods by the hook. A rewrite is a result type, not a hook. The trait `AnalysisAgent` (`trait_def.rs:7`) and `Hook::Analysis` take the name of their hook position, because ticket 19 Q5 drops the term "analysis" (inferred; the name is the implementer's).
- **The Agent kind gives the hook** (finding 13). Delete `AgentConfig.hook` (`agent.rs:159`) and its field in `data/schemas/settings.schema.json` and `data/settings.json`.
- **The Game rules: one call per generation** (interface 4, amended). `game_rules::apply(state: SwipeState, result: &QuantifierResult, world: &GameWorld<'_>, kind: Kind) -> Result<RulesOutcome>`, with `RulesOutcome { state, fired: Option<EventRecord> }`. The `kind` parameter is gap G1 (§12; owner, ticket 28).
  - Narration and Impersonate: movement; the Characters present; the Trigger (it reads `times_met` before the encounters advance it); the encounters. The trigger context becomes the fired event record. Its direction is rendered through `TemplateVars`, once (finding 12).
  - Event: no movement, no Trigger check, no trigger context.
  - `Err` is the contradiction of the diagnostics check, as today.
  - `GameWorld { map, characters, persona_name }` is a domain struct. The storage adapter builds it from `WorldBundle`, so the arch-lint row "model → storage" (rule 12) holds.
  - `SwipeState::pre_analysis` stays until R5 (`swipe_state.rs:49`, its caller `job.rs:256`): it marks an event's one-shot Trigger fired until `GenerationOrder::write` takes that job.
  - The Game-rules step loads the Swipe state at `saved.at` and calls `apply` once. When a Trigger fires, the Job builds the event order from the fired record (Q11).
  - Delete `GameRuleRequest` (3 variants), `TriggerMatch` and its two `From` impls, `trigger_narration`, the double call and the hand-cleared context in `Job::apply_rules`, and `GameRulesOutcome` with its fields (finding 25).
- **Stored data** (interface 5, Q12). The trigger context changes from `TriggerRef` to `EventRecord`. Schema migration v30 deletes the saved Games, as v28 did. This deletes the Games in the dev database.
- **The Retrigger reads the record** (Q3). Until R5 moves the plan rules, the Retrigger code of `gate.rs` takes the event record from the trigger context of the current state, with no World read. `event_record` and `trigger_direction` lose their World reads here, and R5 deletes them.

**Spec scenarios.** `retrigger.md`: a Retrigger sends the rendered direction to the model (new; finding 12). Check `dashboard.md` 39.6 and 39.7: if a step names the debug key `quantifier_confidence`, remove it. No other scenario changes: the behaviour that a player sees stays.

**Tests.**

| Tier | Id | Asserts |
|---|---|---|
| unit, `game_rules_tests.rs` | `game-rules-event-no-movement` | a destination does not move the player for an event |
| unit | `game-rules-event-no-trigger` | no trigger context and no fired record, also when a Trigger condition holds |
| unit | `game-rules-times-met-before-encounters` | a first-encounter Trigger fires on the Narration that brings its Character |
| unit | `game-rules-fired-record-rendered` | the direction of the fired record has `{{user}}` replaced by the Persona name; the trigger context equals the record |
| seam contract, Quantifier | `quantifier-result-output` | the Quantifier returns a `QuantifierResult`; the confidence stays inside |
| tier 1, `retrigger.rs` | the new scenario | the `ScriptedModel` call record holds the rendered direction |
| unit, `game_rules_tests.rs` | rewritten | the tests and the property-test set of P3 move from `apply_game_rules` and `GameRuleRequest` to `apply` |
| seam contracts | rewritten | the Quantifier cases that assert a `StatePatch` or a confidence assert a `QuantifierResult` |

**Guardrail rules.** Rule 12 stays green. Rule 13 (`walker_rows.rs`): rename the `AgentRunner` hook methods in its row, if their names change.

**Docs.**

- Target design: D-29 and D-34 (hooks by position; each result type has one landing); §2.3 does not change.
- The doc comment of `TriggerRef` ("a reference only"): the trigger context holds the event record now.
- `docs/diataxis/reference/narrative/agent_system.md` (it names `AnalysisAgent` and the hooks) and `game_flow.md` (it names `apply_game_rules` and the hook config).
- `CONTEXT.md` is done (Quantifier, Agent, Hook, Game rules).

**Done when** (§2, plus): a search of `src/`, `tests/` and `docs/diataxis/` for `StatePatch`, `AnalysisOutput`, `GameRuleRequest`, `TriggerMatch`, `apply_game_rules` and `quantifier_confidence` finds nothing; `game_rules::apply` has one production caller.

## 7. R5: the kind carries its data, and the plan rules

**Content** (ticket 19 Q1, Q2, Q9, Q13 to Q15, interfaces 1 and 2, amended by ticket 20 Q13; UI F2, F3; findings 1, 3, 5).

- **The kind carries its data** (interface 1). In `domain/model/generation.rs`:
  - `Generation { Narration { guide }, Impersonate { direction: Option<String> }, UserRegen, Event(EventRecord) }`.
  - `GenerationOrder { generation, place, start: SwipeAt }`. The order holds ids and texts only (Q13).
  - `GenerationOrder::kind() -> Kind`: `Kind` stays a fieldless tag (Q15), for the status step, the role health and the D-31 filter.
  - `GenerationOrder::event_after(saved, record)`: the Job builds the event order with it.
  - `GenerationOrder::write(&self, text, start: &SwipeState) -> NewSwipe` (amended). It is the one home of the Message kind, the message type, the maker, the stored inputs and the event header. It marks an event's own one-shot Trigger fired, once. Its Swipe state is the pre-analysis state.
  - R-18 is a match on the order: a user regen ends after its text save.
  - Delete `GenerationOrder.inputs` and `.event`; the missing-data guards (`require_job_data` direction and event fill; `job.rs` `user_message` and the "the event record" read of `apply_rules`; `message_kind_of`; the `None` arm of `generation_record::message_kind`); `HookInput.kind` and `.event`; `SwipeState::pre_analysis` (its three tests in `swipe_state_tests.rs`, lines 75, 91, 100, become `write-event-marks-one-shot`). `generation_record.rs` goes if `write` takes all of it (inferred), and its tests move to `generation_tests.rs`.
- **A bare `/impersonate` is allowed** (Q1; finding 1, UI F2). An Impersonate has a direction or none. A Retry of an Impersonate Input with no direction runs with none.
- **The Retrigger rule** (Q2; UI F3). R-65: a Narration that is not an event Narration, or an Impersonate Input, with a trigger context. `Tail::retrigger_allowed` is the one rule. The story-log button reads it (through `story_window`, R2) and adds no condition.
- **The plan rules** (interface 2; Q9). `JobPlan::for_request(request: &JobRequest, story: StoryEnd { tail, has_text }) -> Result<JobPlan, PlanRefusal>`, a pure domain function. `PlanRefusal::BadRequest(line)` answers 400; `PlanRefusal::Missing(name)` answers 500 with the failure line. `JobRequest`, `JobPlan` and `OptionsRun` move from `gate.rs` to the domain. `Tail::redo` and `Tail::retrigger` become private parts of `for_request`. `Tail::retrigger_allowed` stays on the `Tail`. A Retrigger copies the event record from the trigger context of the current state.
- **`gate.rs` keeps** the guard, the storage checks (the Game row, the Persona, the start state, has text) and its save units. Delete `plan_for`, `generation_plan`, `require_start`, `event_record`, `trigger_direction` and most of `require_job_data` (`gate.rs:427-589`).

**Spec scenarios.**

- `actions.md`: a bare `/impersonate` writes an Impersonate Input (new).
- `swipe_new.md`: a Retry of a bare Impersonate Input runs (new).
- `retrigger.md` 14.3 changes to "a typed Input" (Q2). New: a Retrigger on an Impersonate Input with a trigger context runs.
- `story_log.md` (today's Retrigger-button scenario is at its line 83, test `tests/http/story_log.rs:283`): the Retrigger button shows on an Impersonate Input with a trigger context.

**Tests.**

| Tier | Id | Asserts |
|---|---|---|
| unit, `generation_tests.rs` | `plan-retry-by-last-kind` (5 rows) | each kind of last Message gives its `Generation` variant, `NewSwipeOnLast` and the start of design §4.5 |
| unit | `plan-retry-bare-impersonate` | an Impersonate Input with no direction gives `Impersonate { direction: None }` |
| unit | `plan-retrigger-refused` (R-65 rows) | no trigger context; the last Message is an event Narration; the last Message is a typed Input: `BadRequest` |
| unit | `plan-retrigger-copies-record` | the order holds the record of the trigger context of the current state |
| unit | `plan-action-no-story`, `plan-options-no-text` | `Missing("the story")`; `BadRequest` |
| unit | `plan-user-regen-not-analysed` (ticket 20 Q13) | the plan of a user regen has no analysis step |
| unit | `write-event-marks-one-shot` (amended) | an event's new Swipe state has no trigger context and has its one-shot Trigger in the fired set; a repeat Trigger is not added |
| unit | `write-user-regen-not-analysed` (amended) | the new Swipe holds the start state, and the maker is the player |
| tier 1, `actions.rs` | the bare `/impersonate` scenario | as written |
| tier 1, `swipe_new.rs` | the Retry scenario | as written |
| tier 1, `retrigger.rs`, `story_log.rs` | 14.3 and the new Retrigger scenarios | as written |
| unit, `tail_tests.rs` | changed | the `redo` and `retrigger` tests move to `generation_tests.rs` with the fn; `retrigger_allowed` tests stay |

**Guardrail rules.** None new.

**Docs.** Target design §4.5 (the direction is no longer data that must exist); `retrigger.md` 14.3; `game_flow.md`, where it describes the plan rules in the writer (inferred).

**Done when** (§2, plus): a search of `src/` for `plan_for`, `generation_plan`, `require_start`, `require_job_data`, `trigger_direction`, `message_kind_of` and `pre_analysis` finds nothing; `gate.rs` holds no request planning.

## 8. R6: the preset module

**Content** (ticket 26 Q1, candidate S2; finding 15; finding 28, the presets part).

- **One preset module** in `application/` (inferred: it grows from `PromptPresetService`, today a pass-through at `prompt_preset_service.rs:46`). It gives the preset of a Game for each hook and for the narrator, and it holds the edit and delete rules that are in the HTTP handler today (`prompt_presets.rs` `update_preset_handler`, `delete_preset_handler`).
- **A Game-row read failure is a failure.** `active_quantifier_preset_id` and `active_options_preset_id` (`games.rs:203`, `219`) and `preset_id_or` stop falling back to the registry on a read error (`.ok().flatten()`, `games.rs:237`; finding 15). The preset choice leaves storage.
- **A preset delete is refused while a Game uses it**, the same way as a World delete ("used by N games"). Today the delete checks the registry and the default Options preset, but not the preset slots of each Game (synthesis S2).
- The missing-preset policy of the Job does not change in R6. R7 changes it (§12, gap G3).

**Spec scenarios.** `prompt_presets.md`: a delete of a preset that a Game uses is refused, with the count of Games (new). Check the edit and delete scenarios of `prompt_presets.md`; they keep their results.

**Tests.**

| Tier | Id | Asserts |
|---|---|---|
| tier 1, `prompt_presets.rs` | the new scenario | the refusal line names the count; the preset stays |
| tier 1, `prompt_presets.rs` | changed (finding 28) | the tests observe through the preset module, not `Storage` |
| tier 1 | `preset-game-read-fails` (inferred: finding 15 asks that the failure show) | an injected Game-row read error (`StorageCall`, R1) in a Job gives a failure line, not the registry preset |

**Guardrail rules.** None new. `check_http_storage_leak` stays green when the handler calls the module.

**Docs.** `storage.md` and `architecture_system.md`, where they place the preset choice in storage (inferred: both name presets today). `prompt_system.md`, where it states how a Game's preset is found.

**Done when** (§2, plus): the preset handlers make no edit or delete decision; a search of `src/adapters/driven/storage/` for `preset_id_or` finds nothing.

## 9. R7: the failure lines and the loop of the Job

**Content** (ticket 20 Q1, Q2, Q4, Q14; ticket 26 findings 24, 27, 28; C-4).

- **One read-line rule** (Q2). A read in `accept` and in the Job gives "The data for this request is missing: <record>.", with the raw error in the disclosure. Only a failed write gives "The new text could not be saved." The record names are one `Record` enum (`Game`, `Story`, `StartState`, `World`, `Persona`, `Settings`, `Preset`, …) in `domain/model/state/generation_status.rs` (inferred: next to `GenerationFailure`). Delete `impl From<EngineError> for Failure` (`job.rs:87`), so a read that names no record does not compile.
- **A missing preset is a failure line** (gaps G3 and G6, §12; owner, ticket 28). With `Record::Preset`, a missing Quantifier or Options preset gives the missing line, not `None` (`job.rs:798`, `Ok(None)` at `:800`). A missing narrator preset gives the same line: `GenerationFailureKind::PresetMissing` ("The active prompt preset is missing.", `generation_status.rs:35`, `:64`, `:151`) goes, and the scenario of `failure_display.md` at its line 88 changes its line. R6 makes a missing preset rare, because a preset delete is refused while a Game uses it.
- **The error class** (Q4). `from_engine_error` classifies only the model and prompt-assembly errors. Its `Database | Io` arm goes.
- **One loop** (Q14). `enum JobFailure { EntryMoved, Line(GenerationFailure) }`, and `Job::run` as the loop of ticket 20 Q14: one local "a Narration was saved", one event wrap, one exit. Delete `Failure` (`job.rs:56`), `Ended`, `Failure::missing`, `event_narration_failed`, `into_event_failure` (2 sites become 1), `failed`, the three holders of "a Narration was saved", and `Stopped::SaveFailed` as a separate path: a unit error becomes `Line(save_failed)` at the unit.
- **The Options run follows** every failure after a saved Narration or event Narration, except `EntryMoved` (Q1). This includes a `SaveFailed` of the state update or of the event text save (target design §4.2; today's code skips it). The status shows the first failure. A failure of that Options run is only logged.
- **A refused state update** (finding 24; inferred: CR-S6 names the contradiction of the Game rules). It shows the contradiction line, not "The new text could not be saved.", and the text stays saved.
- **Names** (finding 27). `GenerationFailureKind::SceneMissing` becomes `InvalidRoom`. `Units` stays (closed).
- **`FAILURE_KINDS`** (finding 28; `tests/browser/stub/failure_display.rs:887`) is derived from the enum, so it cannot lag.

**Spec scenarios.**

- `options.md`: the Options run follows a failed state update; the Options run follows a failed event text save (new; ticket 20).
- `failure_display.md`: a read error in a Job shows the missing line with the name of its record (new rows); a state update that the Game rules refuse shows the contradiction line (new; finding 24).
- `game_flow.md` is a Diátaxis doc, not a spec: see Docs.

**Tests.**

| Tier | Id | Asserts |
|---|---|---|
| unit, `domain/model/` | `options-follow` (6 rows; gap G2) | on demand; a Narration saved with no failure, with a line, with `EntryMoved`; no Narration with a failure and with none |
| unit, `generation_status_tests.rs` | `failure-of-event` (gap G2) | an event's line becomes "The event narration failed." with the inner line in the raw text; the lines of the other kinds pass. `EntryMoved` is a `JobFailure` in the application, so `races.rs` covers it at tier 1 |
| unit, `generation_status_tests.rs` | `record-names` | each `Record` gives its fixed name in the line; a miss and a read error give the same line |
| tier 1, `options.rs` | `options-after-state-save-failed`, `options-after-event-text-save-failed` | the save line (or the event line) shows, and the Options run ran |
| tier 1, `failure_display.rs` | `job-read-names-record` (rows) | an injected read error (`StorageCall`, R1) in a Job shows "missing: <record>", not the save line. One row is I.7: a failed start-state read in `accept` (C-4) |
| tier 1, `failure_display.rs` | `job-read-names-record`, preset rows (G3, G6) | a missing narrator preset and a missing Quantifier preset each show "missing: the preset" |
| tier 1, `failure_display.rs` | `state-update-refused` (finding 24) | the contradiction line shows; the Narration stays |
| seam contracts | changed (finding 28) | each "failure text" case checks the exact text, not its presence |
| tier 1, `prompt_presets.rs` or `failure_display.rs` | `preset-game-read-fails` (R6), changed | it asserts the missing line of `Record::Game` |
| tier 1, `failure_display.rs` 38.11 | changed (finding 28) | the test does not depend on which unit writes the dock |

Existing tests that must stay green (ticket 20): `options.rs` (the three dock tests), `failure_display.rs` 38.x, `races.rs`, and the event-failure rows of `retrigger.rs` and `swipe_new.rs`.

**Guardrail rules.** None new.

**Docs.** `game_flow.md`: the Options run after a save failure, and the failure-line rule. Target design §4.7, if a line text changes (inferred: the "missing" line already has its text there).

**Done when** (§2, plus): a search of `src/` for `impl From<EngineError> for Failure`, `event_narration_failed`, `into_event_failure`, `SceneMissing` and `PresetMissing` finds nothing; `Job::run` has one exit.

## 10. R8: the step rule and the narrator

**Content** (ticket 20 Q3, Q5, Q11 to Q13; ticket 26 finding 21).

- **The step rule** (Q3, Q11; sketch 25 §2.1). Each step loads by id what it uses, makes one call, saves small, and returns its result. Between steps only ids, texts and step results move. A later step may read a record again: finding 19 is the accepted cost. The step list of ticket 20 ("Data: the step rule") is the target: `room_check`, `save_input`, `narrate`, `save_text`, the R-18 end, `analyse`, `update_state`.
- **The save units read nothing** (Q3). The step that calls a unit loads the values and gives them to it. Delete from `JobWriter`: `state_at`, `tail_of`, `dock` (`opening_room_report` went in R3). This also removes the duplicate start-state read (`job.rs` `start_state` and `JobWriter::state_at`; ticket 19, handed on).
- **A notice-only write.** An on-demand Options failure writes the notice only, through a new notice-only `Tx` write. `save_options` loses `after_narration`.
- **`GameWriter::stop` loads first, then saves** (finding 21). It reads the dock record before it removes the job entry. A failed read then leaves the Job running and answers "failed".
- **The narrator owns the prompt rules of each kind** (Q5). The narrate step gives the values it loaded to `Narrator::narrate(&Generation, ..)`. The narrator makes the user message (a user regen's Input text from the `Tail` the step read; finding 24, the `player_text` fallback goes), the event continuation text (`continuation_user_message` moves out of the Job), the guide and Impersonate flags, and the role for the role health. `Generation::preset_type` is a domain function. `Narrator::assemble` becomes private. The `agent-edge` rule stays.
- **Each hook builds its own input** (Q5). The after-narration hook and the end-of-Job hook each load their own values and build their `HookInput`. The Options run reads its toggle when it runs.

**Spec scenarios.** None new: the behaviour that a player sees stays. The stop with a failed dock read is a change only on a storage fault (inferred: no scenario states it today).

**Tests.**

| Tier | Id | Asserts |
|---|---|---|
| seam contract, `narrator_tests.rs` | `narrator-user-message-by-kind` (4 rows) | the user message and the flags of each `Generation`, with no storage |
| seam contract, `narrator_tests.rs` | `narrator-kind-role` | the role of each `Generation` |
| storage seam contract | `notice-keeps-set` | the notice-only write keeps the offered set |
| tier 1, `races.rs` or `dashboard.rs` | `stop-dock-read-fails` (finding 21; inferred) | an injected dock read error on a stop leaves the Job entry, and the stop answers "failed" |
| guardrail | row 7, parameter half | no `Job` method takes a record type |

Existing tests that must stay green: `races.rs` (a stop at each model call; the event narration uses the preset that is active when it runs, R-59), and every tier-1 test that runs a Job.

**Guardrail rules.**

| # | Rule | Row |
|---|---|---|
| 7 | Between steps, pass ids (parameter half) | type-check row on `pipeline/job.rs` (`tests/infrastructure/guardrails/walker_rows.rs`): no method of `Job` takes `Game`, `Message`, `SwipeState`, `Tail`, `DockRecord` or `WorldBundle` as a parameter. A free helper fn that builds an input from values is allowed (ticket 20 Q12) |

**Docs.** [Build plan](rust-rebuild-build-plan.md) §9.5, rule 7: the parameter half. `prompt_system.md` and `narration_system.md`, where they place the prompt rules in the Job (inferred).

**Done when** (§2, plus): a search of `src/` for `state_at`, `tail_of`, `continuation_user_message` in `job.rs`, and `fn player_text` in `job.rs` (`job.rs:617`) finds nothing (`HookInput.player_text` stays); `Narrator::assemble` is private.

## 11. R9: the small fixes

**Content** (ticket 26 Q3 to Q5).

- **Finding 10**, the stop table of scenario 39.4 (`dashboard.md`): the row at the Options call asserts the status `options`; a new row stops at the analysis call of an event Narration (the one-shot Trigger is in the new Swipe state).
- **Finding 16**: `ActiveGame::set` (`active.rs:37`) returns `Result`. A Game select answers with the failure line, and the pointer does not move. The doc of `ActiveGame::set` loses the wrong `updated_at` claim (finding 29).
- **Finding 22**: the status handler answers with the failure line when the status read fails, not "idle".
- **Finding 23**: `reset` with no active Game answers 400.
- **P-1**: the stop and the status poll with no active Game.
- **Finding 28**: scenario 3.5 asserts the count of Narrations.
- **Finding 29**: `render().unwrap_or_default()` in `games.rs` (lines 71 and 240) gives the failure line; `architecture_system.md` loses the `AppState` field list.

**Spec scenarios.**

- `dashboard.md` 39.4: the two row changes (finding 10). New: the status poll when the status read fails (finding 22). New: the stop and the status poll with no active Game (P-1).
- `reset.md`: a reset with no active Game answers 400 (new; finding 23).
- `games.md`: a Game select whose pointer write fails answers with the failure line, and the active Game stays (new; finding 16).
- `actions.md` 3.5: the count of Narrations.

**Tests.** One tier-1 test for each new scenario, at the tier of its spec; the 39.4 table test gains its rows; 3.5 gains its assertion. The two `games.rs` render failures and the select failure use the storage failure layer or a broken template, whichever the code allows (inferred).

**Guardrail rules.** None.

**Docs.** `architecture_system.md` (the `AppState` field list goes).

**Done when** (§2, plus): a search of `src/adapters/driving/http/` for `unwrap_or_default()` on a `render()` finds nothing.

## 12. Gaps the plan states

These points are not in the tickets as written. The plan fills each one with the reading that changes the least, and labels it. G1, G3 and G6 change an owner decision or a line that a player sees; the owner confirmed all three on 2026-10-20 ([ticket 28](../../.scratch/rust-rebuild-build/issues/28-confirm-the-fills-of-the-response-plan.md)), so they are decisions now. A reviewer or the owner who disagrees with another fill reopens it as a ticket before its slice starts.

| # | Gap | Fill | Slice | Label |
|---|---|---|---|---|
| G1 | (Amends the signature that ticket 20 Q13 confirmed.) Ticket 20 Q13 deleted `PreAnalysisState`, which told the Game rules that a generation is an event. `apply(state, &QuantifierResult, &GameWorld)` cannot tell an event from a Narration, but an event has no movement and no Trigger check (ticket 19) | `apply` takes the `Kind` of the order as a fourth parameter. The step that calls it has the order (ticket 20: `update_state(order, saved, result)`), and R-18 is a match on the order too | R4 | decided: owner, ticket 28 (option A) |
| G2 | Ticket 20 puts `options-follow` and `failure-of-event` in `job_tests.rs`. A unit test of `job.rs` breaks the one rule (build plan §2; §9.4: tier 1 covers `job.rs`) | The rule of `options-follow` is a pure fn next to `OptionsRun` in `domain/model/generation.rs`, and the event wrap is a method of `GenerationFailure` in `generation_status.rs`. Their tests are plain domain unit tests. The fn takes plain facts (on demand, a Narration saved, the failure is `EntryMoved`, a failure exists), so `JobFailure` stays in the application | R7 | inferred |
| G3 | Ticket 20's read-line rule (`Record::Preset`) makes a missing Quantifier or Options preset a failure line, not `None` (`job.rs:798`). No ticket states this player-visible change | State it: a missing preset of any hook is the missing line. R6 refuses a delete of a preset that a Game uses, so the line shows only on a broken database | R7 | decided: owner, ticket 28 (option A) |
| G6 | The narrator already has its own line for a missing preset, `PresetMissing` ("The active prompt preset is missing."). With G3, one fault (a missing preset) has two lines, against ticket 20 Q2 ("one fault shows one line") | `Record::Preset` for every preset; `PresetMissing` goes; the scenario of `failure_display.md` at its line 88 changes its line | R7 | decided: owner, ticket 28 (option A) |
| G4 | Ticket 27 Context: the Game delete of ticket 19 and the drop of `message_swipes.room_report` (ticket 25) "can be one migration" | Two migrations: v29 (R3) drops the column; v30 (R4) deletes the saved Games. Each slice is green on its own, so each migration lands with the code that needs it. Only v30 deletes Games | R3, R4 | inferred |
| G5 | Ticket 25 shows the room name of each entry, but `StoryRow` holds no Swipe state ("the story reads take no Swipe state", `message_types.rs:29`) | `story_window` also reads the room of each active Swipe state and the World map; a pure domain fn makes the marker (§5) | R3 | inferred |

## 13. The merge

**Content** (ticket 26 Q6; the Destination of the map).

1. Every slice R1 to R9 is **done** (build plan §2). The last commit of R9 passed the full gate and the retro.
2. A last search: `src/`, `tests/` and `docs/diataxis/` hold none of the names that the Done-when lines of R1 to R9 remove.
3. `git merge --no-ff wf/rebuild-response` into `main`, as the earlier slices did. If `main` moved after `22a8f090`, merge `main` into `wf/rebuild-response` first and run the full gate.
4. The full gate runs on `main` after the merge.
5. If `wf/rebuild-review` and its worktree `../chronicler-engine-rebuild-review` still exist, remove them (ticket 26 Q6). The branch holds only two merge commits of its own. Use `scripts/remove_worktrees.py` if it accepts the worktree; else ask the owner.
6. The map's Destination is reached. The lead records it on the map.

## 14. Where each item of the review went

The outside check of ticket 27: each decided item has one slice.

| Item | Slice |
|---|---|
| Finding 1, UI F2 (bare `/impersonate`) | R5 |
| Finding 2 (failure lines) | R7 |
| Finding 3 (kind and data) | R5 |
| Finding 4 (the room lookups, the event rules, the one-shot mark, the `Kind` maps) | R3 (lookups), R4 (rules), R5 (mark, `Kind`) |
| Finding 5 (planning in `gate.rs`) | R5 |
| Finding 6 (the loop) | R7 |
| Finding 7, candidate M2 (in-memory adapter) | R1 |
| Findings 8, 9, 14, candidate S4 | closed (ticket 22) |
| Finding 10 (stop table 39.4) | R9 |
| Finding 11 (newest-Game test data) | R1 |
| Finding 12 (Retrigger render) | R4 |
| Finding 13 (`AgentConfig.hook`) | R4 |
| Finding 15, candidate S2 (presets) | R6 |
| Finding 16 (`ActiveGame::set`) | R9 |
| Finding 17, candidate M4 (analysis type), candidate M5 (story entries) | R4, R1 |
| Findings 18, 20, candidate M6 | closed (ticket 26) |
| Finding 19, candidates M3, S5 (narrator, repeated reads) | R8 (finding 19 is the accepted cost) |
| Finding 21 (stop order) | R8 |
| Finding 22 (status read) | R9 |
| Finding 23 (reset 400) | R9 |
| Finding 24 (`player_text`, refused state update) | R8, R7 |
| Finding 25 (test-only surface) | closed; R4 and R8 delete its named parts |
| Finding 26 (duplication) | R1 |
| Finding 27 (`SceneMissing`) | R7 |
| Finding 28 (tests) | R1 (`db-error-context`), R6 (`prompt_presets.rs`), R7 (failure text, `FAILURE_KINDS`, 38.11), R9 (3.5) |
| Finding 29 (misc) | R1 (no Swipe), R9 (render, `ActiveGame` doc, `architecture_system.md`) |
| C-4, P-1 | R7, R9 |
| UI F1, candidate S3 (story window) | R2 |
| UI F3 (Retrigger button) | R5 |
| Ticket 25 (rooms) | R3 |
| UI F4, F5 | done (ticket 24) |
| The merge, `wf/rebuild-review` | §13 |
