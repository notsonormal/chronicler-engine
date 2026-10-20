# Design 3 — the shape of the Job, through types

Constraint of this draft: wrong states are unrepresentable. A flag that two
places must agree on becomes a value with one maker. A runtime guard becomes a
type the compiler checks. Claims are labelled *known* (read in code) or
*inferred*.

## 1. Interface

### 1.1 The failure of a Job (`src/application/pipeline/job.rs`)

```rust
/// The one read path names its record, so a raw error text can never become
/// the name (Q2). `From<EngineError> for Failure` is deleted: a bare `?` on a
/// storage read does not compile in the Job any more.
pub enum Failure {
    /// A read missed or failed: "The data for this request is missing: <record>."
    Missing { record: Record, source: Option<EngineError> },
    /// A model or rules failure: its kind's own line.
    Line(GenerationFailure),
    /// A write failed: "The new text could not be saved."
    Save(EngineError),
    /// The entry moved on: no write lands, not even the failure line.
    EntryMoved,
}

/// The fixed list of record names (Q2), in `generation_status.rs`. `accept`
/// and the domain plan use it too, so the names stay one list.
pub enum Record { Game, Story, StartState, CurrentState, World, Persona, Settings, Preset, Message }
impl Record { pub fn name(self) -> &'static str { /* "the game", "the story", ... */ } }
```

`Failure::Missing` renders through `GenerationFailure::missing_record(record.name(),
source)` — *known*: that constructor already exists. The preset is the one read
whose **miss** keeps its own line (`PresetMissing`, target design §4.7); a preset
read **error** is `Missing { record: Preset }`.

### 1.2 The one ending of a Job (FINDINGS 6)

The fact "a Narration was saved" lives in exactly one value today's code holds
in three (*known*: the local `saved_a_narration`, `Ended.narration_saved`,
`options_step(after_narration)`). Here it is one type with one writer:

```rust
/// The fact that decides the end-of-Job hook. Only a text save of a kind with
/// `saves_a_narration()` writes it; nothing ever resets it.
pub enum Landed { Nothing, Narration(SwipeAt) }

pub enum JobOutcome {
    Done(Landed),
    /// A failure that is not `EntryMoved`.
    Failed(Failure, Landed),
    /// The entry moved: no failure line and no Options run. It carries no
    /// `Landed`, so "an Options run after EntryMoved" does not compile.
    EntryMoved,
}

impl JobOutcome {
    /// Q1 as one total fn — the only place the eligibility rule exists.
    fn options_follow(&self, plan: OptionsRun) -> bool {
        match self {
            Self::Done(landed) => plan == OptionsRun::OnDemand || landed.is_narration(),
            Self::Failed(_, landed) => plan == OptionsRun::AfterNarration && landed.is_narration(),
            Self::EntryMoved => false,
        }
    }
}
```

`run` has one exit (`finish`); `failed` and `end` go. The event-failure wrap
runs at one site: the loop wraps when the failed order's kind is `Event`
(*known*: today `into_event_failure` runs at two sites in `run`).

### 1.3 The input bundle (Q3, Q5)

