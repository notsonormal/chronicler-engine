# Research 16: which known problems a clean-room design would collapse, at each scope

Ticket: [Research: which known problems a clean-room design would collapse, at each scope](../issues/16-research-clean-room-collapse.md).

Tree: `9cbf3112`. The only code commit after the tree of research 12 (`d6a047cd`) is `72cfffdc`. It holds the work tree that research 12 read, so the code facts of research 12 hold here (inferred from `git diff --stat d6a047cd..HEAD`).

Labels: **known** = read in a file or in git in this session, or taken from a cited asset that labels it known. **inferred** = follows from known facts, not run. **guessed** = depends on what a clean team would design. **tested** = a test run shows it. No fact here is newly tested.

This asset reports facts and options. It does not choose a scope. It does not decide whether to run a clean-room design. It does not edit the `architecture-deepening` map. Those are decisions for the owner, in [Decide whether and how to run a clean-room design of the generation model](../issues/17-decide-clean-room-run.md).

Two premises come from [research 15](research-15-clean-room-method.md):

- The clean specification holds required behaviour only. Code-shape problems never enter it.
- Of the 18 prior decisions in research 11, 8 are behaviour rulings that a specification can carry: 1, 2, 6, 11, 13 (first half), 14, 16 and 17. 8 are module shape (3, 4, 5, 7, 8, 9, 12 and 15), and 2 are process (10 and 18). The module-shape decisions stay out of the specification (research 15 §3).

Cells marked **†** depend on the conflict between owner ruling 1 and `gg01` decision 3, and so on the order of the run (§4.2).

Terms: `CONTEXT.md` (Message, Swipe, Snapshot, generation, Action, Action Pipeline, Retry, Trigger) and `/codebase-design` (module, interface, seam). "Turn" appears only in titles and quotes.

## Summary

- **36 problems.** 17 `architecture-deepening` tickets, this map's open tickets 01–03, the three seed defects, and the 10 fixed and 3 further open defects of research 12 §4c. 20 are behaviour problems and 16 are code-shape problems (§2).
- **Collapse (settled + moot) per scope:** A 14, B 23, C 25. Open: A 5, B 0, C 5. Untouched: A 17, B 13, C 6 (§4.3).
- **A fails the too-narrow test.** Its clean design must assume five answers that belong to B, and no document gives them: which outside writers must hold the slot, where the generation status lives, what the records and the save can hold, when the claim saves, and where notices go. Every open cell at A waits on one of them (§3.3).
- **B fails the too-narrow test on one point only:** which Game a write lands in, and whether one save is one unit. That is a storage answer. B passes if its specification states it as a requirement on storage (§3.4).
- **B to C adds no behaviour settlement.** The 18 settled cells are the same rows. C adds 2 moot cells and 5 open cells. The 5 open cells are storage and Agent trade-offs that the `architecture-deepening` map is also set to decide (§4.5).
- **3 to 5 settled cells per scope depend on order.** 3 cells depend on the conflict between ruling 1 and `gg01` decision 3. They are settled for a run after ticket 01 answers its first question, and open for a run before it. O-1 and O-2 also depend on order, unless the owner accepts that prior decision 6 fixes the rewind point (§4.2).
- **A run at B overlaps 5 `architecture-deepening` tickets, and a run at C overlaps 11.** Ticket 09 of that map (rank 1, its next pick) overlaps at every scope (§5).
- **Two `architecture-deepening` tickets have drifted since 2026-10-04.** The action-area copy of ticket 11 and the builder copy of ticket 14 are gone (§5.4).

## Method

Files read in this session:

- This map: `map.md`; tickets 01, 02, 03, 15, 16 and 17; `seed-evidence.md`; the research 05, 11 and 12 assets in full; the research 15 asset (summary, §3, §5 step 4, §6 and §7); `grep` passes over the research 04, 06 and 07 assets for `last_trigger`, the anchor rule and `check_retry_anchor`.
- `.scratch/_closed/architecture-deepening/map.md` and all 17 of its tickets (Question, Background, Current state). The two HTML reports were not opened, because no ticket was unclear.
- `docs/specs/retrigger.md` (intro and properties) and `docs/agents/issue-tracker.md`.

Commands run:

- `grep -cE '^#{3,4} +Scenario'` and `grep -cE '^- \*\*I\.[0-9]+'` over `docs/specs/*.md`: the scenario and property counts in §3.1.
- `grep` over `docs/specs/` and `docs/diataxis/` for arrival, `/status/reset-generating`, the edit route, and Quantifier and Trigger coverage.
- `grep -rE "fn <name>\b"` over `src/` for 65 functions, and a `(struct|enum|trait|type)` check for 30 types, cited here or in the `architecture-deepening` tickets. Five were missing: `connection_card_html`, `generation_error_summary`, `get_generating_status`, `persist_options_failure` and `ActionAreaViewModel` (§5.4).
- `git log` on the files of `architecture-deepening` tickets 11 and 14; `git diff --stat d6a047cd..HEAD -- src/application src/domain src/bootstrap`.
- `grep '^Status:\|^Blocked by:\|^Assignee:'` over the 17 `architecture-deepening` tickets.

## 1. How to read the cells

### 1.1 Kinds

- **Behaviour problem.** A client can observe it: an HTTP answer, the story log, the state a restore gives, or the text an LLM receives. It can enter a clean specification as a requirement.
- **Code-shape problem.** A shallow module, a duplicated path or a wide interface. It stays out of the specification. It is checked only when the clean design is compared with the code.

### 1.2 Values

The ticket defines four values. These rules apply them.

1. **untouched.** The module or the behaviour that the problem is about is outside the scope's boundary. This includes a problem whose prevention is wholly inside a module that the scope treats as a black box, even when the boundary states that module's contract.
2. **settled.** Only a behaviour problem can be settled, because only behaviour enters the specification. Two conditions apply. (a) The specification writer has a requirement that excludes the problem. (b) The mechanism that meets the requirement is inside the scope, or in a contract that already exists at the boundary. A requirement is available in four cases: a current spec, doc or owner ruling states it; the owner has already ruled a stale spec wrong; it follows from a behaviour ruling; or it is a general correctness property that needs no ruling (no lost update; a write lands in the Game it belongs to).
3. **open.** The problem is inside the scope, but the clean design must still decide it. This is the case when the requirement needs an owner ruling that does not exist, when the mechanism needs an answer from outside the scope that no document gives, or, for a code-shape problem, when any design of that module faces the same trade-off.
4. **moot.** A code-shape problem inside the scope, or a behaviour problem that a duplicate causes, about a structure that exists only because of the code's history: a second copy of a path, a split by entry path, a left-over variant. A design from the specification has no reason to contain it.

