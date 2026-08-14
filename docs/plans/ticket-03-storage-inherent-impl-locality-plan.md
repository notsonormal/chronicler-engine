# Ticket 03 — Refactor Storage and InMemoryData to module-per-type

## Summary

Make `Storage` and `InMemoryData` inherent impls satisfy the `guardrails_inherent_impl_locality` rule (ticket 01, deferred) by relocating impls so each type's impls live where the rule permits. No logic changes — pure relocation + visibility widening + one method-to-free-fn conversion. `build.py` stays green at every phase.

The inherent-impl-locality rule is **not active** during this ticket (ticket 01 is re-added only after 03–08 land, per map strategy). Verification here is structural reasoning + the existing guardrail suite green. Final confirmation (zero Storage/InMemoryData violations) is deferred to ticket 01's re-addition.

**Locked decisions (plan review):**
- **Flatten, not rename.** Move the 11 entity files + `core.rs` + `test_support.rs` + 11 `*_tests.rs` up from `storage/backend/` into `storage/` directly; delete `backend/`. Every `impl Storage` file's parent dir is then `storage` (ends with `/storage`) → folder exemption applies. No `storage/storage/` doubling. Cost: `storage/` gains ~16 direct entries and the "backend ops" grouping dissolves into the flat dir — accepted.
- **`seed_game_data` → free fn.** `bootstrap/load.rs` `impl Storage { seed_game_data }` is a layering violation (bootstrap defining Storage behavior; seeding reads JSON + delegates to `storage.seed_world/character/persona`, i.e. orchestration, not storage). Converted to `pub(crate) fn seed_game_data(storage: &Storage, ...)` in `bootstrap/`. Root-cause fix; 7 call sites change signature.

## Key Changes

1. **`InMemoryData` → flat file `storage/in_memory_data.rs`.** Move `struct InMemoryData` + helper structs (`InMemoryWorld`, `PersonaCardWithKey`, `CharacterSeed`) + all `impl InMemoryData` blocks (from `backend/core.rs`, `backend/messages.rs`, `backend/swipes.rs`) into one file. Def path == impl path → rule-clean without a folder exemption. The 5 private helper methods become `pub(crate)` (cross-file callers in the Storage impls).
2. **`bootstrap/load.rs`: convert `impl Storage { seed_game_data }` → `pub(crate) fn seed_game_data(storage: &Storage, data_dir)`.** Body unchanged except `self` → `storage` in two internal delegations. As a free fn in `bootstrap/` it is `guardrails_free_fn_location`-compliant (parent folder `bootstrap` is allowlisted) and the `impl Storage` violation disappears. 7 call sites updated (1 in `run.rs`, 6 in `load_tests.rs`).
3. **Flatten `storage/backend/` into `storage/`.** `git mv` 11 production files (`core.rs`, `characters.rs`, `games.rs`, `llm_messages.rs`, `messages.rs`, `personas.rs`, `presets.rs`, `settings.rs`, `snapshots.rs`, `swipes.rs`, `worlds.rs`) + `test_support.rs` + 11 `*_tests.rs` up one level. Merge `backend/mod.rs` declarations into `storage/mod.rs`; delete `backend/`. Def stays in `storage/core.rs` (not `mod.rs` — `guardrails_mod_purity` forbids struct defs in mod.rs). The `impl Db*` blocks in 6 files ride along untouched (ticket 06's job; still violations, but the rule isn't active yet).
4. **Import path churn:** all `storage::backend::X` → `storage::X` (top re-export handles `Backend`, `Storage`, `InMemoryData`, `InMemoryWorld`, `PersonaCardWithKey`, `CharacterSeed`, `TestOverride`). ~22 src files + `storage/mod.rs` + 2 integration-test files.
5. **Guardrail path-pin update:** `layers.rs:56` `storage/backend/messages.rs` → `storage/messages.rs` (keeps `check_messages_swipes_separation` firing at the new path).
6. **Live doc update:** `docs/diataxis/reference/coding_standards/unit_test_standards.md` (2 `storage::backend::` import examples, lines 32 + 218). Historical docs (CHANGELOG, old plans, issues, auto-generated test-police inventory) left as-is.

## Implementation

### Phase 1: Extract InMemoryData to a flat file (backend/ still named `backend`)

