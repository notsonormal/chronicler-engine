# Scan 2: Domain (src/domain/)

Scope read: every non-test `.rs` file of `src/domain/` (model, state, utils). I followed state writers into `src/application/` and `src/adapters/` only to find the transitions. Epistemic labels: known = I read the code path. inferred = I reasoned from types. unsure = I did not confirm.

Summary: the domain layer holds almost no running state. It holds the data types and the pure rules that the machines elsewhere use. The implicit machines in the domain are mostly encodings of a mode in data: the story end, the opening, the Trigger lifecycle, the Encounter, the dock record, the Message kind and the settings pairs.

---

### Trigger lifecycle (event record, fired mark, event Narration)
- Where: `src/domain/model/state/swipe_state.rs:28` (`SwipeState.event_record`), `src/domain/model/trigger.rs:44` (`Encounter.trigger_fired`), `src/domain/model/state/event_record.rs:17` (`EventRecord`, `TriggerRef.trigger_repeat`).
- State variables: `SwipeState.event_record: Option<EventRecord>`; `Encounter.trigger_fired: HashMap<usize,bool>`; `Trigger.repeat` / `TriggerRef.trigger_repeat`; `times_met`; `Message.kind` (`EventNarration(record)`).
- States (some implied):
  1. not met: the requirement or the room does not match.
  2. met and not fired: matches, `trigger_fired` has no key.
  3. fired, event pending (implied): the Swipe state has `event_record = Some`, `trigger_fired` has no key yet, and the event Narration is not yet saved.
  4. event Narration saved: `trigger_fired[idx] = true` for a one-shot Trigger (nothing for a repeat Trigger).
  5. retriggerable (implied): the last Message is a Narration or an Impersonate Input whose current state holds `event_record`.
- Transitions:
  - Game rules pick the Trigger -> 2 to 3: `game_rules.rs:73-78` (`fired_trigger`, `state.event_record = fired.clone()`).
  - event generation text save -> 3 to 4: `generation.rs:119-124` (`GenerationOrder::write`, `mark_trigger_fired` only when `!trigger_repeat`) -> `trigger.rs:57`.
  - any new Swipe of a non-event kind clears the record: `generation.rs:138-143` (`without_context`).
  - event generation of the Rules: `game_rules.rs:73-78` (`is_event` sets `fired = None`, so the record becomes `None`).
  - Retrigger: `generation.rs:251-259` (`JobRequest::Retrigger`) -> `tail.rs:42-52` (`retrigger_record`).
  - Retry of an event Narration reuses the record of its `MessageKind`: `generation.rs:326-345` (`redo`).
  - Delete of the event Narration returns the story to state 5 (inferred from `tail.rs`; the delete itself is in `gate.rs:556`).
- Guards / invariants: a one-shot Trigger that is already marked fired is skipped (`game_rules.rs:204-210`). The fired mark is written with the event Narration, not with the firing. `event_record` is `Some` only on the Swipe of the generation that fired it.
- Explicitness: FULLY IMPLICIT. The phase is `Option<EventRecord>` plus a `HashMap` key plus the kind of the last Message. The two writes of the mode sit in two files (`game_rules.rs` and `generation.rs`).
- Already modeled: partly. The "two writes of a new Swipe state" cover the firing. The fired mark and the retrigger are not in that list.
- Risks seen:
  - known: if the event generation fails, the one-shot Trigger stays unmarked. The record stays in the Swipe state. The next analysed generation can fire it again, if the requirement still matches. Probably wanted, but nothing writes it down.
  - known: `evaluate_triggers` loops `characters.values()` of a `HashMap` (`game_rules.rs:166-172`) and `fired_trigger` takes `.next()` (`:90`). With two matching Triggers the choice is not deterministic across runs. The prompts sort by id (`character.rs:75`), the Trigger choice does not.
  - known: `trigger_fired` is keyed by the index in the Character's Trigger list. An edit of the World file moves the indexes of saved Games (inferred for the saved-Game case).
  - known: `Encounter.trigger_fired` stores `bool` and the code reads only the key. `false` would also read as "not fired", so two encodings of one fact exist.