**Collapse** = settled + moot.

For a fixed defect, the cell is about the defect class: does the clean design make that class impossible? The fixed instance is already gone.

A settled cell does not mean that nothing is left to decide. §4.4 lists the decisions that stay inside settled rows.

### 1.3 Labels on cells

- **known:** the definition of the scope alone decides the cell.
- **inferred:** the cell follows from the available requirements and the facts in research 04–15.
- **guessed:** the cell depends on what a clean team would design.

## 2. The problem list

IDs: `AD-nn` = `architecture-deepening` ticket nn; `T-nn` = this map's ticket nn; `S-n` = seed §n; `F-n` and `O-n` = the fixed and the further open defects of research 12 §4c. Row W is extra (see the notes).

The **Requirement** column says where a clean specification gets the requirement that excludes the problem:

- **existing:** a current spec, doc or owner ruling;
- **stale:** a spec that the owner already ruled wrong, so the writer uses the ruling;
- **conflict:** two prior decisions disagree (§4.2);
- **new, general:** no document states it, but it is a correctness property;
- **new, owner:** no document states it, and it is a product choice.

| ID | Problem | Source | Kind | Requirement |
|---|---|---|---|---|
| AD-01 | The constraints of the Storage seam are not written down: the failure injection through `with_backend_mut` and `BackendKind::Test`, and the prior `inherent-impl-locality` decisions. So AD-02 cannot be grilled. | [AD 01](../../architecture-deepening/issues/01-research-storage-seam-constraints.md) | code shape | — |
| AD-02 | `Storage` is a concrete struct with an enum dispatch. It has about 50–65 `pub fn`, and each repeats the `match` on the backend. | [AD 02](../../architecture-deepening/issues/02-deepen-storage-into-backend-trait.md) | code shape | — |
| AD-03 | Thin CRUD services (`PersonaCatalogue`, `WorldCatalogue`, `PromptPresetService`, `SettingsService`) forward to `Storage`, because the `arch-lint.toml` rule forbids HTTP → storage. | [AD 03](../../architecture-deepening/issues/03-replace-crud-services-with-repository-port.md) | code shape | — |
| AD-04 | One lifecycle is split into five `impl ActionPipeline` files by entry path. The phase order lives in `run_from_input` and four other ordering functions. `persist_snapshot_or_err` is called per phase. | [AD 04](../../architecture-deepening/issues/04-consolidate-action-pipeline-turn-lifecycle.md); research 12 §1c, §2a | code shape | — |
| AD-05 | `MessageService` holds four concerns (state load, the save-and-pointer rule, player edits, the Retry anchor) in 12 public methods. `retry.rs` calls `load_snapshot_by_id` past it. | [AD 05](../../architecture-deepening/issues/05-split-message-service-by-concern.md) | code shape | — |
| AD-06 | Shallow helpers surround `PromptAssembler`: `merge_single_user_message` (one `format!`), five public token-budget constants, and `PromptContext::build_narration_prompt`, whose only production caller is arrival. | [AD 06](../../architecture-deepening/issues/06-deepen-prompt-assembler-absorb-helpers.md) | code shape | — |
| AD-07 | `QuantifierAgent::execute` and `OptionsAgent` read the settings and the preset from storage inside the Agent. `AgentContext` has no preset. | [AD 07](../../architecture-deepening/issues/07-move-quantifier-preset-into-agent-context.md) | code shape | — |
| AD-08 | One slot concept is in three files (`gate.rs`, `slot.rs`, `guard.rs`). The registry crosses files. `GenerationGate` exposes seven items. | [AD 08](../../architecture-deepening/issues/08-fold-generation-slot-guard-into-gate.md) | code shape | — |
| AD-09 | The generation status and phase are written by hand: 33 assignment lines in 7 files (39 lines in 8 files on 2026-10-04). | [AD 09](../../architecture-deepening/issues/09-input-buffer-generation-transitions.md); research 12 §1c | code shape | — |
| AD-10 | Arrival re-implements the narrate-and-persist prefix in `ArrivalTaskContext::run_inner`, with its own budget path and its own fallback. | [AD 10](../../architecture-deepening/issues/10-route-arrival-through-narration-generation.md); research 12 §2c | code shape | — |
| AD-11 | The generation-status branch (error → phase → idle) has copies. The poll reads `MessageService` directly. | [AD 11](../../architecture-deepening/issues/11-single-generation-status-strip.md); drift in §5.4 | code shape | — |
| AD-12 | The Options and Quantifier Agents copy one body: the four-entry history window, the `<CurrentRoom>` and `<RecentHistory>` blocks, and a two-attempt retry loop. | [AD 12](../../architecture-deepening/issues/12-share-agent-call-core.md) | code shape | — |
| AD-13 | The `Agent` trait, `AgentRegistry` and `ExecutionPhase` serve two Agents on two dispatch paths. `PreGeneration` and `BackendSelector` have no reader. | [AD 13](../../architecture-deepening/issues/13-retire-or-justify-agent-registry.md) | code shape | — |
| AD-14 | The connection card is defined twice: a builder and the panel template. | [AD 14](../../architecture-deepening/issues/14-single-connection-card.md); drift in §5.4 | code shape | — |
| AD-15 | Each HTTP handler picks its own error policy. A user-displayable Validation error is a 400 in one place and a 200 fragment in another. Failed reads are labelled `EngineError::Config`. | [AD 15](../../architecture-deepening/issues/15-single-fragment-error-policy.md) | behaviour (HTTP contract), with a code-shape part | HTTP contract; outside every scope |
| AD-16 | Bootstrap finds Games with raw SQL in `bootstrap/run.rs`. It creates them through `init_game::resolve_game_id`, a second path next to `GameCatalogue::create_game`. | [AD 16](../../architecture-deepening/issues/16-bootstrap-games-through-game-catalogue.md) | code shape | — |
| AD-17 | `transport/` and `transport/utils/` are re-export layers. The OpenRouter and Ollama adapters map `ChatCompletionResult` to `LlmCallResult` twice. | [AD 17](../../architecture-deepening/issues/17-flatten-llm-transport.md) | code shape | — |
| T-01 | No document states a generation's states, its rewind and result Snapshots, or who may write game state while it runs. The pointer timing follows from the phase order (research 12 §3b). | [ticket 01](../issues/01-model-a-generations-states.md); [seed](seed-evidence.md) "The shared root" | behaviour | conflict (ruling 1 against `gg01` decision 3); writer rule: new, general |
| T-02 | `System` notices live in the Message history. So the LLM prompts read them, they own the last-Message controls, and they take Snapshot pointers. | [ticket 02](../issues/02-replace-system-messages.md); research 09 | behaviour | existing: the owner's rule of 2026-10-11 (no notice reaches an LLM); visibility per source: new, owner |
| T-03 | The location header names the arrival room, but a swipe switch restores the room where the generation started. | [ticket 03](../issues/03-decide-where-the-room-lives.md); research 08 | behaviour | conflict (as T-01); "does the player need the room in the story log": new, owner |
| S-1 | A Swipe's Snapshot is set before the Quantifier runs, so a swipe switch restores the state from before the Quantifier. | seed §1 | behaviour | conflict (as T-01) |
| S-2 | `swipe_new.md` I.6 and 22.1 say that re-impersonate does not re-run the Quantifier. The code runs the full tail. | seed §2; research 11 drifts 1–2 | behaviour (a spec conflict) | stale: the owner ruled for the full tail (prior decision 2) |
| S-3 | Arrival writes game state with no slot. Its save can overwrite a later Action's state, its row sorts after the Action's rows, and its error can overwrite the Action's status. | seed §3; research 05 §5 | behaviour | new, general (no lost update); "arrival on every boot": new, owner |
| F-1 | Concurrent Actions raced (2026-05-10, `994138b6`). | research 12 §4c | behaviour | existing: `swipe_new.md` I.8 and 12.1; `game_flow.md` §Cancellation |
| F-2 | The status stayed on "Generating narration…" while the Quantifier ran (2026-06-01, `d61955d8`). | research 12 §4c | behaviour | existing: `game_flow.md` §Granular Status Phases |
| F-3 | A failed pre-main Snapshot save was dropped (2026-06-23, `2fd24857`). | research 12 §4c | behaviour | existing: `swipe_new.md` I.7 |
| F-4 | One of five room lookups did not have the dynamic-room fallback (2026-07-12, `c5ba1958`). | research 12 §4c | behaviour, caused by a duplicated lookup | — (a duplicate) |
| F-5 | A guided Retry anchored on an older Input's Snapshot (2026-09-06, `7df55ec9`). | research 12 §4c | behaviour | existing: the guided Retry anchor (research 06 found no disagreement with `ai_steering.md`) |
| F-6 | A swipe switch restored a `Generating` Snapshot and locked the page (2026-10-04, `8449a491`). | research 12 §4c | behaviour | existing: `swipe_switch.md` 36.1 |
| F-7 | A stale `Generating` with no slot locked the page (2026-10-06, `5fa46615`). | research 12 §4c | behaviour | existing: `two-state-channels.md`; `dashboard.md` 39.1 |
| F-8 | Event Retry always failed, and the failure rolled the game back. The test fixture had drifted from the real save order (2026-10-11, `9b78b671`). | research 12 §4c; phase-2 ticket 09 | behaviour | existing: prior decisions 11 and 16; `swipe_new.md` I.9, 10.5–10.7, 11.9, 11.10 |
| F-9 | A failed Trigger wrote a raw error as a `System` Message (2026-10-11, `9b78b671`). | research 12 §4c; phase-2 ticket 10 | behaviour | existing: prior decision 13 (first half) |
| F-10 | The location header landed one Message late (2026-10-11, now `72cfffdc`). | research 12 §4c; phase-2 ticket 15 | behaviour | existing: `actions.md` 1.14; prior decision 14 |
| O-1 | A main Retry keeps the previous `last_trigger`. So Retrigger can re-fire a Trigger of a replaced generation. | research 04; research 06 D8 | behaviour | existing, loose: `CONTEXT.md` Retry; prior decision 6; `retrigger.md` I.9 |
| O-2 | The main-Retry anchor is the last Input row. So a Retry of a continue or a post-impersonate Narration rewinds to an earlier Action. | research 06 D10 | behaviour | existing, loose: `CONTEXT.md` Retry; prior decision 6 |
| O-3 | `check_retry_anchor` validates the anchor's pointer, but an event redo restores the event Swipe's pointer. | research 07 §6, point 1 | behaviour | existing: `swipe_new.md` I.7 |
| W (extra) | Research 05 pairs 2–9: writes land in another Game after a switch, reset or create; `handle_cancellation` writes into the new Game; one save splits across two Games; `/status/reset-generating` frees the slot of a running task; pre-claim copies; heal against a finishing generation; edit and delete during a generation; a swipe switch against a new claim. | research 05 §3 | behaviour | new, general |

