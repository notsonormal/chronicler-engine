# Decide the rebuilt repository's module shape, docs and guardrails for agents

Type: grilling
Status: resolved
Blocked by: —

## Question

The rebuild replaces about 3,577 non-test lines and adds about 1,765 across the six areas of
[target design version 2](../../turn-state-model/assets/target-design.md), and it brings the rules
of [sketch 25](../../turn-state-model/assets/sketch-25-new-design.md) §6.2 with it. The crate stays
one crate and its edges are decided. Which docs, which guardrails and which module-shape conventions
must the rebuilt repository have so that an agent can find its way and change it safely — and what
happens to each of today's docs and walkers in the build?

Answer these, in this order:

1. **Guardrails.** Which of §6.2's sixteen rules does the build enforce, with which tool, and what
   does each tool cost? The sketch groups them into three generic walkers (call site, type check,
   call count, about 350 lines with their tests), plus the rules that stay at compiler level and the
   arch-lint scopes. Rule 16 of [sketch 24](../../turn-state-model/assets/sketch-24-agent-hooks.md)
   is in the same set. Two of today's path-limited walkers check no file at all.
2. **Docs.** What happens to `AGENTS.md` (with its generated structure index), `tests/AGENTS.md`,
   `CODING_STANDARDS.md`, `CONTEXT.md`, `docs/AGENTS.md` and the specs in `docs/specs/`: rewritten,
   extended, or dropped? Which of them does the build keep in step automatically, and how?
3. **Module shape for agents.** What must the rebuilt module shape provide for an agent (module
   headers, one type per file, generated indexes, an ownership trail) — and is any of that a rule,
   a doc convention, or neither?

## Context

- **The fog this graduates from.** The map's "Not yet specified" held: "What an AI-native project
  needs from its module shape, its docs (`AGENTS.md`, `CODING_STANDARDS.md`, the specs) and its
  guardrails after the rebuild. It hangs on the crate layout (ticket 07) and the edges." Both are
  now decided: [Prototype: decide the crate split and the dev-build loop](07-prototype-crate-split-and-dev-build-loop.md)
  rules out a crate split, and [Decide the storage seam of the rebuild](11-decide-storage-seam-of-rebuild.md)
  fixes the `application → storage` direction.
- **Origin.** The owner's question "what an AI-native project needs" from
  [Decide whether and how to run a clean-room design of the generation model](../../turn-state-model/issues/17-decide-clean-room-run.md).
- **Already decided, do not reopen here:** the one unit-test rule and where a seam contract may live
  ([Decide the test rule outside the designed scope](08-decide-test-rule-outside-designed-scope.md));
  the crate layout (ticket 07); the storage seam (ticket 11); the rules that exist only as
  consequences of the design.
- **Today's surface (known).** `arch-lint.toml` (`root = "./src"`, one scope per path prefix);
  `tests/infrastructure/guardrails/` (enums, free-fn location, inherent-impl locality, layers,
  location, nesting, structure, style); `scripts/check_test_structure.py`,
  `scripts/generate_structure_index.py`, `scripts/generate_tests_structure_index.py`,
  `scripts/generate_guardrails_doc.py`, `scripts/validate_docs.py`,
  `scripts/validate_feature_spec.py`; the generated blocks in `AGENTS.md`, `tests/AGENTS.md` and
  `docs/AGENTS.md`.
- **Known defect nearby.** Two path-limited walkers check no file
  ([Make the two path-limited guardrails check real files](../../retro-issue-bucket/issues/50-dead-path-guardrails.md)).
- **Skills:** `/codebase-design` for the module shape; `/writing-for-agents` for the docs.

## Done when

- Each of the three answers above, with the tool or doc that carries it.
- A list of the docs and walkers the build adds, rewrites or deletes, and what each one enforces.
- The owner decides what the build plan takes.

## Comments