- [ ] #### Task 1.1: Create `src/adapters/driven/storage/in_memory_data.rs` (3 SP)
  - [ ] ##### SubTask 1.1.1: Create the file with the AGENTS.md-mandated two-line header: `//! [DOC: docs/diataxis/reference/storage.md]` + `//! In-memory backend data structures and their inherent impls`. Then move `struct InMemoryData` + `struct InMemoryWorld` + `struct PersonaCardWithKey` + `struct CharacterSeed` defs from `backend/core.rs` into it. (1 SP)
  - [ ] ##### SubTask 1.1.2: Move `impl InMemoryData { empty() }` from `backend/core.rs` into `in_memory_data.rs` (already `pub(crate)`, unchanged). (1 SP)
  - [ ] ##### SubTask 1.1.3: Move the two `impl InMemoryData` blocks from `backend/messages.rs` (3 methods: `update_active_swipe`, `soft_delete_message`, `restore_soft_deleted`) and `backend/swipes.rs` (2 methods: `update_swipe_text`, `load_swipes_for_messages`) into `in_memory_data.rs`. Widen all 5 methods from private `fn` → `pub(crate) fn` (they are called cross-file by `impl Storage` methods after the split). (1 SP)
- [ ] #### Task 1.2: Rewire imports for the InMemoryData extraction (2 SP)
  - [ ] ##### SubTask 1.2.1: `storage/mod.rs` — add `pub mod in_memory_data;` + `pub use in_memory_data::*;`. (1 SP)
  - [ ] ##### SubTask 1.2.2: `backend/core.rs` — remove the 4 moved struct defs + the `impl InMemoryData` block; add `use crate::adapters::driven::storage::in_memory_data::{InMemoryData, InMemoryWorld, PersonaCardWithKey, CharacterSeed};` (needed for `Backend::InMemory(Box<InMemoryData>)` and the `empty()` call). (1 SP)
  - [ ] ##### SubTask 1.2.3: `backend/messages.rs` + `backend/swipes.rs` — remove their `impl InMemoryData` blocks; add `use crate::adapters::driven::storage::in_memory_data::InMemoryData;`. (1 SP)
  - [ ] ##### SubTask 1.2.4: `backend/worlds.rs` (imports `InMemoryWorld`), `backend/characters.rs` (`CharacterSeed`), `backend/personas.rs` (`PersonaCardWithKey`) — repoint those imports to `storage::in_memory_data::` (or the top re-export `storage::`). (1 SP)
- [ ] #### Task 1.3: Verify Phase 1 green (1 SP)
  - `cargo fmt && cargo clippy --all-targets -- -D warnings && cargo test --lib` then `cargo nextest run --test guardrails`. All green.

### Phase 2: Convert `seed_game_data` from `impl Storage` to a free fn

- [ ] #### Task 2.1: Rewrite `bootstrap/load.rs` (1 SP)
  - [ ] ##### SubTask 2.1.1: Delete the `impl Storage { pub fn seed_game_data(&self, data_dir) }` block. Replace with `pub(crate) fn seed_game_data(storage: &Storage, data_dir: &std::path::Path) -> crate::error::Result<()>`. Body unchanged except `self` → `storage` in the two internal calls (`seed_worlds(self, …)` → `seed_worlds(storage, …)`, `seed_personas(self, …)` → `seed_personas(storage, …)`). The existing helper free fns (`read_json_file`, `seed_worlds`, `process_world_dir`, `seed_personas`) already take `storage: &Storage` — untouched. (1 SP)
- [ ] #### Task 2.2: Update call sites (1 SP)
  - [ ] ##### SubTask 2.2.1: `bootstrap/run.rs:65` — `storage.seed_game_data(&data_dir)` → `seed_game_data(&storage, &data_dir)`; add `use crate::bootstrap::load::seed_game_data;` (or qualify). (1 SP)
  - [ ] ##### SubTask 2.2.2: `bootstrap/load_tests.rs` — 6 call sites (lines 7, 23, 91, 93, 126, 141): `storage.seed_game_data(path)` → `seed_game_data(&storage, path)`; add the import. (1 SP)
- [ ] #### Task 2.3: Verify Phase 2 green (1 SP)
  - `cargo fmt && cargo clippy --all-targets -- -D warnings && cargo test --lib` then `cargo nextest run --test guardrails`.

### Phase 3: Flatten `storage/backend/` into `storage/`