Notes on the list:

- S-1, S-2 and S-3 are also the first three "still open" defects of research 12 §4c. Each has one row (known).
- T-01 overlaps S-1 and S-3. It stays a row because it is an open ticket and its problem is wider: no document states the model at all.
- AD-01 is a research ticket. Its row records the gap that it fills.
- Research 12 §4c lists six open defects. It does not list research 05's pairs 2–9, because they are inferred races, not observed defects (inferred). Row W holds them, so that their effect is visible. W is not in the counts of §4.3.
- Kinds: the "behaviour" rows are the 19 rows from T-01 to O-3, plus AD-15. The 16 other `AD` rows are code shape.

## 3. The scope options

### 3.1 The table

| Unit | Boundary: what the clean team may assume | Specification size | Behaviour only the code defines | Fits the Destination | Too-narrow test |
|---|---|---|---|---|---|
| **A. This map's model** | **Inside:** a generation's states (Input, Narration, Quantifier result, engine commit, Trigger continuation, Options rewrite, finalize) and its failure states; which Snapshot each Message and Swipe points at; the writer rule; the five Retry forms, Retrigger, and on-demand Options. **Black boxes, with a contract that the boundary must state:** the narrator, Quantifier and Options calls (inputs, result shape, failures); Trigger evaluation; the world rules of the engine commit; and the five B answers of §3.3. | About **72 scenarios and 16 properties**: `actions.md` (21, 4), `swipe_new.md` (30, 8), `retrigger.md` (9, 4), `swipe_switch.md` (2), and 10 of the 15 scenarios in `options.md` (counts known; selection inferred). The 8 behaviour rulings of research 15 §3, plus the owner's notice rule of 2026-10-11. **12 code-only behaviours** to write (§3.5). | §3.5, items 1–12 | **yes** (known: A is the Destination's model) | **fails** (inferred): it must assume five B answers that no document gives (§3.3) |
| **B. The game-state lifecycle around the Action Pipeline** | **Inside:** A, plus the Message history, Swipes and Snapshots (records, load, save), the generation status, `GenerationGate` (slot, guard, heal), arrival, and the HTTP handlers that write game state (§3.2). **Black boxes:** the LLM calls, Trigger evaluation, prompt assembly, storage, and the HTTP views. The boundary must state the storage contract of §3.4. | About **86 scenarios and 16 properties**: A, plus `story_log.md` 8.1–8.4, `reset.md` 7.1–7.2, `games.md` 17.1, 18.1 and 18.3, `dashboard.md` 39.1 and 39.3, `failure_display.md` 38.3, 38.5 and 38.7. **20 code-only behaviours** (§3.5). Arrival, `/status/reset-generating` and the edit route have no scenario at all (known). | §3.5, items 1–20 | **yes** (known: B's modules are the "Domain" list in the map's Notes; research 05 added the handler writers) | **fails on one C answer** (inferred): which Game a write lands in, and whether one save is one unit. It passes if the specification states that answer as a requirement on storage (§3.4) |
| **C. The engine core** | **Inside:** B, plus the Agents (Quantifier, Options, the registry, the preset lookup), Triggers, prompt assembly and storage. **Black boxes:** the HTTP views (templates, fragments, error rendering, the status strip) and the LLM transport (the `LlmProvider` adapters). | B, plus about **25–60 scenarios** from `options.md` (24.10, 24.11, 24.19), `narrator_mode.md` (5), `llm_messages.md` (2), and the generation-facing part of `prompt_presets.md` (39) and `settings.md` (18). **Four subsystems have no feature spec:** the Quantifier, Triggers, prompt assembly and storage. Their requirements must come from `docs/diataxis/reference/` and the code (known: no spec file; inferred: the range). **28 code-only behaviours** (§3.5). | §3.5, items 1–28 | **no** (inferred): it adds modules outside the map's Domain, it overlaps 11 `architecture-deepening` tickets, and the Destination's final decision is about the pipeline, not the engine core. It needs its own map | not applied (the ticket applies the test to A and B). Observation (inferred): ticket 03's options B and C need the story-log view, which is a black box at C too |

