# Decide the shape of a generation: its kind, its analysis output and the Game rules

Type: grilling (HITL)
Status: resolved
Blocked by: —

## Question

What types carry one generation from the request to the saved Swipe state, so that the states the design calls impossible cannot be built? Decide, as one design:

1. **The kind carries its data** (finding 3): an event generation carries its event record, an Impersonate carries its direction (or none), and the runtime checks that guard the missing data go away. This decides the answer to a bare `/impersonate` (finding 1, UI F2) and which last Messages allow a Retrigger (UI F3: spec 14.3 against `Tail::retrigger_allowed`).
2. **One analysis output type** from the analysis hook to the Game rules (candidate M4, finding 17): one confidence enum, no `QuantifierResult` to `StatePatch` round trip, no dead `SceneState.quantifier_confidence`.
3. **The Game rules as one call per generation** (candidate M1, findings 4 and 12): one room lookup that respects `dynamic_rooms`, the event rules, the one-shot Trigger mark, the opening room report, and the rendered event record (the Retrigger direction through `TemplateVars`). Inside the `arch-lint.toml` rule that keeps `WorldBundle` out of the inputs of the Game rules.
4. **Where the Job plan rules live** (candidate S1, finding 5): the `Tail` or `gate.rs`.

The answer is the interfaces and the tests that pin them; the build is a later slice.

## Context

- Inputs: [synthesis.md](../assets/architecture-review/synthesis.md) rows M1, M4, S1 and §4; [FINDINGS.md](../assets/whole-rebuild-review/FINDINGS.md) findings 1, 3, 4, 5, 12, 17; [the UI findings](../assets/ui-review/findings.md) F2, F3.
- Thermo-nuclear's proposal: `../assets/whole-rebuild-review/findings/full-review-thermo.md`, "What would make this approvable" (H1, H3, H4).
- Skills: `/grilling`, `/domain-modeling`, `/codebase-design` (its design-it-twice pattern for the interfaces). Terms: `CONTEXT.md`. Design: `.scratch/_closed/turn-state-model/assets/target-design.md`.

## Answer

Resolved with the owner on 2026-10-20 (grilling, 15 questions). Interface drafts (design-it-twice): [design 1, minimal](../assets/generation-shape/design-1-minimal.md) (Opus), [design 2, common caller](../assets/generation-shape/design-2-common-caller.md) (GLM), [design 3, types](../assets/generation-shape/design-3-types.md) (Qwen), from [the brief](../assets/generation-shape/brief.md). Marinara facts: [marinara-agents.md](../assets/generation-shape/marinara-agents.md). The answer below takes parts of all three drafts; where they differ, the owner decision wins.

### Owner decisions

| Q | Decision |
|---|---|
| Q1 | A bare `/impersonate` is allowed (the baseline did this). An Impersonate has a direction or none; a Retry of such an Input runs with none. |
| Q2 | Retrigger follows R-65: a Narration that is not an event Narration, or an Impersonate Input, with a trigger context. `Tail::retrigger_allowed` is the one rule; the story-log button reads it and adds no condition. Spec 14.3 changes to "a typed Input". |
| Q3 | The trigger context holds the whole event record (Trigger reference, header name, rendered direction), as target design §2.3 says. The Game rules render the direction once, when the Trigger fires. A Retrigger copies the record from the current state, with no World read. |
| Q4 | The Quantifier result holds only the Characters present and the destination. Confidence and movement type stay private to the Quantifier, with one confidence enum. `SceneState.quantifier_confidence` and its debug key are deleted. |
| Q5 | No generic term ("analysis" generalises from one Agent). The type is the **Quantifier result**. |
| Q6 | Each Agent kind has its own result type, and each result type has one landing (Marinara model). No merge of several outputs. |
| Q7, Q8 | Hooks are named by position: the before-narration hook, the after-narration hook (after the text save, before the state update) and the end-of-Job hook (after the last generation, or alone for `/options`). What an Agent does is its result type; a rewrite is a result type, not a hook. |
| Q9 | The plan rules are a pure domain fn. `gate.rs` keeps the guard, the storage checks and its save units. |
| Q10 | Moved to [ticket 25](25-messages-rooms-and-mapless-worlds.md): the owner sees a deeper problem in how rooms and Messages are linked and shown. This ticket keeps today's room-report behaviour. |
| Q11 | When a Trigger fires, the Game rules return the fired event record. The Job builds the event order. |
| Q12 | A new migration deletes the saved Games (as v28 did), because the stored trigger context changes shape. |
| Q13 | The order keeps "ids and texts only" (sketch 25 §2.1, "pass ids, load late, save small"): `start: SwipeAt`, not the state. The first step of the generation loads the state. |
| Q14 | One domain fn on the order builds the new Swipe and, for an analysed kind, the `PreAnalysisState`. Only that fn makes one; only the Game rules take one. |
| Q15 | `Kind` stays a fieldless tag that the order derives. `HookInput` loses `kind` and `event` (no Agent reads them). |

