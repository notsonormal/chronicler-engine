# The rebuild at a glance

What the code looks like after the rebuild, and the path to it, on one page. This page holds no decision. Each part links to the ticket or sketch that holds the decision and its reason. [Write the build plan](../issues/14-write-the-build-plan.md) uses this page as the plan's first section.

Labels: **known** (read in a ticket or in the code), **inferred**, **guessed**.

## 1. The code after the rebuild

```text
HTTP handlers
  └─ CurrentGame ............ reads the active-Game pointer once per request
       │
       ├─ job::start(game, request) ...... Action, Retry, Retrigger, Options
       │     GameWriter::accept → a JobPlan (ids only) → one runner
       │     generate: room check → narrator → text save → analysis → game rules → state update
       │     every step goes through run_step, which sets the status name
       │
       ├─ GameWriter / JobWriter  (application/generation/gate.rs)
       │     one guard; the only code that opens a Storage transaction
       │     JobWriter: the only write handle of a running Job; it checks the job id on each save
       │
       ├─ GameViewQuery .......... story log, dock, status, busy
       │
       ├─ Agents at fixed hooks .. Quantifier at Analysis, Options Agent at Options, Narrator
       │     they get values and give back values; they hold no Storage
       │
       ├─ domain ................. each Swipe carries its SwipeState; Tail; the game rules (pure)
       │
       └─ Storage ................ today's Backend enum, same shape
             + reads by GameId: games(), stories(), docks()
             + Tx: one small record write per call; transaction / transaction_new_game
```

| Part | What it is | Decided in |
|---|---|---|
| `CurrentGame`, `ActiveGame` | The one reader of the active-Game pointer. The pointer leaves `Storage`. | [sketch 25](../../turn-state-model/assets/sketch-25-new-design.md) §3, rule 2 |
| `JobPlan` and one runner | `accept` builds the plan as a value: an optional Input save, the first generation, the Options run. One runner in `pipeline/job.rs` holds the order of every request. | [Decide the shape of the Job and the busy state](../issues/05-decide-job-and-busy-state-shape.md) |
| `generate` | One function for every kind of generation. Each step runs through `run_step`. | ticket 05 (gap G1, rule 17) |
| `GameWriter`, `JobWriter` | Today's `GenerationGate`, changed: one guard, one hold per write, a job id check on each save unit. Busy is memory only. | sketch 25 §3.2; [Decide the race tests and guardrail rule 18](../issues/12-decide-race-tests-and-rule-18.md) |
| Agents at hooks | Two hook traits now, `Analysis` and `Options`; `BeforeNarration` and `Rewrite` come with their first Agent (target design D-34). Each Agent gets `HookInput` and its settings as values, and returns a typed value. | [sketch 24](../../turn-state-model/assets/sketch-24-agent-hooks.md) |
| `SwipeState` on the Swipe | The game state that a generation left. It replaces the Snapshot. The current state is that of the last Message's active Swipe. | sketch 25 §3.1 |
| The game rules | `apply_game_rules` in `domain/model/utils/game_rules.rs`: a pure function that changes `SwipeState` and returns an outcome value. About 330 lines move out of `GameState` with no change of logic; `GameState` keeps thin wrappers for the story side effects until the core step. | sketch 25 §7.2; [ticket 17](../issues/17-decide-the-plan-review-gaps.md) Q3, Q4 |
| `Storage` | The `Backend` enum stays. New reads by `GameId`, and a `Tx` for the record writes. The new records exist on SQLite only; the in-memory backend keeps today's kept methods. A typed `TxWrite` key injects a failure. The CRUD services stay. | [Decide the storage seam of the rebuild](../issues/11-decide-storage-seam-of-rebuild.md), amended by [ticket 15](../issues/15-review-design-against-standard-practice.md) Q4 |

## 2. What goes

Snapshot (records, storage, mappers), `GameState`, `NarrativeState`, `MessageService`, `ActionPipeline` with `PipelineRun` and its five entry files, the arrival opening, the `Storage::game_id` atomic, `guard.rs` and `slot.rs`. The full list with the new home of each job is sketch 25 §7.1.