- [ ] #### Task 3.1: Move files up + merge mod.rs (2 SP)
  - [ ] ##### SubTask 3.1.1: `git mv` the 11 production files (`core.rs`, `characters.rs`, `games.rs`, `llm_messages.rs`, `messages.rs`, `personas.rs`, `presets.rs`, `settings.rs`, `snapshots.rs`, `swipes.rs`, `worlds.rs`) + `test_support.rs` from `storage/backend/` to `storage/`. (1 SP)
  - [ ] ##### SubTask 3.1.2: `git mv` the 11 `*_tests.rs` files up to `storage/`. (1 SP)
  - [ ] ##### SubTask 3.1.3: Merge `backend/mod.rs` content into `storage/mod.rs`: the 11 `pub mod` declarations, `pub use core::*;`, `#[cfg(feature = "testing")] pub use test_support::{TestFailureHandle, TestOverride};`, `#[cfg(feature = "testing")] mod test_support;`, and the 11 `#[cfg(test)] mod X_tests;` lines. Remove `pub mod backend;` + `pub use backend::*;` from `storage/mod.rs`. Delete `backend/mod.rs` and the now-empty `backend/` dir. (1 SP)
- [ ] #### Task 3.2: Rewrite import paths in moved files (2 SP)
  - [ ] ##### SubTask 3.2.1: 11 production files — `use crate::adapters::driven::storage::backend::{...}` → `use crate::adapters::driven::storage::{...}` (top re-export). Intra-module `super::test_support` refs in `core.rs` stay valid (test_support moved up alongside, still resolvable as `super::test_support`). (1 SP)
  - [ ] ##### SubTask 3.2.2: 11 `*_tests.rs` files — `use crate::adapters::driven::storage::backend::{Storage, TestOverride}` → `use crate::adapters::driven::storage::{Storage, TestOverride}`. (1 SP)
- [ ] #### Task 3.3: Update external import paths (1 SP)
  - [ ] ##### SubTask 3.3.1: `tests/http/prompt_presets.rs:10` + `tests/http/settings.rs:10` — `chronicler_engine::adapters::driven::storage::backend::TestOverride` → `chronicler_engine::adapters::driven::storage::TestOverride`. (1 SP)
- [ ] #### Task 3.4: Update path-pinned guardrail + live doc (1 SP)
  - [ ] ##### SubTask 3.4.1: `tests/infrastructure/guardrails/layers.rs:56` — `"storage/backend/messages.rs"` → `"storage/messages.rs"`. Keeps `guardrails_messages_swipes_separation` firing at the new path (same rule, new location — not a weakening). (1 SP)
  - [ ] ##### SubTask 3.4.2: `docs/diataxis/reference/coding_standards/unit_test_standards.md` lines 32 + 218 — `storage::backend::Storage` / `storage::backend::TestOverride` → `storage::Storage` / `storage::TestOverride`. (1 SP)
- [ ] #### Task 3.5: Verify Phase 3 green (1 SP)
  - `cargo fmt && cargo clippy --all-targets -- -D warnings && cargo test --lib` then `cargo nextest run --test guardrails` then `cargo nextest run --tests` (integration suite catches the 2 integration-test import updates and any missed `storage::backend::` path). All green.

### Phase 4: Final validation

- [ ] #### Task 4.1: Full gate + structural spot-check (1 SP)
  - [ ] ##### SubTask 4.1.1: `python build.py` green (fmt + clippy + guardrails + tests). (1 SP)
  - [ ] ##### SubTask 4.1.2: Structural spot-check: `rg -n "^impl Storage" src/` — every hit is in `src/adapters/driven/storage/` (parent ends `/storage`) or the def file `storage/core.rs`. `rg -n "^impl InMemoryData" src/` — exactly one hit, in `storage/in_memory_data.rs` (def file). No `impl Storage` remains in `bootstrap/`. (1 SP)

## Test Plan

- **Existing unit tests (unchanged behavior):** `storage/*_tests.rs` (moved up from `backend/`) exercise every Storage method against in-memory + sqlite backends. Must pass unchanged — only import paths change. `core_tests.rs` covers `new_in_memory`/`new_sqlite`/`set_game_id`/failure-injection (Storage core). `load_tests.rs` covers `seed_game_data` (now free-fn form, same behavior).
- **Existing guardrails:** `guardrails_mod_purity` (`storage/mod.rs` stays declaration-only after the merge — all `pub mod`/`mod`/`pub use`/`//!`), `guardrails_file_length_src` (largest file `worlds.rs` ~298 lines, well under 2000), `guardrails_free_fn_location` (`seed_game_data` stays in `bootstrap/`, allowlisted; `in_memory_data.rs` has no free fns), `guardrails_messages_swipes_separation` (path-pin updated to `storage/messages.rs`), `guardrails_test_file_location` (test files move alongside their sources, pairing holds).
- **No new tests:** pure relocation, no new behavior. YAGNI on an `in_memory_data_tests.rs` — InMemoryData methods stay covered indirectly via the Storage tests that call them.
- **Deferred:** the `guardrails_inherent_impl_locality` test itself (ticket 01) is not in the repo during this ticket. When ticket 01 is re-added, it should report **zero** `Storage` and `InMemoryData` violations. That is the final confirmation, out of scope here.

