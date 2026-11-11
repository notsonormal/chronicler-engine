## 1. Encounter / Characters-present lifecycle
- **Where**: `src/domain/model/trigger.rs:43-47` (`Encounter`), `trigger.rs:50-54` (`Encounters.characters`), held in `src/domain/model/state/swipe_state.rs:20-22`
- **State variables**: `times_met: u32` (counter), `currently_meeting: bool`, `trigger_fired: HashMap<usize, bool>`, plus the presence/absence of the map entry itself
- **States**: (a) no entry = never met; (b) `times_met >= 1, currently_meeting = true` = in the room; (c) `currently_meeting = false` = known but gone; (d) per-Trigger "already fired" marks; (e) corrupted: entry with `times_met = 0`, or `currently_meeting = true` with a `times_met = 0` entry
- **Transitions**:
  - entered from the Scenario → `times_met = 1, currently_meeting = true` — `state/swipe_state.rs:66-79`
  - `Entered` (set difference) → `currently_meeting = true`, `times_met += 1` — `utils/game_rules.rs:150-155`
  - `Left` (set difference) → `currently_meeting = false` — `utils/game_rules.rs:156-160`
  - room changed → `currently_meeting = true` for *every* Character in `result.present` — `utils/game_rules.rs:112-121`
- **Guards / invariants**: `times_met` increments only on `Entered`; a non-repeat Trigger is skipped when its index is marked fired (`game_rules.rs:202-211`); the Trigger reads `times_met` before the transitions advance it (`game_rules.rs:41-43` comment)
- **Explicitness**: FULLY IMPLICIT (counter + bool + map, no enum, three writers)
- **Already modeled**: no
- **Risks seen**:
  - `is_currently_meeting` (`trigger.rs:87`) has no production reader — only tests call it (`tests/storage/story_storage.rs:1015`). The flag is written by three paths and read by none. **known**
  - `currently_meeting` is set by a second path (`game_rules.rs:118`) than `Entered`, so "meeting" is not implied by `times_met`. **known**
  - The scenario start sets `times_met = 1` directly (`swipe_state.rs:76`) instead of going through `increment_times_met`, so the first-encounter rule has two homes. **known**
  - A Character that stays in the room keeps `times_met = 1` forever; an `Eq` threshold of 2 can never be met without a leave. Design consequence, but nowhere stated. **inferred**

## 2. Trigger fire-once lifecycle
- **Where**: `src/domain/model/utils/game_rules.rs:70-79` (firing + `event_record`), `src/domain/model/generation.rs:118-133` (the mark), `trigger.rs:62-64` / `trigger.rs:79-85` (the marks)
- **State variables**: `SwipeState.event_record: Option<EventRecord>` (`state/swipe_state.rs:22-31`), `Encounter.trigger_fired[idx]`, `MessageKind::EventNarration(EventRecord)`
- **States**: (a) no Trigger has fired; (b) a Trigger fired and its record is on the analysed Swipe; (c) the event Narration exists and the one-shot mark is set; (d) fired but never narrated (event generation failed) — record present, mark absent
- **Transitions**:
  - rules fire → `state.event_record = Some(record)` — `game_rules.rs:74-78`
  - the event Swipe is written → `mark_trigger_fired` for a non-repeat Trigger, `event_record` cleared — `generation.rs:118-133`
  - the event generation's own rules run → `is_event` skips triggers, `event_record = None` — `game_rules.rs:52-79`
  - Retrigger → the record is read off the current state — `tail.rs:50-62`
- **Guards / invariants**: a non-repeat Trigger must not fire twice; only `MessageKind::{Narration, ImpersonateInput}` with a record may be retriggered; an event swap fires nothing
- **Explicitness**: FULLY IMPLICIT (three separate markers, no enum, no single transition function; the rule is split between `game_rules::apply` and `GenerationOrder::write`)
- **Already modeled**: no
- **Risks seen**:
  - If the event generation fails, the analysed state is saved with `event_record` set and *no* `trigger_fired` mark (`job.rs:316-331` saves the state, then the event order runs). The next ordinary Action re-evaluates and re-fires the same one-shot Trigger. Only Retrigger is the intended recovery. **known**
  - `trigger_fired` is keyed by the Trigger's position in the Character's `triggers` list (`game_rules.rs:169-172`, `event_record.rs:10-13`). A World file edit that reorders Triggers re-points every stored mark. **inferred**

