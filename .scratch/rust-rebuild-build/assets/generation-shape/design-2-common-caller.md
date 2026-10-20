# Design 2: the generation shape, optimised for the common caller

Constraint: the most common caller is the Job's `generate` for a player Action (a Narration). That path becomes trivial: **zero `match` on the kind**. The rarer kinds (event, Impersonate, user regen, Retry of the opening, Retrigger) carry their own data inside one enum, so no guard branch exists anywhere.

## 1. Interface

### The order: `Generation` (the kind is the variant)

```rust
// src/domain/model/generation.rs

/// The order of one generation: ids and texts only. Each kind carries its own
/// data, so an order cannot exist without the records its generation writes.
#[derive(Debug, Clone, PartialEq)]
pub enum Generation {
    /// A Narration of a player Action, a Continue, a `/guide`, or a Retry of one.
    Narration { place: Place, start: SwipeAt, guide: Option<String> },
    /// The player dictating their own Persona's Message. `direction` is none
    /// for a bare `/impersonate` (assumption; flips in one place, §6).
    Impersonate { place: Place, start: SwipeAt, direction: Option<String> },
    /// A Retry that regenerates the player's own Message. No analysis runs.
    UserRegen { place: Place, start: SwipeAt },
    /// A Trigger's continuation Narration. The record is always present.
    Event { place: Place, start: SwipeAt, record: EventRecord },
}

/// What a generation writes as a record. Always complete.
pub struct GenerationRecord {
    pub message_type: MessageType,
    pub message_kind: MessageKind,   // EventNarration(record) built from the variant
    pub maker: Maker,
    pub event_header: Option<String>,
}

impl Generation {
    pub fn start(&self) -> SwipeAt;
    pub fn place(&self) -> Place;
    /// The step the status shows: `GeneratingEvent` for an event, else `Narrating`.
    pub fn step(&self) -> Step;
    /// The role whose health the model call feeds: `Trigger` for an event, else `Narrator`.
    pub fn role(&self) -> Role;
    /// Whether the text save empties the dock and an Options run may follow.
    pub fn saves_a_narration(&self) -> bool;
    /// Whether the analysis and the game rules run (false only for a user regen).
    pub fn is_analysed(&self) -> bool;
    /// The stored inputs of the Swipe this generation writes (stored JSON unchanged).
    pub fn stored_inputs(&self) -> StoredInputs;
    /// The record this generation writes. Always complete.
    pub fn record(&self) -> GenerationRecord;
    /// The pre-analysis state: the start state, no trigger context; an event also
    /// counts its own one-shot Trigger fired; a user regen keeps the start state.
    pub fn pre_analysis(&self, start: &SwipeState) -> SwipeState;
    pub fn rules(&self, opening: bool) -> Rules<'_>;  // § the game rules
}
```

### The analysis: one output type

```rust
// src/domain/model/agent.rs — the analysis-hook output, beside `OfferedSet`

/// The analysis of one generation: what the analysis-hook Agents agree on.
/// The game rules read exactly these fields.
#[derive(Clone, Debug, Default, PartialEq)]
pub struct Analysis {
    /// The Characters the narration shows as present, by id.
    pub present: Vec<String>,
    /// The room the narration moves the player to, if any.
    pub destination: Option<String>,
    /// How sure the analysis is: the least sure of the Agents that contributed.
    pub confidence: Confidence,
}

impl Analysis {
    /// A second destination is a contradiction between two Agents, so the first wins.
    pub fn merge(self, other: Analysis) -> Analysis;
}
```

`Confidence` (Low/Medium/High, ordered) stays the one confidence enum. The quantifier's parse DTO (movement type, per-field confidence) becomes private to the Quantifier; its retry loop reads it inside.

### The game rules: one call per generation

