# Design 1 (minimal interface): three entry points for the plan rules and the Game rules

Constraint: 1 to 3 entry points in total. This design has 3. Each entry point is a step of the generation that the Job already has. Each one takes values only.

| # | Entry point | Rules behind it | Called by |
|---|---|---|---|
| 1 | `JobPlan::for_request` | the plan rules: request + end of the story + World → the plan, or a refusal | `gate.rs` `accept` |
| 2 | `GenerationOrder::write` | the text save rules: Message kind, maker, stored inputs, event header, pre-analysis state, room report | `job.rs` `generate`, before `save_text` |
| 3 | `Pending::apply` | the Game rules: movement, Characters present, encounters, Trigger, own one-shot Trigger, trigger context, room report, the next order | `job.rs` `generate`, before `save_state_update` |

## 1. Interface

### 1.1 The kind carries its data

```rust
// src/domain/model/generation.rs
pub enum Generation {
    /// `opening`: a Retry of the opening Narration. It re-applies no input and reports its room.
    Narration { guide: Option<String>, opening: bool },
    /// A bare `/impersonate` has no direction (assumption A1). The type allows both.
    Impersonate { direction: Option<String> },
    /// The text of the active Swipe of the typed Input, as it is at the accept.
    UserRegen { text: String },
    Event(EventRecord),
}

pub struct GenerationOrder {
    pub generation: Generation,
    pub place: Place,            // unchanged
    pub start: SwipeAt,
    pub start_state: SwipeState, // new: the plan has it in the Tail (see §6)
}

impl GenerationOrder {
    pub fn kind(&self) -> Kind;            // fieldless discriminant: today's `Kind`, with step() and role()
    pub fn saves_a_narration(&self) -> bool;
}
```

`Kind` stays a fieldless enum (moved from `agent.rs` to `generation.rs`). It names the status step and the role for health. It carries no data, so no code can ask it for a direction or a record. `GenerationOrder::kind()` and `saves_a_narration()` are facts, not entry points: they run no rule.

Invariants (by construction):
- An event order always holds its event record. An Impersonate order holds `Option<String>`, so a flip of A1 changes one line of entry point 1, not the type.
- An order always holds its start state. A missing start state is a plan refusal, at the accept.

### 1.2 The World as values

```rust
// src/domain/model/generation.rs
/// The World and the Persona of one Game, as the rules read them.
pub struct GameWorld<'a> {
    pub map: &'a MapDef,
    pub characters: &'a HashMap<String, NpcCard>,
    pub persona: &'a PersonaCard,
}
```

The storage adapter builds it: `WorldBundle::game_world(&self) -> GameWorld<'_>` in `worlds.rs`. Storage may import the model; the model never imports storage.

### 1.3 Entry point 1: the plan rules

```rust
// src/domain/model/generation.rs  (JobRequest, JobPlan, OptionsRun move here from gate.rs)
pub struct StoryEnd<'a> { pub tail: Option<&'a Tail>, pub has_text: bool }

pub enum PlanRefusal {
    /// `400`, with the reason line.
    BadRequest(&'static str),
    /// `500`: the record the request needs is not there. The gate writes the failure line.
    Missing(&'static str),
}

impl JobPlan {
    pub fn for_request(request: &JobRequest, story: StoryEnd<'_>, world: &GameWorld<'_>)
        -> Result<JobPlan, PlanRefusal>;
    pub fn first_step(&self) -> Step; // unchanged fact
}
```

Rules inside (order of checks is fixed):
1. `Options`: `has_text` false → `BadRequest("No scene to generate options for yet")`.
2. `Retry`: no Tail → `BadRequest("No input to retry")`. Else the redo by the kind of the last Message (target design §4.5): Narration → `Narration { guide, opening }`; event Narration → `Event(stored record)`; Impersonate Input → `Impersonate { direction }`; typed Input → `UserRegen { text }`. Start state: `state_before_last`, or the first Swipe of the opening.
3. `Retrigger`: `!tail.retrigger_allowed()` → `BadRequest("No trigger context available")`. Else `Event(record)`, with the record from the private `event_record(trigger_ref, world)`. A Trigger that the World does not hold → `Missing("the trigger")`.
4. `Action`, `Guide`, `Impersonate`: no Tail → `Missing("the story")`. Start: the current state.
5. Any order whose start state is absent → `Missing("the start state")`.

`event_record` is the one rule that builds an event record. It renders the direction through `TemplateVars` with the Persona name. Entry point 3 calls the same fn for a fired Trigger. So a Retrigger and a fired Trigger get the same direction text.