The scenario counts are before the four spec edits that research 15 §3 lists: remove the identifiers, turn mechanism phrases into observable outcomes, correct the stale claims, and add the behaviour rulings that no spec carries.

The edits are not the same size at each scope. Research 15 §3 puts each spec input in a class, and its identifier counts are tested:

- **A:** `swipe_switch.md` is clean. `actions.md`, `swipe_new.md` and `retrigger.md` are mixed, and `retrigger.md` names 9 `src/` identifiers. `options.md` was not checked.
- **B adds** `failure_display.md` (clean) and `reset.md` (mixed, 2 identifiers). `story_log.md`, `games.md` and `dashboard.md` were not checked.
- **C:** research 15 checked none of the specs that C adds.
- **Every scope** needs `CONTEXT.md`, which is mixed. It defines Snapshot, Game, Action Pipeline and Agent by the current mechanism.

### 3.2 How this asset reads the list for B

The ticket lists "the HTTP handlers that write game state (edit, delete, swipe switch, reset)". This asset reads the parenthesis as examples:

- `GameCatalogue::switch_game` and `GameCatalogue::create_game` also write game state. Both call `Storage::set_game_id`, which retargets every later write of every writer (research 05 §2b).
- "Reset" covers both `POST /reset` (`GameCatalogue::reset`) and `/status/reset-generating` (`ActionPipeline::reset_persisted_status`).

The B cells assume this reading. If B leaves out the game switch and create, B must assume their contract. Then row W stays open at B (inferred).

### 3.3 The too-narrow test for A

**Result: fails (inferred).** A's clean design must assume five answers that belong to B. No current document gives them as requirements.

1. **Which writers outside the generation exist, and whether they must hold the slot.** These writers are arrival, `switch_swipe`, `edit_history`, `delete_last`, `/status/reset-generating`, `POST /reset`, and the game switch and create (research 05 §2b). A's writer rule is a rule about these B modules. No document states a writer rule (research 11). `/status/reset-generating` has no spec, and it frees the slot of a running task on purpose (research 05 pair 5). So a rule "only the slot owner writes" conflicts with that route until B decides what the route is for.
2. **Where the generation status lives.** It lives only in the Snapshot (research 05 §0). So every save point in A is also a status write, and every state that A saves during a generation carries `Generating`. `two-state-channels.md` describes this mechanism. It does not state a requirement.
3. **What a Message record and a Swipe record can hold, and whether one save is atomic.** A model with one pointer (the result, with the rewind state taken from the previous Message) fits today's records. A model with two pointers, or the header write-back of ticket 03 option A, needs B's records. `save_message_and_snapshot` is up to five storage calls with no transaction (research 05 §0).
4. **When the claim saves.** `GenerationGate::try_claim` writes the Input row and the claim Snapshot (research 04). So the Input's pointer is set inside a B module, before A's first state.
5. **Where notices go.** A can require that a generation adds no notice Message, but the channel that replaces the Message is a B module. For the NPC-fallback and unknown-room sources, no channel exists today: no source sets `GenerationStatus::Error`, and the banner carries only a failed LLM attempt (research 09).

**The phase order does not make A too narrow** (inferred). The specification can state the data dependencies: the Quantifier reads the Narration, the engine commit applies the Quantifier result, Triggers evaluate the committed state, and Options read the result. The order follows from them.

A passes only if its specification answers items 1–5. Then the boundary carries B's design decisions, so A is no longer a black-box unit (inferred).