### Encounter (per-Character meeting state)
- Where: `src/domain/model/trigger.rs:44` (`Encounter`), `Encounters`.
- State variables: `times_met: u32`, `currently_meeting: bool`, `trigger_fired`; and `SceneState.characters_present` (separate vec).
- States: unseen (no map entry), present and meeting, absent after meeting, plus "entry exists but never met" (implied, made by `mark_trigger_fired` or `set_currently_meeting` through `or_default`, `trigger.rs:58-70`).
- Transitions:
  - scenario start: `swipe_state.rs:75-77` (`times_met = 1`, `currently_meeting = true`).
  - quantifier names a new room: `game_rules.rs:117-119` (`set_currently_meeting(true)` with no `times_met` change).
  - Entered: `game_rules.rs:151-155` (`currently_meeting = true`, `times_met += 1`).
  - Left: `game_rules.rs:157-160` (`currently_meeting = false`).
- Guards / invariants: Entered and Left come from the set difference of the Characters present (`quantifier.rs:49-71`), not from the Quantifier. `currently_meeting` should be true only for a Character in `characters_present`. Nothing checks this: `assert_encounters_consistency` (`game_rules.rs:272`) checks only that the ids are known, and only with feature `diagnostics`.
- Explicitness: FULLY IMPLICIT. A counter and a bool; the transitions sit in three functions in two files.
- Already modeled: no.
- Risks seen:
  - known: `currently_meeting` is written in four places and read nowhere outside its own getter (`rg is_currently_meeting` finds only the definition `trigger.rs:87`). It is a dead state variable.
  - known: the doc comment at `trigger.rs:39` says `times_met` "Increments on first encounter (Entered from !currently_meeting)". The code increments on every Entered (`game_rules.rs:154`), so a Character that leaves and returns counts again. Doc and code disagree.
  - known: `movement()` (`game_rules.rs:117-119`) sets `currently_meeting = true` for Characters of the new result before the Characters present are replaced. A Character that is new in the result gets Entered later and counts. The two places both set the bool, in different order of steps (a comment at `:36-37` says the order matters for `times_met` only).
  - known: the consistency check (`assert_state_consistency`) is a no-op without feature `diagnostics` (`game_rules.rs:235-241`, `Cargo.toml:56`). An unknown id from the Quantifier would then create an Encounter entry. The parser filters ids (`quantifier/utils/parser.rs:36,159`), so this is closed upstream (known).

### The story end (Tail) and the opening
- Where: `src/domain/model/tail.rs:9` (`Tail`), `src/domain/model/generation.rs:206-220` (`StoryEnd`).
- State variables: `Tail.last: Message` (`message_type`, `kind`, `active_swipe_index`), `Tail.state_before_last: Option<(SwipeAt, SwipeState)>`, `Tail.count: usize`, `StoryEnd.tail: Option<Tail>`, `StoryEnd.has_text: bool`, `Swipe.maker` (`WorldOpening`).
- States (all derived, none named): empty story (`tail = None`); opening only (`state_before_last = None`, `count = 1`); last is an Input (awaits narration); last is a Narration or System (ends a cycle); last is an event Narration; last Narration with an event record (retriggerable).
- Transitions (the events that move the story end): a saved text, a Retry (new Swipe), a swipe switch, a delete. The rules that read the state: `tail.rs:42-72` (`retrigger_record`, `retrigger_allowed`, `awaits_narration`, `can_delete`); `generation.rs:166-172` (`next_in_autoplay`); `generation.rs:216-220` (`ends_a_cycle`); `generation.rs:326-345` (`redo`). Writers: `gate.rs:~526,556,891` (not read in depth here).
- Guards / invariants: a Game always has the opening Message, so `tail = None` is a missing-data case (`PlanError::Missing(Record::Story)`, `generation.rs:314`). `can_delete` needs `count > 1`. A Retry names the Message the page showed (`require_last`, `generation.rs:282`).
- Explicitness: FULLY IMPLICIT as one mode. Each rule re-derives the mode from the fields.
- Already modeled: no (the Job and Autoplay machines use it, the story end itself is not modeled).
- Risks seen:
  - known: "the opening" has three encodings: `Maker::WorldOpening` on Swipe 0 (`opening.rs:19`), `state_before_last.is_none()` (`generation.rs:330`, `tail.rs:37-42`), and "first Message". `redo` and `start_at` use the second. A Retry of the opening starts from swipe 0 of the opening itself (`tail.rs:40`). If the first Swipe is swiped away, the start state is still Swipe 0 (known), which is probably wanted.
  - known: `MessageType::System` is never produced by the engine (`rg MessageType::System` shows only matches and a test fixture). `awaits_narration` is false and `ends_a_cycle` is true for it. The variant is a state no transition reaches.
  - inferred: `ends_a_cycle` is true for any non-Input last Message, with no check that a Job ran. The Budget rule (`driver.rs:111`) relies on the Job having saved it.
  - known: `Tail.current_state()` returns `None` when `active_swipe_index` is out of range (`tail.rs:16`). `retrigger_record` then returns `None` silently.