`Tail::retrigger_allowed` stays on `Tail`. It is the one home of the rule (A2). The story log reads it too (§2.3).

### 1.4 Entry point 2: the text save rules

```rust
pub struct NewText {
    pub message_type: MessageType,
    pub message_kind: MessageKind,      // total: Event(record) → EventNarration(record)
    pub swipe: Swipe,                   // text, maker, stored inputs, event header, room report, pre-analysis state
    pub empties_the_dock: bool,         // == saves_a_narration()
}

impl GenerationOrder {
    /// `None` for a user regen: it has no analysis, so the Job cannot apply rules to it.
    pub fn write(&self, text: String, world: &GameWorld<'_>) -> (NewText, Option<Pending>);
}
```

The pre-analysis state: the start state with no trigger context. An event also marks its own one-shot Trigger fired. A user regen keeps the start state. The room report: none, except the opening Retry, which reports the room of its start state. No error mode: every input is valid by type.

### 1.5 Entry point 3: the Game rules

```rust
// src/domain/model/utils/game_rules.rs
pub struct Pending { /* private: the Generation, the start state */ }

pub struct Analysed {
    pub state: SwipeState,
    pub room_report: Option<String>,
    /// The event order that a fired Trigger starts: a new Message, from `saved`, with the analysed state.
    pub next: Option<GenerationOrder>,
}

impl Pending {
    pub fn apply(self, analysis: &Analysis, saved: SwipeAt, world: &GameWorld<'_>)
        -> crate::error::Result<Analysed>;   // Err: a contradiction (diagnostics check)
}
```

Rules inside, applied to the start state:
- Narration and Impersonate: movement; Characters present; Triggers read `times_met`; then the Character events advance it; trigger context = the fired Trigger; `next` = its event order. Room report: the room arrived in, or, for the opening, the room after it.
- Event: no movement; Characters present; Character events; own one-shot Trigger fired; no trigger context; no Trigger evaluated; `next` is `None`; no room report. This makes the limit of two generations per Job a fact of the type.

The room lookup "map room, else added room" is `SwipeState::room(&self, map) -> Option<&Room>` in `swipe_state.rs`. It is a helper, not a rule entry point. The Game rules, the Job room check, `view_query.rs` and `WorldView` all call it.

### 1.6 One analysis output type

```rust
// src/domain/model/agent.rs
pub struct Analysis {
    pub present: Vec<String>,          // Character ids
    pub destination: Option<String>,
}
impl Analysis {
    pub fn merge(self, other: Analysis) -> Analysis; // the first destination wins (unchanged rule)
}
```

`AnalysisAgent::run` returns `Result<Analysis, EngineError>`. `AgentRunner::analysis` returns `Analysis` (`Analysis::default()` when every Agent fails). The Quantifier keeps its confidence private (`quantifier/types.rs`) for its own retry and its NPC filter. No reader outside the Quantifier reads a confidence (known: the Game rules read `npc_ids` and `destination` only).

## 2. Usage

### 2.1 `gate.rs` `accept`

```rust
let tail = self.tail_or_refuse(game, units)?;
let has_text = self.story_has_text(game, units)?;
let bundle = self.bundle_or_refuse(game, units)?;   // game row, persona, World: the line names the one that failed
let plan = JobPlan::for_request(&request, StoryEnd { tail: tail.as_ref(), has_text }, &bundle.game_world())
    .map_err(|refusal| match refusal {
        PlanRefusal::BadRequest(line) => Refusal::BadRequest(line.to_string()),
        PlanRefusal::Missing(name) => self.missing(game, name, units),
    })?;
```

### 2.2 `job.rs` `generate`

```rust
let world = bundle.game_world();
let step = order.kind().step();
self.run_step(step, || order.start_state.room(world.map).map(drop).ok_or_else(invalid_room))?;
// Input save, then the narrator call. The narrator gets `&order.generation` (see §2.4).
let (new_text, pending) = order.write(narrated, &world);
let saved = self.run_step(step, || self.writer.save_text(order.place, new_text))?;
let Some(pending) = pending else { return Ok(Generated { narration: false, next: None }) };
let analysis = self.run_step(Step::Quantifying, || self.analyse_step(order, &world, ...))?;
let analysed = self.run_step(Step::Quantifying, || pending.apply(&analysis, saved.at(), &world).map_err(Self::contradiction))?;
self.run_step(Step::Quantifying, || self.writer.save_state_update(saved, analysed.state.clone(), analysed.room_report))?;
Ok(Generated { narration: order.saves_a_narration(), next: analysed.next })
```