**Amended by [ticket 20](20-shape-of-the-job.md) Q13 (owner, 2026-10-20):** `PreAnalysisState` is deleted, because it carried a derived state from the text-save step to the Game-rules step (against sketch 25 §2.1). `GenerationOrder::write(text, start)` returns only the `NewSwipe`; its Swipe state is the pre-analysis state, and `write` still marks an event's own one-shot Trigger fired, once. The Game-rules step loads the Swipe state at `saved.at` and calls `game_rules::apply(state: SwipeState, &QuantifierResult, &GameWorld)`. R-18 is a match on the order: a user regen ends after its text save. The test `write-user-regen-not-analysed` asserts the start state and the player maker; a plan test asserts that a user regen has no analysis step. The interface text below is the original; read it with this amendment.

**Amended by [ticket 25](25-messages-rooms-and-mapless-worlds.md) (owner, 2026-10-20):** the stored room report is deleted, so `RulesOutcome` has no `room_report` field, and the `room_report` line of interface 4 no longer applies.

**Amended by [ticket 28](28-confirm-the-fills-of-the-response-plan.md) (owner, 2026-10-20):** `game_rules::apply` takes a fourth parameter, `kind: Kind`, because a loaded Swipe state cannot tell an event from a Narration. A missing preset of any kind gives the `Record::Preset` line, and `PresetMissing` goes.

### The interfaces

**1. The kind carries its data** (`src/domain/model/generation.rs`)

```rust
pub enum Generation {
    Narration { guide: Option<String> },
    Impersonate { direction: Option<String> },
    UserRegen,
    Event(EventRecord),
}
pub struct GenerationOrder { generation: Generation, place: Place, start: SwipeAt }
impl GenerationOrder {
    fn kind(&self) -> Kind;                                        // status step, health role, D-31 filter
    fn event_after(saved: SwipeAt, record: EventRecord) -> Self;   // the Job builds the event order
    fn write(&self, text: String, start: &SwipeState) -> (NewSwipe, Option<PreAnalysisState>);
}
```

- `write` is the one home of the Message kind, the message type, the maker, the stored inputs and the event header (today in `generation_record.rs`, `gate.rs` and `job.rs`).
- `write` is the only maker of `PreAnalysisState`. It marks an event's own one-shot Trigger fired, once. A user regen gets `None`, so the Job cannot run the Game rules on it (R-18 as a type).
- Deleted: `GenerationOrder.inputs` and `.event`; the missing-data guards (`require_job_data` direction and event fill, `job.rs` `user_message` and `apply_rules` "the event record", `message_kind_of`, the `None` arm of `generation_record::message_kind`); `HookInput.kind` and `.event`.

**2. The plan rules** (the same file)

```rust
JobPlan::for_request(request: &JobRequest, story: StoryEnd { tail, has_text }) -> Result<JobPlan, PlanRefusal>
// PlanRefusal::BadRequest(line) → 400; PlanRefusal::Missing(name) → 500 with the failure line
```

- `JobRequest`, `JobPlan` and `OptionsRun` move from `gate.rs` to the domain. `Tail::redo` and `Tail::retrigger` become private parts of this fn. `Tail::retrigger_allowed` stays on the Tail.
- `gate.rs` keeps the guard, the storage checks (Game row, Persona, start state, has text) and its save units. Deleted from it: `plan_for`, `generation_plan`, `require_start`, `event_record`, `trigger_direction`, most of `require_job_data`.

**3. The Quantifier result**

```rust
pub struct QuantifierResult { present: Vec<String>, destination: Option<String> }
```

- The Agent at the after-narration hook returns it; the Game rules take it.
- Deleted: `StatePatch`, `AnalysisOutput`, `StatePatch::merge`, the second confidence enum and its two `From` impls, the public `QuantifierParseResult` and `MovementParseResult` (parser-private), `SceneState.quantifier_confidence`, `DebugStateView.quantifier_confidence`.

**4. The Game rules: one call per generation** (`src/domain/model/utils/game_rules.rs`)