**2026-10-16, from [Decide the build path](09-decide-build-path.md) (resolved).** Each guardrail rule lands in the slice that creates the code it guards (ticket 09 Q10), and each doc change lands in the slice that makes it true (Q12). The slices are P0 the test rule, P1 `ActiveGame`/`CurrentGame`, P2 delete the arrival opening, P3 `SwipeState` with `apply_world_rules`, P4 the Agent edge on values, P5 the storage `Tx` mechanism, then the core step. So the answer here should name, for each rule or doc it keeps, the slice where it lands. [Write the build plan](14-write-the-build-plan.md) waits on this ticket.

**2026-10-17, from [Decide the race tests and guardrail rule 18](12-decide-race-tests-and-rule-18.md) (resolved).** The rule set is now 18 rules: sketch 25 §6.2 rules 1 to 15, sketch 24 rule 16, ticket 05 rule 17, and rule 18. Rule 18 adds one row to the call-count walker of rule 8, and that row counts indirect holds: it first collects the `gate.rs` functions that call `hold`, then allows each function at most one call to `hold` or to a collected function (about 20 lines more than a direct count, guessed). Rule 18 lands in the core step. This ticket owns the walker's tool and cost.

## Answer

Resolved 2026-10-17 by grilling with the owner (three rounds). The rebuilt repository enforces **all 18 rules** (sketch 25 §6.2 rules 1 to 15, sketch 24 rule 16, ticket 05 rule 17, ticket 12 rule 18) with four table-driven syn walkers, two arch-lint changes, the compiler rows, and the behaviour tests that ticket 12 lists. The machinery lands in P0; each row lands with the code it guards (ticket 09 Q10).

### 1. The rules and the tool that carries each one

Slice column: P0 the test rule, P1 `ActiveGame`/`CurrentGame`, P3 `SwipeState` with `apply_world_rules`, P4 the Agent edge on values, P5 the storage `Tx` mechanism, **core** the core step.

| # | Rule (short) | Tool | Row or scope the build writes | Lands in |
|---|---|---|---|---|
| 1 | Every read and write names its Game | compiler + syn | banned-name kind: no `static` and no `Atomic*` in `storage/`. The `Storage::game_id` field (`storage/core.rs:17`) goes with `ActiveGame` | P1 |
| 2 | The active-Game pointer has one reader | syn | call-site kind: only `current_game.rs` and `GameCatalogue` call `ActiveGame` | P1 |
| 3 | Only `GameWriter` writes game records | syn | call-site kind: only `generation/gate.rs` calls `Storage::transaction` and `Storage::transaction_new_game`. **Ticket 11's names, not the sketch's `Storage::create`** | core. P5 writes the two entries, but in P5 today's Game writes call them from outside `gate.rs`, so the row would be red there (inferred; ticket 09 Q10 used "rule 3 in P5" as an example before ticket 11 R7 fixed the callers) |
| 4 | A running Job writes only through its `JobWriter` | compiler | private fields; only `accept` makes one; each method checks the job id | core |
| 5 | Edit, delete and swipe switch only while idle | compiler + test | `GameWriter` methods that check busy under the guard; the HTTP test is I6 of ticket 12 | core |
| 6 | The guard never spans an LLM call | compiler + syn | banned-name kind: `gate.rs` names no `Narrator` and no hook trait (inferred: sketch 25 named `LlmRoles`, which sketch 24 replaced) | core |
| 7 | Between steps, pass ids | syn | type-check kind on `pipeline/job.rs`: no field and no parameter of type `Game`, `Message`, `SwipeState`, `Tail`, `DockRecord`, `WorldBundle` | core |
| 8 | A step loads late | syn | call-count kind: each function in `pipeline/job.rs` makes at most one `Narrator` call or one hook call (sketch 24 changed the shape) | core |
| 9 | A save writes only what changed | compiler | `Tx` write methods take one small record each; no `GameState` type | P5 |
| 10 | No carry fields | syn | type-check kind: no `#[serde(skip)]` field on a `domain/model/` type. Today's two live in `narrative_state.rs`, which P3 keeps (`SwipeState` sits inside `GameState` there) and the core step deletes | core |
| 11 | Agents and the narrator get values and do not know storage | arch-lint + compiler | a scope for `application/agents/**` and the narration module, with `storage` on its deny list; `HookInput` and `AgentSettings` carry no `GameId`. No allowlist: sketch 24 and research 04 already moved the token budget and the Agent settings to values | P4 |
| 12 | The game rules are pure | arch-lint | add `model` → `storage` to the deny list (one block; `domain/` imports no `storage` today, so it is green at once) | P3 |
| 13 | One generation path | syn + test | call-site kind: only the step functions of `pipeline/job.rs` call `Narrator::narrate` and the hook methods; a test checks the save units of each kind in order | core |
| 14 | A save unit lands whole | compiler + test | record writes exist only on `Tx`; a storage test: a transaction that fails half-way leaves no row | P5 |
| 15 | The status has one writer per part | compiler | `JobEntry` fields are private to `gate.rs`; the failure line changes only through save units | core |
| 16 | An Agent writes no game record | compiler | each hook trait returns a value; no Agent holds `Storage` or `JobWriter` | P4 |
| 17 | Every step runs through the wrapper | syn | call-site kind: the step functions may be called only from `run_step` | core |
| 18 | One hold per `gate.rs` function | syn | call-count kind, row two: collect the `gate.rs` functions that call `hold`, then allow each function at most one call to `hold` or to a collected function | core |

