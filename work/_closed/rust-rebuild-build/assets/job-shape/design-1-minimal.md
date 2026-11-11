# Design 1: the minimal interface (ticket 20)

Constraint of this draft: the fewest entry points and types. Claims are **known** (read in code on 2026-10-20) or **inferred**. This draft builds on the ticket 19 interfaces (`Generation`, `GenerationOrder::{kind, event_after, write}`, `game_rules::apply`, `RulesOutcome`).

## 1. Interface

### 1.1 The Job (`src/application/pipeline/job.rs`): 1 entry point

```rust
impl Job {
    pub fn start(writer: &GameWriter, deps: JobDeps, game: GameId, request: JobRequest) -> Result<(), Refusal>;
}
```

No change to the signature (known: the HTTP handlers call only `start`). Everything else in the file is private.

### 1.2 The failure type: 1 type, 3 entry points

`JobFailure` replaces `Failure`, `Ended` and `Stopped`. It lives in `gate.rs`, because the writer makes `EntryMoved`.

```rust
pub enum JobFailure {
    /// The job entry no longer holds this Job's id (stop, reset, delete). No write follows.
    EntryMoved,
    /// The failure line that the Job writes at its end.
    Line(GenerationFailure),
}
impl JobFailure {
    /// The one read rule (Q2). `Ok(None)` and `Err` both give
    /// "The data for this request is missing: <record>."; the error goes in the disclosure.
    pub fn need<T>(record: Record, read: Result<Option<T>, EngineError>) -> Result<T, JobFailure>;
    /// The one event wrap: for `Kind::Event`, a `Line` becomes "The event narration failed."
    /// with the inner line in the raw text. Other kinds and `EntryMoved` pass unchanged.
    pub fn of(self, kind: Kind) -> JobFailure;
}
impl From<GenerationFailure> for JobFailure { /* Line */ }
// No `From<EngineError>`: a read must name its record, or it does not compile.
```

In the domain (`generation_status.rs`), one fixed list and three constructors:

```rust
pub enum Record { Game, Persona, World, Settings, Story, StartState, Message, Preset }
impl Record { pub fn name(self) -> &'static str; }  // "the game", "the persona", "the World", ...
impl GenerationFailure {
    pub fn missing_record(record: Record, error: Option<&EngineError>) -> Self; // takes `Record`, not `&str`
    pub fn save_failed(error: &EngineError) -> Self;    // only a failed write
    pub fn contradiction(error: &EngineError) -> Self;  // the Game rules' `Err`
}
```

`accept` and `JobFailure::need` both call `missing_record`, so the line rule has one home.

### 1.3 The reads: 2 types, 1 entry point (`src/application/narration/reads.rs`)

```rust
/// What a hook reads. The Job reads it once, before the hook.
pub struct HookReads {
    pub world: WorldBundle,       // moved to src/domain/model/world.rs
    pub game: Game,               // perspective, tense, preset slots, Options toggle
    pub state: SwipeState,        // the start state; the current state for the Options run
    pub tail: Tail,               // the place of a Retry, the user regen text, the opening fact
    pub story: Vec<MessageEntry>, // the window that ends at the place of the new text
}
impl HookReads {
    /// The one builder of `HookInput`, for the after-narration and end-of-Job hooks.
    /// Story: the last `STORY_WINDOW` entries. `player_text`: the last entry when `text` is set, else "".
    pub fn with_hook<T>(&self, text: Option<&str>, run: impl FnOnce(&HookInput<'_>) -> T) -> T;
}
/// What one generation reads (Q5). The Job reads it once per generation.
pub struct GenerationReads {
    pub hook: HookReads,
    pub room: Room,               // the room check passed: the start state's room (`SwipeState::room`)
    pub preset: PromptPreset,     // the Game's preset of slot `generation.preset_type()`
    pub budget: TokenBudget,      // from one settings read
    pub response_length: String,  // from the same settings read
    pub quantifier: AgentSettings,
}
```

The closure in `with_hook` exists because `HookInput` borrows a `WorldView` and a `[StoryEntry]` (known: `agent.rs` 116 to 134). The builder owns them for the call. `HookInput` keeps its fields, less `kind` and `event` (ticket 19).

