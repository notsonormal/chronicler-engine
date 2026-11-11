# Sketch 24: Agent hooks in `generate` (fork 4, option B)

Ticket: [Decide where the new design of the game-state lifecycle makes sense](../issues/24-decide-fix-or-rewrite.md), round 2, Q2. Updated after Q8 (the term **game rules**, `apply_game_rules`, replaces "world rules") and Q9 (each Agent declares its generation kinds).
Base: [sketch 25](sketch-25-new-design.md) draft 3. This sketch changes sketch 25 §3.3 ("The seam to the LLM roles") and the rows of §6.2 that name `LlmRoles`. Nothing else in sketch 25 changes.
Code: commit `9cbf3112`. Marinara: clone at `c5650149` (the same clone as research 14).

Labels: **known** (read in the code), **inferred**, **guessed** (an estimate). The `rust` blocks list interfaces. They do not compile.

## 0. What the owner asked for

> "I'm not sure how this would work if I had multiple agents, some pregeneration and some post generation, maybe even background agents. Marinara-Engine has a huge amount of agents (writer, tracker, misc, etc). I'm not sure how many I want exactly but I want to get reasonably supportive of it."

So the seam must let a new Agent attach to a generation with no change to `generate`. The Agent catalogue, batching, per-Game enabling and custom Agents stay inside the edge. This sketch does not design them.

## 1. Today and Marinara

**Today (known).**

| Part | What it does |
|---|---|
| `Agent` trait (`trait_def.rs`, 11 lines) | `name`, `phase`, `backend_selector`, `execute(&AgentContext) -> AgentResult` |
| `ExecutionPhase` | `PreGeneration` (no call site runs it), `PostGeneration`, `OptionsGeneration` |
| `AgentResult` | `PromptDirective` (the merge loop drops it), `StatePatch`, `Options`, `NoOp` |
| `AgentContext` | a whole `&GameState`, plus the narrator text, the player text, the room, the map, the Persona and the Characters |
| `AgentRegistry` (82 lines) | built from `AgentConfig`; knows `quantifier` and `options`; `agents_for_phase` |
| Call sites | `core.rs:468` runs every `PostGeneration` Agent and merges the `StatePatch`es. `options.rs:82` runs `OptionsGeneration`. The narrator is not an Agent: `call_narrator` in `pipeline_run.rs`. |
| Storage reads | `QuantifierAgent` and `OptionsAgent` read the Settings and their preset from `Storage`. |

**Marinara (known, research 14 and `docs/agents/`).** Three phases: `pre_generation`, `parallel` and `post_processing`. Each Agent declares one of 32 result types, and the host applies the result. The Agents do not write storage. Rewrite Agents run in their own lane before the trackers. Tracker results are stored by `(messageId, swipeIndex)`. Post-processing runs can outlive the reply.

Today's code has the same idea in a small form. Option B finishes it and fixes where each output lands.

## 2. The shape

### 2.1 Hooks: fixed points in `generate`

A **hook** is a fixed point in `generate` where the Job runs every Agent registered for that hook. Each hook has its own output type, so the compiler stops an Agent from returning an output that its hook cannot save.