### Message kind and Swipe flags (redundant mode encodings)
- Where: `src/domain/model/message.rs:34` (`Swipe`), `:57` (`Message`), `state/message_types.rs:11,22`.
- State variables: `Message.message_type` (Narration/System/Input) and `Message.kind` (TypedInput/ImpersonateInput/Narration/EventNarration(record)); `Swipe.impersonated: bool`, `guide_text: Option`, `impersonate_direction: Option`, `event_header: Option`, `maker`.
- States: each valid (type, kind) pair; the swipe flags that should agree with the kind (`impersonated` with `ImpersonateInput`, `event_header` with `EventNarration`, `guide_text` with a guided Narration).
- Transitions: created in one place for generations (`generation.rs:97-128`, `GenerationOrder::write`); the flags can be re-written later by `Message::set_stored_inputs` (`message.rs:166-176`); `set_event_header` is test-only.
- Guards / invariants: `message_type` follows from `kind` (Input kinds <-> `Input`, Narration kinds <-> `Narration`). Nothing in the types enforces it. Storage keeps both as columns (known: `Message::new` takes both).
- Explicitness: FULLY IMPLICIT (two parallel encodings, plus flags).
- Already modeled: no.
- Risks seen:
  - known: `Message::new(message_type, kind, swipe)` (`message.rs:67`) accepts any pair, so `(Narration, TypedInput)` is constructible. Inferred: a stored row could hold it; the mapper (`mappers/message.rs:55`) rebuilds both from columns.
  - known: the flags are per Swipe but `kind` is per Message. A Retry of an Impersonate Input makes a new Swipe with `impersonated = true` (`generation.rs:~100-110`, the Impersonate arm of `write`). A Swipe of an event Narration carries `event_header` of its record. A Retry that changes the record kind cannot happen (kind is fixed), so the flags stay in step (inferred).

### Message hydration and the active Swipe index
- Where: `src/domain/model/message.rs:131-163`.
- State variables: `Message.swipes: Vec<Swipe>` (empty after `from_db`), `Message.active_swipe_index: usize`.
- States: hydrating (empty `swipes`, made by `from_db`), valid (index < len), invalid index (index >= len).
- Transitions: `from_db` -> empty (`message.rs:131`); the mapper fills the swipes and then calls `ensure_valid_swipe_index` (`mappers/message.rs:55-72`); `set_active_swipe` silently ignores an out-of-range index (`message.rs:117-122`); `ensure_valid_swipe_index` resets to 0 (`:148-155`).
- Guards / invariants: CONTEXT.md says every Message has at least one Swipe and exactly one is active. A hydrating Message breaks the first rule until the mapper finishes. `from_db` is `pub(crate)`.
- Explicitness: FULLY IMPLICIT (an empty Vec means "not yet loaded").
- Already modeled: no.
- Risks seen:
  - known: with `swipes` empty and index 0, `ensure_valid_swipe_index` resets 0 to 0 and reports `true`. A Message with no Swipe would pass the fix and read as `text() == ""` (`message.rs:96`).
  - known: `update_active_swipe_text` and `set_stored_inputs` do nothing when the index is invalid. A caller gets no signal (`message.rs:124-129,166-176`).

### Dock record (offered Options and the Options failure)
- Where: `src/domain/model/dock.rs:8` (`DockRecord`), `OfferedSet` at `src/domain/model/agent.rs:132`.
- State variables: `DockRecord.offered: Vec<String>`, `DockRecord.failure: Option<OptionsFailure>`; storage: row of `game_docks` with `offered` and `notice` columns (`transaction.rs:307-345`, `storage/docks.rs:39`).
- States: no row (never offered); cleared (empty `offered`, no failure); offered (non-empty `offered`, no failure); failed (`failure = Some`); and the combination "offered AND failed" (implied).
- Transitions: `put_dock` replaces both (`transaction.rs:300`); `put_options_failure` replaces the failure and keeps `offered` (`transaction.rs:307-317`); `clear_dock` writes the default record (`:322-326`); a Narration save empties the dock (`agent.rs:75-77`, doc of `saves_a_narration`; the write is in `gate.rs`).
- Guards / invariants: the comment at `dock.rs:11` says the failure "stands in for the offered set". The view shows the failure first and hides the offered list (`templates.rs:64`).
- Explicitness: FULLY IMPLICIT (a Vec plus an Option, two columns).
- Already modeled: no (the Options run in the Job is modeled, the record it writes is not).
- Risks seen:
  - known: `put_options_failure` keeps the old `offered` set, so "offered AND failed" is reachable. The view hides the set, but `GET` of the dock still returns both (`app_state.rs:147` passes both).
  - known: no row and the cleared row read the same (`docks.rs` uses `unwrap_or_default`, `view_query.rs:~110`).