```rust
// src/application/narration/bundle.rs — agent-edge: domain values only, no storage import.
pub struct WorldValues { pub world: Arc<WorldCard>, pub map: Arc<MapDef>,
                         pub persona: Arc<PersonaCard>, pub npcs: HashMap<String, NpcCard> }

/// The room check's witness. Only `GenerationBundle::check_room` makes one, so
/// a narrator call with an unchecked room does not compile.
pub struct CheckedRoom<'a>(&'a Room);

pub enum Placement { NewMessage, OnLast(SwipeAt /* next swipe index on the last Message */) }

/// Read once per generation, before its first step. Every step takes values.
pub struct GenerationBundle {
    world: WorldValues, game: Game, settings: AppSettings,
    preset: PromptPreset,               // by the domain fn `Generation::preset_type`
    quantifier_preset: Option<String>,  // None when the preset row is gone (today's rule, *known*)
    response_length: String, budget: TokenBudget,   // both derived from `settings`, read once
    start: SwipeState,                  // the start state of the order
    history: Vec<MessageEntry>,         // the narrator window, before the place
    typed_input: Option<String>,        // the plan's Input text
    placement: Placement,               // from the one tail read
    opening_room_report: Option<String>,// tail.count == 1 && OnLast: the room name, as a value (Q3)
}

impl GenerationBundle {
    pub fn check_room(&self) -> Result<CheckedRoom<'_>, GenerationFailure>; // SceneMissing line
    pub fn narration_input(&self, room: CheckedRoom<'_>) -> NarrationInput<'_>;
    pub fn scene(&self) -> Scene<'_>;          // start state + WorldView, for the hook
    pub fn game_world(&self) -> GameWorld<'_>; // ticket 19's rules input
    pub fn player_text(&self, story: &[StoryEntry]) -> String;
}

/// The end-of-Job hook reads its own bundle when it runs (the toggle, the tail
/// and the current state can have moved since the last generation).
pub struct OptionsBundle { world: WorldValues, tail: Tail, current: SwipeState,
                           story: Vec<StoryEntry>, settings: AppSettings, preset: Option<String> }
impl OptionsBundle { pub fn scene(&self) -> Scene<'_>; pub fn scene_kind(&self) -> Kind; }
```

Both bundles build their `Scene` in one place (*known*: today `analyse_step`
and `options_step` each build a `WorldView` and a `HookInput` again):

```rust
// src/domain/model/agent.rs — pure domain, next to HookInput and WorldView.
pub struct Scene<'a> { pub state: &'a SwipeState, pub world: WorldView<'a> }
impl<'a> Scene<'a> {
    pub fn hook_input(&self, text: Option<&'a str>, player_text: &'a str,
                      story: &'a [StoryEntry]) -> HookInput<'a>;
}
```

**Q3 verification (asked for explicitly).** *Known* from `gate.rs`: while the
Job holds its entry, `switch_swipe`, `edit` and `delete_last` refuse at
`require_idle`; `stop`, `delete` and `replace` remove the entry under the same
hold, and the job-id check then refuses every later unit with `EntryMoved`. So
no other writer can change the story. The Job's own Input save is the only
write between a bundle read and a text save, and it runs only for
`Placement::NewMessage`, which consumes no placement facts. The `OnLast` facts
are therefore as stable outside the unit as inside it, and a stop in between
still refuses the unit by the id check. Reading them in the bundle is safe.

### 1.4 The narrator (Q5)

```rust
// src/application/narration/narrator.rs
pub struct NarrationInput<'a> {
    pub world: &'a WorldCard, pub room: CheckedRoom<'a>, pub npcs: NpcContext<'a>,
    pub persona: &'a PersonaCard, pub perspective: NarrativePerspective, pub tense: NarrativeTense,
    pub history: &'a [MessageEntry], pub typed_input: Option<&'a str>,
    pub preset: &'a PromptPreset, pub response_length: &'a str, pub budget: TokenBudget,
}

impl Narrator {
    /// The prompt rules of each kind live here: the user message, the event
    /// continuation text, the guide and Impersonate flags, the role for health.
    pub fn narrate(&self, generation: &Generation, input: &NarrationInput<'_>)
        -> Result<Narrated, GenerationFailure>;
}
```

Inside, `user_message` is a **total** fn — every `Generation` variant carries
its own text, so no arm can fail or unwrap (*inferred* from ticket 19's
`Generation`):

| Generation | User message | Flags | Role |
|---|---|---|---|
| `Narration { guide }` | `typed_input` if non-empty, else the last Input of `history` | `with_guide(guide)` | Narrator |
| `Impersonate { direction }` | `direction.unwrap_or_default()` | `with_impersonate(true)` | Narrator |
| `UserRegen { text }` | `text` | none | Narrator |
| `Event(record)` | the continuation text of `history.last()` + `record.direction` | none | Trigger |