## 3. Options dock (offered set / failure notice / busy)
- **Where**: `src/domain/model/dock.rs:8-13` (`DockRecord`)
- **State variables**: `offered: Vec<String>`, `failure: Option<OptionsFailure>`, plus the row's absence
- **States**: (a) no row = never generated; (b) `offered` empty, no `failure` = cleared/empty; (c) `offered` non-empty = a set to pick from; (d) `failure` present = a reason line stands in; (e) `offered` non-empty **and** `failure` present = hidden set under a notice; (f) any of these with the Game busy = rendered but disabled
- **Transitions**:
  - a Narration text save empties it — `gate.rs:845, 917, 938` (`empties_the_dock = kind.saves_a_narration()`)
  - a successful Options run writes the set, no failure — `gate.rs:882-889`
  - a failed Options run writes the failure and **keeps** `offered` — `gate.rs:890`, `transaction.rs:306-318`
  - stop keeps `offered`, drops the failure — `gate.rs:519-524`
  - swipe switch / delete last / create write `DockRecord::default()` — `gate.rs:554, 591, 634`
  - an Input save does **not** empty it — `agent.rs:82-85` (`saves_a_narration`)
- **Guards / invariants**: the template renders `failure` *instead of* `offered` (`adapters/driving/http/templates.rs:64`); buttons are disabled only while busy
- **Explicitness**: FULLY IMPLICIT (a Vec and an Option; state (e) is undocumented and reachable)
- **Already modeled**: no
- **Risks seen**:
  - State (e) is reachable and the model/view layer drops `offered` from view only at render time, not at write time. **known**
  - An Input is saved, the Narration fails → `save_failure` writes the Game failure line and nothing touches the dock, so the *pre-Input* options stay offered under a failure banner, and a pick submits text for a scene that has moved. **known** for the write path, **inferred** for the user-visible consequence
  - `DockRecord::default()` conflates "never generated" with "cleared", so nothing can tell an empty dock from an unread one. **known**

## 4. Game status: Idle / Busy / Failed
- **Where**: `src/domain/model/state/generation_status.rs:8-18` (`GenerationStatus`), `:225-236` (`Step`), `src/domain/model/game.rs:47` (`failure: Option<GenerationFailure>`), reconciled in `src/application/games/view_query.rs:196-212` (`Status`)
- **State variables**: the in-memory busy holder, `games.failure` (nullable column), the Job's `Step` (memory only), and the derived `GenerationStatus`
- **States**: Idle; Busy (`{Narrating, Quantifying, GeneratingEvent, Options}`); Failed(kind); and the reachable pair "busy *and* a stored failure line"
- **Transitions**:
  - accept clears the line — `gate.rs:737-740`
  - a Job end failure writes it — `gate.rs:873-876`
  - a refused request writes it — `gate.rs:684-695`
  - stop clears it — `gate.rs:524`, `732-735`
  - swipe switch / delete last / create clear it — `gate.rs:554, 591, 634`
  - `edit` does **not** clear it — `gate.rs:560-576`
  - the precedence rule: busy wins over the stored line — `view_query.rs:199-201`
- **Guards / invariants**: one line per Game; a busy Game's step is the Job's; a Game that nothing holds and whose column is empty is Idle
- **Explicitness**: EXPLICIT STATES, SCATTERED TRANSITIONS (the enum is explicit, its transitions live in seven handlers of another layer; `GenerationStatus` itself never transitions)
- **Already modeled**: partly — the busy registry and the Job are (`generation/gate.rs`, `pipeline/job.rs`); the failure line's lifecycle is not
- **Risks seen**:
  - "failed" is stored twice (busy holder + column); `GenerationStatus::Failed` is a third, write-only mirror built at read time (`view_query.rs:128-134`). **known**
  - `edit` leaves a failure line that `switch_swipe` and `delete_last` clear, though all three are player repairs of the same story end; no spec states that `edit` keeps it (`docs/specs/story_log.md` says nothing about failure). **inferred** asymmetry
  - `Step` has one writer (`gate.rs:898-904`, memory only), so a status after a restart shows Idle while a Job is not running — correct, but the step and the holder are two separate facts. **known**

