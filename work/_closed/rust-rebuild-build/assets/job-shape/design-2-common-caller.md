# Design 2 (common caller): the shape of the Job — ticket 20

Design constraint: `Job::run` and one generation read as one short loop and one short fn. The types behind them may grow. All claims *known* are read in `src/` on 2026-10-20; the rest are *inferred*.

## 1. Interface

### 1.1 The kind carries the regenerated text (the open detail, decided)

`Generation::UserRegen` takes the text it regenerates: `UserRegen { text: String }`. Reasons: ticket 19's own principle 1 is "the kind carries its data" (an Impersonate its direction, an event its record), and this text is the same class; `JobPlan::for_request` reads the Tail at accept and the Tail holds the last Message's text (*known*: `gate.rs` → `Tail::redo`), so no new read; the order keeps "ids and texts only". The alternative (a bundle field `regenerated_text: Option<String>`) is `None` for three of four kinds.

### 1.2 The run types (`src/application/pipeline/job.rs`)

```rust
/// What one generation left behind when it ended whole.
struct Generated {
    at: SwipeAt,
    fired: Option<EventRecord>,     // the Job builds the event order from it
    narration: bool,                // kind.saves_a_narration()
    scene: Scene,                   // handed on to the Options run
}
/// Where a failed generation stopped. Positional only: the loop owns what landed.
enum Stop { BeforeText, AfterText }
struct Failed { failure: Failure, stop: Stop }
impl Failed {
    /// The ONE home of the event-line wrap and of the stop mark. An event
    /// generation's failure names the event wherever it stops (design §4.3).
    fn new(failure: Failure, kind: Kind, stop: Stop) -> Self;
    /// Whether this failure's generation had saved a Narration's text.
    fn saved_a_narration(&self, kind: Kind) -> bool {
        self.stop == Stop::AfterText && kind.saves_a_narration()
    }
}
```

`Failure` stays the two-way type it is today (`Stopped(Stopped) | Line(GenerationFailure)`), but loses `impl From<EngineError>` (see 1.5). Invariant: `saved_a_narration` is the only reader of `stop`; `run` is the only accumulator of saved Narrations.

### 1.3 The Scene: the values read once (`src/application/pipeline/job.rs`, private)

```rust
/// The request-scoped values, read once per Job: the World bundle, the Game
/// row, the settings. `Clone`: the World values are `Arc`s.
#[derive(Clone)]
struct Scene {
    world: Arc<WorldCard>, map: Arc<MapDef>, persona: Arc<PersonaCard>,
    npcs: HashMap<String, NpcCard>,
    game: Game,                 // perspective, tense, options_always_on
    settings: AppSettings,      // response_length, connection, Agent preset ids
}
```