`save_text(place, NewText)` matches on `Place` only. It holds no kind branch and no room read.

### 2.3 The Tail and the story log

`Tail` keeps `current_state`, `current_at`, `start_at`, `retrigger_allowed`, `can_delete`. `redo` and `retrigger` move into entry point 1 as private fns. `app_state.rs` passes `tail.retrigger_allowed()` to `NarrativeLogTemplate::new`, and `templates.rs` stops its own test (`is_narration && !is_event_continuation`).

### 2.4 What the later ticket gets

- `Job::run` as one loop: `while let Some(order) = next { next = self.generate(&order, input.take())?.next }`. The rules make the chain; the loop does not build an event order.
- The failure lines: `PlanRefusal` (accept) and the `Err` of `apply` (contradiction). `write` has no error.
- The narrator prompt rules: `&Generation`. A total match gives the direction, the guide, the user regen text and the event direction with no missing branch.

## 3. What hides behind the seam (deleted or made private)

| Point | Deleted | Count |
|---|---|---|
| 1 kind + data | `GenerationOrder.inputs`, `.event`; the guards `require_job_data` (impersonate direction, event fill), `message_kind_of`, `job.rs` `user_message` and `apply_rules` `Failure::missing("the event record")`, `generation_record::message_kind` `None` arm, `Tail::retrigger` `event: None`; `JobWriter::empties_the_dock`, `text_swipe`; `Job::event_order`; `HookInput.kind`, `HookInput.event` (known: no Agent reads them); `generation_record.rs` (3 fns become private in `generation.rs`) | 7 runtime branches; 2 fields; 6 fns; 1 module |
| 2 analysis | `QuantifierResult`, `QuantifierParseResult`, `MovementParseResult`, `MovementType`, `AnalysisOutput`, `StatePatch` (renamed `Analysis`), domain `Confidence`, `QuantifierConfidence` (made private), 2 `From` impls, `NpcEventList.confidence`, `SceneState.quantifier_confidence`, `DebugStateView.quantifier_confidence`; `src/domain/model/quantifier.rs` | 9 types; 2 impls; 3 fields; 1 module |
| 3 Game rules | `apply_game_rules`, `GameRuleRequest` (3 variants, `NpcEvents` had no caller), `GameRulesOutcome` (4 fields), `TriggerMatch` + 2 `From` impls, `trigger_narration` and its dead empty-text branch (known: `narrator.rs` fails an empty text), `SwipeState::pre_analysis`, the double call and the manual `trigger_context = None` in `Job::apply_rules`; room lookups `Job::room_of`, `Job::room_name`, `view_query.rs` inline, `opening_room_report` (with its `.ok()`); `Job::is_the_opening`, `save_text` `tail.count == 1` | 4 types; 9 fns; 2 rules for the event record → 1 |
| 4 plan | from `gate.rs`: `plan_for`, `generation_plan`, `require_start`, `event_record`, `trigger_direction`, most of `require_job_data`; `Tail::redo`, `redo_kind`, `event_record`, `retrigger` | 10 fns; `gate.rs` loses about 200 lines |

## 4. Where each piece lives

| Piece | File | arch-lint |
|---|---|---|
| `Generation`, `Kind`, `GenerationOrder`, `Place`, `GameWorld`, `JobRequest`, `JobPlan`, `OptionsRun`, `PlanRefusal`, `StoryEnd`, `NewText`; entry points 1, 2 | `src/domain/model/generation.rs` | model; imports model only |
| `Pending`, `Analysed`; entry point 3 | `src/domain/model/utils/game_rules.rs` | model |
| `SwipeState::room` | `src/domain/model/state/swipe_state.rs` | model |
| `Analysis` | `src/domain/model/agent.rs` | model |
| Quantifier confidence (private) | `src/application/agents/quantifier/types.rs` | agent-edge; no storage |
| `WorldBundle::game_world` | `src/adapters/driven/storage/worlds.rs` | storage → model is allowed |
| `gate.rs` | re-exports `JobRequest` for the HTTP handlers, or the handlers import it from the model | application → model is allowed |

The rules take `GameWorld`, never `WorldBundle`, so "model → storage" holds. The narrator and the Agents take `&Generation` and `Analysis`, which are values, so "agent-edge → storage" holds.

## 5. Tests that pin the interface

Unit tests are rule 2 of `tests/STRATEGY.md` (pure `domain/model/` functions). Each entry point is the test surface; no test calls a private fn.