**Open detail decided:** `Generation::UserRegen { text: String }`. The order's
rule is "ids and texts only" (ticket 19, Q13) — a text belongs on it, like the
guide and the direction already do. `JobPlan::for_request` fills it from
`tail.last.text()` (*known*: the Tail holds the active Swipe's text), so no
extra read and no runtime "missing text" arm. The bundle alternative would
carry a value only one kind uses and give the narrator a fallible arm.

### 1.5 The save units take values (Q3)

```rust
// src/application/generation/gate.rs — JobWriter. No unit reads any record.
pub fn save_input(&self, text: &str, state: &SwipeState) -> Result<SwipeAt, Stopped>;
pub fn save_text(&self, swipe: &NewSwipe, placement: &Placement,
                 opening_room_report: Option<String>) -> Result<SavedText, Stopped>;
pub fn save_state_update(&self, saved: SavedText, state: SwipeState,
                         room_report: Option<String>) -> Result<(), Stopped>;  // unchanged
/// Offered → `put_dock` (the whole record). Unavailable → `Tx::set_dock_notice`
/// (a notice-only patch inside the transaction): the offered set stays as it is,
/// so the `after_narration: bool` and the 3-arm match go (Q3).
pub fn save_options(&self, result: OptionsResult) -> Result<(), Stopped>;
```

`NewSwipe` comes from ticket 19's `GenerationOrder::write(text, &start)` — the
unit inserts what the domain fn built. It derives "empties the dock" from the
swipe's message kind (a match on a value, not a read). *Known*: today's
`save_options(Offered/Unavailable × after_narration)` has one arm that reads
the dock back (`self.dock()?`); a notice-only `Tx` write removes that read and
keeps behaviour: after a Narration the text save already emptied the set, so a
notice alone renders the same dock.

`Stopped` keeps its two variants; `From<Stopped> for Failure` maps
`SaveFailed → Failure::Save` and `EntryMoved → Failure::EntryMoved`. The
job-id check under the guard is unchanged (Q3).

### 1.6 The storage error (Q4)

```rust
// src/error.rs
/// SQLite returned an error; `context` names the operation, fixed at the call site.
Database { context: &'static str, source: rusqlite::Error },

// src/adapters/driven/storage/utils/plumbing.rs — the one wrap point.
pub(crate) fn db<T>(context: &'static str, r: Result<T, rusqlite::Error>) -> Result<T, EngineError>;
```

The ~64 hand wraps as `Config(format!("Failed to ...: {e}"))` become
`db("...", ...)` (*known*: count from the brief; sites under `storage/`).
`Config` stays for a setting with no valid value (e.g. `narration_connection()`);
the Job maps that to `Missing { record: Settings, source }`.

**What `from_engine_error` still classifies (Q4 answer):** only the model seam —
`Llm` → Unreachable / UnreadableAnswer / TokenBudgetSpent; `ContextOverflow` and
`Narrative(PromptBuild)` → PromptTooLong; `Narrative(Generation)` →
UnreadableAnswer; `PresetNotFound` → PresetMissing; `RoomNotFound` →
SceneMissing; everything else → Other. The `Database(_) | Io(_) → SaveFailed`
arm is **deleted**: once every read names its record and every write shows the
save line by position, no storage error reaches `from_engine_error` in the Job
(*inferred*: it is called only by the narrator and agents, which touch no
storage — the agent-edge rule).

## 2. Usage

```rust
fn run(&self, plan: JobPlan) {                       // the one loop, the one exit
    self.finish(self.run_plan(&plan), plan.options);
}

fn run_plan(&self, plan: &JobPlan) -> JobOutcome {
    let mut landed = Landed::Nothing;
    let mut next = plan.first.clone();
    while let Some(order) = next.take() {            // at most two orders (R-14)
        match self.generate(&order, &mut landed) {
            Ok(event) => next = event,               // the fired Trigger's order, or None
            Err(Failure::EntryMoved) => return JobOutcome::EntryMoved,
            Err(failure) => {
                let failure = if order.kind() == Kind::Event { failure.event_narration_failed() }
                              else { failure };      // the one wrap site
                return JobOutcome::Failed(failure, landed);
            }
        }
    }
    JobOutcome::Done(landed)
}

fn finish(&self, outcome: JobOutcome, plan: OptionsRun) {
    let follows = outcome.options_follow(plan);      // Q1, one rule
    match outcome {
        JobOutcome::EntryMoved => tracing::warn!("the entry moved on; the Job ends"),
        JobOutcome::Done(_) => if follows {
            self.run_options(plan).unwrap_or_else(|f| self.write_failure(f));
        },
        JobOutcome::Failed(failure, _) => {
            if follows { self.run_options(plan).unwrap_or_else(|f| tracing::warn!("{f:?}")); }
            self.write_failure(failure);             // the status shows the first failure (Q1)
        }
    }
}
```

One generation — every read is a bundle value, every step a value in, a value out:

```rust
fn generate(&self, order: &GenerationOrder, landed: &mut Landed)
    -> Result<Option<GenerationOrder>, Failure> {
    let b = self.bundle_for(order)?;                 // one read of each record (below)
    let (kind, step) = (order.kind(), order.kind().step());

    let room = self.run_step(step, kind, || b.check_room().map_err(Failure::Line))?;
    if let Some(text) = &b.typed_input {
        self.run_step(step, kind, || self.writer.save_input(text, &b.start).map_err(Failure::from))?;
    }
    let narrated = self.run_step(step, kind, || {
        self.narrator.narrate(&order.generation, &b.narration_input(room)).map_err(Failure::Line)
    })?;

    let (swipe, pre) = order.write(narrated.text.clone(), &b.start);   // ticket 19
    let saved = self.run_step(step, kind, || {
        self.writer.save_text(&swipe, &b.placement, b.opening_room_report.clone())
            .map_err(Failure::from)
    })?;
    if kind.saves_a_narration() { *landed = Landed::Narration(saved.at()); }  // the one writer
    let Some(pre) = pre else { return Ok(None) };    // a user regen: no rules (R-18 as a type)

    let story = self.story_window(saved.at())?;      // the analysis window ends before the new text
    let hook = b.scene().hook_input(Some(&narrated.text), &b.player_text(&story), &story);
    let (result, outcome) = self.run_step(Step::Quantifying, kind, || {
        let result = self.agent_runner.analysis(&hook, &b.quantifier_settings());
        game_rules::apply(pre, &result, &b.game_world()).map(|o| (result, o))
    }).map_err(Self::contradiction_on)?;             // rules Err → Contradiction line
    self.run_step(Step::Quantifying, kind, || {
        self.writer.save_state_update(saved, outcome.state, outcome.room_report)
            .map_err(Failure::from)
    })?;
    Ok(outcome.fired.map(|record| GenerationOrder::event_after(saved.at(), record)))
}
```

`bundle_for` is the only read site; every read goes through one combinator, so
a read cannot forget its record name:

```rust
fn read<T>(&self, record: Record, get: impl FnOnce() -> Result<Option<T>, EngineError>)
    -> Result<T, Failure>;      // Ok(None) → Missing{record, None}; Err(e) → Missing{record, Some(e)}
fn read_rows<T>(&self, record: Record, get: impl FnOnce() -> Result<T, EngineError>)
    -> Result<T, Failure>;      // windows: no miss, only a read error
```

`bundle_for(order)` reads: World, Game, Settings, Preset (miss → `PresetMissing`
line), the quantifier preset (miss → `None`), the start state, the tail
(placement + opening flag), the narrator window. One generation: 8 reads, each
once (today: settings 4×, Game row 2×, Tail up to 4× — *known* from the brief).

`run_options(mode)` reads the Game row for the toggle when
`mode == AfterNarration` (fresh, as today — *known*), then an `OptionsBundle`
(tail, current state, window of 4, world values, options settings), and calls
`Scene::hook_input` with `text: None`. Agent failure stays
`OptionsResult::Unavailable` (a notice, not a Job failure — *known*).

**`accept`** changes only in the failure lines: `missing`/`unreadable` take a
`Record` instead of a `&str`, and the domain `JobPlan::for_request` returns
`PlanRefusal::Missing(Record)` (ticket 19). Its three-way
`missing`/`unreadable`/`save_failed` split stays — it is already the Q2 shape.

## 3. What hides behind the seam

Deleted from `job.rs` (35 fns today → ~11): `failed`, `end`, `log_refusal`
(one `finish`); `narrate_step`, `user_message`, `continuation_user_message`,
`previous_input` (narrator); `analyse_step` (Scene::hook_input + 3 lines in
generate); `apply_rules` with its event double-call and hand-cleared trigger
context (domain `game_rules::apply`, ticket 19); `room_check`, `resolve_room`,
`room_of`, `room_name` (CheckedRoom + `SwipeState::room`, ticket 19);
`event_order` (`event_after`, ticket 19); `place_before`, `is_the_opening`,
`window_before` (bundle); `world_bundle`, `game_row`, `tail`, `start_state`,
`preset`, `budget`, `options_enabled`, `analysis_settings`, `options_settings`,
`preset_text` (10 read fns → `read` + `bundle_for`); `scene_kind`,
`player_text` (OptionsBundle / bundle). Deleted types and flags: `Ended` (+2
constructors, `into_event_failure`), `Generated`, `RuleOutcome`,
`impl From<EngineError> for Failure`, the local `saved_a_narration`,
`options_step(after_narration: bool)`, `save_options`'s 3-arm match,
`Generated.narration`. **Count: 24 fns + 3 types + 1 From impl + 3 bool/flag
paths out of `job.rs`.**

Deleted from `gate.rs` (this ticket's share; ticket 19's plan deletions are
separate): `JobWriter::state_at`, `tail_of`, `opening_room_report` (with its
`.ok()` silent-miss), `dock`, `text_swipe`, `message_kind_of` — **6 fns**; the
units read nothing.

Deleted from `generation_status.rs`: the `Database | Io → SaveFailed` arm of
`from_engine_error`. Converted under `storage/`: ~64 `Config(format!)` wraps →
`db()`. Deleted there: `EngineError::Config` as the carrier of rusqlite errors.

What stays: `run_step` (the status-name guarantee), `Stopped`, the job-id
check, `SavedText` consumption (a state is written at most twice — *known*),
`OptionsResult`, the accept unit ordering.

## 4. Where each piece lives

| Piece | File | arch-lint |
|---|---|---|
| `Record`, `from_engine_error` (shrunk) | `src/domain/model/state/generation_status.rs` | model: domain only |
| `Scene`, `hook_input` builder | `src/domain/model/agent.rs` | model: domain only |
| `Generation::UserRegen { text }`, plan fns | `src/domain/model/generation.rs` (ticket 19) | model |
| `EngineError::Database { context, source }` | `src/error.rs` | — |
| `db()` helper; `Tx::set_dock_notice` | `src/adapters/driven/storage/utils/plumbing.rs`, `transaction.rs` (+ `in_memory_data.rs`) | storage → domain only |
| `WorldValues`, `CheckedRoom`, `Placement`, both bundles, `NarrationInput` | `src/application/narration/bundle.rs` (new) | agent-edge: holds domain values only; the Job (pipeline) imports storage and builds it — the bundle file never does |
| `Narrator::narrate(generation, input)` | `src/application/narration/narrator.rs` | agent-edge: no storage |
| `Failure`, `Landed`, `JobOutcome`, `read`, `bundle_for`, `run_plan`, `finish`, `run_options` | `src/application/pipeline/job.rs` | application → anything below |
| value-taking save units | `src/application/generation/gate.rs` | application |

*Known*: `arch-lint.toml` denies `agent-edge → storage` and
`model → storage`; every new file above respects both. `bundle.rs` holds
`Arc<WorldCard>` etc. — the `WorldBundle → WorldValues` mapping lives in
`job.rs` (pipeline), not in the bundle.

## 5. Tests that pin the interface

| Tier | CASE / file | Asserts |
|---|---|---|
| seam contract, `src/application/narration/narrator_tests.rs` | `narrate-user-message-by-kind` (4 rows) | each `Generation` variant gives §1.4's user message; no arm needs storage |
| seam contract, same | `narrate-flags-and-role` | Impersonate sets the flag and drops the persona layer; `Narration{guide}` renders the guide; an Event calls the recorder as agent "trigger" |
| seam contract, same | `narrate-empty-answer` (existing, kept) | empty text → `EmptyText` |
| pure domain unit, `generation_tests.rs` | `plan-user-regen-carries-text` | a Retry of a typed Input gives `UserRegen { text }` with the active Swipe's text |
| driven adapter, `tests/storage/transaction_storage.rs` | `set-dock-notice-keeps-offered` | the notice-only write leaves the offered set untouched |
| driven adapter, same | `database-error-carries-context` | a real rusqlite failure in a unit surfaces as `Database` naming the operation |
| tier 1, `tests/http/failure_display.rs` | new scenario + existing `test_save_failure_...`, `test_contradiction_...` | an injected read failure inside a Job shows "The data for this request is missing: <record>." — **not** the save line (the FINDINGS 2 bug, now unrepresentable); a failed write still shows the save line |
| tier 1, `tests/http/options.rs` | `test_options_run_follows_a_failed_event_narration_http`, `test_failed_regeneration_keeps_prior_set_http`, `test_failed_always_on_regeneration_clears_set_and_reports_http` | Q1's table; the notice-only save keeps the prior set (pins the flag removal) |
| tier 1, `tests/http/races.rs` | the stop-at-each-model-call rows; `test_a_stopped_job_writes_nothing_...`; `test_the_event_narration_call_uses_the_preset_active_when_it_runs` | `EntryMoved`: no failure line, no Options run, late writes refused; the event bundle reads the preset when the event generation starts (fresh-read behaviour kept) |
| tier 1, `tests/http/retrigger.rs` | `test_retrigger_trigger_narration_failure_sets_error_http` | "The event narration failed." with the inner line in the disclosure; the Narration that fired stays |
| tier 1, `tests/http/swipe_new.rs` | the Retry rows | a user regen ends at its text save (no state update); the opening Retry keeps its room report (value now comes from the bundle) |

## 6. Trade-offs

**High leverage.** `Landed` + `JobOutcome`: one fact, one eligibility rule, one
exit — the three-copies bug class dies, and `EntryMoved` without an Options run
is structural, not a check. `Record` + `read`: the FINDINGS 2 wrong-line bug
becomes a compile error (no `From<EngineError>`, no `&str` names). Generation
carrying its data (ticket 19) plus `UserRegen { text }` makes the narrator's
prompt rules total — no runtime "missing data" guard survives anywhere in the
narration path.

**Thin.** `CheckedRoom` has one call site; it is cheap ceremony, but a plain
`&Room` argument would lose the "checked" guarantee silently. `Scene`
deduplicates only two hook-input sites. The `quantifier_preset`
`Option<String>` keeps today's quiet rule (a gone preset row → no preset); a
stricter type would change behaviour and belongs to a later ticket.

**Costs, named.** The event generation re-reads the World and settings the
first generation just read (~8 reads per fired Trigger). The alternative — one
bundle for the whole Job — breaks the fresh-read behaviour `races.rs` pins and
widens every lifetime; rejected. Within one generation the preset is read at
bundle time, slightly before today's at-call time; the observable window is the
room check (*inferred*: no tier-1 test can see it — `races.rs` changes presets
while a model call is held, which is outside the window).

**Left open.** (1) `WorldValues` duplicates `WorldBundle`'s fields; moving
`WorldBundle` out of storage would delete one mapping, but that is a storage
ticket. (2) `Placement::OnLast(SwipeAt)` reuses `SwipeAt` for a not-yet-written
index — a newtype would be stricter, but adds a conversion at the one use site.
(3) Whether `run_options` on the Done path should reuse the last generation's
`WorldValues` (one saved read, stale-value risk): left as a fresh read.
