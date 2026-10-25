# Research 04: the open architecture-deepening tickets, sorted against the new design

Ticket: [Research: sort the open architecture-deepening tickets against the new design](../issues/04-research-sort-architecture-deepening.md).
Design: [target design](../../turn-state-model/assets/target-design.md) version 2, [sketch 25](../../turn-state-model/assets/sketch-25-new-design.md) draft 3 (as changed by [ticket 05](../issues/05-decide-job-and-busy-state-shape.md)), [sketch 24](../../turn-state-model/assets/sketch-24-agent-hooks.md). Test rulings: [turn-state-model ticket 26](../../turn-state-model/issues/26-decide-test-placement.md) R1 to R11.
Code: read at commit `d067420b` (2026-10-16).

Labels: **known** (read in the code or in a design asset), **inferred**, **guessed**.

## 1. Results

Four results. The ticket named three; the fourth is new, because one ticket's subject no longer exists.

| Result | Meaning |
|---|---|
| **Settled** | The design answers the question. The build carries the answer out. |
| **Ticket here** | The build plan needs an answer that the design does not give. A new ticket in this map holds the question. |
| **Later effort** | The question is real, but the build does not need its answer. Each gets one line in the map's **Out of scope**. |
| **Moot** | The code that the ticket names is gone. |

| AD ticket | Result | Evidence | Where it goes |
|---|---|---|---|
| 01 Research the Storage seam constraints | **Ticket here** | §2.1 | [Research: the storage constraints that the rebuild must keep](../issues/10-research-storage-constraints-for-rebuild.md) |
| 02 Deepen Storage into a StorageBackend trait | **Ticket here** | §2.2 | [Decide the storage seam of the rebuild](../issues/11-decide-storage-seam-of-rebuild.md) |
| 03 Replace CRUD services with a Repository port | **Ticket here** (the repositories); the CRUD-service part goes with it | §2.3 | the same ticket |
| 06 Deepen PromptAssembler | **Settled** in part; the rest is a **later effort** | §2.4 | — |
| 11 One generation-status strip | **Settled** | §2.5 | — |
| 12 Share one Agent call core | **Settled** in part; the rest is a **later effort** | §2.6 | — |
| 14 One definition of the connection card | **Moot** | §2.7 | — |
| 15 One fragment error policy | **Settled** for the routes that the build changes; the rest is a **later effort** | §2.8 | — |
| 16 Bootstrap through GameCatalogue | **Settled** for creation; the lookup is a **ticket here** | §2.9 | [Decide the storage seam of the rebuild](../issues/11-decide-storage-seam-of-rebuild.md) |
| 17 Flatten LLM transport | **Later effort** | §2.10 | — |

The first estimate in the ticket (inferred, from the charting grilling) was close. Two changes: ticket 14 is moot, not "no overlap"; and the lookup half of ticket 16 joins the storage ticket.

## 2. Each ticket

### 2.1 AD-01 Research the Storage seam constraints — ticket here

**What the design fixes** (known, sketch 25 §3.1). Repositories per record. Every read takes a `GameId`. Record writes exist only on a transaction handle `Tx`. "A transaction runs inside one `with_backend_mut` closure: a SQLite transaction, or one change of the in-memory data under the backend mutex." Ticket 26 R5: storage keeps its own tier, one suite on both backends, "with one failure injected in the middle of a save unit".

**Why the research is still needed.** Three facts conflict with that shape or limit it:

1. **Failure injection cannot fail half a save unit** (known). `with_backend_mut` checks the test override by method name at its entry, before `f` runs, and returns the error in place of the call (`src/adapters/driven/storage/core.rs:126-152`; `TestFailureHandle::set(method, ..)`, `test_support.rs:26`). A `Tx` that runs inside one `with_backend_mut` call can only fail whole, at its start. The R5 suite needs a failure between two record writes of one unit. So the failure injection must change, and the research must say what it must carry.
2. **arch-lint forbids storage to import the application layer, ports included** (known, `arch-lint.toml:164-169`: "Storage adapter must not depend on the application layer; depend on domain models only"). A `Repository` port trait in `application/ports/` that `Storage` implements breaks this rule. AD-03 assumed the port trait would satisfy the lint rule; it satisfies 138-142 and breaks 164-169.
3. **The prior decisions** of `.scratch/inherent-impl-locality/` tickets 03 and 11 (the `backend/` folder flattened; one `impl Storage` per file kept; the locality rule name-only). Still not written down as constraints.