### 3.4 The too-narrow test for B

**Result: fails on one answer that belongs to C, unless the specification states it as a requirement on storage (inferred).**

1. **Which Game a write lands in, and whether one save of several rows is one unit.** Every write goes to the process-global `Storage::game_id()`, which is read at each storage call (research 05 §0). B's writer rule must say which Game a write belongs to. Research 05 pairs 2 and 4 come from this. `storage_design.md` says that a `Storage` instance "carries its game id at construction time", which research 11 found loose. If the B specification copies that claim, B is too narrow. If it states "a write names its Game, and a Message, Swipe and Snapshot save is one unit" as a requirement on storage, B passes and gives C a requirement.

B passes on these answers, if the specification states the black-box contract:

2. **The result shapes and failure modes of the Quantifier and Options calls** (Agents). `docs/diataxis/reference/narrative/agent_system.md` and `docs/specs/options.md` describe them.
3. **When a Trigger fires, its one-shot mark, and what `last_trigger` holds** (Triggers). `actions.md` 1.4–1.8 and `retrigger.md` give the observable part.
4. **What a prompt reads from the Message history** (prompt assembly). This answer is needed only if notices stay in the history (ticket 02). A B design that moves notices out of the history needs no C answer.
5. **The preset in force for a generation.** `swipe_new.md` 22.4 states it for the impersonate preset.

### 3.5 Behaviour that only the code defines

Stale and loose claims come from research 11 and were not re-checked.

**A (items 1–12):**