Rules 5, 13 and 14 also keep the behaviour tests that ticket 12 and the plan name; those tests are not walkers.

### 2. The walkers

Four **table-driven walkers**, one per kind, each configured by a table with one row per rule and the row's reason in a comment (sketch 25 §6.2).

| Kind | Rows | Today's precedent |
|---|---|---|
| Banned name in a scope | 1, 6 | `no std thread`, `http storage leak` |
| Call-site allowlist | 2, 3, 13, 17 | `WiredApp` scope |
| Type check (field or parameter types in a folder) | 7, 10 | `form fields urlencoded safe` (field types on form structs) |
| Call count (calls to a named API per function in a folder) | 8, 18 | none — new |

- **Today's 23 rules do not migrate** (Q3, option A). The new rows use the new walkers; the existing rules stay as `check_*` functions.
- **The detection test** is one table-driven test per kind that exercises every row of that kind, not one hand-written test per rule. `registry_tests.rs` assumes the old shape, so the plan names its rework.
- **A dead path is a failure.** The harness asserts that each path-limited row and walker matches at least one file under its root. This is the sketch's rule, and it turns rule rot into a red build: a rule whose target code disappears must have its fate named in the slice that deletes the code.
- **`guardrails.md` generation** (Q1 of the last round): the rows stay in Rust as a table literal with a strict, documented form. `generate_guardrails_doc.py` parses that one form instead of `pub fn check_*` docs; a row without its reason fails `--check`, as a missing comment does today. No new file form and no new dependency.
- **Cost** (guessed, sketch 25 §6.2): about 350 lines for the four walkers with their tests, plus the generator change. The machinery lands in **P0**, with the test-rule change and the `build.py` target selection.

### 3. The fate of today's guardrail set

`mod.rs` registers 28 test entries over 23 distinct rules.

| Fate | Rules |
|---|---|
| **Keep (20)** | import ordering, doc standards, single-letter vars, template raw strings, form fields urlencoded, separator comments, long comment runs, mod purity, no std thread, file length, `WiredApp` scope, test module header, test file location, HTTP storage leak, enum variant docs, nesting depth, inherent impl locality, free fn location, empty rust file, htmx settle |
| **Re-point (2)** | `handler return type` (its path `src/server/` never matched, because `check_src_files` strips the `src/` prefix) and `server layer boundaries` (dead path `server/`), both to `adapters/driving/http/`, in the slice that rewrites those handlers. The banned type of the second becomes **`SwipeState`** |
| **Retire (1)** | `messages swipes separation` — `messages.rs` and `swipes.rs` become `stories.rs`, so its path dies |

Arch-lint: the roles and Agents scope of rule 11, and the `model` → `storage` deny row of rule 12. Everything else in `arch-lint.toml` stays.

