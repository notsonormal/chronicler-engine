` block with the corrected "private in-module helpers, not shared abstractions" framing:

<proposed_plan>
# Plan: Nesting Depth Guardrail Rollout (Revised — Private Helpers, Not Shared Abstractions)

## Summary

The nesting-depth guardrail at `tests/infrastructure/guardrails/nesting.rs` is in probe mode and reports **22 violations across 12 files** after the predicate-closure tightening. All 22 are genuine depth-4+ control-flow chains — no false positives.

This plan dissolves them via **private in-module functions**, not shared/cross-module abstractions. The only shared-helper proposal (H2 — Trigger traversal across `domain/` + `bootstrap/`) is **rejected** as false-deduplication: same shape, different semantics. Each file gets its own private helpers; no cross-module surface is added.

End state: zero nesting-depth violations, the rule registered as a real `#[test]` gating the build, and a §3.11 entry in `docs/architecture/guardrails.md`.

Total: ~13 SP across 4 phases (vs the original inflated estimate of 30+).

## Key Changes

- Refactor `messages.rs`, `swipes.rs` — private `_inmemory` / `_sqlite` helpers completing the existing `with_backend_mut` pattern (6 violations dissolved)
- Refactor declarative-rewrite targets: `game.rs`, `validate.rs` — `filter_map` + `flat_map` chains (2 violations, real −9 LOC)
- Refactor per-file extractions: `assembler.rs`, `bootstrap/load.rs`, `bootstrap/run.rs`, `bootstrap/init_game.rs`, `application/game_catalogue/gate.rs`, `adapters/driving/cli.rs`, `adapters/driving/http/port_utils.rs`, `domain/engine/trigger_eval.rs` (14 violations)
- Promote probe → enforcement in `tests/infrastructure/guardrails/mod.rs` (1 SP)
- Add §3.11 to `docs/architecture/guardrails.md` (1 SP)

## Design Principle (corrected from prior planning)

**Private in-module functions: yes. Shared/cross-module helpers: only when 2+ real callers in different files exist.**

The original antipattern-checker review flagged "single-caller helper extracted 'for clarity'" as a Refactor-be-damned smell. That concern applies to **shared helpers across module boundaries** — not to private file-local helpers that complete an existing local pattern. Specifically:

- `messages.rs` already has `with_backend_mut` as a private-in-module helper. Adding `delete_message_inmemory`, `update_active_swipe_inmemory`, etc. as private functions **completes** the existing pattern rather than inventing a new abstraction.
- Per-file private helpers like `fn validate_npc_triggers(npc, valid) -> Vec<String>` in `validate.rs` are fine — they live next to their caller and don't widen the public API.

Rejected shared-helper proposals:
- **H1 (`find_npc_by_id` across files)** — 2 callers in different modules, both are 1-line iterator expressions. Extraction saves 0 LOC, creates API surface. **Skip.**
- **H2 (shared Trigger traversal in `domain/model/trigger.rs`)** — `validate.rs` collects errors; `trigger_eval.rs` skips then runs 3 more unrelated checks. Same shape, different intent. False-deduplication. **Skip; refactor each file independently.**
- **H3 (shared `Storage::inmemory_*`)** — **Reframe**: not a "shared" helper — instead, a set of **private functions local to `messages.rs` / `swipes.rs`**, completing the existing `with_backend_mut` convention. **Apply.**

## Per-Violation Inventory (the 22 sites)

Line numbers are approximate — re-locate at execution time by reading the function. Depths read as `function(0) → construct(1) → ...`.

### Phase 1 target sites — Declarative rewrites (2 violations, 2 SP)

Real LOC reduction, no extraction needed.

| File:Line | Function | Depth | Fix |
|---|---|---|---|
| `domain/model/game.rs:21` | `generate_game_name` | function(0)→for(1)→if let(2)→if let(3)→if(4) | Replace loop with `existing_names.iter().filter_map(...).filter_map(...).max().unwrap_or(0)` — saves ~3 LOC |
| `bootstrap/validate.rs:34` | `validate_loaded_data` | function(0)→for(1)→for(2)→if let(3)→if(4) | Replace double-for with `npcs.iter().flat_map(\|npc\| npc.triggers.iter().enumerate().zip(std::iter::repeat(npc))).filter_map(...)` chain — saves ~6 LOC |

### Phase 2 target sites — Storage backend extractions (6 violations, 5 SP)