Per generation the Job reads, once each: the Scene, the start state, the story window before the place, and the preset row that `Generation::preset_type` names. `is_opening` (today `tail.count == 1` on a Retry) comes from the plan: `JobPlan::for_request` already reads the Tail, so the plan carries `opening: bool` (*inferred*; ticket 19's sketch plus one bool).

### 1.4 The narrator's input (`src/application/narration/input.rs`)

```rust
/// Domain values only, so `narration/` imports no storage (agent-edge holds).
pub struct Narration<'a> {
    pub generation: &'a Generation,     // kind, guide, direction, event record, regen text
    pub player_input: Option<&'a str>,  // the free text the Job saved as the Input
    pub world: &'a WorldCard, pub map: &'a MapDef,
    pub persona: &'a PersonaCard, pub npcs: &'a HashMap<String, NpcCard>,
    pub perspective: Perspective, pub tense: Tense,   // the Game row's two values
    pub room: &'a Room,                 // the start state's room, already resolved
    pub present: &'a [String],          // the start state's Characters present
    pub story: &'a [MessageEntry],      // the story before the place
    pub preset: &'a PromptPreset,       // picked by Generation::preset_type
    pub response_length: &'a str, pub budget: TokenBudget,
}

impl Narrator {
    /// One call per generation. The prompt rules of each kind live here:
    /// the user message, the event continuation text, the guide and
    /// Impersonate flags, and the role whose health the call feeds.
    pub fn narrate(&self, input: &Narration<'_>) -> Result<Narrated, GenerationFailure>;
}
```

The narrator owns: `user_message` (the four kind rules, today `job.rs`), `continuation_user_message`, `previous_input`, the `with_guide`/`with_impersonate` flags, and the role (`Kind::role`). The Job builds the struct literal and calls once.

### 1.5 The one read rule (Q2) — `src/application/generation/reads.rs`

```rust
/// A miss or a read error of a record: the line names the record, the
/// disclosure holds the raw error. Used by `accept` and by the Job.
pub fn record<T>(name: &'static str, read: Result<Option<T>, EngineError>) -> Result<T, GenerationFailure> {
    match read {
        Ok(Some(value)) => Ok(value),
        Ok(None) => Err(GenerationFailure::missing_record(name, None)),
        Err(error) => Err(GenerationFailure::missing_record(name, Some(error.to_string()))),
    }
}
```

The fixed record-name list (consts beside `record`): "the game", "the story", "the start state", "the World", "the persona", "the settings", "the preset", "the current state", "the trigger context". `accept` wraps the `GenerationFailure` into `Refusal::Failed`; the Job wraps it into `Failure::Line`. `impl From<EngineError> for Failure` is deleted: no read error keeps the blanket "missing: <raw text>" line.

### 1.6 The save units take values (Q3) — `gate.rs`

```rust
impl JobWriter {
    fn save_input(&self, text: &str, start: &SwipeState) -> Result<SwipeAt, Stopped>;
    fn save_text(&self, swipe: NewSwipe) -> Result<SavedText, Stopped>;
    fn save_state_update(&self, saved: SavedText, state: SwipeState,
        room_report: Option<String>) -> Result<(), Stopped>;
    /// Offered replaces the set. Unavailable writes the notice only, so the
    /// offered set stands as it is (a Narration's text save already emptied it).
    fn save_options(&self, result: OptionsResult) -> Result<(), Stopped>;
    fn save_failure(&self, failure: GenerationFailure) -> Result<(), Stopped>;
    fn step(&self, step: Step) -> Result<(), Stopped>;
}
```

- `save_input` takes the start state the Job read (`state_at` goes). `save_text` takes the `NewSwipe` that `GenerationOrder::write` built (`tail_of`, `opening_room_report`, `message_kind_of`, `text_swipe` go). One amendment to ticket 19's sketch: `write(text, start, room_report: Option<String>)` takes the opening's room report as a third value.
- `Tx::put_notice(&Notice)` (*new*, `storage/transaction.rs`): writes the notice column only, leaving the offered column. `save_options` loses `after_narration` and its `dock()` read. Both dock behaviours of today are kept (*known*, pinned by `options.rs`): an on-demand failure keeps the prior set; an after-narration failure shows the notice over the set the text save already emptied. The job-id check under the guard is unchanged.

**Q3 verification (known, from `gate.rs`):** while the Job holds its entry, no other writer can change the story. `switch_swipe`, `edit` and `delete_last` call `require_idle` inside the hold before any unit; `stop`, `replace` and `delete` remove the Job's entry under the same hold, so the Job's next unit is refused by the id check; `create` writes only new Game rows. The writer rule (target design §3.3) is the guarantee, so a value the Job read before its unit is still current when the unit opens, and the read-once bundle is sound. Settings rows are NOT guarded (a settings write can land mid-Job); the Job accepts the values it read once, per Q5.

### 1.7 The typed storage error (Q4) — `src/error.rs`

```rust
#[error("database error while {context}: {source}")]
Database { context: &'static str, source: rusqlite::Error },
impl EngineError {
    /// The one wrap of a rusqlite error; the context names the line.
    pub fn db(context: &'static str, source: rusqlite::Error) -> Self;
}
```

*Known*: grep counts 82 `Config(format!("Failed to …"))` wraps under `storage/`; each becomes `EngineError::db("…", e)`. `#[from] rusqlite::Error` dies with the tuple variant, so every `?` on a rusqlite error gains a `.map_err` — the named cost of the type. `Config` stays for a settings value with no valid value.

**What `from_engine_error` still classifies:** only the model call's failures — `Llm` → Unreachable / UnreadableAnswer / TokenBudgetSpent, `ContextOverflow` and `Narrative::PromptBuild` → PromptTooLong, `Narrative::Generation` → UnreadableAnswer, `PresetNotFound` → PresetMissing, `RoomNotFound` → SceneMissing, `Config` → Other, the rest → Other. Its `Database` arm never fires on the Job path (a read names its record at the read; a write names SaveFailed at the unit); keep it as the SaveFailed fallback for a write no site named.

### 1.8 The hook inputs in one place

`HookInput::build(scene-values, state, story, text, player_text)` — one constructor on the domain type (`domain/model/agent.rs`), called by the Job for the after-narration hook (state = start state, story = the window) and for the end-of-Job hook (state = the current state, story = the 4 newest Messages, which include the new Narration). `HookInput` has already lost `kind` and `event` (ticket 19 Q15).

## 2. Usage

### 2.1 `Job::run` — one loop, one accumulator, one exit for a failure

```rust
fn run(&self, plan: JobPlan) {
    let JobPlan { input, first, options } = plan;
    let mut saved = false;                    // the one fact: a Narration was saved
    let mut scene = None;                     // the values the last generation read
    let mut order = first;
    let mut input = input;
    while let Some(current) = order.take() {
        match self.generate(&current, input.take().as_deref()) {
            Ok(g) => {
                saved |= g.narration;
                scene = Some(g.scene);
                order = g.fired.map(|r| GenerationOrder::event_after(g.at, r));
            }
            Err(f) => return self.finish(f, current.kind(), saved, scene.as_ref(), options),
        }
    }
    // `/options` reads the Scene once here; a generation's Scene is reused.
    let scene = scene.unwrap_or_else(|| self.scene().unwrap());
    if options == OptionsRun::OnDemand || saved {
        if let Err(failure) = self.options_run(options, &scene) { self.end(failure); }
    }
}
```

`finish` is the one exit of a failed Job (Q1): the Options run follows every failure after a saved Narration, except `Stopped::EntryMoved`; a failure of that Options run is only logged; then the first failure's line is written. A `SaveFailed` of the state update, or of the event generation's text save, gets the Options run — this *changes* today's code, which excludes every `Stopped` (*known*: `failed()` checks `!stopped`); Q1 decides the change. `finish` also needs `options == OptionsRun::AfterNarration`, which holds whenever a generation ran.

### 2.2 One generation

```rust
fn generate(&self, order: &GenerationOrder, input: Option<&str>) -> Result<Generated, Failed> {
    let kind = order.generation.kind();
    let scene = self.scene()?;                              // World, Game row, settings — once
    let start = self.read(RecordName::StartState, ...)?;    // once
    let story = self.window(order)?;                        // the story before the place — once
    let preset_id = scene.game.active_preset_id(order.generation.preset_type());
    let preset = self.read(RecordName::Preset, self.storage.get_preset(&preset_id))?;

    let room = start.room(&scene.map).ok_or_else(|| Failure::missing_room(...))?;
    self.step(kind.step(), || Ok(()))?;
    if let Some(text) = input {
        self.step(kind.step(), || self.writer.save_input(text, &start))?;
    }
    let narrated = self.step(kind.step(), || self.narrator.narrate(
        &Narration { generation: &order.generation, player_input: input, room,
                     story: &story, preset: &preset, ..scene.as_ref()  }))?;

    let report = order.opening.then(|| room.name.clone());  // the plan's `opening` flag
    let (swipe, pre) = order.generation.write(narrated.text, &start, report);
    let saved = self.step(kind.step(), || self.writer.save_text(swipe))?;
    let Some(pre) = pre else {
        return Ok(Generated::user_regen(&scene, &start, saved.at()));  // R-18 as a type
    };
    let result = self.step(Step::Quantifying, || self.quantify(&scene, &start, &narrated, &story, input))
        .map_err(|f| Failed::new(f, kind, Stop::AfterText))?;
    let outcome = game_rules::apply(pre, &result, &scene.world())
        .map_err(|f| Failed::new(Failure::contradiction(f), kind, Stop::AfterText))?;
    self.step(Step::Quantifying, || self.writer.save_state_update(
        saved, outcome.state.clone(), outcome.room_report))
        .map_err(|f| Failed::new(f, kind, Stop::AfterText))?;
    Ok(Generated { at: saved.at(), fired: outcome.fired,
                   narration: kind.saves_a_narration(), scene })
}
```

The analysis story is the newest `STORY_WINDOW` entries of `story` — same boundary as the narrator's window in both places (*known*: today the analysis reads `window_before(at.message, 4)`; the boundary equals the narrator's `place_before`). The Quantifier failure keeps today's fallback behaviour: a failed Agent logs and contributes the default result, never a Job failure (*known*: `AgentRunner::analysis`).