The `Current state (2026-10-04)` of AD-01 still holds (known): `with_backend_mut` at `core.rs:126`, now 47 references (48 then); 11 files with `impl Storage`; `BackendKind::Test` at `core.rs:29-35`.

### 2.2 AD-02 Deepen Storage into a StorageBackend trait — ticket here

**What the design fixes** (known, sketch 25 §7.1, `storage/core.rs` row): "backend dispatch kept; `transaction` and `create` enter through `with_backend_mut`". New storage code: `stories.rs`, `docks.rs`, `transaction.rs`, "for both backends", about 330 lines (guessed, sketch 25 §7.2).

**Why the build needs an answer.** The build writes about 330 new storage lines and about 10 new record writes, each for two backends. Under today's dispatch, each record write is one more pair of `match` arms. Under a trait, each is one method in two adapters. The choice decides the shape of the new code, so it cannot wait for a later effort. The friction of AD-02 is unchanged (known): 65 `pub fn` lines in the 11 `impl Storage` files.

### 2.3 AD-03 Replace CRUD services with a Repository port — ticket here

**What the design fixes** (known). Sketch 25 §3.1: "Whether the reads and the `Tx` writes are port traits or methods on `Storage` is the question of `architecture-deepening` ticket 03, for all of storage. The sketch shows methods." The AD-03 comment of 2026-10-14 says the same.

**Why the build needs an answer.** The repositories of area 1 are new code. Their shape (port trait or `Storage` method) sets which way the dependency between `application/` and `storage/` points, and §2.1 fact 2 shows that a port trait needs an arch-lint change. [Prototype: decide the crate split and the dev-build loop](../issues/07-prototype-crate-split-and-dev-build-loop.md) needs that direction for any crate layout.

**The CRUD services** (`PersonaCatalogue`, `SettingsService`, `PromptPresetService`, `WorldCatalogue`) are outside the designed scope. Their state today (known): 2, 5, 6 and 6 `pub fn` (each count includes `new`); `PersonaCatalogue` is 27 lines. The build does not need to touch them. But once the repository shape is chosen, the same session can rule whether the services join it in the build or stay, at little extra cost. So they go into the same ticket, as its last question.

### 2.4 AD-06 Deepen PromptAssembler — settled in part, later effort for the rest

Settled (known):

- `PromptContext::build_narration_prompt` (`assembler.rs:181`) has one production caller, `arrival_service.rs:153`. Arrival is deleted (D-25), so the method goes with it.
- `PromptAssembler::with_storage` (`assembler.rs:72`) reads the token budget from `Storage`. Sketch 24 §7: "the token budget comes in as a value", under rule 11 (Agents and the narrator do not know storage). The new `Narrator` (sketch 24 §2.2) holds the `PromptAssembler`.

Later effort: `prompt_merge.rs` (7 lines) and the five public constants of `token_budget.rs`.

- `merge_single_user_message` has no caller in prompt assembly. Its callers are the OpenRouter and Ollama providers (`providers/openrouter.rs:35`, `providers/ollama.rs:34`) (known). So its home is the LLM transport question (AD-17), not the assembler.
- The constants change no behaviour, and no file of the build needs them to move.
- Reason for "later": prompt assembly is an edge that the design treats as a black box (target design §10, AD-06 "open (outside)"). The build moves it behind the `Narrator` with no change of logic. A change to the prompt also falls under the LLM-test policy, so a behaviour-neutral build keeps the LLM tests valid (inferred).

### 2.5 AD-11 One generation-status strip — settled

Settled (known):