**This answers [Make the two path-limited guardrails check real files](../../retro-issue-bucket/issues/50-dead-path-guardrails.md)** — option C with A — and the build delivers it. That ticket's status is the owner's to change; its question is decided here.

### 4. The docs

The three-layer model stays: the code and its unit tests, the integration tests and the specs, and the Diátaxis docs. Every doc change lands in the slice that makes it true (ticket 09 Q12).

| Doc | Fate |
|---|---|
| `AGENTS.md`, `tests/AGENTS.md` | Keep. The generated blocks regenerate; the hand-written prose changes in the slice that makes it wrong |
| `docs/AGENTS.md` | Keep. `Where a fact lives` still holds |
| `CODING_STANDARDS.md` | Keep. It gains only the rules no walker can carry |
| `CONTEXT.md` | Change in the build: `Snapshot` goes, and the new terms enter |
| `docs/specs/*` | Follow the behaviour: delete the scenarios whose behaviour dies, add scenarios for the new observable behaviour. `validate_feature_spec.py` forces the pairing |
| `docs/diataxis/reference/game_flow.md` | Rewrite **in place**, no split — a split would re-anchor 53 files, and the doc carries the area's invariants |
| `docs/diataxis/reference/storage.md` | Rewrite in place (37 anchors) |
| `docs/diataxis/reference/architecture_system.md` | Rewrite, and carry the ownership trail |
| `docs/diataxis/reference/startup.md` | Edit: `init_game` loses the arrival opening |
| `docs/diataxis/reference/narrative/narration_system.md` | Edit: the arrival narration goes |
| `docs/diataxis/explanation/two-state-channels.md` | Rewrite: both status channels change |
| `docs/diataxis/explanation/storage_design.md` | Rewrite: Snapshot persistence goes |
| `docs/diataxis/reference/coding_standards/guardrails.md` | Regenerates, from the new table form |

**The DOC anchor rule stays**: line 1 `//! [DOC: docs/diataxis/reference/<area>/<name>.md]`, line 2 the summary. 212 files carry an anchor against 12 distinct targets, so the rebuild's new modules point at the rewritten area docs, not at one new doc per module. No new reference doc is added.

### 5. The module shape for agents

| Item | Rule, doc convention, or neither |
|---|---|
| Module header (DOC anchor + summary) | **Rule** — `check_doc_standards`, today a warning |
| One type per file | **Neither, beyond today's rule** (option A). `inherent impl locality` stays: an inherent impl lives in the type's defining file. A stricter rule would split `gate.rs` (`GameWriter`, `JobWriter`) and `job.rs` against the decided design. The convention is stated in the reference doc |
| Generated indexes (`AGENTS.md`, `tests/AGENTS.md`, `docs/AGENTS.md`, `guardrails.md`) | **Kept mechanism**: the generators plus the pre-commit hook, checked by `build.py` |
| Ownership trail | **Doc convention**: a *who owns what* section in `architecture_system.md` — for each part, what it owns and what it may write. Not a module → decision map, which would duplicate the generated structure index and rot; and not a code comment, which `CODING_STANDARDS.md` forbids for task references |

### 6. Notes for the build plan (ticket 14)

- The walker machinery, the generator change, the dead-path assertion, and the rework of `registry_tests.rs` are **P0** work. Every row lands with the code it guards.
- The row of rule 3 names `Storage::transaction` and `Storage::transaction_new_game` (ticket 11), not the sketch's `Storage::create`.
- The re-point of the two dead walkers belongs to the slice that rewrites the HTTP handlers, and it retires the `TODO` at `layers.rs:91`.
- `docs/specs/` and `tests/` rework stays ticket 08's rule and ticket 12's list; this answer adds no test requirement of its own.
- No `prototype` ticket measures the walkers: the cost guess rides in the plan, and P0's own gate shows the real number.


**2026-10-17, amended by [Review: the design and the build decisions against standard engineering practice](15-review-design-against-standard-practice.md).** The owner's rulings there change parts of this answer (see its Comments: Q2 for ticket 08, Q4 for ticket 11, Q3 for ticket 13). Where the two conflict, ticket 15 wins.
