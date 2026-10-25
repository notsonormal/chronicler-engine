# Sketch 25: the new design next to today's code

Ticket: [Sketch the new design next to today's code](../issues/25-sketch-new-design-next-to-code.md).
Design: [`target-design.md`](target-design.md). Spec: [`clean-spec.md`](clean-spec.md) version 3.
Code: commit `9cbf3112`, no change under `src/`.

Status: **draft 3, final for ticket 25.** **Ticket 24 (2026-10-14) changed two things:** fork 4 is decided as Agent hooks, so [sketch 24](sketch-24-agent-hooks.md) replaces the `LlmRoles` seam of §3.3 and rules 8, 11 and 13 of §6.2; and "world rules" is now **game rules** (`apply_game_rules`). The rest stands. The owner reacted to draft 1 and to draft 2 (§9). [Ticket 24](../issues/24-decide-fix-or-rewrite.md) judges this sketch area by area. One fork is still open: fork 4 waits for [ticket 26](../issues/26-decide-test-placement.md).

## 0. How to read this sketch

- **Terms.** The design's terms for the new shape: Swipe state, start state, current state, pre-analysis state, job, job entry, job id, save unit, dock record, event record. "Snapshot" names today's code only.
- **Job.** The design calls this concept "work". The owner chose **Job** (grilling Q7): the background work that one accepted Action, Retry, Retrigger or Options request starts. While a Job runs, its Game is busy, and a Stop ends it. A second request is refused, not queued. `CONTEXT.md` gets the term with ticket 24's verdict.
- **Rust names.** Module paths, types and traits are placeholders in today's crate (owner, ticket 24, Q9). The `rust` blocks list interfaces. They are not code stubs and do not compile.
- **Labels.** **known**: read in the code at `9cbf3112`, or in research 04, 05, 12 or 23. **inferred**: follows from the code or the design, not run. **guessed**: an estimate.
- **Line counts** are non-test lines from `wc -l` (known). Test lines are those of the matching `*_tests.rs` file (known).
- **Design references.** "D-nn" points at the decision log of [`target-design.md`](target-design.md) §9. "Qn" points at the owner's grilling answers to draft 2 (§9). These references are for this sketch only. The code does not keep them. A guardrail explains its reason in its own comment (§6).
- **What the sketch leaves out.** The inside of the edges: the narrator call, the Quantifier, the Options call, the world rules and Triggers, prompt assembly, the storage engine and the LLM call log. Each edge shows up only as a seam with its contract (`clean-spec.md` §3). Behaviour differences of research 23 are not a topic here. Where a behaviour choice changes the shape, the text says so.

## 1. The shape at a glance

**Today** (known; research 12 §1–§3):

```text
HTTP handlers ── read Storage::game_id (one atomic for the process)
   │
   ├─ ActionPipeline::{process_action, retry, retrigger}      5 files, split by entry path
   │     claim_and_spawn: heal → 400 checks → GenerationGate::try_claim (+ claim Snapshot) → 500 check → spawn
   │     5 ordering functions: run_from_input, retry_event_continuation, retry_user_regen,
   │                           execute_options_refresh, (arrival) ArrivalTaskContext::run_inner
   │     PipelineRun: 17 phase methods, no order
   │     NarrationGeneration: the one deep phase
   ├─ MessageService: load, save + pointer rule, switch/edit/delete with no guard, Retry anchor
   ├─ GenerationGate: busy registry in memory; it also writes the claim Snapshot and heals
   │
   └─ GameState (one record): world state + story + status + offered set + 2 memory-only carry fields
         built whole by load_or_fresh (17 production call sites), passed as &mut through the phases,
         saved whole as a Snapshot at ~10 points of one Action; a Swipe points at one of them
```

**New** (draft 3):

```text
HTTP handlers ── CurrentGame: the only reader of the active-Game pointer; once per request
   │
   ├─ job::start(game, request)                Action, Retry, Retrigger, Options request
   │     GameWriter::accept: guard → busy → 400 → 500 → job entry + accept unit → plan (ids only)
   │     the order of the request (design §4.2): the build plan picks the code shape (Q1)
   │     generate: room check → narrator → text save → analysis → world rules → state update
   │               (one function for every kind of generation: the one fixed constraint)
   ├─ GameWriter (today's GenerationGate, changed): one guard; the only code that opens a transaction
   │     JobWriter: the only write handle of a running Job; the job id check on each save unit
   ├─ GameViewQuery: story log, dock, status, busy — each from one rule
   │
   ├─ domain: Swipe with its SwipeState; Tail (the end of the story) with the derived rules
   ├─ LlmRoles (one trait): narrate, analyse, offer          ── the edges (fork 4: ticket 26)
   └─ Storage: repositories that read by GameId; record writes only on a transaction handle
```

## 2. Two rules that shape every area

### 2.1 Pass ids, load late, save small

The owner's rule (§9, draft 1 reaction 1, and Q2): pass an id and save at once. Do not populate large objects and pass them around.

**Today (known).** `MessageService::load_or_fresh` builds one `GameState`: the latest Snapshot plus every Message with every Swipe. It and `load_expecting_valid_state` have 17 production call sites. Each view poll builds a whole one. The status line builds one to read two fields. One `&mut GameState` goes through the phases of `core.rs` and `pipeline_run.rs`, and each save writes it whole. Two fields (`retry_target`, `arriving_location_write`) exist only to carry data from one phase to the next save. A whole copy that is written whole can overwrite another writer's save (research 05).

**The rule** (Q2):

1. Between steps, pass ids, never loaded records.
2. A step loads what it needs when it starts. It gives the values to the LLM call or the world rules that it calls.
3. A save writes only what changed.

An LLM answer is not in the database yet, so it goes by value to the save that stores it.

**The code carries the rule, not a written list** (Q6). The owner: a rule must be "encoded into the code structure so it's the most obvious and logical way to do things, or via guardrails". For each part, the structure comes first, and a guardrail closes the gap that the structure leaves:

| Part | Structure (the obvious way is the right way) | Guardrail (§6.2) |
|---|---|---|
| 1. Pass ids | The `Job` struct has only its `JobWriter` (Game id and job id), the repositories and the LLM roles. It has no field for a record. A plan holds ids: the start Swipe is a `SwipeAt`, not a `SwipeState`. | Type check: in the Job folder, no struct field and no function parameter has a record type (`Game`, `Message`, `SwipeState`, `Tail`, `DockRecord`, `WorldBundle`). Result types (narrator text, `Analysis`) are allowed. |
| 2. Load late | The only way to get a record is a repository read that takes a `GameId`. A step cannot receive a record (part 1), so it must load one. | Call count: each function in the Job folder calls at most one LLM role. So a function cannot load the Game once and use it for two LLM calls. `generate` calls step functions and makes no LLM call itself. |
| 3. Save small | Writes exist only on the transaction handle (Q3). Each write method takes one small record: one Message, one Swipe, one Swipe state, one dock record or the failure line. No method writes a whole state or a whole story, and no `GameState` type exists. So a "save everything" write cannot be written. | Call site: only `GameWriter` opens a transaction. |

This rule fixes an error of draft 1. Draft 1 loaded the Game record at accept and kept it for the whole Job. But the narrator must use the preset that is active when its call starts (`clean-spec.md` §3.1, R-59). Scenario: a Trigger fires, and the player changes the preset in Settings between the Narration and the event Narration. Under part 2, the event Narration loads the Game record again and uses the new preset.

**The boundaries.**

| Boundary | Carries |
|---|---|
| HTTP → application | `GameId` (from `CurrentGame`), `MessageId`, swipe index, the request text |
| Accept → Job | a `JobWriter` (holds `GameId` and job id), and a plan of ids and texts: the Input text, the first `GenerationOrder` (its start as a `SwipeAt`), the Options run |
| Step → step | ids, and the results of the step before. Never a loaded record. |
| Step → storage write | one `JobWriter` method per save unit, with only what changed |
| Step → storage read | only what that step uses (below) |
| Step → LLM role, world rules | values that the step loaded |

**Narrow reads.** No read loads the whole story with every Swipe state.

| Read | Returns | Used by |
|---|---|---|
| `stories.tail(game)` | the last Message with all its Swipes, the active Swipe state of the Message before it, and the Message count | accept, swipe switch, delete, the story-log controls, the sidebar |
| `stories.swipe_state(game, at)` | one Swipe state | step 1 of `generate`: the start state |
| `stories.entries(game, before, limit)` | for each Message: active Swipe text, kind, room report, event header, Swipe count. No Swipe states. | the narrator's story, the window of the analysis and the Options call, the story log |
| `games.get(game)` | the Game record | the preset at call time, the toggle, the failure line |
| `docks.get(game)` | the dock record | the dock |

### 2.2 Less code, not a copy of today's modules

The owner's concern (§9, draft 1 reaction 4): LLM-written solutions tend to add code. Draft 1 was partly a like-for-like copy. It had a new twin for most of today's modules. Draft 2 applied the deletion test of `/codebase-design` to each new module of draft 1: delete it, and see whether complexity comes back in its callers. Draft 3 keeps that result and changes the storage seam (Q3).

| Today | Draft 1 twin | Deletion test | Draft 3 |
|---|---|---|---|
| `GenerationGate` + `GenerationGuard` + `GenerationSlot` | `WorkRegistry` | a `Mutex<HashMap>` and nothing more: a pass-through | **gone.** Today's `GenerationGate` is changed into `GameWriter`: the same map, now held around each check and save unit. |
| — | `WorkTicket` + the writer methods | two types for one job | **one type:** `JobWriter`. Its `Drop` removes the job entry, as `GenerationGuard` does today. |
| `ActionPipeline` (5 files) | `Lifecycle::start` | accept, then spawn: a pass-through | **gone:** one function, `job::start` |
| `RetryMode`, `RetryTarget` | `RetryOf` | an enum between the story and the order | **gone:** `Tail::redo()` returns the `GenerationOrder` |
| 5 ordering functions | `Work::run` + `Plan` enum | — | **left to the build plan** (Q1), with one constraint: one `generate` |
| `PipelineRun` + `NarrationGeneration` | `Work::generate` | the six steps come back in every request | **kept:** this is the deep module |
| `MessageService` | `GameWriter` + `Story` | — | `GameWriter` + `Tail` |
| `Agent` trait + `AgentRegistry` + `ExecutionPhase` (an edge) | 3 traits + 3 adapters | three seams with one adapter each, toward one LLM | **one trait,** `LlmRoles`, with one adapter. Fork 4 (ticket 26). |
| 22 storage methods | `load_story` + `commit(SaveUnit)` | — | repositories: 5 reads; record writes only on a transaction handle (Q3). The `SaveUnit` enum is gone: each save unit is one method of `GameWriter` or `JobWriter`. |
| `Storage::game_id` | `ActiveGame` | — | `ActiveGame` (moved, not new) |
| 6 `current_game_id()` calls in the handlers (known) | `CurrentGame` | the lookup and the `400` come back in each handler | **kept** (about 20 lines) |

**Where the real reduction is.** Whole mechanisms of today's code go, because the design removes the problem that each one solves:

| Today's mechanism | Why it is not needed |
|---|---|
| heal: `heal_stale`, `heal_stale_if_changed`, `heal_stale_status` | no saved busy mark (D-05) |
| the claim Snapshot, `restore_click_time_state_after_failed_redo`, `durable_world_after_failed_redo` | a failed redo writes nothing |
| the Retry anchor: `find_retry_anchor*`, `check_retry_anchor`, `reconstruct_retry_state`, `last_input_index` | the start state is derived (D-02) |
| the carry fields `retry_target` and `arriving_location_write` | the text save says where the text goes, and the state update writes the room report |
| 5 ordering functions and arrival | one `generate` for every kind; no LLM opening (D-25) |
| the Snapshot copies in `switch_swipe`, `edit_history`, `delete_last` | a swipe switch writes an index; the state follows (D-01) |
| `check_game_unchanged`, `handle_cancellation` | the job id check (D-08) |
| 33 status lines in 7 files | `JobWriter::step` only |

**The hexagon's layers.** The new shape has the layers that the hexagon needs: the HTTP adapter, the application (`GameWriter`, `Job`), the domain (records and pure rules), and two driven seams (storage, LLM). It adds one seam that today's code does not have: `LlmRoles` in front of the `LlmProvider` port. Fork 4 says when that seam earns its place.

**Size.** Draft 1 guessed about 2,060 new lines, draft 2 about 1,735. Draft 3 guesses about 1,765: the save units move from one storage `match` into `GameWriter` methods, and storage gets record writes in their place. About 580 of these lines are code that moves without change of logic: the world rules (about 330) and the bodies of the LLM roles (about 250). Today's code that is replaced or deleted is 3,577 lines (known, §7). All sizes of new code are **guessed**.

## 3. The six areas

### 3.1 Area 1: the records and their storage

**Today (known).**

| Module | Lines | Holds |
|---|---|---|
| `domain/model/state/game_state.rs` | 553 | `GameState { movement, narrative, scene, npc_encounter_log }`. Its `narrative` part also holds the story (`history`), the status (`input_buffer`), `last_trigger` (with the assembled event prompt), `current_options`, `pending_location`, `pending_event`, backend names, and the two carry fields. Its methods mix the story (`push_message`, `add_message*`) with the world rules. |
| `domain/model/state/{narrative_state, game_state_snapshot, generation_status, trigger_context, message_types, movement, scene_state}.rs` | 373 | `NarrativeState`, `GameStateSnapshot`, `InputBuffer`, `StoredTriggerContext`, `MessageType` (with `System`), `MessageEntry` |
| `domain/model/message.rs`, `message_history.rs` | 333 | `Message`, `Swipe { text, snapshot_id, location_header, event_header, guide_text, impersonate_direction, impersonated }`, `MessageHistory` with the anchor helpers |
| `adapters/driven/storage/{core, messages, swipes, snapshots, games}.rs` | 1079 | `Storage { game_id: AtomicU64, backend }`. 22 public methods for Messages, Swipes, Snapshots and the Game id. Each is a separate storage call that reads `game_id()`. One transaction exists in all of `storage/`, in `worlds.rs`. |
| Tables | — | `games`, `messages`, `message_swipes(snapshot_id, location_header, event_header)`, `game_state_snapshots` |

**New: the records.**

```rust
// domain/model/state/swipe_state.rs — new. Replaces GameStateSnapshot and the state part of GameState.
pub struct SwipeState {
    pub room_id: String,
    pub added_rooms: HashMap<String, Room>,     // today MovementState.dynamic_rooms
    pub present: Vec<String>,                   // Character ids; today SceneState.npcs_in_area
    pub encounters: NpcEncounterLog,            // times met, met now, fired one-shot Triggers (as today)
    pub trigger_context: Option<TriggerRef>,    // which Trigger fired; no assembled prompt
}
impl SwipeState {
    pub fn starting(scenario: &Scenario, characters: &Characters) -> Self;   // create and reset
    pub fn pre_analysis(&self, event: Option<&EventRecord>) -> Self;          // design §1
}

// domain/model/message.rs — changed
pub enum MessageKind { TypedInput, ImpersonateInput, Narration, EventNarration(EventRecord) }
pub struct EventRecord { pub trigger: TriggerRef, pub name: String, pub direction: String }
pub struct Message { pub id: MessageId, pub kind: MessageKind, pub active: usize, pub swipes: Vec<Swipe> }
pub struct Swipe {
    pub text: String,
    pub inputs: StoredInputs,           // guide, impersonate direction, impersonated mark
    pub maker: Maker,                   // WorldOpening | Player | Narrator
    pub room_report: Option<String>,    // today location_header
    pub state: SwipeState,              // today snapshot_id → a Snapshot row
}
pub struct SwipeAt { pub message: MessageId, pub swipe: usize }   // the id of one Swipe

// domain/model/tail.rs — new. The end of the story: the input of every derived rule (design §2.3).
pub struct Tail { pub last: Message, pub state_before_last: Option<SwipeState>, pub count: usize }
impl Tail {
    pub fn current_state(&self) -> &SwipeState;
    pub fn start_at(&self) -> SwipeAt;                 // the opening: its first Swipe
    pub fn retrigger_allowed(&self) -> bool;
    pub fn can_delete(&self) -> bool;                  // more than one Message
    pub fn redo(&self) -> Result<GenerationOrder, Refusal>;     // design §4.5
    pub fn retrigger(&self) -> Result<GenerationOrder, Refusal>;
}

// domain/model/generation.rs — new. What one generation needs: ids and texts only (§2.1).
pub struct GenerationOrder {
    pub kind: Kind,                     // Narration | Impersonate | UserRegen | Event
    pub place: Place,                   // NewMessage | NewSwipeOnLast
    pub start: SwipeAt,                 // step 1 loads its Swipe state
    pub inputs: StoredInputs,
    pub event: Option<EventRecord>,
}

// domain/model/game.rs — changed: one new field
pub struct Game { /* as today */ pub failure: Option<GenerationFailure> }
// domain/model/dock.rs — new
pub struct DockRecord { pub offered: Vec<String>, pub notice: Option<Notice> }
```

**New: storage** (Q3: repositories, with writes only inside a transaction).

```rust
// adapters/driven/storage/stories.rs, docks.rs — new. Replace messages.rs, swipes.rs and snapshots.rs.
// Reads: every read names its Game.
impl Storage {
    pub fn games(&self) -> GameReads;      // get(game)
    pub fn stories(&self) -> StoryReads;   // tail(game), swipe_state(game, at), entries(game, before, limit)
    pub fn docks(&self) -> DockReads;      // get(game)

    // adapters/driven/storage/transaction.rs — new. The only way to write a game record.
    pub fn transaction<T>(&self, game: GameId, f: impl FnOnce(&mut Tx) -> Result<T, EngineError>)
        -> Result<T, EngineError>;
    pub fn create<T>(&self, f: impl FnOnce(&mut Tx) -> Result<T, EngineError>) -> Result<(GameId, T), EngineError>;
}
// Every write on a Tx goes to the Tx's Game. There is no other write method.
impl Tx {
    pub fn games(&mut self) -> GameWrites;        // insert, set_failure, remove
    pub fn messages(&mut self) -> MessageWrites;  // insert, set_active, delete_last
    pub fn swipes(&mut self) -> SwipeWrites;      // insert, set_text, set_state(at, state, room_report)
    pub fn docks(&mut self) -> DockWrites;        // put, clear
}
// adapters/driven/storage/games.rs — changed: get(game) takes a GameId and returns the failure line.
```

- **The edge contract** (`clean-spec.md` §3.6): "a write names its Game, and a save unit lands whole". The `Tx` holds the Game id, so every write names its Game. A transaction runs inside one `with_backend_mut` closure: a SQLite transaction, or one change of the in-memory data under the backend mutex. So both backends land a unit whole (inferred).
- **Where a save unit is defined.** A save unit is one `GameWriter` or `JobWriter` method that opens one transaction and makes 1 to 4 record writes. So the content of each unit sits in the application, next to the rules that decide it. The storage adapter only maps records to rows. Each record write exists once. For example, `docks().clear()` serves 6 units.
- **Trait or struct.** Whether the reads and the `Tx` writes are port traits or methods on `Storage` is the question of `architecture-deepening` ticket 03, for all of storage. The sketch shows methods.
- **Tables.** `games` gets the failure line. A new `game_docks` table holds the dock record. It is separate from `games`, because `update_game_config` writes the Game record from the config routes, which have no busy check (known). `messages` gets `kind` and the event record. `message_swipes` gets `maker` and `state` (JSON), and loses `snapshot_id`. `game_state_snapshots` goes. Saved Games may be discarded (OR-01), so no migration.
- **What goes:** `Storage::game_id`, `set_game_id`, `current_game_id`. The active-Game pointer moves to `application/games/active.rs` (area 2).

**Depth and locality.** Today 22 storage methods serve this job, and the pointer rule lives in three places (research 12 §3b). New: five reads, about 10 record writes, and one transaction entry. The deletion test: delete `Tail`, and the derived-value rules come back in each reader. The `SwipeState` is a full copy per Swipe, so storage grows with the Swipe count (design §7).

### 3.2 Area 2: the writer rule and the guard

**Today (known).**

| Module | Lines | Job |
|---|---|---|
| `generation/{gate, guard, slot, mod}.rs` | 248 | `GenerationGate`: `Mutex<HashMap<u64, GenerationSlot>>`. `try_claim` checks busy, inserts the slot, sets `Generating`, and writes the claim Snapshot through `MessageService`. `heal_stale` changes the status. `GenerationGuard` frees the slot on drop. A `generation_id` exists, and only `release_owned_slot` reads it. |
| `ActionPipeline::claim_and_spawn` (in `core.rs`) | ~57 | heal → `before_claim` (the `400` checks) → `try_claim` (busy) → `pre_spawn` (the `500` check) → `spawn_blocking` with the guard |
| `PipelineRun::check_game_unchanged` (in `pipeline_run.rs`) | ~13 | compares the Game id at three points. A Game switch makes the work stop and throw its text away ("game changed — discarding in-flight generation"). |
| `MessageService::{switch_swipe, edit_history, delete_last}` | ~80 | `switch_swipe` gets a `bool` from the handler; edit and delete check nothing |
| `Storage::game_id` (in `core.rs`) | ~40 | one atomic; every storage call reads it |

Shape: the busy check is in memory, but no save runs under it. The Game id of a write is read at the time of the write. 23 save sites write the latest Snapshot (research 23, D-03).

**New.**

```rust
// application/games/active.rs — moved from Storage. The active-Game pointer (design §2.2, D-26).
pub struct ActiveGame { /* Mutex<Option<GameId>> */ }
impl ActiveGame { pub fn get(&self) -> Option<GameId>; pub fn set(&self, game: GameId); }
// set() keeps today's touch of games.updated_at, because resolve_game_id picks the newest by it (known).

// adapters/driving/http/utils/current_game.rs — new. Every handler in scope takes it.
pub struct CurrentGame(pub GameId);   // FromRequestParts: reads ActiveGame once; none → 400 (R-45)

// application/generation/gate.rs — changed. Today's GenerationGate becomes the only writer.
pub struct GameWriter { storage: Arc<Storage>, entries: Mutex<HashMap<GameId, JobEntry>> }  // one guard (Q4)
pub enum Refusal { StillThinking, BadRequest(String), MissingData(String) }   // 200 / 400 / 500

impl GameWriter {
    // Action, Retry, Retrigger, Options request. Under the guard: busy → 400 → 500 → job entry → accept unit.
    pub fn accept(&self, game: GameId, request: JobRequest) -> Result<(JobWriter, JobPlan), Refusal>;
    // The short writers. Under the guard: busy → StillThinking; else the checks and one save unit.
    pub fn switch_swipe(&self, game: GameId, message: MessageId, index: usize) -> Result<(), Refusal>;
    pub fn edit(&self, game: GameId, message: MessageId, text: String) -> Result<(), Refusal>;
    pub fn delete_last(&self, game: GameId) -> Result<(), Refusal>;
    pub fn create(&self, new: NewGame) -> Result<GameId, EngineError>;
    pub fn replace(&self, old: GameId, new: NewGame) -> Result<GameId, EngineError>;   // reset
    pub fn stop(&self, game: GameId) -> Result<(), EngineError>;   // the entry goes, cancel, stop unit
    pub fn busy_step(&self, game: GameId) -> Option<Step>;
}

// The only write handle of a running Job. Only accept makes one. One method per save unit of a Job.
pub struct JobWriter { /* Arc<GameWriter>, GameId, job id: private */ }
impl JobWriter {
    // Each: under the guard, check that the job entry still holds this job id, then one transaction.
    pub fn save_input(&self, text: &str, start: SwipeAt) -> Result<SwipeAt, Stopped>;
    pub fn save_text(&self, order: &GenerationOrder, text: String, pre: SwipeState) -> Result<SwipeAt, Stopped>;
    pub fn save_state_update(&self, at: SwipeAt, state: SwipeState, room_report: Option<String>) -> Result<(), Stopped>;
    pub fn save_failure(&self, failure: GenerationFailure) -> Result<(), Stopped>;
    pub fn save_options(&self, result: OptionsResult, after_narration: bool) -> Result<(), Stopped>;
    pub fn step(&self, step: Step) -> Result<(), Stopped>;
    pub fn game(&self) -> GameId;
}
impl Drop for JobWriter { /* removes the entry if it still holds this job's id */ }
```

**What the guard does.** The guard is one lock. `GameWriter` holds it for a few milliseconds around "check, then save". It makes two things safe:

1. Two fast clicks on one Game cannot both start a Job. The busy check and the new job entry happen under the lock.
2. No save can slip past its check. A player edit checks "busy?" and saves under the lock. A Job's save checks "is the entry still my job id?" and saves under the lock. So a stop or a reset cannot come between the check and the save.

`GameWriter` never holds the lock during an LLM call, because the LLM calls happen in `Job`, outside `GameWriter`. So the guard never makes one Game wait for another Game's LLM call. One lock for all Games is enough (Q4): every storage call already takes the one backend mutex (known), so one lock per Game gains nothing (inferred). D-07's words change from "a per-Game guard" to "a guard".

**Two Games at the same time** (Q4).

- **Today (known):** a Game switch during a Job stops it and throws its text away.
- **New (R-74):** a Job keeps the Game that it started on. The player can act in Game A, switch to Game B and act there while A's LLM call runs, then switch back to A and see its result.
- **Side by side in two browser tabs: not in this design.** The server has one active Game (D-26). The owner accepted this on the condition that side by side stays cheap to add. It stays cheap because only `CurrentGame` reads the pointer (guardrail rule 2): to add it, `CurrentGame` reads the Game id from the request (the URL, or a header that HTMX sends) in place of the pointer (inferred). The rest of the cost is in the spec's HTTP interface (§2), the routes and the templates. This is a wish for the [platform choice](../../platform-choice/map.md) map.

- **Seams.** Above: the HTTP handlers, through `CurrentGame` and `Refusal`. Below: the storage reads and `Storage::transaction`. `GameWriter` is not a seam for swapping. Its tests use the in-memory `Storage`.
- **What the shape makes true.** No write exists without a `GameId` (D-09). A running Job can write only through its `JobWriter`, so no save unit of a Job lands without the job id check (D-08). No player edit lands while a job entry exists (R-70). The guard is held for one check and one save unit, never across an LLM call (D-07).
- **Status writes.** Today 33 hand-written lines in 7 files set the status (research 12 §1c). New: `JobWriter::step` only. The failure line has the save units of design §3.2 as its only writers.

**Depth and locality.** One module answers "who may write now". Today the answer is split across `GenerationGate`, `claim_and_spawn`, `check_game_unchanged`, three `MessageService` methods and `Storage::game_id`.

### 3.3 Area 3: the Job and its steps

**Today (known; research 12 §1–§2).**

| Module | Lines | Test lines | Job |
|---|---|---|---|
| `pipeline/action_pipeline/core.rs` | 605 | 1397 | `claim_and_spawn`; heal; `run_from_input` (164 lines, 6 of 10 entry paths); `retry_event_continuation`; `finalize_phase_error`; `persist_generation_error`; `run_post_generation_agents`; `phase_engine_commit` |
| `pipeline/pipeline_run.rs` | 487 | — | 17 phase methods; `call_narrator`; `build_trigger_request`; `persist_snapshot_or_err`; `check_game_unchanged`; `handle_cancellation` |
| `pipeline/action_pipeline/retry.rs` | 394 | 2114 | `RetryMode`, `RetryTarget`, `resolve_retry_target`, `reconstruct_retry_state`, `check_retry_anchor`, `retry_user_regen`, `restore_click_time_state_after_failed_redo` |
| `pipeline/action_pipeline/options.rs` | 223 | 365 | `process_options`, `execute_options_refresh`, `run_options_generation`, `finalize_options_failure`, `rewrite_options_after_turn` |
| `pipeline/narration_generation.rs` | 174 | 386 | `NarrationGeneration::run`: bundle, room, preset, prompt, narrator call, add Message, save |
| `pipeline/action_pipeline/{action, retrigger}.rs`, `phase_error.rs`, `mod.rs` ×2 | 182 | 870 | `process_action`, `retrigger`, `PhaseError` |
| `arrival_service.rs` | 190 | 159 | a sixth ordering function, with its own narrator call and no slot |
| `GameState::{push_message, add_message*, log_movement_completion, commit_trigger_narration}` | ~120 | — | the domain decides whether a text is a new Message or a Swipe on `retry_target` |

Shape: ten entry paths, five ordering functions plus arrival. `PipelineRun` holds phases but no order. The event tail exists twice, the `retry_target` append three times, the Options step twice, and there are five failure writers (research 12 §2c).

**New.**

```rust
// application/pipeline/job.rs — new. Replaces the files of pipeline/ and action_pipeline/.
pub struct JobPlan {                           // built by GameWriter::accept; ids and texts only
    pub input: Option<String>,                 // free text: the Input save
    pub first: Option<GenerationOrder>,        // None for an Options request
    pub options: OptionsRun,                   // AfterNarration | OnDemand
}
pub fn start(writer: &GameWriter, roles: Arc<dyn LlmRoles>, game: GameId, request: JobRequest)
    -> Result<Started, Refusal>;               // accept, then spawn_blocking: run the plan

struct Job { writer: JobWriter, roles: Arc<dyn LlmRoles>, storage: Arc<Storage> }   // no record fields
impl Job {
    fn generate(&self, order: GenerationOrder) -> Result<Generated, Stopped>;   // design §4.3; no LLM call itself
    fn narrate_step(&self, order: &GenerationOrder) -> Result<String, Stopped>;   // one LLM call
    fn analyse_step(&self, at: SwipeAt) -> Result<Analysis, Stopped>;            // one LLM call
    fn options_step(&self, run: OptionsRun) -> Result<(), Stopped>;              // one LLM call
}
pub struct Generated { pub narration: bool, pub at: SwipeAt, pub fired: Option<EventRecord> }
```

**The order of a request** (design §4.2). The order is behaviour: the Input save if the plan has one; `generate` the first order; if a Trigger fired, `generate` an event order that starts from the Swipe just saved; if a Narration was saved, or for an Options request, the Options run; then `JobWriter` drops. The code shape that holds this order is the build plan's choice (Q1: one runner over the plan, or one function per request kind). The one fixed constraint: every kind of generation runs through the one `generate`.

`generate` holds the six steps of design §4.3 for every kind:

| Step | Call | Loads (by id, at the start of the step) |
|---|---|---|
| 1. Room check | the World's map has the start state's room, or it is one of its added rooms | `stories.swipe_state(order.start)`, the World data |
| 2. Narrator call | `narrate_step` → `roles.narrate(&request, &cancel)` | `games.get` (the preset now), the World data, `stories.entries` before the place, the start state |
| 3. Text save | `writer.save_text(..)`; a user regen returns here | — |
| 4. Analysis | `analyse_step` → `roles.analyse(&request)`; it never fails, the fallback is inside | `stories.entries` (the window of 4) |
| 5. World rules | `apply_world_rules(&start, &analysis, &world, mode)`; `mode` is `Full` or `Event` | the start state and the World data again |
| 6. State update | `writer.save_state_update(..)` | — |

The seam to the LLM roles (`application/agents/roles.rs`, new; fork 4, ticket 26):

```rust
pub trait LlmRoles: Send + Sync {     // contracts: clean-spec §3.1, §3.2, §3.3, §3.5
    fn narrate(&self, req: &NarratorRequest, cancel: &CancellationToken) -> Result<String, GenerationFailure>;
    fn analyse(&self, req: &AnalysisRequest) -> Analysis;
    fn offer(&self, req: &OptionsRequest) -> Result<Vec<String>, OptionsFailure>;   // NotAvailable is a result
}
pub struct NarratorRequest<'a> {      // design §4.6; values only, no GameId, no dock, no failure line
    pub kind: Kind, pub game: &'a Game, pub world: &'a WorldBundle, pub start: &'a SwipeState,
    pub story: &'a [StoryEntry], pub inputs: &'a StoredInputs, pub event: Option<&'a EventRecord>,
}
// domain/model/world_rules.rs — the world rules as a pure function. Contract: clean-spec §3.4.
pub fn apply_world_rules(start: &SwipeState, analysis: &Analysis, world: &WorldBundle, mode: Mode)
    -> Result<RulesOutcome /* state, moved_to, fired */, Contradiction>;
```

- **Behind the seams** (inside not designed here). The one `LlmRoles` adapter takes today's `call_narrator`, `load_preset_and_response_length`, `build_trigger_request`, `run_post_generation_agents` and `run_options_generation`, with `PromptAssembler::assemble` and `LlmCallRecorder::complete` under them. The event prompt is built at call time from the event record. `apply_world_rules` takes `GameState::execute_freeaction_impl` and the methods that it calls, moved onto `SwipeState`.
- **What the shape makes true.** Every kind runs the same six steps in the same order, so the save points are the same for every kind. Only step 3 and step 6 give a Swipe its state. The start state comes from `Tail`, so no restore step and no anchor rule exist. A failed redo saves nothing, so nothing must be put back.
- **Test seam** (inferred; ticket 26 decides the test home). Today a model test can see a generation only through storage, and no interface answers "which state does this Swipe hold" (research 12 §5c). New: start a Job with a fake `LlmRoles` and the in-memory `Storage`, then read `stories.tail(game)` and the Swipe's `state`.

**Depth and locality.** `generate` is the deep module: one call, six steps, every kind. The deletion test: delete `generate`, and the six steps come back in each request, which is today's state.

### 3.4 Area 4: the reads of the views

**Today (known).**

| View | Reads today | Where the rule is |
|---|---|---|
| Story log | `GameViewQuery::get_story_log_entries`: a whole `GameState`; `System` entries too. Retrigger from `last_trigger` of the latest Snapshot (`NarrativeLogTemplate::new`). | `view_query.rs`, a template |
| Dock | `GameViewQuery::get_current_options` (a whole `GameState`) and `GenerationGate::is_busy`, in `AppState::render_options_dock` | `app_state.rs` |
| Status line | `generating_status_handler` builds a whole `GameState` to read two fields, then asks `is_busy` | the HTTP handler |
| Debug route, input box | `GenerationGate::is_busy` | handlers |
| Header banner | `GameViewQuery::role_health` | `view_query.rs` |
| Sidebar, headshots (not in the design's list) | `get_current_room_view`, `get_npc_headshots`: a whole `GameState` each | `view_query.rs` |

**New.**

```rust
// application/games/view_query.rs — changed. Each method takes the captured GameId.
impl GameViewQuery {
    pub fn story_log(&self, game: GameId) -> Result<StoryLogView, ApplicationError>;  // stories.entries + stories.tail
    pub fn dock(&self, game: GameId) -> Result<DockView, ApplicationError>;           // docks.get; enabled = !busy
    pub fn status(&self, game: GameId) -> Result<Status, ApplicationError>;           // busy_step, else games.get().failure
    pub fn is_busy(&self, game: GameId) -> bool;
    pub fn current_scene(&self, game: GameId) -> Result<SceneView, ApplicationError>; // stories.tail().current_state()
    pub fn role_health(&self) -> Result<Vec<RoleHealth>, ApplicationError>;           // unchanged
}
```

- **Shape change.** Each view reads only its own record. The status rule of design §2.3 moves from the handler into `status`. `can_retrigger` uses `Tail::retrigger_allowed`, the same rule that accept checks for `/retrigger`.

### 3.5 Area 5: create, reset and the opening

**Today (known).**

| Module | Lines | Job |
|---|---|---|
| `games/catalogue.rs` | 299 | `create_game`, `reset`, `persist_initial_state_with_swipes` (a Snapshot, then the opening Message, in separate calls; no Message when the opening text is empty), `switch_game` |
| `MessageService::build_fresh_initial_state` | ~30 | builds the start `GameState` |
| `bootstrap/init_game.rs` | 176 | `resolve_game_id` (creates a Game with `insert_game_from_request` when the World has none); `load_game_state` seeds a Snapshot when none exists; `try_persist_initial_message`; `spawn_arrival_task_if_needed` |
| `arrival_service.rs` | 190 | the LLM opening at each start for a World with no opening text, with no slot |
| `ActionPipeline::heal_stale_status` | — | at boot and at each Game switch |

**New.**

```rust
impl GameCatalogue {   // application/games/catalogue.rs — changed
    pub fn create_game(&self, world: &str, persona: &str) -> Result<GameId, ApplicationError>;
        // GameWriter::create: one transaction — the Game, an empty dock record, and the opening Narration
        // with one Swipe (opening text, may be empty; SwipeState::starting; the starting room as its room report);
        // then ActiveGame::set
    pub fn reset(&self, game: GameId) -> Result<GameId, ApplicationError>;   // GameWriter::replace
    pub fn switch_game(&self, game: GameId) -> Result<(), ApplicationError>; // games.get, then ActiveGame::set; no heal
}
// bootstrap/init_game.rs — changed: keeps resolve_game_id only. When the World has no Game,
// it calls GameCatalogue::create_game, so a Game made at boot gets the same create unit.
```

- **Shape change.** `arrival_service.rs` and the boot spawn go (D-25). The opening is a normal Message, so a Retry of it is a normal Retry: `Tail::redo` gives its first Swipe as the start.
- **Depth and locality.** One create unit replaces three writers of the first state (research 04 P14, P15).

### 3.6 Area 6: the dock and notices

**Today (known; research 09).**

- The offered set is `NarrativeState::current_options`, inside every Snapshot. `rewrite_options_after_turn` clears it at the end of the work, and `switch_swipe` clears it in its copy.
- Notices are `MessageType::System` Messages from five sources: `phase_post_generation` (low confidence), `GameState::attempt_movement` (unknown room), `finalize_options_failure`, and `rewrite_options_after_turn` (twice). They reach every prompt and take the last-Message controls (research 07).

**New.** `DockRecord` (area 1) is the only home of the offered set and the notice. Only these save units write it:

| Save unit (method) | Offered set | Notice |
|---|---|---|
| `JobWriter::save_text` of a Narration (any kind) | empty | none |
| `JobWriter::save_input` | as it was | as it was |
| `JobWriter::save_options`, success | the new set | none |
| `JobWriter::save_options`, failure or `NotAvailable`, after a Narration | empty | the reason |
| `JobWriter::save_options`, failure or `NotAvailable`, for an Options request | as it was | the reason |
| `GameWriter::switch_swipe`, `GameWriter::delete_last` | empty | none |
| `GameWriter::stop` | as it was | none |
| `GameWriter::create` | empty | none |

- One notice source: the Options run (R-83, R-86). `MessageType::System` goes. The header banner already reports a failed analysis call.
- No request type of `LlmRoles` has a field for the dock, so no prompt can hold a notice (R-82).
- The templates and CSS for `System` entries change with it. They are outside the designed scope.

## 4. Walk-throughs

### 4.1 A free-text Action that fires a Trigger

**Today** (known; research 04 P1 and P2). `S*` are Snapshot rows. `[x → Sn]` is a pointer that a save sets.

```text
POST /action → action_handler → dispatch_action → ActionPipeline::process_action
  claim_and_spawn
    MessageService::load_or_fresh                    a whole GameState; Game id from the Storage atomic
    heal_stale_if_changed                            may save a Snapshot
    before_claim: GameState::add_message(Input)      memory
    GenerationGate::try_claim: status Generating   → Sc   [Input → Sc]
    spawn_blocking (the guard frees the slot at the end)
  task: execute_action_with_inputs → run_from_input
    storage.current_game_id() again; load_or_fresh again; last_trigger = None
    phase_pre_main_snapshot                        → S0
    NarrationGeneration::run: bundle, room, preset, assemble, LLM, check_game_unchanged, add Narration
                                                   → S1   [Narration → S1]    the story log shows the text
    phase_post_generation: Quantifying             → S2
      Quantifier; low confidence → System Message  → S2b
    phase_engine_commit: execute_freeaction_impl (move, Characters, evaluate_triggers, encounters)
      log_movement_completion queues the header write
    build_trigger_request: the event prompt is assembled now (None → the event is dropped)
                                                   → S3   header row update
    phase_trigger_continuation_llm_call: GeneratingEvent → S4; LLM; commit_trigger_narration
                                                   → S5   [event Message → S5]
    reconcile_post_trigger_npcs (2nd Quantifier, no save)
    rewrite_options_after_turn: clear; Options; failure → System Message → S6
    phase_finalize: Idle                           → Sf
```

About 10 whole Snapshot rows and one header row update, none in a transaction. Every storage call reads the process-wide Game id. The Narration's Swipe points at S1, from before its analysis.

**New.** `[guard]` is the writer's guard. `#j` is the job id check. Each save unit is one transaction.

```text
POST /action → action_handler(CurrentGame(g), form) → job::start(g, request)
  GameWriter::accept(g)                                         [guard]
    busy? → 200 "Still thinking..."
    stories.tail(g); 400 and 500 checks
    JobEntry { j, step: Narrating }; accept unit                failure line → none
    plan { input: text, first: Narration order with start = tail.start_at(), options: AfterNarration }
  → 200 "Thinking..."; spawn_blocking: run the plan
Job
  room check: stories.swipe_state(start); World data            fails → save_failure, no Input
  save_input(text)                                              [guard] #j
  generate(Narration order)
    narrate_step: games.get(g), World data, stories.entries; roles.narrate        LLM
    save_text(text, pre-analysis state; dock emptied)           [guard] #j   the story log shows the text
    step(Quantifying); analyse_step: stories.entries (window); roles.analyse      LLM
    apply_world_rules(Full) → state', moved_to, fired T
    save_state_update(state' with trigger context T, room report moved_to)        [guard] #j
  T fired → generate(Event order: start = the Swipe just saved, event = record of T)
    step(GeneratingEvent); narrate_step: games.get(g) again (the preset now); roles.narrate
    save_text(event text, pre-analysis state with T marked fired)                 [guard] #j
    step(Quantifying); analyse_step; apply_world_rules(Event)
    save_state_update                                           [guard] #j
  games.get(g): toggle on → step(Options); options_step: roles.offer → save_options   [guard] #j
  JobWriter drops: the entry goes if it holds j
```

Seven save units, each one transaction, each with only what changed. A stop at any line keeps what landed before it and refuses the rest.

### 4.2 A Retry, then a swipe switch

The last Message is a Narration that followed a typed Input.

**Today** (known; research 04 P5, P6, P16).

```text
POST /swipe/new → retry_handler → ActionPipeline::retry
  claim_and_spawn: load_or_fresh; heal; before_claim: last_turn_is_retryable
    try_claim                                      → Sc
    pre_spawn: check_retry_anchor (anchor = the last Input row anywhere; its Snapshot must load)
  task: retry_last_response
    load_messages; resolve_retry_target → ReNarrate(main); target = the last Narration
    state_at_click = load_latest_snapshot
    restore the anchor's Snapshot (the Input's Sc); reconstruct_retry_state: history cut after the Input;
      retry_target = the Narration (memory only)
    retry_main_narration → run_from_input, as in 4.1 from S0:
      NarrationGeneration::run → push_message adds a Swipe to retry_target → S1'  [new Swipe → S1']
      Quantifier; engine commit; header onto retry_target's active Swipe → S3'
      retry_target → history; a Trigger adds a new event Message
      rewrite_options_after_turn → S6'; phase_finalize → Sf
    no Swipe landed → restore_click_time_state_after_failed_redo → Sr
POST /message/:id/swipe/:i → switch_swipe_handler → MessageService::switch_swipe(is_busy, id, i)
  busy → ConcurrentGeneration → 503
  last Message? update_active_swipe; load Swipe i's Snapshot (S1: before its analysis);
  force Idle, clear the options; save the copy → Ss
  → the live state is Swipe i's world from before its analysis (the seed defect)
```

**New.**

```text
POST /swipe/new → retry_handler(CurrentGame(g)) → job::start(g, Retry)
  GameWriter::accept(g)                                         [guard]
    busy? → "Still thinking..."
    stories.tail(g); tail.redo() → Narration order, NewSwipeOnLast, start = the Input's Swipe
    JobEntry; accept unit
Job
  generate(order): narrate (story = stories.entries before the last Message); save_text (new Swipe, active);
                   analyse; apply_world_rules; save_state_update
    narrator failure → save_failure; no Swipe; the current state did not change, so nothing to restore
  fired → event as a new Message; the Options run; done
POST /message/:id/swipe/:i → switch_swipe_handler(CurrentGame(g), id, i)
  GameWriter::switch_swipe(g, id, i)                            [guard]
    busy → 200 "Still thinking..."
    stories.tail(g): last Message? index in range?
    one transaction: active index; dock emptied; failure line none
  → stories.tail(g).current_state() is now Swipe i's own analysed state. Nothing is copied.
```

## 5. Forks

| # | Fork | A | B | Ruling |
|---|---|---|---|---|
| 1 | Where the order of a request lives (D-16) | one runner over a plan that accept builds as data | one function per request kind | **Neither now** (Q1, C). The build plan picks. The design adds one constraint: one `generate` for every kind of generation. |
| 2 | The current state (D-01) | derived from the last Message's active Swipe | a live record plus copies (GLM, Kimi, today) | **A**, by the owner in ticket 22 (R1 Q1) |
| 3 | The busy mark (D-05) | memory only | saved, with a start-up pass (GLM, Kimi) | **A**, by the owner in ticket 22 (R2 Q3) |
| 4 | The seam to the LLM roles | one trait `LlmRoles` with one adapter, which replaces today's `Agent` trait, `AgentRegistry` and `ExecutionPhase` | no new seam; tests fake `LlmProvider` with `MockBackend`, as today | **Open: [ticket 26](../issues/26-decide-test-placement.md) decides.** Where the tests sit sets whether this seam earns its place. Today the HTTP apps pass an empty registry, so every unseeded Action runs on `QuantifierResult::default()` (research 10). |
| 5 | The storage write | one `commit(game, SaveUnit)` (draft 2) | repositories, with record writes only on a transaction handle; each save unit is one `GameWriter` or `JobWriter` method | **B** (Q3). The normal shape. The compiler still refuses a write outside a transaction, and the content of a unit sits in the application. |
| 6 | What a read loads | the whole story with every Swipe state (draft 1) | narrow reads | **B** (Q2, with the persistence rule of §2.1) |
| 7 | The guard | one guard for all Games | one guard per Game (D-07's words) | **A** (Q4). Same behaviour, less code. Two Games' Jobs still run at the same time. |

## 6. Enforcement

The owner (§9, draft 1 reactions 2 and 3; Q6, Q8): a good design stays in place when the code structure and guardrails enforce it, not when people follow a written list of complex rules. **The rules below are part of the design** (Q8). The build writes each guardrail before or together with the code that it guards. Each guardrail explains its reason in a comment in plain words. The "Design" column below is for this sketch only.

### 6.1 The tools of this repo (known)

| Level | Tool | What it can check |
|---|---|---|
| 1. Compiler | Rust types and visibility | a wrong type, a private constructor, a missing parameter. A rule at this level cannot be broken. |
| 2. arch-lint | `arch-lint.toml`, 13 scope rules, configured in TOML | "folder A must not import folder B" |
| 3. syn walkers | `tests/infrastructure/guardrails/`, 25 rules | any rule on the syntax tree: who calls a method, which types a field or a parameter may have |
| 4. Behaviour tests | unit and HTTP tests | order and outcome |
| 5. Review | — | everything else |

### 6.2 The rules

| # | Rule | Reason (for the guardrail's comment) | Level | How | Design |
|---|---|---|---|---|---|
| 1 | Every read and write names its Game | A write that finds its Game from a global lands in whichever Game is active at that moment. A switch during a Job then sends the Job's saves to the wrong Game. | compiler + syn | `Storage` has no Game id field. Every read takes a `GameId`, and a `Tx` holds one. syn: no `static` and no `Atomic*` in `storage/`. | D-09 |
| 2 | A request reads the active-Game pointer once, in one place | All the writes of one request must go to one Game, also when the player switches during the request. One reader also keeps side by side cheap to add (Q4). | syn | call site: only `current_game.rs` and `GameCatalogue` call `ActiveGame` | D-09, Q4 |
| 3 | Only `GameWriter` writes game records | Two writers that do not share one guard can overwrite each other's saves. | syn | call site: only `generation/gate.rs` calls `Storage::transaction` and `Storage::create` | D-03, D-07 |
| 4 | A running Job writes only through its `JobWriter` | After a stop or a reset, a late save of the old Job must not land. The job id check refuses it. | compiler | `JobWriter` has private fields, only `accept` makes one, and each of its methods checks the job id | D-08 |
| 5 | Edit, delete and swipe switch only while idle | A player edit during a Job would change the story under the Job. | compiler + test | they exist only as `GameWriter` methods that check busy under the guard; an HTTP test checks "Still thinking..." | R-70 |
| 6 | The guard never spans an LLM call | An LLM call takes seconds. Every request would wait for it. | compiler + syn | the guard is a private `MutexGuard` that no function returns; syn: `gate.rs` names no `LlmRoles` | D-07 |
| 7 | Between steps, pass ids | A copy of a record that a Job keeps gets stale: a preset change during the Job would not reach the next narrator call. | syn (type check) | in `pipeline/job.rs`, no struct field and no function parameter has the type `Game`, `Message`, `SwipeState`, `Tail`, `DockRecord` or `WorldBundle` | §2.1 part 1 |
| 8 | A step loads late | A function that loads a record once and then makes two LLM calls uses a stale copy for the second call. | syn (call count) | each function in `pipeline/job.rs` calls at most one `LlmRoles` method | §2.1 part 2 |
| 9 | A save writes only what changed | A write of a whole copy can overwrite another save. | compiler | the `Tx` write methods take one small record each; no `GameState` type exists | §2.1 part 3 |
| 10 | No carry fields | A field that holds data only until the next save makes a save depend on what an earlier call left in memory. | syn | no `#[serde(skip)]` field on a type in `domain/model/` | §2.1 |
| 11 | The LLM roles get values and do not know storage | A role that reads storage hides an input from its contract and needs a database in its tests. | arch-lint + compiler | a scope for the roles' adapter that may not import `storage`; the request types have no `GameId`, no dock and no failure line | D-17, R-82 |
| 12 | The world rules are pure | The world rules decide the game state. A pure function can be tested with no storage and no LLM. | arch-lint | they live in `domain/model/`, where the `model` rules already apply; add `model` → `storage` to the denied list | D-12 |
| 13 | One generation path | A second path that calls the narrator gets its own save points, as arrival did. | syn + test | call site: only the step functions of `pipeline/job.rs` call `narrate`, `analyse` and `offer`; a test checks the save units of each kind in order | D-16, Q1 |
| 14 | A save unit lands whole | A reader or a restart must never see a Message without its Swipe, or a Swipe without its state. | compiler + test | record writes exist only on `Tx`; a storage test: a transaction that fails half-way leaves no row | R-90 |
| 15 | The status has one writer per part | Today 33 lines in 7 files set the status, and two of them disagree after a restart. | compiler | `JobEntry` fields are private to `gate.rs`; the failure line changes only through save units | D-05 |

**Three generic walkers, not one per rule.** Rules 2, 3, 6 and 13 are one kind: "method X may be called only in files Y" (call site). Rules 7 and 10 are one kind: "types X may not be fields or parameters in folder Y" (type check). Rule 8 is a third kind: "functions in folder Y call at most N methods of X" (call count). Today's guardrails already have walkers of the first two kinds; the call-count walker is new. Each walker is configured with one table row per rule, and each row carries its reason as a comment. Guessed cost: about 350 lines with their tests.

**A walker that is limited to a path must fail when the path matches no file.** Two of today's walkers check nothing, because their path no longer exists (§6.3). The new walkers must not repeat this.

### 6.3 Limits

- **Rust visibility.** `pub(in path)` can only name an ancestor module. So "only `gate.rs` may open a transaction" needs a syn rule; the compiler cannot express it (known).
- **Name matching.** Today's walkers match text and identifiers, not resolved paths (known, `layers.rs`). A `use … as` alias gets past them (inferred).
- **Dead guardrails (known).** Two of today's walkers check no file. `check_handler_return_type` looks for paths that start with `src/server/` and `check_server_layer_boundaries` for `server/` (`tests/infrastructure/guardrails/layers.rs:94` and `:203`), but the paths are relative to `src/` and neither folder exists. Their unit tests use fake `server/…` paths, so the tests pass. The owner had an issue opened for this outside this map (Q9): [Make the two path-limited guardrails check real files](../../retro-issue-bucket/issues/50-dead-path-guardrails.md).
- **Conflicts with today's edges (known).** `PromptAssembler::with_storage` reads the token budget from `Storage`. `QuantifierAgent` and `OptionsAgent` hold `Storage` through `from_config_with_storage`. Rule 11 needs those reads moved into the step, or an allowlist.
- **What no tool checks:** whether the design is good, which fields belong in a Swipe state, and the order of steps inside one function.
- **For the platform-choice map.** The rows at compiler level depend on Rust. A language with weaker visibility rules would move them down to the walker level (inferred).

## 7. Fate table

Fate: **kept** (no change of shape), **changed** (same module, new shape), **replaced** (the job moves to a named new module), **deleted** (the job does not exist in the design). Files marked "part" hold jobs outside the designed scope as well.

### 7.1 Today's modules

| Module | Lines | Test lines | Fate | Where the job goes |
|---|---|---|---|---|
| `application/pipeline/action_pipeline/core.rs` | 605 | 1397 | replaced | `claim_and_spawn` → `GameWriter::accept` and `job::start`; `run_from_input`, `retry_event_continuation` → the order of the request and `generate`; heal → deleted; the failure writers → `JobWriter::save_failure`; `run_post_generation_agents` → `LlmRoles` adapter; `phase_engine_commit` → `apply_world_rules` |
| `application/pipeline/pipeline_run.rs` | 487 | — | replaced | phases → the steps of `generate`; `call_narrator`, `build_trigger_request`, preset lookups → `LlmRoles` adapter; `persist*`, `check_game_unchanged`, `handle_cancellation` → deleted |
| `application/pipeline/action_pipeline/retry.rs` | 394 | 2114 | replaced | `resolve_retry_target` → `Tail::redo`; `reconstruct_retry_state`, `check_retry_anchor`, `restore_click_time_state_after_failed_redo`, `durable_world_after_failed_redo` → deleted |
| `application/pipeline/action_pipeline/options.rs` | 223 | 365 | replaced | `Job::options_step`; `run_options_generation` → `LlmRoles` adapter; `finalize_options_failure` → `JobWriter::save_options` |
| `application/pipeline/narration_generation.rs` | 174 | 386 | replaced | steps 1–3 of `generate`, and the `LlmRoles` adapter |
| `application/pipeline/action_pipeline/action.rs` | 75 | 640 | replaced | the plan built in `accept` |
| `application/pipeline/action_pipeline/retrigger.rs` | 49 | 230 | replaced | `Tail::retrigger` |
| `application/pipeline/phase_error.rs` | 24 | — | replaced | `Stopped`, `Refusal` |
| `application/pipeline/{mod, action_pipeline/mod}.rs` | 34 | — | replaced | `pipeline/job.rs` |
| `application/message_service.rs` | 273 | 376 | deleted | loads → the story reads; saves and the pointer rule → `JobWriter` save units; switch, edit, delete → `GameWriter`; Retry anchor → `Tail::start_at`; `build_fresh_initial_state` → `SwipeState::starting` |
| `application/generation/gate.rs` | 145 | 154 | changed | becomes `GameWriter` and `JobWriter`: the same map, held around each check and save unit |
| `application/generation/mod.rs` | 13 | — | changed | — |
| `application/generation/{guard, slot}.rs` | 90 | 109 | deleted | `JobWriter`'s `Drop` |
| `application/arrival_service.rs` | 190 | 159 | deleted | — (D-25) |
| `bootstrap/init_game.rs` | 176 | — | changed | keeps `resolve_game_id`; the rest is deleted |
| `application/errors.rs` (part) | 86 | — | changed | `ProcessActionResult` → `Refusal`; `ConcurrentGeneration` (`503`) goes |
| `application/games/catalogue.rs` (part) | 299 | 724 | changed | create, reset and switch as in area 5; rename, delete, mode, posture and presets kept |
| `application/games/view_query.rs` (part) | 233 | 166 | changed | area 4; `role_health` kept |
| `application/debug/dto.rs` | 28 | — | changed | reads `busy_step` |
| `domain/model/state/game_state.rs` | 553 | 1840 | changed (split) | world-rule methods (about 330 lines, inferred) → `apply_world_rules` on `SwipeState`; story methods, `arriving_message_mut`, `inject_scenario_logs` → deleted; the `GameState` type → deleted |
| `domain/model/state/narrative_state.rs` | 63 | — | deleted | — |
| `domain/model/state/game_state_snapshot.rs` | 73 | — | replaced | `SwipeState` |
| `domain/model/state/generation_status.rs` | 138 | 166 | changed | `GenerationFailure` and its kinds kept (the failure line); `GenerationPhase` → `Step`; `InputBuffer`, `GenerationStatus` → deleted |
| `domain/model/state/message_types.rs` | 59 | — | changed | `MessageType` → `MessageKind` without `System`; `MessageEntry` → `StoryEntry` |
| `domain/model/state/{movement, scene_state}.rs` | 24 | — | changed | fields of `SwipeState` |
| `domain/model/state/trigger_context.rs` | 16 | — | replaced | `TriggerRef` and `EventRecord`, with no assembled prompt |
| `domain/model/state/mod.rs` | 18 | — | changed | — |
| `domain/model/message.rs` | 186 | 186 | changed | `Swipe` gains `state`, `maker`, `room_report`; loses `snapshot_id`; `Message` gains `kind` |
| `domain/model/message_history.rs` | 147 | 197 | replaced | `Tail` |
| `domain/model/game.rs` (part) | 117 | — | changed | gains the failure line |
| `adapters/driven/storage/core.rs` (part) | 208 | 191 | changed | the Game id atomic goes; backend dispatch kept; `transaction` and `create` enter through `with_backend_mut` |
| `adapters/driven/storage/messages.rs` | 241 | 366 | replaced | `storage/stories.rs` |
| `adapters/driven/storage/swipes.rs` | 208 | 569 | replaced | `storage/stories.rs` |
| `adapters/driven/storage/snapshots.rs` | 125 | 260 | deleted | — |
| `adapters/driven/storage/games.rs` (part) | 297 | 275 | changed | the failure line; a `GameId` parameter on `get` and the preset lookups; Game writes move onto `Tx` |
| `adapters/driven/storage/in_memory_data.rs` (part) | 132 | — | changed | the new records |
| `adapters/driven/storage/mappers/message.rs` | 72 | 163 | changed | the new Message and Swipe rows |
| `adapters/driven/storage/mappers/state_snapshot.rs` | 60 | 192 | deleted | — |
| `adapters/driven/storage/models/{message, swipe, game}.rs` | 135 | — | changed | the new columns |
| `adapters/driven/storage/models/game_state_snapshot.rs` | 26 | — | deleted | — |
| `adapters/driven/storage/utils/plumbing.rs` (part: 4 tables) | of 728 | — | changed | the schema of area 1 |
| `adapters/driving/http/action/handlers/actions.rs` | 115 | — | changed | `CurrentGame`; `Refusal` → `200`/`400`/`500` |
| `adapters/driving/http/chat_window/handlers/chat_window.rs` | 68 | — | changed | the same; the swipe switch through `GameWriter::switch_swipe` |
| `adapters/driving/http/history/handlers/history.rs` | 33 | — | changed | the same |
| `adapters/driving/http/layout/handlers/endpoints.rs` (part) | 99 | — | changed | the status rule moves to `GameViewQuery::status`; the stop calls `GameWriter::stop` |
| `adapters/driving/http/games/handlers/games.rs` (part) | 255 | — | changed | the switch does not heal |
| `adapters/driving/http/debug/handlers/debug.rs` | 39 | — | changed | — |
| `adapters/driving/http/app_state.rs` (part) | 184 | — | changed | the dock renders `DockView` |

Totals (known, whole files): 7,289 non-test lines. The "part" files hold jobs outside the scope too, so the scoped part is smaller, near ticket 24's rough 6,600 (inferred).

| Fate | Lines (whole files, known) |
|---|---|
| replaced | 2,750 |
| deleted | 827 |
| changed | 3,712 |
| kept | 0 |

The pipeline's 124 unit-test functions (5,132 lines) test `ActionPipeline` methods directly (research 12 §5a). Their modules are replaced, so these tests go with them (inferred). The HTTP tests go through the routes. Some of them seed Snapshots by hand (`tests/http/{actions, options, retrigger, swipe_new, games_switch}.rs`), and those parts change (known: they name Snapshot functions). Where the new tests sit is ticket 26's question.

### 7.2 New modules

All sizes are **guessed**. "Moved" is code that moves with no change of logic.

| Module | Interface | Lines | Of which moved |
|---|---|---|---|
| `domain/model/state/swipe_state.rs` | `SwipeState`, `starting`, `pre_analysis` | 70 | — |
| `domain/model/world_rules.rs` | `apply_world_rules` | 330 | 330 (from `GameState`) |
| `domain/model/tail.rs`, `domain/model/generation.rs` | `Tail` and its derived rules; `GenerationOrder`, `SwipeAt` | 140 | — |
| `domain/model/dock.rs` | `DockRecord` | 25 | — |
| `application/games/active.rs`, `adapters/driving/http/utils/current_game.rs` | `ActiveGame`, `CurrentGame` | 40 | 20 (from `Storage`) |
| `application/generation/gate.rs` (changed, counted here in full) | `GameWriter`, `JobWriter`, and one method per save unit | 320 | — |
| `application/pipeline/job.rs` | `start`, `JobPlan`, `Job`, `generate`, the step functions | 230 | — |
| `application/agents/roles.rs` | `LlmRoles` and its adapter | 280 | 230 (from `pipeline_run.rs`, `core.rs`, `options.rs`) |
| `adapters/driven/storage/{stories, docks, transaction}.rs` | the reads, `Tx` and its record writes, for both backends | 330 | — |
| Total | | about 1,765 | about 580 |

## 8. Open points

- **Fork 4** (§5): [ticket 26](../issues/26-decide-test-placement.md) decides.
- **For ticket 24** (inferred from the owner's answers; ticket 24 decides):
  - D-07's words change from "a per-Game guard" to "a guard" (Q4).
  - D-16 gains one constraint: one `generate` for every kind of generation (Q1).
  - The term "work" becomes **Job** in the design, and `CONTEXT.md` gets it with the verdict (Q7). "Action Pipeline" goes. "Turn" stays deprecated.
  - The rules of §6.2 are part of the design (Q8).
- **For the platform-choice map:** side by side in two browser tabs is a wish (Q4). The rows of §6.2 at compiler level depend on Rust.
- **For `architecture-deepening` ticket 03:** the repositories of area 1 are the first instance of its question (trait or struct, one per record).

## 9. Owner reactions

### To draft 1 (2026-10-12)

1. **Persistence.** "Often huge objects are being populated and passed around. This is rather cumbersome and seems pointless when we are a database and CRUD system. Doesn't it make more sense to just pass in the id, save it, at least within a single context." Draft 2: §2.1 and fork 6.
2. **Enforcement.** The owner asked whether the boundaries can be defined in a programmatic, semantic or structural way, and enforced with guardrails: "if we decide on a design we should be able to force it in a practical sense. And as long as the design itself is good, by enforcing it, we can maintain it more easily." Draft 2: §6.
3. **Guardrail comments.** "Each new guardrail should EXPLAIN its decision (we allow comments on them for that reason), not some random number D-05." The sketch may map a guardrail to its decision. The final code does not keep the numbers. Draft 2: the "Reason" column of §6.2.
4. **Code volume.** "The problem with LLM generated code is that the solution to problems tends to be adding more code, not removing code or reducing code." The owner asked whether draft 1 was like-for-like with today's layers and classes, and noted that a hexagonal architecture adds some layers anyway. Draft 2: §2.2.

### To draft 2: grilling (2026-10-14)

Forks 2 and 3 were not asked again: the owner ruled on them in ticket 22. Fork 4 waits for ticket 26.

| Q | Question | Answer |
|---|---|---|
| Q1 | Fork 1: where the order of a request lives | **C.** Not decided in the design. The build plan picks. One constraint: one `generate` for every kind of generation. |
| Q2 | The persistence rule | Draft 2's rule was "really complicated. It should be a simpler rule." **A:** the three-line rule of §2.1 — "makes sense, but this needs to be encoded into the code structure so it's the most obvious and logical way to do things, or via guardrails. This is the most effective way to keep the codebase consistent. Not writing down complex rules to follow." |
| Q3 | Fork 5: one `commit`, or repositories | The owner asked: "is that really the best solution? One big generic Storage object? What are the pros and cons vs a more normal repository model?" **B:** one repository per record, with record writes only inside a transaction. |
| Q4 | Fork 7: the guard, and two Games at the same time | The owner asked what the guard does, and whether one guard would block playing two Games at once "to deal with LLM latency". It does not. **A:** switching only in this design, "as long as it is cheap to add B [side by side] later". |
| Q5, Q7 | The term for the engine work of one request | The owner rejected the count of uses in the spec and the design as an argument ("that's just find and replace"): "consider the right word based on its own merits." The owner then weighed "turn" ("player turn, npc turn, event turn"). Against it: one Job holds an Input and two narrator texts, so "turn" would again name two things, a Retry and an Options request are Jobs but not turns, and "npc turn" conflicts with the glossary. **A: Job.** "Turn" stays deprecated. |
| Q6 | How the code carries the persistence rule | **As written:** the structure and the three guardrails of §2.1 (rules 3, 7 and 8 of §6.2), and one reader of the active-Game pointer to keep side by side cheap (rule 2). |
| Q8 | Do the guardrails belong to the design? | **A:** yes. Ticket 24 takes the rule list. The build writes each guardrail before or together with the code that it guards. |
| Q9 | The two guardrails that check nothing | **A:** open an issue outside this map. |