```rust
// src/domain/model/utils/game_rules.rs

/// The World as the game rules read it: values only. Replaces `WorldBundle` at this seam.
pub struct RulesWorld<'a> {
    pub map: &'a MapDef,
    pub characters: &'a HashMap<String, NpcCard>,
    pub persona_name: &'a str,
}

/// The one rules run of one generation. The kind's own rules data.
pub enum Rules<'a> {
    /// A Narration or an Impersonate: movement, the Characters present, which
    /// Trigger fires. `opening`: this generation redoes the opening, so its
    /// Swipe reports its starting room.
    Analysed { opening: bool },
    /// An event Narration landing: no movement, no newly fired Trigger, its own
    /// one-shot Trigger counted fired, no trigger context left, no room report.
    Event { record: &'a EventRecord },
}

/// The state the generation's Swipe keeps, and what it shows.
pub struct Outcome {
    pub state: SwipeState,
    pub room_report: Option<String>,
    /// The Trigger that fired, its direction rendered for the Persona. `Analysed` only.
    pub fired: Option<EventRecord>,
}

/// One call per analysed generation. Pure: no storage, no LLM. `Err` is a
/// contradiction: the caller keeps the pre-analysis state.
pub fn apply(pre: &SwipeState, analysis: &Analysis, rules: Rules<'_>, world: &RulesWorld<'_>)
    -> Result<Outcome>;

/// The room of an id: a map room, else an added room. The one lookup.
pub fn room_of<'a>(state: &'a SwipeState, map: &'a MapDef, room_id: &str) -> Option<&'a Room>;

/// The event record of a Trigger: its name for the header, its narration prompt
/// rendered for the Persona as the direction. One rule for a Trigger the rules
/// just fired and a Trigger the current state still holds. `None`: the World no
/// longer holds that Character or Trigger.
pub fn event_record(trigger: &TriggerRef, world: &RulesWorld<'_>) -> Option<EventRecord>;
```

### The orders from the story: the Tail

```rust
// src/domain/model/tail.rs

pub enum RetriggerRefused {
    /// The last Message and the current state do not allow one (R-65).
    NotAllowed,
    /// The World no longer holds the Trigger the current state names.
    MissingTrigger,
}

impl Tail {
    /// The order of the last Message again, of the kind the last Message holds.
    pub fn redo(&self) -> Generation;
    /// A Retrigger: the event Narration as a new Message from the current state,
    /// its record built by `game_rules::event_record` from the World.
    pub fn retrigger(&self, world: &RulesWorld<'_>) -> Result<Generation, RetriggerRefused>;
    /// Unchanged: the one home of the R-65 rule, which both surfaces read.
    pub fn retrigger_allowed(&self) -> bool;
}
```

### Invariants, ordering, error modes

- **An accepted order is complete.** `accept` returns only orders that hold every record their generation writes. The Job can never fail with "the event record is missing" or "the impersonate direction": those states are unrepresentable. Its failures are model failures, save failures and contradictions only.
- **The rules always derive from the pre-analysis state.** The rules never read `trigger_context` (known: `evaluate_triggers` reads room, encounter log, fired set), and the event fired-marking already sits in the pre state, so `apply` needs no fired-marking of its own. The one-shot rule gets one writer: `Generation::pre_analysis` (today two: `SwipeState::pre_analysis` and `trigger_narration`, known).
- **Ordering inside `apply`**: Triggers are evaluated before the Character events advance `times_met`; the NPC diff comes from the state's previous present set against `analysis.present`; the event run strips `destination` itself, so the Job passes the analysis unchanged.
- **Error modes**: `apply` → `Err` (contradiction); `Tail::retrigger` → `NotAllowed` (the `400` of R-65) or `MissingTrigger` (the `500` "the trigger" line), fed by `event_record`'s `None`. No other error mode crosses this seam.

## 2. Usage

### The accept (`src/application/generation/plans.rs`, new `PlanMaker`)

```rust
match request {
    JobRequest::Action(text) => {
        let start = self.require_start(tail, game)?;               // 500: the story
        Ok(JobPlan::leads(Generation::Narration {
            place: Place::NewMessage, start, guide: None,
        }, (!text.is_empty()).then(|| text.clone())))
    }
    JobRequest::Retrigger => {
        let tail = tail.ok_or_else(|| self.bad_request("No trigger context available"))?;
        let order = tail.retrigger(&self.rules_world(game)?)
            .map_err(|r| self.retrigger_refusal(game, r))?;        // 400 or 500, one map
        Ok(JobPlan::leads(order, None))
    }
    // Guide, Impersonate, Retry (`tail.redo()`), Options: same shape, no data checks.
}
```

`require_job_data` keeps only the storage 500-checks (game row, persona, start state). The direction check and the event-record fill are deleted: the variants carry the data.