1. Which Snapshot a swipe switch restores. `swipe_switch.md` pins only `Idle` and the cleared options. `storage_design.md` §Per-swipe state binding states the intent, and the code differs (research 11, drifts 11–12).
2. The rewind point of a Retry of a continue or a post-impersonate Narration. No scenario covers it (research 06 D10).
3. Re-impersonate runs the full tail. `swipe_new.md` I.6 and 22.1 are stale (drifts 1–2).
4. Event Retry and Retrigger run a reconciliation Quantifier pass that cannot move the player. The reasons in I.6, 10.2 and `retrigger.md` 13.2 are stale (drifts 3, 5).
5. A Retry or a Retrigger can add Messages: an event Message and `System` Messages. See I.2 against 9.5, and `retrigger.md` I.3 (drifts 4, 6).
6. The Input's pointer is the claim save, which still holds the previous `last_trigger` (research 04).
7. `last_trigger` after a main Retry (research 04; the map's fog "Retry tails").
8. The writer rule during a generation. No document states one (research 11).
9. Which Message a Retry targets when `System` Messages trail. `CONTEXT.md` Retry is loose (drift 7; research 06 D4).
10. Every Message has Swipes, and user regen and re-impersonate add a Swipe to an Input. `CONTEXT.md` Swipe is loose (drift 8); prior decision 6 rules it.
11. Most Snapshots belong to no Message. `CONTEXT.md` Snapshot is loose (drift 9).
12. The event-Retry anchor can be a `System` Message (research 06 D9).

**B adds (items 13–20):**

13. Arrival runs on every boot of a World with no scenario text, holds no slot, and writes one Message and one Snapshot (research 05 §5). No spec or doc describes it (known: `grep` found no arrival text in `docs/specs/` or in `docs/diataxis/reference/startup.md`).
14. `/status/reset-generating` frees the slot of a running task (research 05 pair 5). No spec covers it. Only the generated route reference names it (known).
15. `POST /history/:id` (`edit_history`). No scenario covers it. `swipe_new.md` 9.4 only uses an edited Input (known).
16. Where the writes of a cancelled generation land after a game switch (research 05 pairs 2–3). `game_flow.md` §Cancellation says only that a switch "invalidates stale work at the next boundary".
17. The generation status lives in every Snapshot (research 05 §0). No requirement says where the status lives.
18. `delete_last` and `edit_history` act on a trailing `System` Message (research 07).
19. `pending_location` seeds the Narration of the starting scenario. Phase-2 ticket 15 kept it, and `subplan-c-mapless-enablement.md` plans to delete it (research 11, drifts 13–14).
20. Every write targets the process-global current Game. `storage_design.md` is loose (research 11).

**C adds (items 21–28):**

21. The Quantifier retries on low confidence, and Options retries on zero parsed options (AD-12). No spec covers either rule.
22. Each Agent reads the last four history entries and renders the same `<CurrentRoom>` and `<RecentHistory>` blocks (AD-12).
23. An Agent reads its prompt preset from storage inside `execute`, during the generation (AD-07).
24. Every prompt that carries history renders `System` Messages, the Trigger continuation prompt included (research 07). This is against the owner's rule of 2026-10-11.
25. Arrival takes its token budget from values read at spawn time, not through `PromptAssembler` (AD-10).
26. A save of several rows is several storage calls, each with its own lock, and no transaction (research 05 §0; `storage_design.md` partial).
27. Tests inject failures per storage operation through `BackendKind::Test` (AD-01).
28. The order in which Triggers are evaluated. `actions.md` 1.8 pins the one-shot rule only (inferred: no spec states the order).

## 4. The collapse matrix

### 4.1 Matrix

| ID | A | B | C | Note |
|---|---|---|---|---|
| AD-01 | untouched (known) | untouched (known) | open (inferred) | At C the specification writer must state the storage requirements: two backends, failure injection per operation, the `inherent-impl-locality` rules. The research moves into the specification. It does not vanish. |
| AD-02 | untouched (known) | untouched (known) | open (inferred) | Any design of storage must still choose the seam. Two real backends and a test adapter exist (`BackendKind`). |
| AD-03 | untouched (known) | untouched (known) | untouched (inferred) | The services serve the World, Persona, Settings and Preset views, which C does not list. C's storage decision changes the premise (AD-03 is blocked by AD-02). |
| AD-04 | moot (inferred) | moot (inferred) | moot (inferred) | A model has one sequence of states, with an entry state for each Retry form. Five ordering functions split by entry path have no counterpart. |
| AD-05 | open (inferred) | moot (guessed) | moot (guessed) | A owns two of the four concerns (the pointer rule and the Retry anchor). The record and edit concerns are B's (§3.3, items 1 and 3). At B a design groups the concerns by its own model. |
| AD-06 | untouched (known) | untouched (known) | moot (guessed) | At B, one narration path for arrival removes the only production caller of `build_narration_prompt`. That is an interaction, not a settlement. At C, the visibility of the token-budget values stays a decision. |
| AD-07 | untouched (known) | untouched (known) | open (inferred) | Each Agent has a per-Game preset (`games.md` 20.11). So any design must choose where the preset is resolved. |
| AD-08 | untouched (known) | moot (guessed) | moot (guessed) | The width of the gate's interface depends on B's answer for `/status/reset-generating`. |
| AD-09 | open (inferred) | moot (inferred) | moot (inferred) | At A, the home of the status is a B answer (§3.3, item 2). At B, one owner writes the status. |
| AD-10 | untouched (known) | moot (inferred) | moot (inferred) | The decision that stays: does arrival run the Quantifier and the Triggers, and on every boot? |
| AD-11 | untouched (known) | untouched (known) | untouched (known) | An HTTP view. B's status model decides what the strip reads. Drift in §5.4. |
| AD-12 | untouched (known) | untouched (known) | open (guessed) | "Contract or convenience" (the ticket's constraint) is a decision that any design of two Agents must make. |
| AD-13 | untouched (known) | untouched (known) | open (guessed) | The specification must say whether the set of Agents is fixed. The unread `BackendSelector` and `PreGeneration` would be moot. |
| AD-14 | untouched (known) | untouched (known) | untouched (known) | An HTTP view. Drift in §5.4. |
| AD-15 | untouched (known) | untouched (known) | untouched (known) | The HTTP adapter's policy. B can add failure kinds that the views must map (ticket 02). |
| AD-16 | untouched (known) | untouched (inferred) | untouched (inferred) | B includes one creation path (`GameCatalogue::reset`), and C includes the storage lookup. Neither includes the composition root, where the raw SQL and the second creation path are. |
| AD-17 | untouched (known) | untouched (known) | untouched (known) | The LLM transport is a black box at every scope. |
| T-01 | open (inferred) | settled† (inferred) | settled† (inferred) | At A, the states and the pointers are inside, but the writer rule needs B's writers (§3.3, item 1). At B and C, the clean design is a candidate model. Ticket 01's grilling stays, with a candidate to grill. |
| T-02 | open (inferred) | settled (inferred) | settled (inferred) | At A, the replacement channel is a B answer (§3.3, item 5). At B, the history and the channel are both inside. |
| T-03 | settled† (inferred) | settled† (inferred) | settled† (inferred) | A result pointer settles the mismatch, and today's records can hold it. Where the header lives stays a decision (§4.4). |
| S-1 | settled† (inferred) | settled† (inferred) | settled† (inferred) | One pointer that holds the result, with the rewind state taken from the previous Message, fits today's records. Two pointers need B's records. |
| S-2 | settled (inferred) | settled (inferred) | settled (inferred) | The specification must carry the owner ruling, not I.6 and 22.1. The same spec edit is needed without a clean room. |
| S-3 | open (inferred) | settled (inferred) | settled (inferred) | At A, the writer rule is inside, but arrival is a B module. So the reach of the rule needs §3.3, item 1. |
| F-1 | untouched (inferred) | settled (inferred) | settled (inferred) | At A, the slot is assumed. |
| F-2 | settled (inferred) | settled (inferred) | settled (inferred) | A defines the states that the phase reports. |
| F-3 | settled (inferred) | settled (inferred) | settled (inferred) | A failed save of the rewind state is a failure state. |
| F-4 | untouched (inferred) | untouched (inferred) | moot (inferred) | Room lookups build the input of the narrator call, which is a black box at A and B. |
| F-5 | settled (inferred) | settled (inferred) | settled (inferred) | — |
| F-6 | settled† (inferred) | settled (inferred) | settled (inferred) | At A, settled if the result state is idle. With no result pointer, the restored state carries `Generating` unless B moves the status. |
| F-7 | untouched (inferred) | settled (inferred) | settled (inferred) | The display channel and the heal are B's. |
| F-8 | settled (inferred) | settled (inferred) | settled (inferred) | The fixture drift is test code, outside every clean design (§4.4). |
| F-9 | settled (inferred) | settled (inferred) | settled (inferred) | The status line and the banner carry this failure today (phase-2 ticket 10). |
| F-10 | settled (inferred) | settled (inferred) | settled (inferred) | — |
| O-1 | settled (inferred) | settled (inferred) | settled (inferred) | A redo starts where the replaced generation started (prior decision 6), and Retrigger recovers the last generation's failed Trigger (`retrigger.md` I.9). So the redo cannot keep a Trigger that the replaced generation did not see. |
| O-2 | settled (inferred) | settled (inferred) | settled (inferred) | "A new swipe redoes the last generation" (prior decision 6) excludes a rewind to an earlier Action. |
| O-3 | settled (inferred) | settled (inferred) | settled (inferred) | I.7 applies to the event Message too, so a design validates the Snapshot that the redo restores. |
| W (extra) | open (inferred) | settled (inferred) | settled (inferred) | B must include the game switch and create (§3.2). Pair 4 also needs the storage answer of §3.4. |

### 4.2 The † cells: ruling 1 against `gg01` decision 3

Research 11 records two prior decisions that disagree:

- **Ruling 1** (seed §1): a Swipe holds the state that its own generation produced, the Quantifier's updates included.
- **`gg01` decision 3:** "Swipe snapshots are captured pre-tail at message-add time in every path, so `switch_swipe` semantics are unchanged."

Ticket 43's rejected fix (commit `8449a491`) gives the reason for the pointer before the tail: a Swipe's Snapshot is also the rewind point of the guided, impersonate and user-regen redos (research 12 §4d).

Research 15 §3 classes decision 3 as module shape. It also says that decision 3 has a behaviour consequence (a swipe switch restores the state from before the Quantifier) that conflicts with ruling 1, that a clean specification cannot carry both, and that the conflict is ticket 01's first question. Research 15 §6 lists this conflict among the questions that stay open for a run before ticket 01.

Inferred: as requirements, "a swipe switch restores the result" and "a redo starts from the state before the generation" do not conflict. They conflict only when one pointer must serve both. The number of pointers is a mechanism.

So the † cells depend on the order of the run:

- **A run after ticket 01 answers its first question for ruling 1:** the † cells are settled.
- **A run before ticket 01:** the † cells are open, and the clean agent chooses there. Research 15 §6 says that such choices are options, not evidence. The owner can answer that one question before the run. Then the † cells are settled.

Research 15 §6 lists five questions that stay open for a run before ticket 01. This asset treats them as follows:

| Question (research 15 §6) | Cells | Treatment here |
|---|---|---|
| The conflict between decisions 1 and 3 | T-01 (B, C), T-03, S-1, F-6 (A) | † |
| The room | T-03 | † |
| The rewind point | O-1, O-2 | Settled, on a reading of prior decision 6: "a new Swipe redoes the last generation", so the redo starts where the replaced generation started. The map's fog patch "Retry tails" points the same way ("per Action, not the last Input row"), but it still lists the rewind point and `last_trigger` as open. If the owner does not accept the reading, O-1 and O-2 are open for a run before ticket 01, as the † cells are. |
| The notice channel | T-02 (at B and C) | Settled. The owner's rule of 2026-10-11 (no notice reaches an LLM) excludes the defect, whatever channel the design picks. The channel stays a decision (§4.4). |
| The writer rule | S-3 and row W (at B and C) | Settled. "No lost update" excludes the defects, whatever writer rule the design picks. The rule itself stays a decision (§4.4). |

In the last three rows, the choice of the clean agent is an option, not evidence (research 15 §6). The cell does not change, because the cell asks only whether the problem can still occur. At A, T-02, S-3 and row W are open, because the channel and the outside writers are B modules (§3.3).

### 4.3 Counts

The 36 rows of §2. Row W is not counted.

| Value | A | B | C |
|---|---|---|---|
| settled | 13 | 18 | 18 |
| moot | 1 | 5 | 7 |
| open | 5 | 0 | 5 |
| untouched | 17 | 13 | 6 |
| **collapse (settled + moot)** | **14** | **23** | **25** |

**† cells:** 3 of the settled cells at each scope (A: T-03, S-1, F-6; B and C: T-01, T-03, S-1). For a run before ticket 01, they are open: A 10 settled and 8 open; B 15 and 3; C 15 and 8. Collapse is then A 11, B 20 and C 22. If O-1 and O-2 are open too (§4.2): A 8 settled and 10 open; B 13 and 5; C 13 and 10. Collapse is then A 9, B 18 and C 20.

**By kind:**

| Kind (rows) | A | B | C |
|---|---|---|---|
| Behaviour (20) | 13 settled, 3 open, 4 untouched | 18 settled, 2 untouched | 18 settled, 1 moot, 1 untouched |
| Code shape (16) | 1 moot, 2 open, 13 untouched | 5 moot, 11 untouched | 6 moot, 5 open, 5 untouched |

**Row W:** A open, B settled, C settled. With W, B and C each have 19 settled cells, and A has 6 open cells.

### 4.4 Decisions that stay inside settled rows

- **T-01.** Ticket 01 stays a grilling. A clean design is a candidate model for it, not its answer.
- **T-02.** The channel for each notice source, and whether the player sees the NPC-fallback and unknown-room notices at all (ticket 02's questions). Research 15 §6 lists the notice channel among the questions that stay open for a run before ticket 01. Legacy Games with stored `System` Messages need a migration, because the mapper's `?` in the row loop blanks the whole history of a legacy Game (research 09).
- **T-03.** Where the header lives (ticket 03's options). Options B and C need the story-log view, which is a black box at every scope. Legacy rows keep their stored headers. `pending_location` survives or goes.
- **S-2.** The spec edit to I.6 and 22.1. It is needed with or without a clean room.
- **S-3 and AD-10.** Whether arrival runs on every boot, and whether it runs the Quantifier and the Triggers.
- **S-3 and row W.** The writer rule itself: which writers must hold the slot, and what `/status/reset-generating` is for (§3.3, item 1). Research 15 §6 lists the writer rule among the questions that stay open for a run before ticket 01.
- **O-1 and O-2.** The exact rewind point, and whether a main Retry clears `last_trigger`. The cells rest on a reading of prior decision 6 (§4.2). The owner may rule otherwise.
- **F-8.** The fixture drift. Test fixtures are outside every clean design. A comparison of pointers needs a test that loads a pointer back and compares the restored world. No tier has one (research 10).

### 4.5 What the matrix shows

Facts, with no weighting:

- At A, 14 of 36 cells collapse. Each open cell at A (T-01, T-02, S-3, AD-05, AD-09) waits on one of the five B answers of §3.3 (inferred).
- At B, 23 cells collapse, and no cell is open. Every cell that B leaves is outside it: the HTTP views, the LLM transport, storage, the Agents, prompt assembly, and the composition root (inferred). §4.4 lists the decisions that stay inside B's settled rows.
- From B to C, the behaviour rows do not change: 18 settled at both. C adds 2 moot cells (AD-06, F-4) and 5 open cells (AD-01, AD-02, AD-07, AD-12, AD-13). The open cells are trade-offs that the `architecture-deepening` map is also set to decide (inferred).
- The fixed-defect classes: at B and C, 9 of 10 are settled, and F-4 is untouched at B and moot at C. At A, 7 of 10 are settled. F-1, F-4 and F-7 are untouched at A, because their prevention is the slot, the room lookup and the status display (inferred).
- 3 settled cells at each scope depend on the order of the run. 2 more (O-1, O-2) also do, unless the owner accepts a reading of prior decision 6 (§4.2).

## 5. Coordination with `architecture-deepening`

### 5.1 Overlap

An `architecture-deepening` ticket overlaps a scope when its cell is not "untouched".

| Scope | Overlapping tickets | Touched indirectly (cell untouched, but the premise changes) |
|---|---|---|
| A | AD-04, AD-05, AD-09 | — |
| B | AD-04, AD-05, AD-08, AD-09, AD-10 | AD-06 (arrival is the only production caller of `build_narration_prompt`); AD-11 (B's status model decides what the strip reads); AD-16 (B includes `GameCatalogue::reset`) |
| C | AD-01, AD-02, AD-04, AD-05, AD-06, AD-07, AD-08, AD-09, AD-10, AD-12, AD-13 | AD-03 (blocked by AD-02); AD-11; AD-16 |

B's set is the fog patch "Generation-state cluster ordering" of that map (tickets 09, 04, 08 and 10), plus AD-05 (known, by comparison).

### 5.2 The three options for a run at B or C

| Option | Cost (inferred) | Conflict with that map's Out of scope rule | Conflict with that map's Priority table |
|---|---|---|---|
| **Proceed as planned.** Sessions on that map take tickets in Priority order while the run goes on. | The same modules get two decisions: one grilled against the current code, one from the comparison. Each grilling anchors on the current shape, which the run exists to avoid. The decisions are code-shape decisions, so they cannot enter the clean specification as requirements (research 15 §3). The comparison must treat each one as one more shape to compare. A disagreement re-opens a resolved ticket, and the wayfinder process has no re-open step. Exposure: 5 tickets at B, 11 at C. | None. The run belongs to another effort, and that map keeps its scope. | None. Rank 1 (AD-09) is that map's next pick. |
| **Pause until the run ends.** Overlapping tickets are skipped until the comparison lands. | Delay. At B, 4 tickets on that map's frontier wait, rank 1 among them. At C, 8 wait (ranks 1, 2, 4, 5, 9, 10, 11 and 15). The pause needs a marker: the wayfinding operations define only `open`, `claimed` and `resolved` (known, `docs/agents/issue-tracker.md`). AD-01 is research, and its asset is input to a C specification. So a pause of AD-01 at C costs time and gives nothing. | None. | Yes. The table ranks by review strength and leverage. A pause is an order that the table does not express. Someone must mark it in that map, and this ticket may not. |
| **Fold into a new effort.** Overlapping tickets close in that map and move into the clean-room effort. | Each folded question must be re-stated in the new effort. That map loses its generation-state cluster and the fog patch about it. Its Destination says that every candidate gets a decision (known), so each folded ticket needs a closing line that points to the new effort. | Partial. The rule puts "rewrites not surfaced by the reviews" in a future review. The folded tickets were surfaced by the reviews, so the rule does not cover them. Closing them needs that map's Destination redrawn, or a resolution that says "decided in <effort>" (inferred). | Yes. The folded tickets leave the table. The fog patches "Generation-state cluster ordering" and (at C) "Agent cluster shape" change, and so does the exception for tickets 12 and 13 (inferred). |

That map's fog patch "Cross-cutting migration shape" asks whether the accepted deepenings land as one migration or as independent refactors. A run at B is one answer to that question for the generation-state cluster (inferred).

### 5.3 Overlapping tickets that a session can resolve before this research lands

Known at `9cbf3112`: each of the 17 tickets of that map has `Status: open` and `Assignee: (unclaimed)`. Four are blocked: 02 by 01, 03 by 02, 07 by 13, and 08 by 09. That map's pick rule takes the highest-ranked open, unblocked and unclaimed ticket (its Notes). So a session on that map can take any of these overlapping tickets now:

| Ticket | Rank | Overlaps at | An early resolution, if a run goes ahead (inferred) |
|---|---|---|---|
| AD-09: Give the input buffer named generation transitions | 1 | A, B, C | **The next pick of that map.** Its transition set and its home anchor on the status inside the Snapshot, which B must decide. Likely double work at B and C. |
| AD-01: Research the Storage seam constraints | 2 | C | Research, not a decision. Its asset is input to a C specification. It keeps its value. |
| AD-10: Route arrival narration through the one narration-generation path | 4 | B, C | A "commit" agrees with the likely clean shape. A "reject" conflicts with it (guessed). |
| AD-12: Share one Agent call core between Options and Quantifier | 5 | C | Open at C: the clean design faces the same choice. |
| AD-13: Retire or justify the Agent registry | 9 | C | Open at C. |
| AD-04: Consolidate the ActionPipeline turn lifecycle | 10 | A, B, C | Moot at every scope. Its module shape anchors on the current code. |
| AD-05: Split MessageService by concern | 11 | A, B, C | Open at A, moot at B and C. |
| AD-06: Deepen PromptAssembler, absorb shallow helpers | 15 | C | Moot at C. |

These overlapping tickets are blocked, so no session can resolve them first: AD-08 (by AD-09), AD-02 (by AD-01) and AD-07 (by AD-13).

### 5.4 Drift in the "Current state (2026-10-04)" sections of that map

- **AD-11.** Commit `41807c36` (2026-10-08, "retire the caller-less action-area fragment and dead code") removed `ActionAreaViewModel`, one of the three copies (known). `GameViewQuery` has no `get_generating_status`, and `generation_error_summary` is now `generation_failure_summary` (known). The poll `generating_status_handler` still loads the state through `MessageService::load_expecting_valid_state` and branches failure → phase → `idle` (known). The copy in `tests/test_utils/stub_server.rs` was not re-checked.
- **AD-14.** Commit `6916556a` (2026-10-07, "split Settings into Connections and Text Check sub-tabs") deleted `builders/connections.rs` and its `connection_card_html` (known). The panel loop in `settings/templates/settings.rs` renders `connection-row`, the only such markup in `src/` (known). Inferred: the duplicate that AD-14 names is gone.
- **AD-04, AD-05, AD-08, AD-09, AD-10.** Research 12 measured their sizes and counts again (for example, 33 status lines in 7 files, not 39 in 8). Those counts hold at `9cbf3112` (inferred, see the header).
- **This map's ticket 02** names `persist_options_failure`. The function is `finalize_options_failure` (known).

## 6. Facts for the fog patch "Fix or rewrite"

- A clean design at A cannot stand alone. It must assume five B answers that no document gives (§3.3).
- B fits the map's Domain, and at B no cell is open. Its specification needs about 86 scenarios, 16 properties, 20 code-only behaviours and one requirement on storage (§3.1, §3.4). Decisions stay inside its settled rows (§4.4).
- C settles no behaviour row that B does not settle. It adds 5 open code-shape cells that it shares with the `architecture-deepening` map, and it needs its own map (§4.5).
- 3 settled cells per scope depend on the order of the run against ticket 01. 2 more do too, unless the owner accepts a reading of prior decision 6. One owner answer before the run settles each group (§4.2).
- At B, 9 of the 10 fixed-defect classes are settled (§4.5).
- A run at B or C overlaps 5 or 11 `architecture-deepening` tickets. AD-09, the next pick of that map, overlaps at every scope (§5).