Private `_inmemory` helpers in `messages.rs` / `swipes.rs`. Net LOC ≈ 0; depth drops from 4+ to 2.

| File:Line | Function | Depth | Helper to extract (private, same file) |
|---|---|---|---|
| `messages.rs:58` | `delete_message` InMemory arm | closure(1)→match(2)→if(3)→if(4) | `fn delete_message_inmemory(messages: &mut HashMap<u64, Vec<Message>>, id: u64, game_id: u64)` |
| `messages.rs:76` | `load_message_rows` Sqlite arm | closure(1)→match(2)→for(3)→body(4) | `fn load_message_rows_sqlite(conn, game_id) -> Result<Vec<Message>>` (the `.map_err(\|e\| ...)` closure is fine — predicate-only) |
| `messages.rs:131` | `get_active_swipe_index` InMemory arm | closure(1)→match(2)→match(3)→body(4) | `fn get_active_swipe_index_inmemory(messages, id, game_id) -> Option<usize>` |
| `messages.rs:179` | `update_active_swipe` InMemory arm | closure(1)→match(2)→if let(3)→if let(4) | `fn update_active_swipe_inmemory(messages, game_id, message_id, index)`; collapse double-`if let` with `and_then` |
| `messages.rs:206` | `soft_delete_message` InMemory arm | closure(1)→match(2)→if let(3)→if let(4)→body(5) | `fn soft_delete_message_inmemory(messages, id, game_id)` |
| `swipes.rs:45` | `update_swipe_text` InMemory arm | closure(1)→match(2)→if let(3)→if let(4) | `fn update_swipe_text_inmemory(data, ...)`; double-`if let` collapse |
| `swipes.rs:107` | `load_swipes_for_messages` InMemory arm | closure(1)→match(2)→for(3)→if let(4) | `fn load_swipes_for_messages_inmemory(swipes, message_ids) -> HashMap<u64, Vec<Swipe>>`; `filter_map` flattens |

Note: some of these have multiple InnerMemory arms (e.g. `restore_soft_deleted`, `purge_soft_deleted`) — if they violate, extract similarly; if not, leave alone.

### Phase 3 target sites — Per-file private extractions (11 violations, 4 SP)

Each is a private function local to the file. No shared abstractions.

| File:Line | Function | Depth | Helper to extract (private) |
|---|---|---|---|
| `domain/engine/trigger_eval.rs:12` | `evaluate_triggers` | function(0)→for(1)→for(2)→if let(3)→if(4)→continue(5) | Private `fn trigger_should_skip_for_room(trigger, current_room_id) -> bool`; called before the other 3 checks. (NOT shared with validate.rs — different intent.) |
| `application/narrative_prompt/assembler.rs:197` | `render_npc_cards_layer` | function(0)→for(1)→for(2)→body(3) — verify depth; may actually be <4 | `fn render_npc_summary(npc, in_area_ids) -> String` — extract the inner-loop body |
| `application/narrative_prompt/assembler.rs:253` | `render_npc_cards_layer` | function(0)→for(1)→if(2)→for(3)→find-closure(4) | Note: `.find(\|n\| n.id == rel.with)` is a predicate closure, doesn't bump. Likely real depth is 3, not 4. **Re-verify by reading.** If 3, drop this from scope. |
| `bootstrap/load.rs:54` | `seed_worlds` | function(0)→for(1)→if(2)→match(3)→for(4)→if(5) | Private `fn process_world_dir(storage, world_dir, data_dir) -> Result<()>`; the inner character-seeding loop becomes `seed_characters_for_world(...)` |
| `bootstrap/load.rs:70,80` | `seed_worlds` (continuation) | (same function, same fix shape) | (covered by above) |
| `bootstrap/run.rs:195` | `ensure_presets` | function(0)→for(1)→if(2)→if let(3)→if(4) (×2 sites) | Private `fn process_preset_file(storage, path, existing_ids, preset_type) -> Result<bool>` |
| `bootstrap/init_game.rs:62` | `load_game_state` | (singleton — re-verify depth) | Private `fn try_load_initial_message(msg, storage, snapshot_id) -> Result<u64, EngineError>` |
| `bootstrap/init_game.rs:69,80` | `load_game_state` | match(1)→if let(2)→if let(3)→body(4) | Same helper; arm body extraction |
| `application/game_catalogue/gate.rs:115` | `persist_initial_state_with_swipes` | function(0)→if let(1)→match(2)→for(3)→body(4) (×2 sites) | Private `fn persist_swipes_for_message(storage, msg_id, swipes) -> ()` **in gate.rs** — NOT shared with `bootstrap/run.rs` despite the similar shape. False-deduplication concerns don't apply because they're literally in separate private scopes. |
| `adapters/driving/cli.rs:55` | `scan_worlds` | function(0)→for(1)→if(2)→if(3)→if let(4)→if let(5) | Private `fn discover_worlds_in_dir(dir) -> Result<Vec<(String, String)>>`; outer `for` becomes one helper call |
| `adapters/driving/http/port_utils.rs:27` | `bind_with_retry` (async) | function(0)→loop(1)→match(2)→Err(3)→if let(4) | Private `async fn try_bind_and_kill(addr) -> Result<TcpListener, io::Error>`; the `loop`'s `match` arm body becomes the helper call |