### The common path (`Job::generate`)
```rust
fn generate(&self, order: &Generation, input: Option<&str>) -> Result<Generated, Ended> {
    let world = self.rules_world()?;                                // RulesWorld values
    let start = self.start_state(order.start())?;
    let opening = order.place() == Place::NewSwipeOnLast && self.tail()?.count == 1;

    self.run_step(order.step(), || self.room_check(&world, &start))?;
    if let Some(text) = input {
        self.run_step(order.step(), || self.save_input(text, order.start()))?;
    }
    let narrated = self.run_step(order.step(), || self.narrate(order, &world, &start, input))?;
    let pre = order.pre_analysis(&start);
    let saved = self.run_step(order.step(), || self.save_text(order, narrated.clone(), pre, opening))?;

    if !order.is_analysed() {                                       // only a user regen
        return Ok(Generated::input_saved(order, saved));
    }
    let analysis = self.run_step(Step::Quantifying, || self.analyse(order, &world, &start, &narrated))?;
    let outcome = self.run_step(Step::Quantifying, || {
        game_rules::apply(&pre, &analysis, order.rules(opening), &world).map_err(Self::contradiction)
    })?;
    self.run_step(Step::Quantifying, || {
        self.save_state_update(saved, outcome.state, outcome.room_report)
    })?;
    Ok(Generated::narration(order, saved, outcome.fired))
}
```

The spine matches on no kind. The one branch, `is_analysed`, is a question about the order, not a guard for missing data. The second generation of a Narration's Job is `Generation::Event { place: Place::NewMessage, start: at, record }` — a plain constructor.

What this hands the later tickets: the `Job::run` loop gets its per-kind facts as methods (`step`, `is_analysed`, `saves_a_narration`); the narrator prompt ticket gets `role()`, `preset_type()` (Impersonate vs System), `steering()` (Impersonate vs Guide) and the event record, with the history window the Job already loads.

### The runner

`AgentRunner::analysis` merges `Analysis` values (`reduce(Analysis::merge)`); the `AnalysisAgent` trait returns `Analysis`. `HookInput` loses `kind` and `event`: no production Agent reads either (known: grep over `src/application/agents`). D-31 kind filtering stays possible: the runner filters before the call.

## 3. What hides behind the seam

**Guard branches deleted (8):** `require_job_data`'s Impersonate-direction check and Event-record fill; `job.rs` `user_message` and `apply_rules` missing-record failures; `gate.rs` `message_kind_of`'s `Stopped`; `generation_record::message_kind`'s `Option`; `job.rs` `apply_rules`'s event double-call, movement strip and manual `trigger_context` surgery; `JobWriter::empties_the_dock` (a duplicate of `saves_a_narration`, known).

**Fns deleted or absorbed (13):** gate.rs `event_record`, `trigger_direction`, `generation_plan`, `opening_room_report`; job.rs `user_message`, `scene_kind`, `is_the_opening`, `room_of`, `room_name`, `event_order`; game_rules.rs `trigger_narration` (the landing rules are one `Rules::Event` arm); `SwipeState::pre_analysis` (moves to `Generation`); `StatePatch::merge` (rewritten as `Analysis::merge`).

**Types deleted (11):** `Kind` (the variant is the kind; its methods move), `QuantifierConfidence` + both `From` impls, `QuantifierResult`, `QuantifierParseResult`, `MovementParseResult` (public shapes; the parse DTO goes private to the Quantifier), `StatePatch`, `AnalysisOutput`, `GameRuleRequest` (replaced by `Rules`, which drops the caller-less `NpcEvents` variant), `NpcEvent`/`NpcEventList`/`NpcTransitionType` (the diff stays a private helper).

**Duplicated rules unified (4):** the room lookup, 4 copies → `game_rules::room_of` (this also fixes `opening_room_report` ignoring added rooms and dropping read errors with `.ok()`, known); the opening room report, 3 sites → the `opening` fact and `Outcome.room_report`; the event-record build, 2 rules (TemplateVars render vs raw copy, known) → `game_rules::event_record`; the one-shot fired marking, 2 sites → `Generation::pre_analysis`. `SceneState.quantifier_confidence` gains its first production writer: `apply` records `analysis.confidence` in the outcome state, so `/debug/state` reads a live value.

## 4. Where each piece lives

| Piece | Path | arch-lint |
|---|---|---|
| `Generation`, `Place`, `GenerationRecord` | `src/domain/model/generation.rs` | model; imports domain only |
| `Analysis`, `Confidence`, `HookInput` | `src/domain/model/agent.rs` | model |
| `Rules`, `Outcome`, `apply`, `room_of`, `event_record`, `RulesWorld` | `src/domain/model/utils/game_rules.rs` | model → storage denied: `RulesWorld` takes `MapDef`, `NpcCard` map, persona name — no `WorldBundle` |
| `Tail::redo`/`retrigger`, `RetriggerRefused` | `src/domain/model/tail.rs` | model |
| `PlanMaker` (request → plan, 400/500 checks) | `src/application/generation/plans.rs` (new) | application; may read storage; a new type, so the inherent-impl guardrail holds |
| Guard, registry, save units | `src/application/generation/gate.rs` | application; keeps its module doc's promise |
| The `generate` spine | `src/application/pipeline/job.rs` | application |
| Runner, Quantifier | `src/application/agents/**` | agent-edge: `Analysis` is a value; no storage |

