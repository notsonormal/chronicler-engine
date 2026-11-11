# Brief: the shape of a generation (ticket 19, design-it-twice)

You design one interface proposal. Read-only on `src/` and `tests/`: do not edit code. Write your proposal to the file your task names. Write in Simplified Technical English. Label claims *known* (read in code) or *inferred*.

## The problem

One generation goes from a request to a saved Swipe state. Today the types let states exist that the design calls impossible, and the domain rules are spread over the Job and the writer. Design **one** set of interfaces that fixes these four points together:

1. **The kind carries its data** (FINDINGS 3, 1). `GenerationOrder` (`src/domain/model/generation.rs`) has `kind: Kind` beside `inputs: StoredInputs` and `event: Option<EventRecord>`. About 7 runtime branches guard the missing data: `gate.rs` `require_job_data` (an Impersonate needs a direction; an event needs a record, which it fills), `job.rs` `user_message` and `apply_rules` (`Failure::missing("the event record")`), `generation_record::message_kind` (an event without a record writes no kind). `Kind` (`src/domain/model/agent.rs:55`) is also used by `HookInput`, `Kind::role`, `Kind::step`, `saves_a_narration`, and by the narrator. `Tail::redo` and `Tail::retrigger` (`src/domain/model/tail.rs`) build orders. `Tail::retrigger` cannot fill the event record, because the direction comes from the World (`gate.rs` `event_record`, `trigger_direction`).
2. **One analysis output type** (synthesis M4, FINDINGS 17). Today: the Quantifier makes a `QuantifierResult` (`src/domain/model/quantifier.rs`), the agent turns it into a `StatePatch` (`src/application/agents/quantifier/agent.rs`), the runner merges the patches and builds a new `QuantifierResult` (`src/application/agents/runner.rs`), and the Game rules read `npcs.npc_ids` and `movement.destination` only. Two confidence enums (`Confidence` in `agent.rs:217`, `QuantifierConfidence`) with two `From` impls. `SceneState.quantifier_confidence` (`src/domain/model/state/scene_state.rs`) has no production writer; only the debug view copies it. `movement_type` and the movement confidence are dropped; only the parser and a log line read them. The Quantifier's own retry reads `npcs.confidence.is_low()` (`quantifier/utils/orchestration.rs:125`).
3. **The Game rules as one call per generation** (synthesis M1, FINDINGS 4, 12). Today: `apply_game_rules(state, GameRuleRequest, map, npcs)` in `src/domain/model/utils/game_rules.rs` with three request variants; `NpcEvents` has no production caller. For an event generation, `Job::apply_rules` (`src/application/pipeline/job.rs` ~442-510) removes the movement, calls the rules twice and clears `trigger_context` by hand. The one-shot Trigger is marked fired in two places (`SwipeState::pre_analysis` in `src/domain/model/state/swipe_state.rs` and `game_rules.rs` `trigger_narration`). The room lookup "map room, else added room" is in four places (`game_rules.rs` `room_of`, `job.rs` `room_of`, `src/application/games/view_query.rs` ~61, `gate.rs` `opening_room_report` ~806, which ignores added rooms and drops read errors with `.ok()`). The opening room report is in three places (`src/domain/model/utils/opening.rs`, `gate.rs` `save_text` `tail.count == 1`, `job.rs` `is_the_opening`); a Retry of the opening writes it twice. Two rules build the event record: a fired Trigger renders its direction through `TemplateVars` (`game_rules.rs` ~119), a Retrigger copies the raw `narration_prompt` (`gate.rs` `trigger_direction`).
4. **Where the Job plan rules live** (synthesis S1, FINDINGS 5). `GameWriter::plan_for`, `generation_plan`, `require_start`, `require_job_data`, `event_record`, `trigger_direction` are in `src/application/generation/gate.rs` (~427-589), a 910-line file whose module doc promises the guard and its save units. Candidates: the `Tail` (`tail.rs`), or a new domain module, or stay in `gate.rs`.

## Hard constraints (known)

- `arch-lint.toml` rule "model → storage": `src/domain/model/**` must not import the storage adapter. `WorldBundle` (`src/adapters/driven/storage/worlds.rs:26`: `world`, `map`, `persona`, `npcs`) is a storage type. So the Game rules and any domain plan code take domain types (`MapDef`, `NpcCard` map, `PersonaCard`, `WorldCard`), or a new domain struct.
- The narrator and the Agents are in the `agent-edge` scope: they take values, not storage.
- The rules stay pure: no storage, no LLM. The Job does the reads and the writes through `JobWriter` save units (`save_text`, `save_state_update`).
- Behaviour to keep: `.scratch/_closed/turn-state-model/assets/target-design.md` §1 (pre-analysis state), §4.3 step table and the table "The Swipe state that each kind leaves" (event: no movement, no trigger context, its own one-shot Trigger in the fired set; room report rules; the opening reports its starting room), §4.5 (Retry by kind of last Message; Retrigger), §2.3 "Retrigger allowed". Triggers read `times_met` before the Character events advance it (`game_rules.rs` `analysis`).
- A later ticket decides `Job::run` as one loop, the failure lines and the narrator prompt rules. Do not design those; only say what your interface hands them.

## Assumptions to design for (the owner decides them in parallel; your interface should not break if they flip)

- A bare `/impersonate` is allowed: an Impersonate has a direction or none. A Retry of an Impersonate Input with no direction runs with none.
- A Retrigger is allowed when the last Message is a Narration that is not an event Narration, or an Impersonate Input, and the current state holds a trigger context (R-65, target design §2.3). The rule has one home that the server check and the story log button both read.

## Output (one Markdown file, at most ~250 lines)

1. **Interface**: the Rust types and signatures (sketch code), plus invariants, ordering and error modes. Name each type with `CONTEXT.md` terms.
2. **Usage**: how `gate.rs` (accept), `job.rs` (`generate`) and the Tail use it, as short code.
3. **What hides behind the seam**: which of today's branches, fns and types are deleted. Count them.
4. **Where each piece lives** (file paths) and how it stays inside `arch-lint.toml`.
5. **Tests that pin the interface**: name each test (a `CASE` id style like `game-rules-event-no-movement`), its tier (unit in `src/**/*_tests.rs` or tier-1 in `tests/`), and what it asserts. Read `tests/STRATEGY.md` for the tiers.
6. **Trade-offs**: where leverage is high, where it is thin, what migrates (stored JSON of `StoredInputs` and `EventRecord` must still read).

Vocabulary: `CONTEXT.md` for domain terms; `.agents/skills/codebase-design/SKILL.md` for module, interface, seam, depth, leverage, locality.