```rust
pub struct GameWorld<'a> { map: &'a MapDef, characters: &'a HashMap<String, NpcCard>, persona_name: &'a str }
pub fn apply(pre: PreAnalysisState, result: &QuantifierResult, world: &GameWorld<'_>) -> Result<RulesOutcome>;
pub struct RulesOutcome { state: SwipeState, room_report: Option<String>, fired: Option<EventRecord> }
impl SwipeState { fn room(&self, map: &MapDef) -> Option<&Room>; }   // the one room lookup
```

- Narration and Impersonate: movement; the Characters present; the Trigger (it reads `times_met` before the encounters advance it); the encounters. The trigger context becomes the fired event record, with its direction rendered through `TemplateVars`, once.
- Event: no movement, no Trigger check, no trigger context.
- `Err` is the contradiction of the diagnostics check, as today.
- `GameWorld` is a domain struct; the storage adapter builds it from `WorldBundle`, so the `arch-lint.toml` rule "model → storage" holds.
- `room_report`: today's behaviour, until ticket 25 decides it.
- Deleted: `GameRuleRequest` (3 variants), `TriggerMatch` and its two `From` impls, `trigger_narration`, `SwipeState::pre_analysis`, the double call and the hand-cleared context in `Job::apply_rules`, three of the four room lookups (`job.rs` `room_of`, `view_query.rs` inline, `gate.rs` `opening_room_report` with its `.ok()`).

**5. Stored data.** The trigger context changes from `TriggerRef` to `EventRecord`. A new migration deletes the saved Games.

**6. Doc changes that come with the build.** Target design §4.5 (the direction is no longer data that must exist), §2.3 unchanged; D-29 and D-34 (hooks by position; each result type has one landing); spec `retrigger.md` 14.3; the `TriggerRef` doc ("a reference only"). `CONTEXT.md` is already done: Quantifier, Agent, Hook and Game rules entries.

### Tests that pin the interfaces

| Tier | CASE id | Asserts |
|---|---|---|
| unit, `generation_tests.rs` | `plan-retry-by-last-kind` (5 rows) | each kind of last Message gives its `Generation` variant, `NewSwipeOnLast` and the start of design §4.5 |
| unit | `plan-retry-bare-impersonate` | an Impersonate Input with no direction gives `Impersonate { direction: None }` |
| unit | `plan-retrigger-refused` (R-65 rows) | no context; last is an event Narration; last is a typed Input: `BadRequest` |
| unit | `plan-retrigger-copies-record` | the order holds the record of the current state's trigger context |
| unit | `plan-action-no-story`, `plan-options-no-text` | `Missing("the story")`; `BadRequest` |
| unit | `write-event-marks-one-shot` | an event's pre-analysis state has no trigger context and its one-shot Trigger in the fired set; a repeat Trigger is not added |
| unit | `write-user-regen-not-analysed` | the Swipe holds the start state, the maker is the player, no `PreAnalysisState` |
| unit, `game_rules_tests.rs` | `game-rules-event-no-movement` | a destination does not move the player for an event |
| unit | `game-rules-event-no-trigger` | no trigger context and no fired record, even when a Trigger condition holds |
| unit | `game-rules-times-met-before-encounters` | a first-encounter Trigger fires on the Narration that brings its Character |
| unit | `game-rules-fired-record-rendered` | the fired record's direction has `{{user}}` replaced by the Persona name; the trigger context equals the record |
| unit | `room-added-room` | map room, else added room, else none |
| seam contract | `quantifier-result-output` | the Quantifier returns a `QuantifierResult`; confidence stays inside |
| tier 1 | `actions.rs` | a bare `/impersonate` writes an Impersonate Input (new scenario) |
| tier 1 | `swipe_new.rs` | a Retry of a bare Impersonate Input runs (new scenario) |
| tier 1 | `retrigger.rs`, `story_log.rs` | a Retrigger on an Impersonate Input with a trigger context runs, and the button shows (spec 14.3 changed) |
| tier 1 | `retrigger.rs` | a Retrigger sends the rendered direction to the model (`ScriptedModel` call record) |

### Handed on

- [Decide the shape of the Job](20-shape-of-the-job.md): the Job loop, the failure lines, the narrator prompt rules, and the duplicate start-state read (`job.rs` `start_state` and `JobWriter::state_at`).
- [The story log past 50 Messages](23-story-log-past-50-messages.md): the story-log window; the Retrigger button reads `Tail::retrigger_allowed`.
- [Rethink Messages and rooms](25-messages-rooms-and-mapless-worlds.md): the room report, the opening fact, mapless Worlds.
- Later Agent work: if Agent result types grow toward Marinara's open set with custom Agents, the result type may become a trait object, not a closed enum.