## 5. Story end (the `Tail` machine)
- **Where**: `src/domain/model/tail.rs:11-17` (`Tail`), read at `src/adapters/driven/storage/stories.rs:69-95`
- **State variables**: `last: Message`, `state_before_last: Option<(SwipeAt, SwipeState)>`, `count: usize`, plus `last.message_type`, `last.kind`, `last.active_swipe_index`, `current_state().event_record`
- **States**: no story; opening only (`count = 1`); awaits narration (last is an Input); cycle ended (last is a non-event Narration); event Narration last; retriggerable (kind allows **and** the current state holds a record)
- **Transitions**:
  - append a Message — `gate.rs:903-922`, `gate.rs:820-834`
  - append a Swipe on the last — `gate.rs:924-941`
  - delete the last — `gate.rs:574-595`
  - switch the active Swipe (changes which state is "current") — `gate.rs:528-558`
  - `None → Some` for `state_before_last` when a second Message lands
- **Guards / invariants**: the story always keeps its last Message (`can_delete`, `tail.rs:70-72`); `start_at` must name the state the last Message was written from (`tail.rs:32-46`); a trailing Input is a valid rest state (`game_rules.rs:215-221`)
- **Explicitness**: EXPLICIT STATES (implied by field values), SCATTERED TRANSITIONS (the appends live in two other modules)
- **Already modeled**: no
- **Risks seen**:
  - A *missing* state row on the second-to-last Message yields `state_before_last = None` (`stories.rs:86`), the same value as "only one Message". `start_at()` then returns `(last.id, 0)` — the last Message's own swipe 0 — so a Retry starts from the wrong state with no error. **known** (code path), consequence **inferred**
  - `last.message_type` and `last.kind` are read by different rules (`awaits_narration` vs `retrigger_record`/`redo`); nothing keeps the pair consistent (see finding 6). **known**

## 6. Message kind vs message type vs Swipe flags
- **Where**: `src/domain/model/state/message_types.rs:9-30` (`MessageType`, `MessageKind`), `src/domain/model/message.rs:52-60`, `message.rs:26-35` (`Swipe` flags)
- **State variables**: `message_type`, `kind`, `Swipe.impersonated: bool`, `Swipe.guide_text: Option<String>`, `Swipe.impersonate_direction: Option<String>`, `Swipe.event_header: Option<String>`
- **States**: typed Input; Impersonate Input; Narration; event Narration; guided Narration; directed Impersonate. Invalid: `MessageType::Input` with `MessageKind::Narration`, `MessageType::Narration` with `MessageKind::TypedInput`, `kind = ImpersonateInput` with `impersonated = false`, `kind = EventNarration` with an empty `event_header`
- **Transitions**: one writer — `GenerationOrder::write` (`generation.rs:88-136`) and `Message::new` for the opening (`utils/opening.rs:15`) and the typed Input (`gate.rs:821-823`)
- **Guards / invariants**: none in the type or in `Message::new` (`message.rs:65-75` takes both as free parameters); `MessageType::System` has no `MessageKind` counterpart and is mapped to `Narration` only in test fixtures (`test_support/fixtures.rs:576-577`)
- **Explicitness**: FULLY IMPLICIT (two correlated enums plus three redundant flags)
- **Already modeled**: no
- **Risks seen**:
  - The pair is constructible in any combination; `Tail::awaits_narration` reads only `message_type` (`tail.rs:65-67`) while `Tail::redo` reads only `kind` (`generation.rs:326-345`), so an inconsistent pair changes planning silently. **known**
  - `impersonated` repeats `MessageKind::ImpersonateInput`, and one DB migration depends on the flag being the authority (`storage/utils/plumbing.rs:609-623`). Two sources for one fact. **known**