### Phase 4 target sites — Verification & docs (3 violations of buffer, 2 SP)

3 violations are accounted for in the buffer (sites that turn out to be false positives under re-reading, or extractions that don't fully dissolve the violation). If the count is exactly 22 as predicted, Phase 4 has no extra refactoring.

## Implementation

### Phase 1: Declarative Rewrites (2 SP)

- [ ] #### Task 1.1: Rewrite `generate_game_name` with `filter_map + max` (1 SP)
  - In `chronicler_engine/src/domain/model/game.rs`, replace the `for-if let-if let-if` chain (~8 LOC) with a single `existing_names.iter().filter_map(strip_prefix).filter_map(parse).max().unwrap_or(0)` chain (~4 LOC)
  - **Validate:** `cargo nextest run --lib -- domain::model::game` passes; `cargo nextest run --test guardrails guardrails_nesting_depth_probe --no-capture` reports **0** in game.rs
- [ ] #### Task 1.2: Rewrite `validate_loaded_data` trigger loop with `flat_map + filter_map` (1 SP)
  - In `chronicler_engine/src/bootstrap/validate.rs`, replace the `for npc → for (i, trigger) → if let Some → if` chain with `npcs.iter().flat_map(|npc| npc.triggers.iter().enumerate().map(move |t| (npc, t))).filter_map(...)`
  - **Validate:** `cargo nextest run --test integration -- bootstrap::validate` passes; probe reports **0** in validate.rs

### Phase 2: Storage Backend Extractions (5 SP)

- [ ] #### Task 2.1: Refactor `messages.rs` (3 SP — 4–5 violations)
  - Add private `fn delete_message_inmemory`, `fn load_message_rows_sqlite`, `fn get_active_swipe_index_inmemory`, `fn update_active_swipe_inmemory`, `fn soft_delete_message_inmemory` at the bottom of the file (alongside the existing `db_message_to_model` convention)
  - `with_backend_mut` arms become `Backend::InMemory(data) => delete_message_inmemory(&mut data.messages, id, game_id)` — depth 2
  - Use `and_then` to collapse double-`if let` patterns where they appear
  - **Validate:** `cargo nextest run --lib -- adapters::driven::storage::backend::messages` passes (or whichever test module covers it); `cargo nextest run --test integration -- storage::messages` passes; probe reports **0** in messages.rs
- [ ] #### Task 2.2: Refactor `swipes.rs` (2 SP — 2 violations)
  - Add private `fn update_swipe_text_inmemory`, `fn load_swipes_for_messages_inmemory`
  - `filter_map` for `load_swipes_for_messages` if the LOC math works out cleanly; otherwise plain `for` loop inside the helper
  - **Validate:** `cargo nextest run --test integration -- swipes` passes; probe reports **0** in swipes.rs

### Phase 3: Per-File Private Extractions (4 SP)

- [ ] #### Task 3.1: Refactor `trigger_eval.rs` (1 SP)
  - Add private `fn trigger_should_skip_for_room(trigger, current_room_id) -> bool` (returns true for mismatch); use `continue` at call site — first of 4 checks
  - **Validate:** `cargo nextest run --lib -- domain::engine::trigger_eval` passes (or `--test integration -- engine`); probe reports **0** in trigger_eval.rs
- [ ] #### Task 3.2: Refactor `bootstrap/load.rs` (1 SP)
  - Add private `fn process_world_dir(storage, world_dir, data_dir) -> Result<()>`; inner character-seeding loop becomes a `for` inside the helper (depth 1, fine)
  - **Validate:** `cargo nextest run --test integration -- bootstrap` passes; probe reports **0** in load.rs
- [ ] #### Task 3.3: Refactor `bootstrap/run.rs` + `bootstrap/init_game.rs` + `game_catalogue/gate.rs` (2 SP)
  - `run.rs`: private `fn process_preset_file(storage, path, existing_ids, preset_type) -> Result<bool>` — dissolves 2 violations
  - `init_game.rs`: private `fn try_load_initial_message(msg, storage, snapshot_id) -> Result<u64, EngineError>` — dissolves 2 violations
  - `gate.rs`: private `fn persist_swipes_for_message(storage, msg_id, swipes) -> ()` — dissolves 2 violations. **NOT** shared with `run.rs` (different files, different scopes, same shape is OK).
  - **Validate:** `cargo nextest run --test integration` passes; probe reports **0** in all three files
- [ ] #### Task 3.4: Refactor `adapters/driving/cli.rs` + `adapters/driving/http/port_utils.rs` (1 SP)
  - `cli.rs`: private `fn discover_worlds_in_dir(dir) -> Result<Vec<(String, String)>>`; the outer `for` becomes a single helper call returning a Vec
  - `port_utils.rs`: private `async fn try_bind_and_kill(addr) -> Result<TcpListener, io::Error>`; the retry loop becomes one match arm calling the helper
  - **Validate:** `cargo nextest run --test integration -- http` passes; probe reports **0** in both files
- [ ] #### Task 3.5: Verify `assembler.rs` sites (1 SP)
  - The 2 reported violations at `assembler.rs:197, 253` may be at depth 3, not 4 (predicate closures don't bump; the `.find(|n| n.id == rel.with)` at line 253 is predicate-only).
  - Re-read; if depth < 4, **leave alone** (false positive under tightened rule; the probe should have filtered them out — investigate why it didn't).
  - If genuine, extract `fn render_npc_summary(npc, in_area_ids) -> String` and `fn render_relationships(npc, all_npcs) -> String`
  - **Validate:** probe output matches expectation; if extracted, `cargo nextest run --test integration -- narrative_prompt` passes

### Phase 4: Enforcement + Docs (3 SP)

- [ ] #### Task 4.1: Promote probe to enforcement (1 SP)
  - Edit `chronicler_engine/tests/infrastructure/guardrails/mod.rs`:
    - Delete `#[test] fn guardrails_nesting_depth_probe()`
    - Add `#[test] fn guardrails_nesting_depth_src() { check_src_files("nesting depth (src)", check_nesting_depth); }`
  - **Validate:** `cargo nextest run --test guardrails` — **all tests pass** (including the new enforcement test). If any violations remain, fix-or-grandfather decision required.
- [ ] #### Task 4.2: Add §3.11 to `docs/architecture/guardrails.md` (1 SP)
  - New section following the §3.10 pattern:
    ```markdown
    ### 3.11 Nesting Depth (`guardrails_nesting_depth_src`)

    **Standard**: Function-body control-flow nesting depth must not exceed 3 (`MAX_NESTING_DEPTH = 3`). Depth 0 = function body; depth 4 = violation.

    **Counting constructs**: `if`/`if let`, `match`, `for`/`while`/`loop`, closure with control flow, async block with control flow.

    **Non-counting constructs**: `ExprBlock` (regular `{}` scoping), `?` operator, `try {}` blocks, macros. **Predicate-only closures** (`.retain(|m| ...)`, `.map_err(|e| ...)`, `.filter(|x| ...)`) do NOT bump depth — only closures/async blocks whose body contains direct control flow (`if`/`match`/`for`/`while`/`loop`) bump.

    **Severity**: error
    **Scope**: `src/`
    **Checks**: `syn::visit::Visit` walk of every `ItemFn` / `ImplItemFn` / `ExprClosure` / `ExprAsync`; per-construct depth increment via `NestingVisitor::enter`; `ControlFlowDetector` filters predicate-only closures/async.
    **Exemptions**: None (as of plan t12).
    ```
  - Update §3 intro paragraph: count "22 registered conventions" (was 21)
  - **Validate:** manual check — section renders; intro count updated
- [ ] #### Task 4.3: Full `python build.py` validation (1 SP)
  - `cd chronicler_engine && python build.py`
  - **Validate:** all gates green — fmt, clippy, arch-lint, guardrails (47+ tests), integration tests
- [ ] #### Task 4.4: Archive plan + CHANGELOG (1 SP)
  - Move `docs/plans/t12-nesting-depth-guardrail-rollout.md` → `old-docs/archived-plans/`
  - Add entry to `docs/CHANGELOG.md`: "t12: tightened nesting-depth guardrail (predicate closures excluded); refactored 22 violations across 12 files via private in-module helpers (no shared abstractions)"

## Test Plan

1. **Probe-based verification** (after every refactor task):
   - `cargo nextest run --test guardrails guardrails_nesting_depth_probe --no-capture` — count must drop as predicted
2. **Code-level tests** (after every refactor task):
   - Run the relevant `*_tests.rs` sibling or `--test integration -- <module>` to confirm no behavioral regression
3. **Full gate** (Phase 4.3):
   - `python build.py` from `chronicler_engine/` — must pass fmt + clippy + arch-lint + all guardrails + integration

## Per Task / Sub Task Validation Steps

| Task | Validation command | Pass criterion |
|------|--------------------|----------------|
| 1.1 | probe + `cargo nextest run --lib -- domain::model::game` | game.rs count = **0**; tests green |
| 1.2 | probe + `cargo nextest run --test integration -- bootstrap::validate` | validate.rs count = **0** |
| 2.1 | probe + `cargo nextest run --test integration -- storage::messages` | messages.rs count = **0**; sibling tests green |
| 2.2 | probe + `cargo nextest run --test integration -- swipes` | swipes.rs count = **0** |
| 3.1 | probe + `cargo nextest run --lib -- domain::engine::trigger_eval` | trigger_eval.rs count = **0** |
| 3.2 | probe + `cargo nextest run --test integration -- bootstrap` | load.rs count = **0** |
| 3.3 | probe + `cargo nextest run --test integration` | run.rs, init_game.rs, gate.rs counts all **0**; full integration suite green |
| 3.4 | probe + `cargo nextest run --test integration -- http` | cli.rs, port_utils.rs counts = **0** |
| 3.5 | probe | assembler.rs count = **0** (or documented false-positive resolution) |
| 4.1 | `cargo nextest run --test guardrails` | All tests pass including new enforcement test |
| 4.2 | (manual) | §3.11 added to `docs/architecture/guardrails.md`; intro count updated to 22 |
| 4.3 | `python build.py` | All gates green |
| 4.4 | (manual) | Plan archived; CHANGELOG updated |

## Assumptions

1. **Probe count is authoritative.** 22 violations as the target. If 1–2 sites turn out to be tightened-rule false positives (e.g. `assembler.rs:253` where `.find(|n| ...)` is predicate-only), Phase 3.5 documents them and skips.
2. **Private in-module functions only.** No shared/cross-module helper abstractions are introduced. Each refactor adds file-local helpers next to existing private helpers like `db_message_to_model`, `db_game_to_game`.
3. **No behavioral changes.** Refactors are pure code-shape changes. All existing integration tests continue to pass without modification.
4. **`Cargo.toml` unchanged.** No new dependencies; `syn`/`walkdir`/`quote` already dev-deps.
5. **Loan/borrow checker concerns are localized** to storage backend extractions (Phase 2). Pattern: helper takes `&mut HashMap<u64, Vec<Message>>` directly, not `&mut InMemoryData`, so the match arm can borrow `data.messages` cleanly.
6. **Story points are tight** — Phase 2 (`messages.rs` = 3 SP) is the largest single task; well under the 8 SP break-up threshold.
7. **Guardrail code is already in place**; the file content above is for recreate-on-revert purposes only. No code changes to `nesting.rs` in this plan.
8. **Phase 4 enforcement has no grandfather list.** If any violations remain after Phase 3, fix them or revert the promotion — do NOT add grandfathered exemptions.

## Open Questions

1. **`assembler.rs` depth re-verification** (Task 3.5): Are lines 197/253 actually depth-4 under the tightened rule, or did the probe report them under the old rule and they're now false positives? Resolve at execution time. If genuine, simple helper extraction; if false-positive, document why the probe is over-counting.
2. **Should the rule eventually apply to `tests/`?** Currently the probe (and enforcement) scans `src/` only. Future decision; not in scope here.
3. **Should `MAX_NESTING_DEPTH` ever drop to 2?** Phase 2 leaves storage call sites at depth-2 (closure + match). Lowering the threshold to 2 would force the trait-dispatch refactor (Alternative A from the prior planning session). Out of scope for this plan; flag for future consideration if rule proves too permissive.