### 2.3 `options_run` — the end-of-Job hook

Reads the story once (the Tail for the current state, the 4 newest Messages — a window that, unlike the analysis window, includes the new Narration), reuses the Scene, builds one `HookInput`, and calls `save_options(result)`. The toggle check reads `scene.game.options_always_on`; no Game-row read.

### 2.4 `accept`

Keeps the guard, the accept unit and the entry. Its reads go through `record(name, …)`; `plan_for`, `generation_plan`, `require_start`, `require_job_data`'s direction and event fill, `event_record` and `trigger_direction` are already gone into `JobPlan::for_request` (ticket 19).

## 3. What hides behind the seam

Deleted or moved out of `job.rs` (*known*, 804 lines today; the file lands near 300 lines, *inferred*):
- The three holders of "a Narration was saved" (`saved_a_narration`, `Ended.narration_saved`, `options_step(.., after_narration)`) → one accumulator in `run`. The two `into_event_failure` sites in `run` → one site inside `Failed::new`.
- `Ended` (+2 methods, 1 `From`), `Failure::missing`, `Failure::event_narration_failed`, `impl From<EngineError> for Failure` → `Failed::new` and `record`.
- `narrate_step`, `user_message`, `continuation_user_message`, `previous_input`, `preset`, `budget`, `game_row`, `world_bundle` (≈90 lines) → the narrator behind `Narration` + one `Scene` read.
- `analyse_step`'s and `options_step`'s `WorldView`/`HookInput` builds (≈40 lines, twice) → `HookInput::build`; `scene_kind` goes with `HookInput.kind`.
- `apply_rules` (≈60 lines), `RuleOutcome`, the double `apply_game_rules` call → ticket 19's `game_rules::apply`.
- `place_before`, `window_before` (two limits), `is_the_opening`, `tail`, `start_state`, `options_enabled`, `analysis_settings`, `options_settings`, `preset_text`, `event_order`, `room_of`, `room_check`, `resolve_room`, `player_text` — 14 fns; `options_step`'s re-reads → one Tail read, one window read, the reused Scene.