## 7. Swipe text vs Swipe state (un-analysed Swipe)
- **Where**: `src/domain/model/message.rs:26-35` (`Swipe.text` + `Swipe.state`), `src/application/pipeline/job.rs:316-331`
- **State variables**: `text`, `state`, and implicitly "the analysed write ran or not"
- **States**: text saved, state still the start state (pre-analysis); text saved, state written by the rules (analysed); text edited, state untouched
- **Transitions**: two-transaction protocol (`gate.rs:837-860` then `gate.rs:862-871`, documented at `gate.rs:862`); `edit` writes text only — `gate.rs:560-576` → `transaction.rs:206-216`
- **Guards / invariants**: a Swipe's state should describe its text; the contradiction path (`GenerationFailure::Contradiction`, `generation_status.rs:44-51`) is the one declared exception
- **Explicitness**: EXPLICIT STATES, SCATTERED TRANSITIONS
- **Already modeled**: yes for generation (the two writes of a new Swipe state)
- **Risks seen**:
  - `edit` leaves the state describing the old text forever; no marker distinguishes it from an analysed Swipe, so Retrigger and the next generation both read a state that no longer matches the story. **known**

## 8. Movement: map Room vs added Room
- **Where**: `src/domain/model/state/movement.rs:8-11` (`MovementState`), `src/domain/model/utils/game_rules.rs:104-145`
- **State variables**: `current_room_id: String`, `dynamic_rooms: HashMap<String, Room>`
- **States**: standing in a World Room; standing in an added Room; (invalid) standing in an id that is neither
- **Transitions**: a named walk that the map knows (`walk`, `game_rules.rs:129-140`); a walk the map does not know, which invents a Room (`attempt_movement`, `game_rules.rs:114-127`); no destination = no move (`game_rules.rs:109-111`)
- **Guards / invariants**: `current_room_id` must resolve through `SwipeState::room_by_id` (`swipe_state.rs:54-57`). This is checked only under the `diagnostics` feature (`game_rules.rs:223-241`); the non-diagnostics build returns `Ok(())` and the feature is not default (`Cargo.toml:56`). The Job's own `room_check` (`job.rs:183`) is the only production guard
- **Explicitness**: FULLY IMPLICIT (a String naming one of two sources; the fallback branch invents a state)
- **Already modeled**: no
- **Risks seen**:
  - Without `diagnostics`, an inconsistent state (unknown room, unknown Character in `characters_present`, an encounter naming an absent Character) is written and only surfaces on the next Job's room check. **known**
  - `Room::new_dynamic` builds its id from whole seconds (`map.rs:53-62`): two walks to two unknown places inside one second produce the same id, and the second Room overwrites the first in `dynamic_rooms` while `current_room_id` points at the survivor. **known**

## 9. Text check: mode + auto-check
- **Where**: `src/domain/model/settings.rs:298-333` (`TextCheckSettings`), `TextCheckMode` at `settings.rs:13-17`
- **State variables**: `mode: TextCheckMode`, `enable_auto_check: bool`
- **States**: Disabled (auto-check meaningless); Enabled + auto-check on; Enabled + auto-check off; the stored invalid pair (`Disabled` + `true`)
- **Transitions**: `set_mode_and_auto_check` normalises both fields (`settings.rs:326-329`), called from one handler (`http/settings/handlers/settings.rs:196`); the stored invalid pair is repaired at read (`settings.rs:322-333`)
- **Guards / invariants**: a disabled check never accepts auto-check, on read **and** on write
- **Explicitness**: EXPLICIT STATES, SCATTERED TRANSITIONS (one normaliser, but two `pub` fields and a serde path that bypass it)
- **Already modeled**: mostly — this is the best-defended pair in the area
- **Risks seen**:
  - The fields are `pub` and deserialised directly, so the invalid pair remains storable; the repair is at one read site (`effective_enable_auto_check`) which every caller must remember. **inferred**