## 5. Tests that pin the interface

Unit, pure domain (`src/domain/model/**/*_tests.rs`, plain unit tests per STRATEGY.md rule 2):

| CASE | File | Asserts |
|---|---|---|
| `generation-event-carries-its-record` | `generation_tests.rs` | an Event order's `record()` always yields `EventNarration` with the header; no `Option` |
| `generation-pre-analysis-per-kind` | `generation_tests.rs` | Narration/Impersonate strip the context; Event also marks its one-shot; UserRegen keeps the start state |
| `generation-user-regen-not-analysed` | `generation_tests.rs` | `is_analysed` false only for UserRegen; `saves_a_narration` true only for Narration/Event |
| `game-rules-analysed-fires-and-keeps-context` | `game_rules_tests.rs` | one call: movement, present set, fired record with rendered direction, context = the fired Trigger |
| `game-rules-event-no-movement` | `game_rules_tests.rs` | a destination in the Analysis is ignored; context none; one-shot fired; room report none |
| `game-rules-trigger-times-met-order` | `game_rules_tests.rs` | a first-encounter Trigger still fires |
| `game-rules-opening-room-report` | `game_rules_tests.rs` | `Analysed { opening: true }` with no movement reports the starting room, added rooms included |
| `game-rules-contradiction-keeps-pre-state` | `game_rules_tests.rs` | `apply` returns `Err`; nothing is written |
| `room-of-added-room` | `game_rules_tests.rs` | map room, else added room, else none |
| `event-record-rendered-direction` | `game_rules_tests.rs` | a fired Trigger and a Retrigger get the same rendered direction |
| `tail-redo-per-last-message` | `tail_tests.rs` | each last Message kind maps to its variant; an Impersonate redo keeps a none direction |
| `tail-retrigger-builds-full-order` | `tail_tests.rs` | `retrigger(world)` returns a complete Event order; `NotAllowed` per R-65; `MissingTrigger` when the World dropped the Character |

Seam contract (CASE-tagged): `quantifier-analysis-output` in `src/application/agents/quantifier/agent_tests.rs` — the agent returns `Analysis`; the parse DTO stays inside the box. `analysis-merge-conflict-first-wins` (id union, first destination, least confidence) sits with the model file `agent_tests.rs`.

Tier-1 (`tests/http/`, domain outcomes, most already exist and must stay green): `actions.rs` (the state each kind leaves; the event generation's no-movement rule), `retrigger.rs` (the R-65 allowed set; the event Narration as a new Message), `swipe_new.rs` (Retry by kind of last Message; a Retry of the opening reports its starting room), `races.rs` (the event Narration's preset).

## 6. Trade-offs

**High leverage.** `Generation` makes 8 guard branches and 2 failure lines unrepresentable, and the common path matches on no kind: every caller of the order (gate, Job, Tail, the later loop ticket) reads facts, not data it must check. `apply` deletes the event double-call, the manual state surgery, one fired-marking site, one event-record rule and 3 of 4 room lookups. `Analysis` deletes a 4-hop conversion chain and a duplicate confidence enum.

**Thin.** `Rules` is near-flat (`Analysed` carries one bool). Accepted: it is exactly the kind's rules data, and it keeps `apply`'s signature at four parameters. The 4 variants repeat `place`/`start`; flatness buys exhaustive construction with no partial states. `Generation::rules(opening)` takes one fact the caller computes from the tail; the alternative (rules reading the story) would need storage.

**Behaviour changes, flagged.** A Retrigger's direction becomes the rendered prompt (today the raw copy, known): this makes it equal to what a Retry of the same event Narration reads, which today differs. The rules derive from the pre-analysis state instead of the start state (inferred equivalent: the rules never read `trigger_context`, and the fired-marking is idempotent).

**Migration.** `StoredInputs` and `EventRecord` stored JSON is unchanged, so old rows read; `Generation` is in-memory only. `scene.quantifier_confidence` rows read as `None` (serde default) until the rules write one. The `Kind` deletion is mechanical for the agent test files.

**If the assumptions flip.** A required Impersonate direction: `Impersonate { direction: String }` plus one check at the two constructors (`plans.rs`, `Tail::redo`); the common path does not change. A tighter Retrigger rule: `retrigger_allowed` is already the one home both surfaces read.