Deleted from `gate.rs`: `state_at`, `tail_of`, `opening_room_report`, `dock`, `message_kind_of`, `text_swipe`, `save_options`'s param — 7 items, plus the 6 plan fns ticket 19 already takes. Under `storage/`: 82 hand wraps → `EngineError::db` (*known* count).

Duplicate reads, today → new: settings 3 → 1 per Job; Game row 2 → 1 per Job; Tail up to 3 → 0 in `generate` (the plan carries the opening flag and the regen text); window 2 → 1; World bundle 2 (generation + Options) → 1 per Job.

## 4. Where each piece lives

| Piece | Path |
|---|---|
| `Generation` (+`UserRegen { text }`), `preset_type`, `Kind` mirror | `src/domain/model/generation.rs` |
| `JobPlan::for_request`, `opening` flag, `OptionsRun`, `JobRequest` | `src/domain/model/generation.rs` (ticket 19's move) |
| `HookInput::build` | `src/domain/model/agent.rs` |
| `GenerationFailure` lines, `from_engine_error` narrowed | `src/domain/model/state/generation_status.rs` |
| `EngineError::Database { context, source }`, `EngineError::db` | `src/error.rs` |
| `record` + the record-name consts | `src/application/generation/reads.rs` |
| `Narration`, the prompt rules per kind | `src/application/narration/` |
| `Job`, `Scene`, `Failed`, `run`/`finish`/`generate`/`options_run` | `src/application/pipeline/job.rs` |
| The guard, `accept`, the value-taking units, `Tx::put_notice` | `src/application/generation/gate.rs`, `src/adapters/driven/storage/transaction.rs` |

`arch-lint.toml`: `agent-edge` (`application/agents/**`, `application/narration/**`) still denies storage imports. `Narration` and the narrator's rules hold domain values only; `WorldBundle` is destructured by the Job (pipeline, `application` scope, allowed). No rule change is needed.

## 5. Tests that pin the interface

| Tier | CASE id | Asserts |
|---|---|---|
| unit, `generation_tests.rs` | `generation-preset-slot` | `preset_type`: System for Narration/UserRegen/Event, Impersonate for Impersonate |
| unit, `generation_tests.rs` | `plan-opening-flag` | a Retry of the opening Narration sets `opening: true`; any other order `false` |
| seam, `narrator_tests.rs` | `narrator-user-message-action` / `-continue` / `-impersonate-bare` / `-user-regen` / `-event-continuation` | the five user-message rules, from `Narration` alone (no Job) |
| seam, `narrator_tests.rs` | `narrator-kind-role` | the call record's agent is the Trigger's role for an event, the Narrator's otherwise |
| seam, `tests/storage/` (docks) | `dock-notice-only` | `put_notice` writes the notice and leaves the offered column |
| tier 1, `failure_display.rs` | `job-missing-record-line` | a failed read of each record shows "The data for this request is missing: <record>." with the raw error in the disclosure (new; today's blanket line is the bug) |
| tier 1, `failure_display.rs` | `job-save-failed-line` (exists) | only a failed write shows "The new text could not be saved.", now with the typed `Database` context in the disclosure |
| tier 1, `options.rs` | `job-options-after-save-failed` | a `SaveFailed` of the state update still gets the Options run (Q1; new — today's code refuses it) |
| tier 1, `options.rs` | `job-entry-moved-no-options` | after a stop, no Options run and no failure line |
| tier 1, `options.rs` | existing `test_failed_regeneration_keeps_prior_set_http`, `test_failed_always_on_regeneration_clears_set_and_reports_http`, `test_options_run_follows_a_failed_event_narration_http` | the dock behaviours and the event-failure Options run, unchanged |
| tier 1, `actions.rs`, `swipe_new.rs`, `retrigger.rs`, `story_log.rs` | ticket 19's rows | the step table and §4.5, which this design must not move |

## 6. Trade-offs

- **High leverage:** the failure lines become one rule with a fixed name list (Q2) — today's worst line class dies; `run` becomes a loop a tier-1 test walks with a `ScriptedModel` script of one entry per call; one `Scene` read kills the settings/Game-row/World re-reads; the narrator's kind rules become seam-testable with no storage.
- **`Narration` is wide** (14 fields). A builder with per-kind methods would add caller code for no plainness gain; a value struct the Job fills in one literal is plainer.
- **`Scene` is per Job, not per generation.** One read per Job is fewer reads, and the World and Game row cannot change under a busy entry (*known*, §1.6) — only settings can, and the design accepts the once-read values. Honouring a mid-Job settings change later is a one-line hoist back into `generate`.
- **The `?`-ergonomics cost of Q4**: losing `#[from] rusqlite::Error` makes every storage `?` a `map_err`. The typed context is worth it (the disclosure improves, `from_engine_error` stops guessing), but the diff is wide and mechanical.
- **Left open:** the before-narration hook has no Agent yet; `Scene` is where its input would come from. `Refusal` and `Failure` could merge further (both wrap `GenerationFailure`) — left for ticket 19's build. `put_notice` assumes a game always has a dock row (`DockRecord::default` writes an empty set — *inferred*). `job-options-after-save-failed` changes today's behaviour; Q1's text wins and the test pins the new rule.