## 10. Connection slots: a named Connection vs "inherit Narration"
- **Where**: `src/domain/model/settings.rs:458-474` (`AppSettings`), the slot table at `settings.rs:507-538`, `slot_reference` at `settings.rs:621-629`
- **State variables**: `narration_connection_id: String`, `quantifier_connection_id: String` (required), `impersonate_connection_id: Option<String>`, `options_connection_id: Option<String>` (optional), and `connections: Vec<LlmConnection>` whose membership decides validity
- **States**: initialized; narrated; **where** the value is absent — for a required slot, this means the slot does not name an existing Connection (invalid); for an optional slot, it means it falls back to the Narration slot
- **Transitions**: a removal clears the optional slots that name the removed id (`settings.rs:647-676`); an empty slot takes the Narration Connection (`settings.rs:521, 533` → `slot_reference` fallback); a required slot refuses the removal (`settings.rs:654-668`)
- **Guards / invariants**: every slot must resolve (`check_connection_slots`, `settings.rs:638-644`); at least one Connection stays
- **Explicitness**: FULLY IMPLICIT (presence/absence of a String)
- **Already modeled**: no
- **Risks seen**:
  - "empty" and "explicitly the same Connection as Narration" behave identically until the Narration slot moves, then diverge (`settings.rs:621-629`); nothing records which the player meant. **inferred**
  - `slot_reference` falls back to the Narration slot when a *required* slot's closure yields `None`; today the closures always return `Some`, so this fallback is defensive dead weight that would silently serve the wrong Connection if a slot were ever made optional. **inferred**

## 11. Agent: present / disabled / replaced / failed
- **Where**: `src/domain/model/agent.rs:107-128` (`AgentConfig.enabled`), `src/application/agents/runner.rs:29-70`
- **State variables**: the `agents: Vec<AgentConfig>` membership, `enabled: bool`, `AgentRunner.after_narration: Option<..>` / `.end_of_job: Option<..>`
- **States**: runner built from defaults (empty config); Agent absent (not configured, or `enabled = false`); Agent present; Agent replaced by a later config; Agent present but failing at run time
- **Transitions**: empty list → defaults (`runner.rs:33-38`); replace on duplicate (`runner.rs:46-58`, `warn!` only); a failed Quantifier → `keeping_present` (`runner.rs:76-84`); a missing end-of-Job Agent → `Ok(None)` → reported unavailable (`runner.rs:88-96`, `job.rs:358-365`)
- **Guards / invariants**: one Agent per hook
- **Explicitness**: FULLY IMPLICIT (four different "no Agent" outcomes, one `Option` field)
- **Already modeled**: no
- **Risks seen**:
  - A **failed** Quantifier returns the same `QuantifierResult::keeping_present` as a successful "nothing changed" (`runner.rs:81-84` vs `quantifier.rs:16-24`), so the Job cannot report "the Quantifier did not run" and the failure leaves no line. Contrast the Options Agent, which does report `Unavailable`. **known**
  - A duplicate `agent_type` silently replaces the earlier Agent, with a log line and no validation error (`runner.rs:50-58`). **known**

## 12. Mode preset registry: stored bundle vs computed default
- **Where**: `src/domain/model/settings.rs:154-215` (`ModePresetRegistry`), `settings.rs:580-604`
- **State variables**: `Vec<ModePresetBundle>` (possibly partial or empty), the per-mode slot strings
- **States**: mode has a stored bundle ("customised"); mode has none ("never written") and reads the constructed defaults
- **Transitions**: `set_active_preset` writes a whole bundle, materialising the default first (`settings.rs:592-604` + `settings.rs:200-205`)
- **Guards / invariants**: a missing entry must never fail a lookup (`bundle_for` fallback)
- **Explicitness**: EXPLICIT STATES, SCATTERED TRANSITIONS
- **Already modeled**: partly
- **Risks seen**:
  - "no stored bundle" is invisible to the player and to the DB, so a mode whose defaults change in code silently changes its options for every Game that never wrote one. **inferred**
  - A Game copies the three mode-tagged preset ids at switch time (`games/catalogue.rs:169-208`); later Settings changes for that mode never reach an existing Game. **inferred**

## 13. `HistoryCut` two-phase cut
- **Where**: `src/domain/model/history_cut.rs:15-72`, used at `src/application/prompting/assembler.rs:192-217`
- **State variables**: `len`, `kept_from`, `blocks`
- **States**: freshly cut at the token space; cut again to free characters; rendered
- **Transitions**: `HistoryCut::of` (`history_cut.rs:23-45`) → `render_prompt` (`assembler.rs:209`) → optional `drop_to_free` (`history_cut.rs:51-64` / `assembler.rs:213`) → `render_prompt` again
- **Guards / invariants**: `blocks` cover `0..kept_from` with no gap, oldest first (`history_cut.rs:70-72`)
- **Explicitness**: FULLY IMPLICIT (call order required, no type-level phase, no enum)
- **Already modeled**: no
- **Risks seen**:
  - The second cut measures characters while the first measures tokens (`assembler.rs:205-213`); the invariant is a comment only. **known**
  - `drop_to_free` indexes `entry_sizes` up to `self.len` (`history_cut.rs:55-57`), so a shorter slice panics; the one caller passes a same-length slice. **inferred** (API hazard, not reachable today)