## Per Task/Sub Task Validation Steps

- After each Phase: `cargo fmt && cargo clippy --all-targets -- -D warnings && cargo test --lib && cargo nextest run --test guardrails` — must be green before starting the next phase.
- Phase 3 additional: `cargo nextest run --tests` (integration suite) — catches the 2 integration-test import updates and any missed `storage::backend::` path.
- Final: `python build.py`.
- Structural check (Task 4.1.2): the two `rg` commands confirm the relocation matches the rule formula by path analysis, since the rule test isn't active.

## Assumptions

- **Rule formula** (from map): violation iff `impl_path != def_path` AND NOT (`impl_path`'s parent dir ends with `/{snake(type)}`). `snake_case(Storage) == "storage"`, `snake_case(InMemoryData) == "in_memory_data"`.
- **Shape decision — flatten (locked):** entity files move up into `storage/` so every `impl Storage` file's parent dir is `storage` (folder exemption). Def in `storage/core.rs` (not `mod.rs` — `guardrails_mod_purity`). The ticket's Option A put the def in `mod.rs` — **that violates `guardrails_mod_purity`**, so def goes in `core.rs`; `mod.rs` stays pure. `storage/` ends up with ~16 direct entries; the "backend ops" grouping dissolves — accepted tradeoff vs the `storage/storage/` doubling a rename would introduce.
- **`InMemoryData` → flat file (not folder):** ~110 lines total (def + 4 helper structs + 6 methods). A 3-file folder for 110 lines is over-engineering; def==impl path makes the rule clean without a folder exemption. `ponytail:` flat file; split into a folder if InMemoryData grows past ~300 lines or gains a third impl file.
- **`seed_game_data` → free fn (locked):** it is bootstrap orchestration (reads JSON, dispatches to `storage.seed_world/character/persona`), not storage behavior. Moving the impl into `storage/` would either drag its helper free fns into a non-`free_fn_location`-allowlisted folder, or split the orchestrator from its helpers across modules. The free-fn form is the root-cause fix: encodes that seeding is bootstrap, keeps helpers cohesive in `bootstrap/load.rs`.
- **`seed_game_data` was never failure-injectable:** the method body does not route through `with_backend_mut("seed_game_data", ...)`, so no test could inject a failure on it via `with_failure`/`TestOverride`. Grep confirms zero `with_failure("seed_game_data", ...)` / `TestOverride`-on-`seed_game_data` usages. Converting to a free fn therefore loses zero test capability — no silent break of the failure-injection seam.
- **`Db*` entanglement (6 files have both `impl Storage` and `impl Db*`):** ticket 03 does **not** modify any `impl Db*` block. When the files flatten up in Phase 3, the `impl Db*` blocks ride along into `storage/{characters,games,personas,presets,settings,worlds}.rs` — still rule-violations (folder `storage` ≠ `db_character` etc.), but the rule is inactive and ticket 06 owns their extraction to `models/db_*.rs`. Ticket 03's constraint "Do NOT touch Db*" = don't edit the impl blocks; relocating the containing file is unavoidable and leaves the blocks intact. The map's ticket-02 resolution table double-assigned `Db*` to both 03 and 06; this plan resolves the overlap: 03 relocates files (Db* ride along), 06 extracts Db* impls into `models/`.
- **Visibility widening:** 5 InMemoryData private methods → `pub(crate)`. Unavoidable: they're called by `impl Storage` methods in sibling files after the split. `pub(crate)` matches the existing convention (`game_id`, `with_backend_mut` are already `pub(crate)`). Not a `pub` leak — crate-internal only.
- **`storage::backend::` import path:** simplified to `storage::` (top re-export) in all callers. The top-level re-export already exists today (`storage/mod.rs` does `pub use backend::*`); after flatten it does `pub use core::*` + `pub use in_memory_data::*`. No compat shim kept — paths are rewritten in full (codebase rule: no backward-compat unless asked).
- **`guardrails_application_storage_direct`** (referenced in map + ticket 03): **does not exist** in `tests/infrastructure/guardrails/`. Grep for `grandfather` / `application_storage_direct` hits only the issue/map markdown, no test. The constraint is aspirational. This refactor adds no new `application/ → Storage` imports (relocation is within `storage/`), so even if it existed it wouldn't fire. Surfaced: map's claim "already on the books" is inaccurate.
- **External `Storage` call sites (45 files):** all import via `crate::adapters::driven::storage::Storage`, preserved by the re-export chain. Untouched.
- **`test_support.rs`** (Storage failure-injection: `TestOverride`, `TestFailureHandle`): moves up with the flatten to `storage/test_support.rs`. Its own impls are in-file (def==impl path) → rule-clean regardless of location. Distinct from `src/test_support/` (top-level test fixtures) — different paths, pre-existing naming, not introduced here.
- **`super::test_support` ref in `core.rs`:** stays valid after flatten — `test_support.rs` moves up alongside `core.rs`, so `super::test_support` (super = `storage` module) still resolves. No edit needed for that intra-module ref.
- **Historical docs left as-is:** `docs/CHANGELOG.md`, `docs/plans/*`, `.agents/skills/test-police/TEST_INVENTORY.md` (auto-regenerated by the test-police skill), `docs/.../assets/lifecycle-arrival-disposition.md` reference old `storage/backend/` paths. These are historical snapshots / auto-generated — updating them is scope creep. `TEST_INVENTORY.md` regenerates next test-police run; the others are point-in-time records.
- **Post-implementation:** run `chronicler-after-plan-workflow` skill (per map Notes) to refresh tests/AGENTS.md, docs index, and structure index after the landed code changes.

## NOT in scope

- The `guardrails_inherent_impl_locality` rule file itself (ticket 01 — re-added after 03–08 land).
- Wiring the rule into `build.py` as a gate (ticket 09).
- Any `impl Db*` block edits (ticket 06 — they ride along in the flatten but aren't modified).
- `AppState`, `PromptContext`, `PromptPreset`, `QuantifierResult`, `QuantifierParseResult`, `DefaultApplicationService`, `PipelineRun`, `ActionPipeline` (tickets 04, 05, 07, 08 + new AppState ticket).
- Trait-impl locality, free-function location rule, test-target-location rule (map Out of scope).
- Updating historical docs (CHANGELOG, old plans, auto-generated test inventory).

## What already exists (reuse, don't reimplement)

- `check_src_files` / `discover_rs_files` / `Violation` harness in `tests/infrastructure/guardrails/mod.rs` — the rule (ticket 01) will ride this; not this ticket's concern.
- Top-level re-export pattern (`storage/mod.rs` does `pub use backend::*`) — already present; flatten just changes `backend` → `core` + `in_memory_data`.
- `pub(crate)` visibility convention for cross-file storage internals (`game_id`, `with_backend_mut`, `read_json_file`) — the 5 widened InMemoryData methods follow it.
- `free_fn_location` allowlist includes `bootstrap` — `seed_game_data` free fn lands compliant with zero new rule mechanism.
- Existing `load_tests.rs` already exercises `seed_game_data` against in-memory + temp dirs — covers the signature change with no new test.

## Failure modes

- **Phase 1 — missed visibility widening:** if an InMemoryData helper isn't raised to `pub(crate)`, the cross-file `impl Storage` caller won't compile. Surfaced as a compile error (not silent). Fix: widen, rebuild.
- **Phase 2 — missed call site:** if a `storage.seed_game_data(...)` call isn't rewritten, compile error (method no longer exists on Storage). 7 sites enumerated (1 + 6); compiler confirms none missed.
- **Phase 3 — missed `storage::backend::` path:** compile error (module `backend` no longer exists). Full reference set grep-verified (22 src + 2 integration + 1 guardrail + 1 live doc). `cargo nextest run --tests` in Phase 3 catches integration-test misses.
- **Phase 3 — `mod.rs` merge omission:** if a `pub mod` / `#[cfg(test)] mod` declaration isn't carried over, compile error (undeclared module) or a dead test file. Compiler + `check_test_file_location` guardrail (orphan test file) catch it.
- **`guardrails_messages_swipes_separation` silent death:** if the path-pin at `layers.rs:56` isn't updated to `storage/messages.rs`, the rule returns early on every file (no match) and stops firing — a *silent* weakening. Mitigation: Task 3.4.1 updates it; Phase 3 guardrail run confirms no regression in the suite's active rules. (This is the one non-compiler failure mode — called out because it's silent.)
- No new runtime code paths → no runtime failure modes, no logging/metrics gaps.

## Unresolved decisions

None remaining. Both shaping decisions (flatten layout, free-fn `seed_game_data`) are locked via plan review. The two map-level fog items ("does flattening backend/ require a separate ticket" — answered: bundled into 03; "AppState refactor ticket" — out of scope for 03, separate frontier item) are not blocked by this ticket.
