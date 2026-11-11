# Design 3: make impossible states unrepresentable (ticket 19)

Constraint: every rule the brief names moves into the type system.
Data-carrying enums replace `kind + Option<data>`; a planned order and a
ready order are two types, so an event order without its record cannot reach
`generate`. Claims: *known* (read in code), *inferred*.

## 1. Interface

### 1.1 The order: planned, then ready (`src/domain/model/generation.rs`)

```rust
/// From the Tail alone. An event from a trigger context holds no direction
/// yet: the direction lives in the World, which the domain cannot read.
pub struct PlannedOrder { pub place: Place, pub start: SwipeAt, pub kind: PlannedKind }
pub enum PlannedKind {
    Narration { guide: Option<String> },
    Impersonate { direction: Option<String> }, // a bare /impersonate is allowed
    UserRegen,
    Event { source: EventSource },
}
pub enum EventSource {
    Record(EventRecord),  // Retry of an event Narration: the Message keeps it
    Context(TriggerRef),  // Retrigger or just-fired: direction comes from the World
}
/// The order with every datum in hand. `generate`, `save_text`, the game
/// rules and `pre_analysis` accept only this type.
pub struct GenerationOrder { pub place: Place, pub start: SwipeAt, pub kind: ReadyKind }
pub enum ReadyKind {
    Narration { guide: Option<String> },
    Impersonate { direction: Option<String> },
    UserRegen,
    Event { record: EventRecord }, // not an Option
}
pub enum Place {
    NewMessage,
    /// `opening`: the story holds only its opening; its Retry re-reports the
    /// starting room. The Tail sets the flag once.
    NewSwipeOnLast { opening: bool },
}
impl PlannedOrder {
    /// The one resolution step: `direction_of` is the caller's World read,
    /// `persona_name` renders through `TemplateVars`. `None` only for a
    /// `Context` the World cannot fill; accept maps that to its refusal.
    pub fn into_ready(
        self,
        persona_name: &str,
        direction_of: impl Fn(&TriggerRef) -> Option<String>,
    ) -> Option<GenerationOrder>;
}
```

Total projections on `ReadyKind` (one match each, no failure arm):
`kind() -> Kind` (for `HookInput`, role, step, narrator), `message_kind() ->
MessageKind` (the Event arm carries its record), `message_type()`,
`maker()`, `saves_a_narration()` (replaces `empties_the_dock`),
`stored_inputs() -> StoredInputs` (the Swipe-row projection),
`event_record()`, `impersonate_direction()`.

What each type rules out:

- `ReadyKind::Event { record }` — an event generation without a record (today: `event: Option<EventRecord>` plus 5 failure branches, known).
- `GenerationOrder` vs `PlannedOrder` — an unresolved `Context` reaching the Job: only `into_ready` makes a `GenerationOrder`.
- `ReadyKind::Impersonate { direction }` — a direction on a Narration, or `impersonated: true` on an Event (today `StoredInputs` allows both, known).
- `Place::NewSwipeOnLast { opening }` — re-deriving opening-ness from a fresh `tail.count` read (today: three derivations, known).

Stored JSON stays readable: `StoredInputs` and `EventRecord` keep their serde
shapes (known); `stored_inputs()` writes the fields the Swipe row holds. If
the owner later bans a bare `/impersonate`, the flip is one check where
accept builds `PlannedKind::Impersonate` — no type changes (inferred).

### 1.2 One analysis output (`src/domain/model/analysis.rs`, new)

```rust
/// The analysis hook's one output: the agent's result, the runner's merge
/// input and the game rules' input are this type.
pub struct Analysis {
    pub characters_present: Vec<String>,
    /// `None` is "did not move": a movement without a destination cannot exist.
    pub movement: Option<Destination>,
    pub confidence: Confidence, // the one enum, moved from agent.rs
}
pub struct Destination(pub String); // the rules resolve it, or add a room
impl Analysis {
    /// Union of Characters, first destination wins, least confidence wins.
    pub fn merge_all(analyses: Vec<Analysis>) -> Analysis;
}
```