- **One rule.** Target design §2.3 (status: busy → step name; else failure line; else idle) and §6 (the status line and `GET /status/generating` read that rule). Sketch 25 §3.4: `GameViewQuery::status(game) -> Status` holds it. Sketch 25 §7.1: `endpoints.rs` "the status rule moves to `GameViewQuery::status`".
- **The read path.** The poll moves onto `GameViewQuery`. Today it reads `state.message_service.load_expecting_valid_state()` (`endpoints.rs:47`), and `MessageService` is deleted.
- **The order changes.** Today the poll checks the failure before busy (`endpoints.rs:71-84`); the design checks busy first. The design is the source.

What is left is not a deepening (inferred). After the build, the poll and the action area render one `Status` value. They render it into two different outputs on purpose: the poll answers endpoint strings (`narrating`, `idle`, …) and the action area answers markup. Both are HTTP contracts in `docs/specs/` (`dashboard.md`, `failure_display.md`). The failure branch already shares `error_disclosure`. The deletion test: delete one renderer, and the other does not grow, because it renders a different contract. The stub server (`tests/test_utils/stub_server.rs:876`) is a double of the HTTP contract, so it restates the poll by design.

### 2.6 AD-12 Share one Agent call core — settled in part, later effort for the rest

Settled (known):

- **The history window.** Both Agents take the last 4 entries themselves (`options/utils/orchestration.rs:36`, `quantifier/utils/orchestration.rs:40`). Sketch 24 §2.2: the step loads the window and passes it in `HookInput.story`. Target design §4.6 fixes the window of 4. So the two copies of `.take(4)` go.
- **Settings and storage.** D-33 and rule 11: the step loads each Agent's settings and preset and passes them as values. Today both Agents hold `Option<Arc<Storage>>` through `from_config_with_storage` (`options/agent.rs:21,33`; `quantifier/agent.rs:22,34`). This half was already closed as AD-07.

Later effort: the byte-identical `<CurrentRoom>` and `<RecentHistory>` blocks (`options/prompt.rs:43-84`, `quantifier/prompt.rs:57-98`) and the two-attempt retry loop with two retry rules.

- Reason: they sit inside the Agent edge, which sketch 24 §2.2 leaves undesigned ("Behind each trait sits an Agent's inside: its prompt, its parse, its fallback and its LLM call"). The build moves both Agents onto the hook traits with their prompts unchanged, under the LLM-test policy (inferred).
- Consequence for the build (inferred): ticket 26 R5 and sketch 24 §6 write one seam contract per Agent. With the blocks not shared, each Agent's contract states its own `<CurrentRoom>` and `<RecentHistory>` case. A later effort that shares them merges two cases into one.

### 2.7 AD-14 One definition of the connection card — moot

Known: commit `6916556a` (2026-10-07, "split Settings into Connections and Text Check sub-tabs") deleted `builders/connections.rs` and `builders/connections_tests.rs`. Its message: "Remove … the connections HTML builders with their fixtures and tests". Edit now opens one shared form page that swaps the whole panel. The connection row exists once, in `settings/templates/settings.rs:59` (`<div class="connection-row">`), and `tests/http/settings.rs:74,95` asserts on it. The duplicate that AD-14 named no longer exists. The 2026-10-04 review came three days before that commit.

### 2.8 AD-15 One fragment error policy — settled for the build's routes, later effort for the rest

Settled (known):

- **The generation routes.** Sketch 25 §3.2: `Refusal { StillThinking, BadRequest, MissingData }` maps to `200` / `400` / `500`. Target design §3.3: the busy check comes first, before every `400` or `500` check. Sketch 25 §7.1: `application/errors.rs` loses `ConcurrentGeneration` (`503`); `actions.rs`, `chat_window.rs`, `history.rs` map `Refusal`.
- **The failure lines.** Target design §4.7 fixes the text of each failure kind; the status line shows them through the existing `error_disclosure` (§2.5).

Later effort: the policy for the other handlers. The Validation conflict (`http/error.rs:14` maps `Validation` to `400`; `games/handlers/games.rs:218` and `worlds/handlers/worlds.rs:222` render user-displayable errors as a `200` fragment), the 7 `EngineError::Config` sites and 13 `render_error_context` calls in `app_state.rs`, and the four return conventions.