### Generation status and the failure line
- Where: `src/domain/model/state/generation_status.rs:9` (`GenerationStatus`), `:226` (`Step`), `src/domain/model/game.rs:44` (`Game.failure`).
- State variables: `GenerationStatus {Idle, Busy, Failed(GenerationFailure)}`, `Step`, `Game.failure: Option<GenerationFailure>` (stored column).
- States: Idle, Busy (with an optional Step), Failed.
- Transitions: Busy comes from the gate registry; the failure line is saved at `gate.rs:828` and cleared at `gate.rs:841`; the view joins them at `view_query.rs:111-125` (`Status`, busy wins over a stored failure).
- Explicitness: EXPLICIT STATES, SCATTERED TRANSITIONS. The domain enum `GenerationStatus` is built only for the debug DTO (`view_query.rs:128-134`). The real status enum is `Status` in `application/games/view_query.rs:32`. Two enums name the same three states.
- Already modeled: yes (gate, Job).
- Risks seen:
  - known: duplication of `GenerationStatus` (domain) and `Status` (application). Only the second drives behaviour.
  - known: `Step` and `Kind::step()` (`agent.rs:49-54`) map `InputRetry` to `Narrating`, so a Retry of a typed Input shows "narrating" although no model writes it. The same code path says so in `Kind` docs, so this is a wording matter.

### Generation order and plan (the shape of one request)
- Where: `src/domain/model/generation.rs:20-75` (`Place`, `Generation`, `GenerationOrder`), `:178-260` (`OptionsRun`, `JobPlan`).
- State variables: `Place {NewMessage, NewSwipeOnLast}`, `Generation {Narration, Impersonate, InputRetry, Event}`, `JobPlan.input: Option<String>`, `JobPlan.first: Option<GenerationOrder>`, `JobPlan.options: OptionsRun`, `OptionsRun::follows(narration_saved, entry_moved)`.
- States: the valid (Place, Generation) pairs: Narration/Impersonate/Event x NewMessage; Narration/Impersonate/Event/InputRetry x NewSwipeOnLast. `InputRetry` only with `NewSwipeOnLast` (implied). `JobPlan.first = None` only with `OptionsRun::OnDemand` (implied).
- Transitions: built in one place, `JobPlan::for_request` (`generation.rs:234-305`); later steps follow in `pipeline/job.rs` (already modeled).
- Guards / invariants: `first == None` iff the request is `/options`. `input = Some` only for a non-empty `Action`.
- Explicitness: EXPLICIT (enum + one plan place), but the struct allows invalid combinations.
- Already modeled: yes (the Job steps), the plan rules are pure.
- Risks seen:
  - known: types allow `(InputRetry, NewMessage)` and `JobPlan{first: None, options: AfterNarration}`. Only `for_request`, `of_one` and `event_after` construct them, so it is closed in practice (known).
  - inferred: `is_analysed()` (`generation.rs:86`) is a mode flag in disguise: `InputRetry` ends after its text save. The Job's end rules read it elsewhere.

### History cut (cut builder)
- Where: `src/domain/model/history_cut.rs:16` (`HistoryCut`).
- State variables: `len`, `kept_from`, `blocks: Vec<HistoryBlock>`.
- States: no cut (`kept_from = 0`, no blocks); cut by size (`of`); cut again to free more (`drop_to_free`).
- Transitions: `HistoryCut::of` (`:22-45`) builds the first blocks; `drop_to_free` (`:49-60`) adds one more block and moves `kept_from`.
- Guards / invariants: blocks cover `0..kept_from` with no gap (doc at `:66`). The call order is `of` first, then any number of `drop_to_free`.
- Explicitness: FULLY IMPLICIT (counters; a required call order).
- Already modeled: no.
- Risks seen:
  - inferred: `drop_to_free` indexes `entry_sizes[self.kept_from]` (`:53`). A slice shorter than `len` panics. The type does not tie the two slices to `len`.
  - known: the two methods do not agree on empty blocks: `of` can only push non-empty blocks when `kept_tokens > history_space`; `drop_to_free` checks `kept_from > block_start`. Both keep the "no gap" rule.