| CASE id | Tier | Asserts |
|---|---|---|
| `plan-retry-by-last-kind` (4 rows) | unit, `generation_tests.rs` | each kind of last Message gives its `Generation` variant, `Place::NewSwipeOnLast` and the start state of the Message before |
| `plan-retry-opening` | unit | `Narration { guide: None, opening: true }`; start = first Swipe of the opening |
| `plan-retry-bare-impersonate` | unit | an Impersonate Input with no direction gives `Impersonate { direction: None }` (A1) |
| `plan-retrigger-refused` (3 rows) | unit | no context; last is an event Narration; last is a typed Input → `BadRequest` |
| `plan-retrigger-direction-rendered` | unit | the record direction equals the fired-Trigger direction for the same Trigger (Persona name filled) |
| `plan-retrigger-unknown-trigger` | unit | `Missing("the trigger")` |
| `plan-options-no-text` / `plan-action-no-story` | unit | `BadRequest` / `Missing("the story")` |
| `write-event-pre-state` | unit | no trigger context; own one-shot Trigger in the fired set; a repeat Trigger is not added |
| `write-user-regen-no-pending` | unit | the Swipe holds the start state; `Pending` is `None`; maker is the player |
| `write-opening-room-report` | unit | the opening Retry reports its starting room; other kinds report none |
| `game-rules-event-no-movement` | unit, `game_rules_tests.rs` | a destination in the analysis does not move the player for an event |
| `game-rules-event-no-trigger` | unit | no trigger context, `next` is `None`, even when a Trigger condition holds |
| `game-rules-times-met-before-events` | unit | a first-encounter Trigger fires on the Narration that brings its Character |
| `game-rules-fired-next-order` | unit | trigger context set; `next` is an event order at `saved`, with the analysed state |
| `game-rules-opening-room-after` | unit | the opening Retry reports the room after its analysis |
| `game-rules-added-room` | unit | an unknown destination adds a room; the next lookup finds it |
| `analysis-merge-first-destination` | unit, `agent_tests.rs` | the first destination wins; present ids are the union in order |
| retrigger direction scenario (new, `retrigger.md`) | tier 1, `tests/http/retrigger.rs` | the `ScriptedModel` call record of a Retrigger holds the rendered direction |
| opening Retry room scenario (new, `swipe_new.md`) | tier 1, `tests/http/swipe_new.rs` | the new Swipe shows the room after it |
| Retrigger button on an Impersonate Input (new, `story_log.md`) | tier 1, `tests/http/story_log.rs` | the button shows when `Tail::retrigger_allowed` is true |
| unreadable World at accept (extends actions 2.6) | tier 1, `tests/http/actions.rs` | `500` with the missing-data line; no Job starts |

The Quantifier seam contract keeps its retry case on low confidence; the confidence is now private to it.

## 6. Trade-offs

| Topic | This design | Cost or risk |
|---|---|---|
| Leverage, high | 3 entry points replace about 25 fns and 13 types. Each kind rule (pre-analysis, room report, event record, one-shot fire) has one home. | `generation.rs` grows to about 300 lines. |
| Leverage, thin | `StoryEnd.has_text` is a bool that only `Options` reads. | The gate reads `has_text` on every accept: one more query under the guard (inferred: cheap). |
| The order holds its start state | No `start_state` read in the Job; the start-state `500` moves to the plan. Inferred safe: nothing changes a Swipe state while the Game is busy (target design §3.1). | Breaks today's rule "ids and texts only" in `generation.rs`. One `SwipeState` clone per order. |
| The World is read at accept | Retrigger needs it. One read serves every plan. | Behaviour change (inferred): an unreadable World now answers `500` at the accept. Today the Job fails later with a failure line. This matches target design §4.1 step 4, but the owner must accept it. |
| `Event` evaluates no Trigger | Today the rules evaluate and then ignore. The result is the same. | None known. |
| Story log button | One rule. | Behaviour change (known): today `templates.rs` hides the button on an Impersonate Input; `Tail::retrigger_allowed` allows it. |
| A1 or A2 flips | A1: one refusal line in entry point 1. A2: one fn on `Tail`. | None. |

Migration (known): `StoredInputs` is three columns on `swipes` (`impersonated`, `guide_text`, `impersonate_direction`), not JSON. `EventRecord` is inside `messages.kind_json` as `MessageKind::EventNarration`. Neither shape changes: entry point 1 reads them back into `Generation`, and entry point 2 writes them out. `SceneState.quantifier_confidence` goes away; old `state_json` still reads, because no type uses `deny_unknown_fields` (known, grep).