### 1.4 The narrator (`src/application/narration/narrator.rs`): 1 call entry point

```rust
impl Narrator {
    pub fn narrate(&self, generation: &Generation, reads: &GenerationReads) -> Result<String, GenerationFailure>;
    // kept: new, backend_info (debug page), recorder (wiring tests). `assemble` becomes private.
}
impl Generation { pub fn preset_type(&self) -> PresetType; } // Impersonate → Impersonate; else System (domain)
```

The narrator owns every prompt rule of a kind:

| Kind | User message | Flag | Health role |
|---|---|---|---|
| Narration | the last story entry, if it is an Input; else "" | guide | Narrator |
| Impersonate | the direction, or "" | Impersonate | Narrator |
| UserRegen | `reads.hook.tail.last.text()` | — | Narrator |
| Event | the continuation text (last entry + the record's direction) | — | Trigger |

**Open detail of ticket 19 (user regen text): from the bundle.** The Tail is in the bundle already, and §4.5 asks for "the text of the active Swipe as it is now". So `Generation::UserRegen` keeps "no data".

The Narration row needs no `input` argument: the Job adds the saved Input to `reads.hook.story` (§2.1). Then "the Input text, else the previous Input" (today `user_message` + `previous_input`) is one rule: the last entry, if it is an Input (inferred: same text in every case).

### 1.5 The save units (`JobWriter`, `gate.rs`): 5 entry points, all take values

```rust
impl JobWriter {
    pub fn step(&self, step: Step) -> Result<(), JobFailure>;
    /// The Input save and the text save. `place`/`tail` give the new Message, or the next Swipe of
    /// `tail.last`. The dock empties when the Message kind is a Narration or an event Narration.
    pub fn save_swipe(&self, swipe: NewSwipe, place: Place, tail: &Tail, room_report: Option<String>) -> Result<SavedText, JobFailure>;
    pub fn save_state(&self, saved: SavedText, state: SwipeState, room_report: Option<String>) -> Result<(), JobFailure>;
    /// `Offered`: the set, no notice. `Unavailable`: the notice only (`Tx::set_notice`), so it reads no set.
    pub fn save_options(&self, result: OptionsResult) -> Result<(), JobFailure>;
    /// Writes the failure line of a `Line`, logs an `EntryMoved`. Takes `self`: no write follows.
    pub fn end(self, failure: Option<JobFailure>);
}
```

Invariants (known: today's guard; kept):
- Every method checks the job id under the guard. A late write gives `EntryMoved`.
- A unit error becomes `Line(GenerationFailure::save_failed(&e))` at one site (`JobWriter::unit`). Only a write gives "The new text could not be saved."
- `save_state` takes `SavedText` by value, so a Swipe's state is written at most twice.
- 5 entry points is above the aim of 3. I found no merge that keeps the by-value `SavedText` rule and the separate Input unit (§4.4).

**Q3 check: nothing else can change the story while the Job holds its entry (known).** In `src/`, only `gate.rs` opens a unit (`Units::open`, `open_new_game`); `test_app_builder.rs` is test code. `accept`, `switch_swipe`, `edit` and `delete_last` call `require_idle`. `stop`, `replace` and `delete` remove the entry, so the Job's next write gives `EntryMoved`. `create` writes a new Game. The Game row and the settings **can** change during a Job (preset and settings handlers do not go through the guard). So the Job reads them per generation, not per Job (`races.rs` `test_the_event_narration_call_uses_the_preset_active_when_it_runs`).

### 1.6 Storage errors (Q4): 1 reshaped variant

```rust
#[error("Database error: {context}: {source}")]
Database { context: &'static str, source: rusqlite::Error },   // replaces `Database(#[from] rusqlite::Error)`
impl EngineError { pub fn db(context: &'static str) -> impl FnOnce(rusqlite::Error) -> Self; }
// .map_err(EngineError::db("read the story entries"))?
```

- Known: 95 `EngineError::Config(format!(` sites under `storage/` (`plumbing.rs` 30, `stories.rs` 15, `transaction.rs` 14, ...). Inferred: about 7 wrap a serde or datetime error. Those use the existing `Serde` and `Parse` variants. The rest use `db`.
- `Config` stays for settings with no valid value.
- **What `from_engine_error` still classifies:** only the model call and the prompt assembly (the narrator is its only caller, known). Kept arms: `Unreachable`, `PromptTooLong`, `UnreadableAnswer`, `TokenBudgetSpent`, `_ => Other`. Deleted arms: `Database | Io => SaveFailed` (a write names it at the call site), `RoomNotFound` (the read names the room), `PresetNotFound` (the preset read names it).

## 2. Usage

### 2.1 `Job::run`: one loop, one fact, one wrap, one exit

```rust
fn run(self, plan: JobPlan) {
    let JobPlan { input, first, options } = plan;
    let mut next = first.map(|order| (order, input));
    let mut narration_saved = false;               // the one home of the fact
    let mut failure = None;
    while let Some((order, input)) = next.take() {
        match self.generate(&order, input, &mut narration_saved) {
            Ok(event) => next = event.map(|order| (order, None)),
            Err(f) => failure = Some(f.of(order.kind())),   // the one event wrap
        }
    }
    if options_follow(options, narration_saved, failure.as_ref()) {
        if let Err(f) = self.options_run(options) {
            match failure { None => failure = Some(f), Some(_) => tracing::warn!(?f, "Options run after a failure") }
        }
    }
    self.writer.end(failure);                       // the one exit
}
/// Q1. OnDemand: always (no generation runs). EntryMoved: never. Else: when a Narration was saved.
fn options_follow(run: OptionsRun, narration_saved: bool, failure: Option<&JobFailure>) -> bool;
```

### 2.2 One generation

```rust
fn generate(&self, order: &GenerationOrder, input: Option<String>, narration_saved: &mut bool)
    -> Result<Option<GenerationOrder>, JobFailure> {
    let kind = order.kind();
    self.writer.step(kind.step())?;
    let mut reads = self.read_generation(order)?;  // every read once, then the room check
    if let Some(text) = input {                    // a plan with an Input is always `NewMessage`
        self.writer.save_swipe(NewSwipe::typed_input(&text, &reads.hook.state), Place::NewMessage, &reads.hook.tail, None)?;
        reads.hook.story.push(MessageEntry::input(text));
    }
    let text = self.narrator.narrate(order.generation(), &reads)?;
    let opening = order.place() == Place::NewSwipeOnLast && reads.hook.tail.count == 1;
    let (swipe, pre) = order.write(text.clone(), &reads.hook.state);
    let saved = self.writer.save_swipe(swipe, order.place(), &reads.hook.tail, opening.then(|| reads.room.name.clone()))?;
    *narration_saved |= kind.saves_a_narration();
    let Some(pre) = pre else { return Ok(None) };  // a user regen ends here (R-18)
    self.writer.step(Step::Quantifying)?;
    let result = reads.hook.with_hook(Some(&text), |hook| self.agent_runner.analysis(hook, &reads.quantifier));
    let outcome = game_rules::apply(pre, &result, &reads.hook.game_world())
        .map_err(|e| GenerationFailure::contradiction(&e))?;
    let at = saved.at();
    self.writer.save_state(saved, outcome.state, outcome.room_report /* + the opening rule, ticket 25 */)?;
    Ok(outcome.fired.map(|record| GenerationOrder::event_after(at, record)))
}
```

`read_generation` is one private fn. Each read calls `JobFailure::need(Record::.., ..)`. The room check is its last line (`state.room(map)` or `missing_room`), so a bad room and a failed read both save no Input. `options_run` reads one `HookReads` (window 4, the current state) and calls `with_hook(None, ..)`.

### 2.3 `accept`

`missing` and `unreadable` merge into one `fn need<T>(&self, game, record: Record, read, units) -> Result<T, Refusal>`. It calls `missing_record` and `refuse`. `save_failed` stays for the accept unit. The plan comes from `JobPlan::for_request` (ticket 19).

## 3. What hides behind the seam (deleted or merged)

| Where | Deleted or merged | Count |
|---|---|---|
| `job.rs` types | `Failure`, `Ended`, `Generated`, `RuleOutcome` | 4 |
| `job.rs` impls | `Failure::missing`, `event_narration_failed`, `From<Stopped>`, `From<EngineError>`, `From<Failure> for Ended`, `after_text_save`, `into_event_failure` | 7 |
| `job.rs` exits and steps | `failed`, `run_step`, `end`, `log_refusal`, `event_order`, `scene_kind`, `contradiction`, `room_name`, `player_text` | 9 |
| `job.rs` reads → `read_generation`, `read_hook` | `world_bundle`, `game_row`, `tail`, `start_state`, `preset`, `budget`, `options_enabled`, `analysis_settings`, `options_settings`, `preset_text`, `window_before`, `place_before`, `is_the_opening`, `room_check`, `resolve_room`, `room_of` | 16 → 2 |
| `job.rs` → narrator | `narrate_step`, `user_message`, `continuation_user_message`, `previous_input`; the `PromptContext` build and the two flag branches | 4 fns, 2 branches |
| `job.rs` | `analyse_step` and the second `WorldView`/`HookInput` build in `options_step` → `with_hook` | 2 → 1 |
| The flag | `saved_a_narration`, `Ended.narration_saved`, `after_narration: bool` | 3 → 1 |
| The event wrap / the exits | 2 sites → 1; `run` + `failed` → `end` | 2 → 1, 2 → 1 |
| `gate.rs` | `Stopped` (2 variants, `From<EngineError>`); `state_at`, `tail_of`, `opening_room_report`, `dock`; `save_input` + `save_text` → `save_swipe`; `save_failure` → `end`; `missing` + `unreadable` → `need` | 1 type, 4 reads, 3 merges |
| `narrator.rs` | `Narrated`; `narrate` goes from 5 parameters to 2; `assemble` becomes private | 1 type |
| `storage/games.rs` | `active_quantifier_preset_id`, `active_options_preset_id`, `preset_id_or` (known: the Job is their only caller) | 3 |
| `generation_status.rs` | 3 arms of `from_engine_error` | 3 |
| `storage/**` | the hand wraps of rusqlite errors → `EngineError::db` | ~88 |

Read counts per generation (today, from the brief → this design): settings 4 → 1, Game row 2 → 1, Tail up to 4 → 1, start state 2 → 1. The Options run adds 1 World read, 1 Game row read and 1 Tail read per Job.

## 4. Where each piece lives

| Piece | File | `arch-lint.toml` |
|---|---|---|
| `Job`, `run`, `generate`, `options_run`, `read_generation`, `read_hook`, `options_follow` | `src/application/pipeline/job.rs` | application; may import storage |
| `JobFailure`, `JobWriter` | `src/application/generation/gate.rs` | same |
| `HookReads`, `GenerationReads` | `src/application/narration/reads.rs` (new) | agent-edge: imports domain and `prompting::TokenBudget` only, no storage |
| `Narrator::narrate` | `src/application/narration/narrator.rs` | agent-edge |
| `WorldBundle` (moved) | `src/domain/model/world.rs` | model; storage → domain is allowed. Needed, because agent-edge may not import `storage/worlds.rs` |
| `Record`, `missing_record`, `save_failed`, `contradiction` | `src/domain/model/state/generation_status.rs` | model |
| `Generation::preset_type`, `NewSwipe::typed_input` | `src/domain/model/generation.rs` | model |
| `Tx::set_notice`, `TxWrite::SetNotice` | `src/adapters/driven/storage/transaction.rs` | storage |
| `EngineError::Database { context, source }`, `EngineError::db` | `src/error.rs` | known: already imports rusqlite |

## 5. Tests that pin the interface

| CASE id | Tier | Asserts |
|---|---|---|
| `failure-need-none`, `failure-need-error` | unit, `gate_tests.rs` | `Ok(None)` and `Err` give the same line with the record name; `Err` puts the error text in `raw` |
| `failure-of-event` | unit | `Line` of an event: "The event narration failed.", the inner line in `raw`; `EntryMoved` and other kinds pass |
| `options-follow` (6 rows) | unit, `job_tests.rs` | on demand; after a Narration with no failure, a `Line`, `EntryMoved`; no Narration with a failure; no Narration with no failure |
| `record-names` | unit, `generation_status_tests.rs` | each `Record` renders its fixed name in the line |
| `db-error-context` | unit, storage | a failed query gives `Database { context, .. }`, and `from_engine_error` no longer sees it |
| `narrator-user-message-by-kind` (4 rows) | seam contract, `narrator_tests.rs` | the table of §1.4: the message, the flag and the role per `Generation` |
| `narrator-regen-text-from-tail` | seam contract | a user regen sends `tail.last` text, with a window that ends before it |
| `hook-input-window` | unit, `reads_tests.rs` | `with_hook` gives the last 4 entries, the room by `SwipeState::room`, and `player_text` by the §1.3 rule |
| `notice-keeps-set` | unit, storage | `Tx::set_notice` keeps the offered set |
| `options-after-state-save-failed` | tier 1, `tests/http/options.rs` (new) | inject `TxWrite::SetSwipeState`: the status shows "The new text could not be saved.", and the dock shows the offered set |
| `options-after-event-text-save-failed` | tier 1 (new) | inject the event's `InsertMessage`: "The event narration failed.", the Options run ran |
| `options-failure-after-failure-logged` | tier 1 (new) | the status shows the first failure, not the Options failure |
| `job-read-names-record` (rows: World, settings, preset read, story) | tier 1, `failure_display.rs` (new) | an injected read error gives "The data for this request is missing: <record>.", the error in the disclosure, and no Input |
| storage guardrail | `tests/guardrails` | no `EngineError::Config(format!` with a rusqlite error under `storage/` |

Existing tier-1 tests this design touches (they must stay green): `options.rs` `test_options_run_follows_a_failed_event_narration_http`, `test_failed_regeneration_keeps_prior_set_http`, `test_failed_always_on_regeneration_clears_set_and_reports_http`; `failure_display.rs` `test_save_failure_clamps_with_the_save_line_and_saves_no_narration`, `test_contradiction_clamps_with_its_own_line_and_keeps_the_text`, `test_missing_preset_failure_clamps_with_the_preset_line`; `actions.rs` `test_trigger_narration_failure_preserves_main_sets_error_http`, `test_missing_persona_answers_500_with_the_missing_data_line_http`; `retrigger.rs` `test_retrigger_trigger_narration_failure_sets_error_http`; `swipe_new.rs` `test_event_retry_trigger_narration_failure_sets_error_http`, `test_event_retry_keeps_the_room_name_off_the_event_swipe_http`; `races.rs` `test_a_stop_at_each_model_call_keeps_the_records_of_that_step`, `test_the_event_narration_call_uses_the_preset_active_when_it_runs`. All of `narrator_tests.rs` changes its call shape (known: it builds `PromptContext` and imports `Narrated`).

## 6. Trade-offs

**High leverage.**
- `JobFailure::need` with no `From<EngineError>`: the compiler makes every read name its record. That fixes FINDINGS 2 in one place.
- The narrator call goes from 5 parameters and 4 kind branches in the Job to `narrate(&generation, &reads)`. The prompt rules are tested at the seam with no storage.
- The loop holds the fact, the wrap and the exit once each. `options_follow` is a pure fn, so the Q1 table is a unit test.
- The save units read nothing. `Tx::set_notice` removes the `after_narration` flag.

**Thin.**
- `JobWriter` has 5 entry points, not 3.
- `GenerationReads` and `HookReads` are wide structs with `pub` fields. They carry data, not behaviour; their depth is low.
- `with_hook` takes a closure because of the `HookInput` lifetimes. A `HookInput` that owns its window would remove it, but that changes the Agent edge.
- `narration_saved` is a `&mut bool` out-parameter. It is one home, but not a type.
- The in-memory Input push and the Tail that is stale after the Input save are safe only while a plan with an Input is always `NewMessage` (ticket 19 plan rules; inferred). A `debug_assert!` should pin it.

**Behaviour changes (inferred; no tier-1 test pins them now).**
- A settings or Game row read error now comes before the Input save, so it saves no Input. Today it saves the Input, then fails.
- A narration connection with no valid value shows "The data for this request is missing: the settings." Today it shows "missing: Configuration error: ...".

**Left open.**
- The opening room report of the state update stays in the Job beside `RulesOutcome.room_report` until ticket 25.
- A model call `Config` error (for example a provider that did not build) still maps to `Other`. It may need its own line.
- The Options run does its own read of `HookReads` and does not reuse the last generation's bundle. It reads the stored truth and the toggle as it is at the end of the Job, at the cost of 3 more reads per Job.