- Reason: these handlers are outside the designed scope (target design §10, AD-15 "open (outside)"). The build changes `games.rs` only where the switch stops healing (sketch 25 §7.1). Each mapping is an observable contract that needs a spec check, and none of them blocks the build.
- One note for the build (inferred): the `Refusal` mapping gets one home, next to `impl IntoResponse for ApplicationError` in `http/error.rs`, so the build adds no fifth convention. [Decide the build path](../issues/09-decide-build-path.md) carries this note.

### 2.9 AD-16 Bootstrap through GameCatalogue — settled for creation, ticket here for the lookup

Settled (known): sketch 25 §3.5 and the AD-16 comment of 2026-10-14. `init_game` keeps `resolve_game_id` and calls `GameCatalogue::create_game`, so a Game made at boot gets the one create unit. The second creation path (`init_game.rs:39-55`: `bundle_for`, `NewGame`, `insert_game_from_request`) goes.

Ticket here: the lookup. `find_latest_game_for_world` and `list_game_names_for_world` are raw SQL in `bootstrap/run.rs:168-215` (known). The first orders by the newest `messages.timestamp`, else `games.updated_at`. Both run on SQLite only, so startup cannot run on the in-memory backend (known). Three reasons put the lookup in the build plan, not a later effort:

1. The build changes the `messages` table (sketch 25 §3.1: new columns `kind` and the event record), and this SQL reads it.
2. Sketch 25 §3.2 moves the `games.updated_at` touch from `Storage::set_game_id` (`core.rs:155-172`) to `ActiveGame::set`, "because `resolve_game_id` picks the newest by it". So the build already touches this ordering.
3. The design's storage reads all take a `GameId`. "The newest Game of a World" is a read by World, which no design read covers.

Where it lives is a storage-seam question, so it joins [Decide the storage seam of the rebuild](../issues/11-decide-storage-seam-of-rebuild.md).

### 2.10 AD-17 Flatten LLM transport — later effort

Known: no file under `adapters/driven/llm/` is in the fate tables of sketch 25 §7.1 or sketch 24 §7. The `Narrator` calls `LlmCallRecorder`, which calls the `LlmProvider` port, as today. Ticket 26 R6 puts the tier-1 fake at `LlmProvider`, so the transport sits under neither the build nor the new tests. The friction of AD-17 is unchanged: `transport/mod.rs` 8 lines, `transport/utils/mod.rs` 12 lines, the two `complete` bodies (`openrouter.rs` 70 lines, `ollama.rs` 89 lines).

Reason for "later": it touches no file of the build, and the review rated it Speculative. The `prompt_merge.rs` part of AD-06 goes with it, because the providers are its only callers.

## 3. What this sort shows about the edges

The map's fog "The design beyond scope B (the edges)" asks which edges the build must touch. The answer, from the fate tables and §2 (inferred from known fate rows):

| Edge | What the build does to it | Needs a new design? |
|---|---|---|
| Storage | new reads, `Tx`, record writes for both backends; failure injection; the lookup by World | **yes**: tickets 10 and 11 |
| The Agents (Quantifier, Options) | move onto the hook traits; settings and the window come in as values; prompts unchanged | no |
| Triggers and the game rules | about 330 lines move from `GameState` into `apply_game_rules` with no change of logic (sketch 25 §7.2) | no |
| Prompt assembly | moves behind `Narrator`; the token budget comes in as a value; `build_narration_prompt` goes | no |
| HTTP views | the handlers and view reads of sketch 25 §7.1; `Refusal`; the status rule | no |
| LLM transport | untouched | no |

So the fog item graduates into tickets 10 and 11 for storage. The other edges need no design for the build. Their open questions are the later efforts of §2.4, §2.6, §2.8 and §2.10. Whether their tests follow the new test rule is [Decide the test rule outside the designed scope](../issues/08-decide-test-rule-outside-designed-scope.md); this sort gives it the module list above.
