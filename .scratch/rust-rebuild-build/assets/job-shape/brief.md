# Brief: the shape of the Job (ticket 20, design-it-twice)

You design one interface proposal. Read-only on `src/` and `tests/`: do not edit code. Write your proposal to the file your task names. Write in Simplified Technical English. Label claims *known* (read in code) or *inferred*.

## The problem

One Job runs the generations of one request (`src/application/pipeline/job.rs`, 804 lines), through the save units of `JobWriter` (`src/application/generation/gate.rs`, 910 lines). The narrator is `src/application/narration/narrator.rs`. Design **one** set of interfaces for the Job, its failure type, its reads and the narrator, so that these points are fixed together:

1. **`Job::run` as one loop** (FINDINGS 6). Today one fact ("a Narration was saved") is held in three places: the local `saved_a_narration`, `Ended.narration_saved`, and `options_step(.., after_narration: bool)` (which copies `OptionsRun`). The event-failure wrap (`into_event_failure`) runs at two sites. `run` and `failed` are two exits.
2. **The failure lines** (FINDINGS 2). `impl From<EngineError> for Failure` (`job.rs` ~87) makes every read error "The data for this request is missing: <raw error text>". `accept` (`gate.rs` `missing`, `unreadable`, `save_failed`) does it right. `JobWriter` reads inside its units (`state_at`, `tail_of`, `dock`, `opening_room_report` with `.ok()`), and a miss there shows "The new text could not be saved."
3. **Storage error type** (thermo TN-M7). Rusqlite errors are wrapped by hand as `EngineError::Config(format!("Failed to ...: {e}"))` under `src/adapters/driven/storage/` (about 64 sites). `GenerationFailure::from_engine_error` (`src/domain/model/state/generation_status.rs`) maps `Config` to `Other` and `Database` to `SaveFailed`.
4. **The narration prompt rules behind the narrator** (synthesis M3, S5; FINDINGS 19). Today `Job::narrate_step` reads the Game row, the World bundle, the settings (twice), the preset and the story window; picks the preset by kind; builds the user message by kind (`user_message`, `continuation_user_message`, `previous_input`); sets the guide and Impersonate flags on `PromptContext`; then calls `Narrator::narrate(role, &context, &preset, &response_length, budget)`. `analyse_step` and `options_step` each build a `WorldView` and a `HookInput` again. One generation reads the settings 4 times, the Game row 2 times, the Tail up to 4 times.

## Owner decisions for this ticket (fixed; design to them)

| Q | Decision |
|---|---|
| Q1 | The Options run (the end-of-Job hook, when `OptionsRun::AfterNarration`) follows every failure that comes after a saved Narration or event Narration, **except** `Stopped::EntryMoved` (a stop or reset; every write is refused anyway). So a `SaveFailed` of the state update, or of the event generation's text save, still gets the Options run (target design §4.2). The status shows the first failure. A failure of the Options run after a failure is only logged. Before any text save, no Options run follows a failure (except `/options` itself, which is only the Options run). |
| Q2 | One line rule for a read, in `accept` and in the Job: a miss or a read error of a record shows "The data for this request is missing: <record name>." with the raw error in the disclosure. The record names are one fixed list ("the game", "the story", "the start state", "the World", "the persona", "the settings", ...). Only a failed **write** shows "The new text could not be saved." Model failures keep `from_engine_error`'s kinds. |
| Q3 | The save units read nothing. The Job reads each value once, before the unit, and the unit takes values. The job-id check under the guard still refuses a late write. `state_at`, `tail_of`, `opening_room_report` go. An on-demand Options failure writes only the notice (a `Tx` write), so it reads no offered set. Verify (and say so) that nothing else can change the story while the Job holds its entry (`require_idle` and the other `GameWriter` methods). |
| Q4 | A typed `EngineError::Database { context: &'static str, source: rusqlite::Error }` (or close) replaces every hand wrap of a rusqlite error under `storage/`. `Config` stays for settings with no valid value. Say what `from_engine_error` still has to classify once the call site names the line. |
| Q5 | The Job reads one input bundle per generation (World values, Game row, settings, start state, story window, preset), once each. It passes the bundle and `&Generation` to the narrator. The narrator owns the prompt rules of each kind: the user message, the event continuation text, the guide and Impersonate flags, the role for health. The preset slot of each kind is a domain fn `Generation::preset_type`. The same bundle builds the `HookInput` for the after-narration hook and the end-of-Job hook in one place. The `agent-edge` rule stays: `src/application/narration/**` and `src/application/agents/**` import no storage (`arch-lint.toml`). |

## The ticket 19 interfaces (decided, not built yet; build on them)

Read `.scratch/rust-rebuild-build/issues/19-shape-of-a-generation.md`, section "The interfaces". In short:

```rust
pub enum Generation { Narration { guide: Option<String> }, Impersonate { direction: Option<String> }, UserRegen, Event(EventRecord) }
pub struct GenerationOrder { generation: Generation, place: Place, start: SwipeAt }   // ids and texts only
impl GenerationOrder {
    fn kind(&self) -> Kind;
    fn event_after(saved: SwipeAt, record: EventRecord) -> Self;
    fn write(&self, text: String, start: &SwipeState) -> (NewSwipe, Option<PreAnalysisState>);
}
JobPlan::for_request(&JobRequest, StoryEnd { tail, has_text }) -> Result<JobPlan, PlanRefusal>   // domain
pub struct QuantifierResult { present: Vec<String>, destination: Option<String> }
game_rules::apply(pre: PreAnalysisState, &QuantifierResult, &GameWorld) -> Result<RulesOutcome>
pub struct RulesOutcome { state: SwipeState, room_report: Option<String>, fired: Option<EventRecord> }
```

Hooks are named by position: before-narration, after-narration (the Quantifier; after the text save, before the state update), end-of-Job (the Options run). `HookInput` loses `kind` and `event`. The room report keeps today's behaviour (ticket 25 decides it later); the Job must hand the opening's room report to the save unit as a value.

One open detail for you: a user regen's narrator needs the text of the typed Input it regenerates. That Input is the last Message, and the window of a Retry ends before the last Message. Decide where that text comes from (the bundle, or `Generation::UserRegen { text }`, which bends ticket 19's "no data" for `UserRegen`).

## Behaviour to keep (known)

Target design `.scratch/turn-state-model/assets/target-design.md` §4.1 to §4.7: the step table and status names; "a step fails → the later generations do not run"; the event generation starts from the saved Swipe; a failed event generation shows "The event narration failed." with the inner line in the disclosure, and the Narration that fired stays; the Options-run table; the failure lines of §4.7. The tier-1 tests in `tests/` (read `tests/AGENTS.md`, `tests/STRATEGY.md`) pin most of it; name the ones your design touches.

## Output (one Markdown file, at most ~250 lines)

1. **Interface**: the Rust types and signatures (sketch code), with invariants, ordering and error modes. Use `CONTEXT.md` terms.
2. **Usage**: `Job::run` and one generation as short code; how `accept` and the save units change.
3. **What hides behind the seam**: which of today's fns, types, flags and branches are deleted. Count them.
4. **Where each piece lives** (file paths), and how it stays inside `arch-lint.toml`.
5. **Tests that pin the interface**: a `CASE`-style id, the tier (unit in `src/**/*_tests.rs`, seam contract, or tier-1 in `tests/`), and what it asserts.
6. **Trade-offs**: where the leverage is high, where it is thin, and what is left open.

Vocabulary: `CONTEXT.md` for domain terms; `.agents/skills/codebase-design/SKILL.md` for module, interface, seam, depth, leverage, locality.