## 14. The generation plan (the Job's order)
- **Where**: `src/domain/model/generation.rs:15-36` (`Generation`), `:57-92` (`GenerationOrder`), `:177-196` (`OptionsRun`), `:198-350` (`JobPlan::for_request`)
- **State variables**: `Generation`, `Place`, `OptionsRun`, `SwipeAt`, `StoryEnd`
- **States**: plan for an Action / Guide / Impersonate / Retry / Retrigger / Options; analysed vs not analysed; the last Message moved on
- **Transitions**: one function — `JobPlan::for_request` (`generation.rs:234-283`), with `require_last` (`:285-291`) and `redo` (`:326-345`) as its rules
- **Guards / invariants**: the named Message must still be the last (`PlanError::LastMessageChanged`); a typed Input Retry is not analysed (`generation.rs:86-89`)
- **Explicitness**: EXPLICIT (enum + one transition place)
- **Already modeled**: yes (the Job life, `pipeline/job.rs`)
- **Risks seen**: none seen

## 15. Change counters
- **Where**: `src/domain/model/change.rs:129-192` (`GameVersions`, `ChangeVersions`), impl in `src/adapters/driven/storage/changes.rs:19-82`
- **State variables**: `story`, `options`, `busy_state`, five engine counters, `revision`, `game: Option<ActiveGameVersions>`
- **States**: a Game with no raise yet (zeros) vs a Game with numbers; a snapshot with a Game vs without
- **Transitions**: `raise_game` / `raise_engine` (`changes.rs:27-45`), `forget_game` on delete
- **Guards / invariants**: a number only goes up; every raise raises `revision`; a Game snapshot holds both or neither
- **Explicitness**: EXPLICIT STATES, SCATTERED TRANSITIONS (the invariant lives in one struct, outside the domain type)
- **Already modeled**: no
- **Risks seen**:
  - `forget_game` drops a deleted Game's numbers, so they read as zeros — indistinguishable from "never raised" unless the client also notices `game: null` → `game: <id>`. **inferred**

| name | file | explicitness |
|---|---|---|
| Encounter / presence lifecycle | `src/domain/model/trigger.rs:43` | FULLY IMPLICIT |
| Trigger fire-once lifecycle | `src/domain/model/utils/game_rules.rs:70` | FULLY IMPLICIT |
| Options dock | `src/domain/model/dock.rs:8` | FULLY IMPLICIT |
| Game status Idle/Busy/Failed | `src/domain/model/state/generation_status.rs:8` | EXPLICIT STATES, SCATTERED TRANSITIONS |
| Story end (`Tail`) | `src/domain/model/tail.rs:11` | EXPLICIT STATES, SCATTERED TRANSITIONS |
| Message kind vs type vs flags | `src/domain/model/state/message_types.rs:9` | FULLY IMPLICIT |
| Swipe text vs state | `src/domain/model/message.rs:26` | EXPLICIT STATES, SCATTERED TRANSITIONS |
| Movement: map Room vs added Room | `src/domain/model/state/movement.rs:8` | FULLY IMPLICIT |
| Text check mode + auto-check | `src/domain/model/settings.rs:298` | EXPLICIT STATES, SCATTERED TRANSITIONS |
| Connection slots: empty = inherit | `src/domain/model/settings.rs:458` | FULLY IMPLICIT |
| Agent present/disabled/replaced/failed | `src/domain/model/agent.rs:107` | FULLY IMPLICIT |
| Mode preset registry stored vs default | `src/domain/model/settings.rs:154` | EXPLICIT STATES, SCATTERED TRANSITIONS |
| `HistoryCut` two-phase cut | `src/domain/model/history_cut.rs:15` | FULLY IMPLICIT |
| Generation plan | `src/domain/model/generation.rs:234` | EXPLICIT |
| Change counters | `src/domain/model/change.rs:129` | EXPLICIT STATES, SCATTERED TRANSITIONS |