### Game config write kinds and mode-bound presets
- Where: `src/domain/model/game.rs:112` (`ConfigChange`), `game.rs:25-45` (`Game` preset ids, `narrator_mode`), `src/domain/model/settings.rs:158-215` (`ModePresetRegistry`), `prompt_preset.rs:126` (`allows`).
- State variables: `Game.narrator_mode`, four `active_*_prompt_preset_id`, `narrative_perspective`, `narrative_tense`, `PromptPreset.allowed_modes`; `ModePresetRegistry(Vec<ModePresetBundle>)` (a missing mode means "use the constructed default").
- States: one mode (Novel / InteractiveFiction) with the three mode-tagged presets of its bundle; Options preset is mode-free.
- Transitions: `ConfigChange::Mode` sets mode, three presets and perspective (`game.rs:140-149`); `Posture` sets perspective and tense (`:136-139`); `Presets` sets four ids (`:150-154`). The save writes only the named columns (`transaction.rs:~255-292`).
- Guards / invariants: the preset of a slot must allow the mode of the Game (`allows`). The `Presets` change does not say so in its type.
- Explicitness: EXPLICIT STATES (`ConfigChange`), SCATTERED TRANSITIONS (the check of `allows` is in the application layer, not read here).
- Already modeled: no (the config write race with the busy registry is mentioned in the gate scope; this part is the data rule).
- Risks seen:
  - inferred: `Presets` can write a preset that does not allow the current mode. I did not read the service that validates it, so I mark it unsure.
  - known: `ModePresetRegistry::bundle_for` and `active_preset_id` fall back to defaults when a mode is missing (`settings.rs:200-204,580-589`). A partial list is a valid, different state from the full default list.

### Connection slots (settings)
- Where: `src/domain/model/settings.rs:458-478` (`AppSettings`), `:489-536` (`SlotKind`, `CONNECTION_SLOTS`), `:604-678`.
- State variables: `narration_connection_id`, `quantifier_connection_id` (required), `impersonate_connection_id: Option`, `options_connection_id: Option` (None = use Narration), `connections: Vec`.
- States: slot empty (falls back), slot set, slot dangling (names no Connection).
- Transitions: `remove_connection` refuses `NotFound`, `LastConnection`, `InUse(roles)` before any change, then clears optional slots that name it (`:647-678`); settings writes by the service (not in domain).
- Guards / invariants: required slots always name a stored Connection; `check_connection_slots` (`:638`) tests it.
- Explicitness: EXPLICIT (one table, one removal function).
- Already modeled: no, but it is small and guarded.
- Risks seen:
  - known: the fields are `pub`, so a write can leave a dangling slot. Only `connection_for` and `check_connection_slots` detect it, as a `Config` error.
  - known: `LlmConnection.pin_provider_order = true` with an empty `provider_order` is meaningless (`settings.rs:362-364`); nothing rejects it (inferred: no validation seen in the domain).

### Text-check settings pair
- Where: `src/domain/model/settings.rs:299-335` (`TextCheckSettings`).
- State variables: `mode: TextCheckMode`, `enable_auto_check: bool`.
- States: invalid pair `(Disabled, true)`; others valid.
- Transitions: `set_mode_and_auto_check` normalises (`:326-329`); `effective_enable_auto_check` normalises on read (`:322-324`).
- Explicitness: FULLY IMPLICIT (two fields), but guarded by one rule in two methods.
- Already modeled: no.
- Risks seen: known: both fields are `pub`, so a caller that writes them directly can store the invalid pair. The read path hides it.

### Dynamic Rooms (added Rooms of a Game)
- Where: `src/domain/model/state/movement.rs:9`, `src/domain/model/map.rs:52` (`Room::new_dynamic`), `game_rules.rs:104-145`.
- State variables: `MovementState.current_room_id: String`, `MovementState.dynamic_rooms: HashMap<String, Room>`.
- States: in a map Room; in an added Room; in a Room that neither holds (invalid, checked only under `diagnostics`, `game_rules.rs:245-253`).
- Transitions: destination in map or added Rooms -> `walk` (`game_rules.rs:136-145`); unknown destination -> new Room inserted and entered (`:124-134`).
- Guards / invariants: `current_room_id` must resolve through the map or `dynamic_rooms`.
- Explicitness: FULLY IMPLICIT (a string id plus a map).
- Already modeled: the write of the Swipe state is modeled; this mode is not.
- Risks seen:
  - known: the id is `dynamic_{unix seconds}` (`map.rs:52-59`). Two unknown destinations in one second map to the same key; the second replaces the first and both Swipes point at one id.
  - known: `walk` looks up by id (`room_by_id`). A later destination by the name of an added Room does not match its id, so a new added Room is made each time (inferred from `walk`, since the Quantifier prompt may give ids; I did not check what it names).
  - known: with no `diagnostics`, a missing current Room is not detected by the rules.