| Hook | When | Agents today | Output | Where it lands |
|---|---|---|---|---|
| `BeforeNarration` | after step 1 (room check), before the narrator call | none | `PromptContext(String)` | by value into this narrator request. Not saved: a Retry runs the Agent again. |
| `Rewrite` | after step 3 (text save), before the analysis | none | `Rewrite(String)` | that Swipe's text, through a new save unit `save_rewrite` |
| `Analysis` | step 4 | the Quantifier | `Analysis` (movement, Characters present, confidence: today's `StatePatch`) or `Tracker(Value)` | `Analysis` feeds the game rules (step 5). `Tracker` goes into that Agent's own part of the `SwipeState`. Both are saved by the state-update save unit (step 6). |
| `Options` | the Options run, after the Job's last generation | the Options Agent | `Options(Vec<String>)` | the dock record, through `save_options` |

**Not a hook now: `parallel`.** In Marinara a parallel Agent runs during the narrator call and cannot change it. In this design it would have no place to land: it cannot change the narrator request, and the trackers are post-generation. Adding it later is one hook and one runner call next to the narrator call (inferred).

**Background Agents** (Q6) are not a hook. Section 5 shows why the shape keeps them cheap to add.

### 2.2 Interfaces

```rust
// domain/model/agent.rs — changed. ExecutionPhase → the hooks; AgentResult → one output type per hook.
pub enum Hook { BeforeNarration, Rewrite, Analysis, Options }

pub struct PromptContext(pub String);
pub struct Rewrite(pub String);
pub enum AnalysisOutput { Analysis(StatePatch), Tracker(serde_json::Value) }
pub struct OfferedSet(pub Vec<String>);

// The values a hook gives its Agents. The step loads them when it starts (sketch 25 §2.1).
// No GameId, no Storage, no dock record, no failure line (rule 11).
pub struct HookInput<'a> {
    pub kind: Kind,                          // Narration | Impersonate | UserRegen | Event
    pub start: &'a SwipeState,               // the start state (each tracker reads its own last value here)
    pub text: Option<&'a str>,               // the saved text; None before the narrator call
    pub story: &'a [StoryEntry],             // the window
    pub player_text: &'a str,
    pub world: &'a WorldView<'a>,            // room, map, Persona, Characters
    pub event: Option<&'a EventRecord>,
}
// What each Agent gets from its own config, loaded by the step (Q3 = A): its preset text and its token budget.
pub struct AgentSettings { pub preset: Option<String>, pub max_tokens: u32, /* … */ }
// The kinds an Agent runs for come from its AgentConfig (`kinds`, default: every kind). The runner filters on HookInput.kind.

// application/agents/trait_def.rs — changed. One trait per hook, so the output type is fixed per hook.
pub trait BeforeNarrationAgent: Send + Sync { fn name(&self) -> &str;
    fn run(&self, input: &HookInput, settings: &AgentSettings) -> Result<PromptContext, AgentFailure>; }
pub trait RewriteAgent: Send + Sync { fn name(&self) -> &str;
    fn run(&self, input: &HookInput, settings: &AgentSettings) -> Result<Rewrite, AgentFailure>; }
pub trait AnalysisAgent: Send + Sync { fn name(&self) -> &str;
    fn run(&self, input: &HookInput, settings: &AgentSettings) -> Result<AnalysisOutput, AgentFailure>; }
pub trait OptionsAgent: Send + Sync { fn name(&self) -> &str;
    fn run(&self, input: &HookInput, settings: &AgentSettings) -> Result<OfferedSet, OptionsFailure>; }

// application/agents/runner.rs — changed from registry.rs. Holds the Agents of each hook.
pub struct AgentRunner {
    before_narration: Vec<Box<dyn BeforeNarrationAgent>>,
    rewrite: Vec<Box<dyn RewriteAgent>>,
    analysis: Vec<Box<dyn AnalysisAgent>>,
    options: Option<Box<dyn OptionsAgent>>,
}
impl AgentRunner {
    pub fn before_narration(&self, input: &HookInput, s: &HookSettings) -> Vec<PromptContext>;  // a failure drops that Agent's output
    pub fn rewrite(&self, input: &HookInput, s: &HookSettings) -> Option<Rewrite>;              // Agents run in turn; each reads the last text
    pub fn analysis(&self, input: &HookInput, s: &HookSettings) -> Analysed;                    // never fails; the fallback is inside (as today)
    pub fn options(&self, input: &HookInput, s: &HookSettings) -> Result<OfferedSet, OptionsFailure>;
}
pub struct Analysed { pub patch: StatePatch, pub trackers: BTreeMap<String, serde_json::Value> }
// HookSettings: the AgentSettings of every Agent of that hook, keyed by Agent name.

// application/narration/narrator.rs — the narrator is not an Agent. It is the generation.
pub struct Narrator { /* PromptAssembler, LlmCallRecorder */ }
impl Narrator {
    pub fn narrate(&self, req: &NarratorRequest, cancel: &CancellationToken) -> Result<String, GenerationFailure>;
}
// NarratorRequest (sketch 25 §3.3) gains one field: pub context: &'a [PromptContext].
```

**`SwipeState` gains one field:**

```rust
pub struct SwipeState {
    /* room_id, added_rooms, present, encounters, trigger_context — as in sketch 25 */
    pub trackers: BTreeMap<String, serde_json::Value>,   // each Agent's own part, keyed by Agent name
}
```

The game rules copy the start state forward. So a tracker value that no Agent updated in this generation stays as it was (inferred from sketch 25 step 5).

**The edge.** Behind each trait sits an Agent's inside: its prompt, its parse, its fallback and its LLM call through `LlmCallRecorder`. Inside the runner sit the order of Agents in one hook, batching by connection, and concurrency. None of this is designed here (ticket 24 Scope).

### 2.3 `generate` with hooks

Sketch 25 §3.3 had six steps. With hooks:

| Step | Call | LLM calls |
|---|---|---|
| 1. Room check | as in sketch 25 | 0 |
| 2a. Before narration | `before_narration_step`: loads the `HookInput` and the hook's settings, then `agents.before_narration(..)` | 0 to N (one per Agent, or fewer when batched) |
| 2b. Narrator call | `narrate_step`: `narrator.narrate(..)` with the prompt contexts | 1 |
| 3. Text save | `writer.save_text(..)` (a user regen returns here) | 0 |
| 3b. Rewrite | `rewrite_step`: `agents.rewrite(..)`; on a result, `writer.save_rewrite(at, text)` | 0 to N |
| 4. Analysis | `analyse_step`: `agents.analysis(..)` | 1 to N |
| 5. Game rules | `apply_game_rules(&start, &analysed.patch, &world, mode)` | 0 |
| 6. State update | `writer.save_state_update(at, state' with analysed.trackers, room_report)` | 0 |

- **Step names for the status** (`JobWriter::step`): `Preparing` (2a), `Narrating`, `Rewriting` (3b), `Quantifying` (4), `Options`. A hook with no Agent is skipped and sets no step name.
- **Generation kinds** (owner, ticket 24, Q9 = B). Each Agent declares, in its config, the kinds that it runs for. The default is every kind. The runner skips an Agent for the other kinds. No Agent config can override the limits of the table below: they live in `generate` and in the game rules.

  | Kind | Before narration | Narrator | Rewrite | Analysis | Game rules | Can fire a Trigger |
  |---|---|---|---|---|---|---|
  | Narration | runs | runs | runs | runs | `Full` | yes |
  | Impersonate | runs | runs (as the Persona) | runs | runs | `Full` | yes |
  | User regen | runs | runs | runs | skipped (R-18) | skipped | no |
  | Event | runs | runs (from the Trigger's direction) | runs | runs | `Event`: Characters and encounters only (D-12) | no (R-14) |

  "Runs" means: the hook runs the Agents that declare this kind. The Options run is not a column: it runs once per Job, after the last generation, and not after an Impersonate.
- **A Trigger starts no new Job.** It starts the event generation inside the same Job, as today's `run_from_input` does (known, `core.rs:333`). R-14 limits a Job to two generations: the game rules in `Event` mode fire no Trigger.
- **A failed Agent.** A `BeforeNarration` or `Rewrite` failure drops that output, and the generation goes on. A tracker failure keeps the tracker's last value. The LLM call log records each call, so the header banner shows it (D-23). The dock notice stays the Options run's only notice (R-86).
- **Stop** drops the hook step that runs (OR-51), as it drops a narrator call today.

### 2.4 The save units that change

| Save unit | Change |
|---|---|
| `save_rewrite(at, text)` | **new.** One transaction: the Swipe's text. The job id check, as for every Job save unit. |
| `save_state_update(at, state, room_report)` | `state` now carries `trackers` |
| every other unit | no change |

## 3. Walk-through: an Action, with three Agents

The Agents: a knowledge lookup (`BeforeNarration`), the Quantifier (`Analysis`), and a time-and-weather tracker (`Analysis`, `Tracker`). No Trigger fires.

```text
Job
  room check
  before_narration_step: stories.swipe_state(start), stories.entries; settings of the lookup Agent
    agents.before_narration → [PromptContext("The old mill burned down last winter.")]        LLM
  narrate_step: games.get (the preset now); narrator.narrate(.., context)                       LLM
  save_text(text, pre-analysis state)                       [guard] #j   the story log shows the text
  rewrite_step: no Rewrite Agent → skipped
  analyse_step: stories.entries (window); settings of the Quantifier and the tracker
    agents.analysis → Analysed { patch (moved to "mill"), trackers { "weather": {"time":"dusk","sky":"rain"} } }   LLM ×2
  apply_game_rules(Full) → state' (room "mill"; trackers copied, then "weather" replaced)
  save_state_update(state', room report "mill")             [guard] #j
  options_step (toggle on)
```

**Then a Retry, then a swipe switch back.** The Retry starts from the start state (the Input's Swipe), so the lookup runs again and its context is new. The tracker reads its last value from the start state, not from the Swipe that the Retry replaces. The swipe switch back to the first Swipe gives back `"weather": {"time":"dusk","sky":"rain"}` with the rest of that Swipe's state. Nothing is copied.

## 4. Rules (changes to sketch 25 §6.2)

| # | Rule now | Change |
|---|---|---|
| 8 | Each function in `pipeline/job.rs` makes at most one of: a `Narrator::narrate` call, or one `AgentRunner` hook call | was "at most one `LlmRoles` method" |
| 11 | Agents and the narrator get values and do not know storage | the scope covers `application/agents/` and `application/narration/`. `HookInput` and `AgentSettings` have no `GameId`, no dock record and no failure line. |
| 13 | Only the step functions of `pipeline/job.rs` call `Narrator::narrate` and the `AgentRunner` hook methods | was `narrate`, `analyse`, `offer` |
| new 16 | An Agent writes no game record | compiler: each hook trait returns a value, and no Agent holds a `Storage` or a `JobWriter` |

## 5. Background Agents, kept cheap to add (Q6, option C)

A background Agent would run after the Job ends, as Marinara's trackers do. Two parts of this sketch make it cheap to add later:

1. Each tracker owns one named part of the `SwipeState` (`trackers[name]`). A background run then needs one small write: "set `trackers[name]` on Swipe `at`". It overwrites no other part.
2. "Save small" already gives the shape of that write: one record, one transaction.

Adding it later then needs: a `BackgroundAgent` trait; a second entry kind in `GameWriter`, anchored to a `SwipeAt`; one save unit that checks "this Swipe is still the active Swipe of the last Message, and no later Job started", else drops the write. The Job, `generate` and the other save units do not change (inferred).

The cost: D-03 ("only the Job that made a Swipe writes its Swipe state") gets one exception, for the background Agent's own part. That is a design change for the day it is built, not now.

## 6. Option A and option B side by side

| | A: `LlmRoles` (sketch 25) | B: hooks |
|---|---|---|
| A new Agent at an existing hook | a new trait method, a new step in `generate`, a new save path | a new type that implements the hook trait, and one registry line |
| A new kind of output | a new method | a new hook: a new trait, one runner method, one step in `generate`, and a save unit if it lands somewhere new |
| Where an output lands | decided per method | decided per hook, and the compiler holds it |
| Code for today's two Agents | about 280 lines (guessed, sketch 25 §7.2) | about 360 lines (guessed): the runner, four small traits and the narrator module. About 230 of them move from `pipeline_run.rs`, `core.rs` and `options.rs` with no change of logic. |
| Seams | `LlmRoles` with one adapter (a hypothetical seam); `LlmProvider` | the hook traits: `Analysis` has 1 Agent today and 2 or more as soon as a tracker exists; `LlmProvider` |
| Tests (ticket 26) | tier 1 fakes `LlmProvider` | the same. The seam contract of "the roles adapter" becomes one contract per Agent, written in that Agent's test file (`CASE` tags). |

**Build now, or later.** The hooks `BeforeNarration` and `Rewrite` have no Agent today. The build can write all four hooks now, or only `Analysis` and `Options` and add the other two when their first Agent comes. The second keeps less code, and each later hook costs one trait, one runner method, one step and (for `Rewrite`) one save unit (guessed: about 40 lines each).

## 7. Fate of today's Agent code

| Module | Lines (non-test, known) | Fate | Where it goes |
|---|---|---|---|
| `application/agents/trait_def.rs` | 11 | changed | the hook traits |
| `application/agents/registry.rs` | 82 | changed | `AgentRunner`; built from `AgentConfig` as today |
| `domain/model/agent.rs` | 175 | changed | `ExecutionPhase` → `Hook`; `AgentResult` → the output types; `AgentContext` (`&GameState`) → `HookInput` (values); `StatePatch`, `Confidence`, `Role` kept |
| `application/agents/quantifier/agent.rs` | 101 | changed | implements `AnalysisAgent`; the Settings and preset read moves to the step (Q3) |
| `application/agents/options/agent.rs` | 85 | changed | implements `OptionsAgent`; the same |
| `pipeline_run.rs`: `call_narrator`, preset lookups | part | replaced | `application/narration/narrator.rs` |
| `core.rs`: the `PostGeneration` merge loop | part | replaced | `AgentRunner::analysis` |
| `prompting/assembler.rs`: `with_storage` | part | changed | the token budget comes in as a value |