Ruled out: `movement_type` with no destination (today's dead shape, known);
two confidence enums with two `From` impls; a `StatePatch` diverging from the
parser result. `MovementType` stays parser-internal for its log line
(inferred; only the parser and one log read it, known); the retry reads
`analysis.confidence.is_low()`. `NpcEvent`, `NpcTransitionType` and
`from_diff` become private to `game_rules.rs` (the `NpcEvents` variant has no
production caller, known). `SceneState.quantifier_confidence` is deleted: no
production writer (known), and no state type has `deny_unknown_fields`
(known), so stored JSON with the key still reads.

### 1.3 The rules: one call per generation (`src/domain/model/utils/game_rules.rs`)

```rust
pub struct GameRules<'a> {           // domain types only, never WorldBundle
    pub map: &'a MapDef,
    pub characters: &'a HashMap<String, NpcCard>,
    pub persona_name: &'a str,
}
pub struct RuleEffect {
    pub state: SwipeState,           // context set/cleared per kind
    pub room_report: Option<String>, // the event arm builds it as None
    pub fired: Option<EventRecord>,  // TemplateVars-rendered — the one build
}                                    // rule; None for an event order
impl GameRules<'_> {
    /// One call per generation. The order's kind selects the rule; the caller
    /// passes no request variant and clears no field by hand.
    pub fn apply(
        &self,
        pre: PreAnalysisState,
        order: &GenerationOrder,
        analysis: &Analysis,
        text_landed: bool, // the event landing rule: empty text lands nothing
    ) -> Result<RuleEffect, EngineError>;
}
/// The state a Swipe holds from its text save until its analysis updates it.
/// Only `SwipeState::pre_analysis` makes one; the rules consume one. A state
/// update without the rules does not typecheck.
pub struct PreAnalysisState(SwipeState);
impl SwipeState {
    pub fn pre_analysis(&self, order: &GenerationOrder) -> PreAnalysisState;
    /// The one room lookup: map room, else added room. The rules, the Job's
    /// room check, the view query and the opening report all use it.
    pub fn room_of<'m>(&self, map: &'m MapDef) -> Option<&'m Room>;
    pub fn room_name(&self, map: &MapDef) -> String; // room_of, else the id
}
```

`pre_analysis` matches the ready kind: Narration/Impersonate clear the
trigger context; Event clears it *and* marks the record's one-shot Trigger
fired — the marking's one home, because the rules take the
`PreAnalysisState` as their base and never re-mark; UserRegen copies the
start state (today's `match kind` in `job.rs` moves here, known). Ruled out:
the event double-apply with a hand-cleared context; the second fired-marking
in `trigger_narration`; a room report on an event generation. `Err` comes
only from the diagnostics consistency check, as today (known). The ordering
invariant stays: Triggers read `times_met` before the Character diff advances
it — inside one `apply`, unswappable by a caller.

### 1.4 Where the plan rules live (point 4)

Pure plan rules live in the domain: `Tail::redo`/`retrigger` return
`PlannedOrder`, set the `opening` flag and pick the `EventSource`;
`retrigger_allowed` stays the one home the server check and the story-log
button read (known). `gate.rs` keeps only what needs storage: the request →
`PlannedOrder` match, the `500` presence checks, and the resolver closure
(`world_bundle` + `persona` reads) for `into_ready`. `event_record` and
`trigger_direction` shrink into that closure; a Retrigger's direction now
renders through `TemplateVars` like a fired Trigger's, because `into_ready`
holds the one build rule (today the two rules differ, known).

## 2. Usage

`gate.rs` accept:

```rust
let planned = match request { // Action, Guide, Impersonate build PlannedOrder directly
    JobRequest::Retry     => tail.map(Tail::redo).ok_or(no_input_refusal)?,
    JobRequest::Retrigger => tail.and_then(Tail::retrigger).ok_or(no_context_refusal)?,
    ...
};
let order = planned
    .into_ready(&persona_name, |trig| self.trigger_direction(game, trig, units))
    .ok_or_else(|| self.missing(game, "the trigger direction", units))?;
```

`job.rs` generate — no `Option` unwrap, no per-kind data branch:

```rust
let pre = start.pre_analysis(order);                    // every kind, one call
let saved = self.writer.save_text(order, narrated, &pre)?;
if order.kind.kind() == Kind::UserRegen { return Ok(Generated::input(saved.at())); }
let analysis = self.analyse_step(order, &bundle, &start, ...)?;
let effect = self.rules(&bundle)?.apply(pre, order, &analysis, text_landed)?;
self.writer.save_state_update(saved, effect.state, effect.room_report)?;
Ok(Generated { narration: order.kind.saves_a_narration(), at, fired: effect.fired })
// the fired event: GenerationOrder::fired_event(record, at) — NewMessage, start = at
```

`save_text(&GenerationOrder, String, &PreAnalysisState)`: the opening report
at text save reads `order.place` (one flag, no `tail.count` read).
`save_state_update(saved: SavedText, ...)` unchanged — `SavedText`'s consume
trick already rules out a third state write (known). `Tail::redo` sets
`NewSwipeOnLast { opening: state_before_last.is_none() }`; `Tail::retrigger`
builds `Event { source: Context(trigger) }` at `NewMessage`, `current_at()`.

## 3. What hides behind the seam (deleted, counted)

- Runtime branches / failure arms: **9** — `require_job_data`'s
  impersonate-direction refusal and its event-record fill; `apply_rules`'s
  and `user_message`'s `Failure::missing("the event record")`;
  `message_kind`'s `None` arm; `job.rs`'s `match kind` for the pre state; the
  event double-apply with hand-cleared context; `trigger_narration`'s second
  fired-marking; `empties_the_dock` as a free match.
- Types deleted: **8** — `QuantifierResult`, `QuantifierParseResult`,
  `MovementParseResult`, `QuantifierConfidence` (+2 `From` impls),
  `StatePatch`, `AnalysisOutput`'s patch payload, `GameRuleRequest`
  (3 variants), `SceneState.quantifier_confidence`. `MovementType` leaves the
  domain; `NpcEvent`/`NpcEventList` become `game_rules`-private.
- Functions collapsed: **10** — `generation_record::{message_kind,
  message_type, maker}` become total methods; four `room_of` copies become
  `SwipeState::room_of`; three opening-report derivations become one `Place`
  flag; two event-record builds become `into_ready`'s one render; the
  runner's `QuantifierResult` rebuild becomes `Analysis::merge_all`;
  `gate.rs`'s `event_record`/`trigger_direction` become one closure (~160
  lines leave the 910-line file, inferred).

## 4. Where each piece lives

| Piece | File | arch-lint |
|---|---|---|
| `PlannedOrder`, `GenerationOrder`, `ReadyKind`, `Place`, projections | `src/domain/model/generation.rs` | pure: ids, texts, domain types |
| `Analysis`, `Destination`, `Confidence` | `src/domain/model/analysis.rs` (new; `agent.rs` re-exports `Confidence`) | pure |
| `pre_analysis`, `PreAnalysisState`, `room_of` | `src/domain/model/state/swipe_state.rs` | pure |
| `GameRules`, `RuleEffect`, private diff/trigger fns | `src/domain/model/utils/game_rules.rs` | takes `MapDef`, `NpcCard` map, `&str` — never `WorldBundle` (a storage type, known) |
| request match, `500` checks, resolver closure | `src/application/generation/gate.rs` | application may read storage |
| parser → `Analysis` | `src/application/agents/quantifier/utils/parser.rs` | agent-edge: values in, values out |

The World read enters the domain only as the closure, so `into_ready` stays a
pure function, testable with a fake resolver.

## 5. Tests that pin the interface

Unit (pure `domain/model`, STRATEGY.md one-rule #2):

- `generation-planned-context-needs-direction` — `into_ready` on `Context`: resolver `None` → `None`; `Some(raw)` → record with the rendered direction.
- `generation-ready-projections-total` — each `ReadyKind` variant: kind, message_kind, message_type, maker, stored_inputs, saves_a_narration.
- `tail-redo-opening-flag` — opening redo: `opening: true`, inputs default; redo after an Input: `opening: false`, the Swipe's inputs.
- `tail-retrigger-one-home` — `retrigger_allowed` per last-Message kind × trigger-context presence (R-65 table).
- `pre-analysis-event-marks-own-trigger-once` — one-shot record: fired after `pre_analysis`; `apply` leaves the set; a repeat Trigger marks none.
- `game-rules-event-no-movement` — event order + analysis with a destination: unmoved, `fired: None`, `room_report: None`, `trigger_context: None`.
- `game-rules-narration-fired-record` — fired Trigger: rendered direction in `fired`; context set; `times_met` read before the diff (first encounter fires).
- `game-rules-opening-retry-report` — opening flag + no movement: report is the current room's name, resolved through an added room.
- `analysis-merge-first-destination-wins` — union, min confidence, first wins.
- `swipe-state-room-of-map-else-added` — the one lookup, both sources.

Tier-1 (`tests/http/`), plus one driven-adapter:

- `retry-event-narration-keeps-record` (`swipe_new.rs`) — the new Swipe's event header equals the Message's record.
- `retrigger-direction-renders-template` (`retrigger.rs`) — the event prompt holds the persona name where the direction had the placeholder (LLM-message read seam).
- `retry-opening-reports-starting-room` (`swipe_new.rs`) — the story-log fragment shows the starting room on the retried opening, once.
- `bare-impersonate-runs` (`actions.rs`) — `/impersonate` with no text is accepted and saves an Impersonate Input (the assumption's pin).
- `stored-inputs-of-old-swipe-still-read` (`tests/storage/story_storage.rs`) — a Swipe row in today's serde shape reads back through `stored_inputs`.

## 6. Trade-offs

- **High leverage**: `ReadyKind` deletes 5 failure arms and makes
  `message_kind` total; the planned/ready split puts the World-read seam in
  one closure and makes "event without record" unconstructable; one `apply`
  makes movement-stripping, context-clearing and fired-marking unswappable.
- **Thin**: the `opening` flag is a small win — the report text still has two
  writers, because D-19 requires both save units to write it (known).
  `PreAnalysisState` buys ordering safety for one more type and a
  borrow/consume dance in `generate`.
- **Type cost**: two order types and two kind enums where today one flat
  struct stands. Justified only by the `Context` resolution seam; if a World
  read ever reaches the domain, the pair collapses to one.
- **Migration**: `StoredInputs`, `EventRecord`, `SwipeState` serde shapes
  unchanged; deleting `quantifier_confidence` is safe (no
  `deny_unknown_fields`, known). `Analysis` is never stored (no serde on
  today's quantifier types, known) — no migration.
- **Not designed here** (later ticket): `Job::run`'s loop, failure lines,
  narrator prompt rules. This interface hands them a `GenerationOrder` whose
  projections feed `HookInput` and the narrator.