### Counters of the changes (snapshot only)
- Where: `src/domain/model/change.rs:111-160` (`GameVersions`, `ChangeVersions`).
- State variables: `story`, `options`, `busy_state`, five engine numbers, `revision`, `server` id, `game: Option<ActiveGameVersions>`.
- States: with an active Game (all Game numbers) or none (`game = None`).
- Transitions: raised in storage (`adapters/driven/storage/changes.rs`, `write_guard.rs`); not in the domain. Not read here (unsure about the details).
- Guards / invariants: a number only goes up (`change.rs:129`); a restart changes the server id; "a snapshot holds both or neither" (`change.rs:~103`).
- Explicitness: EXPLICIT data, transitions in another area (storage).
- Already modeled: unsure.
- Risks seen: none seen in the domain types.

### Story window room headers (fold)
- Where: `src/domain/model/story_window.rs:23-38`.
- State variables: local `before: Option<&String>` in the fold.
- States: previous entry had a name / had none.
- Transitions: per entry, shown when name differs from `before`; `before` is reset to `None` after an unnamed entry.
- Explicitness: pure function with a local fold state. Not an implicit machine; listed for completeness.
- Already modeled: n/a.
- Risks seen: none seen. Known: after an unnamed entry the next named entry always shows, even with the same name as the one before the gap; the doc comment says so.

---

## Checked and not counted (no time-varying mode)
`action.rs` (`Action::parse` is pure), `quantifier.rs` (`from_diff` is pure), `prompt_part.rs`, `llm_message.rs` (`sent_parts` falls back from the new column to the old two columns, a data-format fallback with no state), `llm_backend.rs`, `character.rs`, `world.rs`, `scenario.rs`, `template.rs`, `generation_parameters.rs`, `saved_games_page.rs`, `utils/game_name.rs`, `utils/name_uniqueness.rs`, `utils/xml.rs`, `utils/*_defaults.rs`, `utils/opening.rs`, `dock.rs` struct `OptionsFailure`, `prompt_preset.rs` (`validate` guards `option_count` only on Options presets, `:103-121`; `Option` fields mean "part included").
There is no `static`, `Mutex`, `Atomic`, `Drop` or retry loop in `src/domain/`. (known: `rg` over the folder).

## Table of contents
name | file | explicitness
- Trigger lifecycle | `model/state/swipe_state.rs`, `model/trigger.rs`, `model/generation.rs`, `model/utils/game_rules.rs` | FULLY IMPLICIT
- Encounter | `model/trigger.rs`, `model/utils/game_rules.rs` | FULLY IMPLICIT
- Story end (Tail) and the opening | `model/tail.rs`, `model/generation.rs` | FULLY IMPLICIT
- Message kind and Swipe flags | `model/message.rs`, `model/state/message_types.rs` | FULLY IMPLICIT
- Message hydration / active Swipe index | `model/message.rs` | FULLY IMPLICIT
- Dock record | `model/dock.rs` | FULLY IMPLICIT
- Generation status / failure line (already modeled) | `model/state/generation_status.rs`, `model/game.rs` | EXPLICIT STATES, SCATTERED TRANSITIONS
- Generation order and plan (already modeled) | `model/generation.rs` | EXPLICIT
- History cut | `model/history_cut.rs` | FULLY IMPLICIT
- Game config write kinds / mode-bound presets | `model/game.rs`, `model/settings.rs` | EXPLICIT STATES, SCATTERED TRANSITIONS
- Connection slots | `model/settings.rs` | EXPLICIT
- Text-check settings pair | `model/settings.rs` | FULLY IMPLICIT
- Dynamic Rooms | `model/state/movement.rs`, `model/map.rs`, `model/utils/game_rules.rs` | FULLY IMPLICIT
- Change counters (snapshot) | `model/change.rs` | EXPLICIT (transitions elsewhere)
- Story window room headers | `model/story_window.rs` | pure fold, not a machine