| Measure | Size |
|---|---|
| Non-test lines replaced or deleted | about 3,577 (known) |
| New lines, with the Agent hooks | about 1,845 (guessed) |
| Unit-test lines that go with the deleted modules | about 7,400 (known) |

## 3. What holds the shape in place

These tools add no code shape. They stop the shape from drifting. Detail: [Decide the rebuilt repository's module shape, docs and guardrails for agents](../issues/13-decide-module-shape-docs-and-guardrails.md).

| Tool | What it holds |
|---|---|
| The compiler | The rules that types can carry: only `accept` makes a `JobWriter`, record writes exist only on `Tx`, an Agent returns a value and holds no `Storage`. |
| Four table-driven syn walkers | Banned name in a scope, call-site allowlist, type check, call count. Seven rows in total (rules 1, 2, 6, 7 field half, 13, 18 direct, and the rule-3 token-constructor row), each with its reason. A row whose path matches no file fails. 15 rules in all; rule 3 is a `TxToken` in `storage/` that every transaction takes, and the token-constructor row lets only `gate.rs` and `test_support` build it ([ticket 15](../issues/15-review-design-against-standard-practice.md) Q3; [ticket 16](../issues/16-decide-the-build-plan-gaps.md) Q5; [ticket 17](../issues/17-decide-the-plan-review-gaps.md) Q2). |
| arch-lint | Two new rows: the Agents and the narrator may not import `storage`; `model` may not import `storage`. |
| Tests | The tier-1 scenarios on SQLite `:memory:`, the storage seam contract, and the race tests in `tests/http/races.rs`. A unit test lives inside a seam contract or on a pure `domain/model/` function; legacy unit tests stay where the build does not break them ([ticket 08](../issues/08-decide-test-rule-outside-designed-scope.md), amended by [ticket 15](../issues/15-review-design-against-standard-practice.md) Q2). |
| Docs | `game_flow.md` and `storage.md` are rewritten in place. `architecture_system.md` gets a *who owns what* section. `CONTEXT.md` loses Snapshot. |

## 4. The path to it

[Decide the build path](../issues/09-decide-build-path.md). P0 to P5 each merge green to `main`. The core step is the one large change, on its own branch: the plan names the commit where tier 1 goes red and the commit where it is green again, and the target tests (I2 to I6, the R-59 scenario) are written before the branch opens ([ticket 15](../issues/15-review-design-against-standard-practice.md) Q1). One implementer works at a time.

| Slice | Content |
|---|---|
| P0 | The test rule and the legacy list; target selection in `build.py test-pattern`; the walker machinery |
| P1 | `ActiveGame` and `CurrentGame`; the active-Game pointer leaves `Storage` (about 240 call sites, known) |
| P2 | Delete the arrival opening |
| P3 | `SwipeState` as a field inside today's `GameState`; the game rules move into `apply_game_rules` ([ticket 17](../issues/17-decide-the-plan-review-gaps.md) Q3, Q4) |
| P4 | Agents at hooks on values, the `Narrator`, the strict fake model that a test can hold, the real Quantifier at tier 1, the first busy tests; the value types of `HookInput` ([ticket 16](../issues/16-decide-the-build-plan-gaps.md) Q3; [ticket 17](../issues/17-decide-the-plan-review-gaps.md) Q6) |
| P5 | The storage `Tx` with the Game record writes ([ticket 17](../issues/17-decide-the-plan-review-gaps.md) Q7), `TxWrite`, the two transaction entries, the lookup of the newest Game of a World; tier 1 on SQLite `:memory:`; the target tests of the core step, `#[ignore]`d while red ([ticket 16](../issues/16-decide-the-build-plan-gaps.md) Q1, Q2) |
| Core | The records change; `GameWriter`/`JobWriter` and `job.rs` replace the old pipeline; the view reads; create and reset; the deletions, which also discard today's saved Games (OR-01; [ticket 17](../issues/17-decide-the-plan-review-gaps.md) Q5) |
