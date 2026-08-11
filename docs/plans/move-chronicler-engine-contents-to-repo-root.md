# Move `chronicler_engine/` contents to repo root

## Summary
Repo extracted from `mrn-general/` with history but still nests the engine under `chronicler_engine/`. Make it a standalone monorepo: move the folder's *contents* to the repo root via `git mv`, then rewrite every path reference that breaks. Root `AGENTS.md` already migrated. The **crate name** `chronicler_engine` stays unchanged — only filesystem *paths* change.

## Key Changes
- Physical move: `chronicler_engine/{src,tests,scripts,docs,assets,data,styles,old-docs,CONTEXT.md,Cargo.toml,Cargo.lock,build.py,arch-lint.toml,clippy.toml,rust-toolchain.toml,rustfmt.toml,.vale.ini,.cargo,.config}` → repo root. `docs/` merges into existing root `docs/` (no collision: root `docs/` has only `agents/`).
- Drop `chronicler_engine/.pi/tasks/` (gitignored ephemeral) and the `python` symlink (build.py uses `sys.executable`).
- Load-bearing code rewrites: `scripts/validate_docs.py`, `tests/infrastructure/guardrails/{structure.rs,structure_tests.rs}`, `scripts/install_git_hooks.py`, `scripts/check_python_docstrings.py`, `scripts/tests/{test_validate_docs.py,test_extract_http_routes.py}`.
- Mechanical rewrite: all `\[chronicler_engine/docs/` refs (src DOC anchors ~150, tests scenario comments 92 across 11 files, arch-lint.toml) via one sed.
- Prose path fixes: root `AGENTS.md`, `docs/AGENTS.md`, `docs/agents/domain.md`, `tests/STRATEGY.md`, guardrail comment prose, script docstrings.
- `.gitignore`: drop `chronicler_engine/` prefix from ~10 rules, dedupe.
- KEEP unchanged: crate name everywhere, `arch-lint.toml root="./src"`, pre-commit hook's `SCRIPT_DIR/../..` (move fixes it).

## Implementation

### Phase 1: Physical move (git mv, preserve history)

- [ ] #### Task 1.1: Move directories via git mv (3 SP)
  - `git mv chronicler_engine/{src,tests,scripts,assets,data,styles,old-docs} ./`; `git mv chronicler_engine/.cargo chronicler_engine/.config ./`
  - Move `docs/` children individually: `git mv chronicler_engine/docs/{diataxis,specs,plans,external_applications,AGENTS.md,CHANGELOG.md} docs/`
- [ ] #### Task 1.2: Move root files via git mv (1 SP)
  - `git mv chronicler_engine/{CONTEXT.md,Cargo.toml,Cargo.lock,build.py,arch-lint.toml,clippy.toml,rust-toolchain.toml,rustfmt.toml,.vale.ini} ./`
- [ ] #### Task 1.3: Clean up residual folder (1 SP)
  - Remove `chronicler_engine/.pi/tasks/` (gitignored) and `chronicler_engine/python` symlink; `rmdir chronicler_engine`; `git status` — confirm all moves tracked

### Phase 2: Path-anchor + scenario-comment mechanical rewrite

`\[chronicler_engine/docs/` (opening bracket) appears in src `//! [DOC:...]` (~150), tests `// [chronicler_engine/docs/specs/...]` (92 across 11 files), and arch-lint.toml. Never in crate refs — blanket sed is safe.

- [ ] #### Task 2.1: Rewrite all `\[chronicler_engine/docs/` refs via one sed (2 SP)
  - `grep -rl '\[chronicler_engine/docs/' src tests *.toml | xargs sed -i 's|\[chronicler_engine/docs/|[docs/|g'`
  - Spot-check: `head -2 src/lib.rs`; `grep -rc '\[chronicler_engine/docs/' src tests` → 0

### Phase 3: Load-bearing code rewrites

- [ ] #### Task 3.1: Fix `scripts/validate_docs.py` repo_root logic (3 SP)
  - `repo_root = engine_root.parent` → `repo_root = engine_root`; drop `"chronicler_engine"` segment in `reference_root`/`resolved` joins
  - Self-test: `python scripts/validate_docs.py` clean
- [ ] #### Task 3.2: Fix guardrail anchor-prefix check (3 SP)
  - `structure.rs` L86,94,100: `starts_with("chronicler_engine/docs/diataxis/reference/")` → `starts_with("docs/diataxis/reference/")`; update 2 message strings
  - `structure_tests.rs` L18,53,283: fixture anchor strings → `docs/...`
  - `cargo nextest run --test architecture`
- [ ] #### Task 3.3: Fix `scripts/install_git_hooks.py` (1 SP)
  - L19: drop `"chronicler_engine"` segment from git-hooks path
- [ ] #### Task 3.4: Fix `scripts/check_python_docstrings.py` (1 SP)
  - Simplify to `root = Path(__file__).parent.parent; dirs = [root / "scripts"]`; drop dead `issue_tracker` line (YAGNI)
- [ ] #### Task 3.5: Fix `scripts/tests/test_validate_docs.py` (3 SP)
  - `parents[3]`→`parents[2]`; drop `"chronicler_engine"` segments; fake-layout drops subfolder; anchor fixture strings + assertion strings → `docs/...`; `engine_root = REPO_ROOT`
- [ ] #### Task 3.6: Fix `scripts/tests/test_extract_http_routes.py` (2 SP)
  - `parents[3]`→`parents[2]`; `ENGINE_ROOT = REPO_ROOT`; drop `"chronicler_engine"` segment in path joins / fake-layout mkdir

### Phase 4: Root AGENTS.md path references

- [ ] #### Task 4.1: Update root AGENTS.md (1 SP)
  - DOCUMENTATION INDEX: `chronicler_engine/{docs,tests}/AGENTS.md`→`{docs,tests}/AGENTS.md`, `python chronicler_engine/scripts/...`→`python scripts/...`
  - DEVELOPMENT LOOP: `chronicler_engine/logs`→`logs`, `chronicler_engine/tmp`→`tmp`
  - DOC STRATEGY Core Principle 2: anchor format → `docs/diataxis/...`
  - STRUCTURE auto-index: regenerated in Phase 7 (self-corrects)

### Phase 5: Other prose path references

- [ ] #### Task 5.1: Fix docs/AGENTS.md preamble (1 SP) — `chronicler_engine/AGENTS.md`→`AGENTS.md`; AUTO-INDEX regenerated separately
- [ ] #### Task 5.2: Fix docs/agents/domain.md (1 SP) — `chronicler_engine/CONTEXT.md`→`CONTEXT.md`
- [ ] #### Task 5.3: Fix guardrail comment prose (1 SP) — `layers.rs`/`location.rs`: `chronicler_engine/src/...`→`src/...`; `tests/STRATEGY.md`
- [ ] #### Task 5.4: Fix script docstrings (1 SP) — drop `chronicler_engine/` prefix in ~9 script docstrings (user choice: keep polish)

### Phase 6: .gitignore cleanup

- [ ] #### Task 6.1: Drop chronicler_engine/ prefixes (1 SP)
  - `/.sisyphus`, `/coverage*.json`, `/.pi-lens/cache`, `/jscpd-report`, `/report/jscpd-report.json`
  - Drop redundant `/tmp` (L213 covers), `.pi/task[s]` (L230/L247 cover)

### Phase 7: Regenerate indexes + verify

- [ ] #### Task 7.1: Regenerate indexes (1 SP) — `scripts/{generate_docs_index,generate_structure_index,generate_tests_structure_index,extract_http_routes}.py`
- [ ] #### Task 7.2: Run cargo gate (2 SP) — `cargo fmt`; `cargo clippy --all-targets -- -D warnings`; `cargo test --lib`; `cargo nextest run --test architecture` (canary); `cargo nextest run --test guardrails`
- [ ] #### Task 7.3: Run python + final gate (2 SP) — `python -m unittest discover scripts/tests`; `python scripts/validate_docs.py`; `python scripts/validate_feature_spec.py`; `python build.py`; final whole-repo sweep `grep -rn 'chronicler_engine/' ... | grep -v crate-refs | grep -vE '(docs/CHANGELOG|docs/old-docs|docs/plans)/'` → expect 0
- [ ] #### Task 7.4: Install + smoke-test pre-commit hook (1 SP) — `python scripts/install_git_hooks.py`; verify `SCRIPT_DIR/../..` resolves to repo root (move fixes previously-broken hook)

## Test Plan
- **Architecture test** (`cargo nextest run --test architecture`): canary for the DOC-anchor prefix rewrite — fails if any src anchor keeps `chronicler_engine/` prefix or `structure.rs` check is half-updated.
- **`python -m unittest scripts.tests.test_validate_docs`**: canary for `validate_docs.py` repo_root logic — `parents[3]→parents[2]` and fake-layout changes break silently if wrong.
- **`python scripts/validate_docs.py`**: end-to-end check that real anchors resolve under `docs/diataxis/reference/` at root.
- **`python build.py`**: full gate. Green = move complete.
- **Pre-commit hook smoke**: stage trivial change, run hook, confirm index-regeneration fires or no-ops cleanly.

## Per Task/Sub Task Validation Steps
- Task 1.3: `git status` shows renames, `rmdir chronicler_engine` succeeds, no unexpected untracked files.
- Task 2.1: `grep -rc '\[chronicler_engine/docs/' src tests` returns 0; `head -2 src/lib.rs` shows `docs/diataxis/...`.
- Task 3.1: `python scripts/validate_docs.py` exits 0, no BROKEN_DOC_ANCHOR.
- Task 3.2: `cargo nextest run --test architecture` green; `cargo nextest run --test guardrails` green.
- Task 3.5/3.6: `python -m unittest scripts.tests.{test_validate_docs,test_extract_http_routes}` green.
- Task 4.1: `grep -c 'chronicler_engine/' AGENTS.md` returns 0.
- Task 7.3: `python build.py` exits 0; `logs/build_*.log` Step Timing Summary all-pass; final sweep returns 0 lines.

## Assumptions
- Crate name `chronicler_engine` kept intentionally (only paths change). `Cargo.toml name=`, `use chronicler_engine::`, `cargo -p chronicler_engine`, `RUST_LOG=chronicler_engine=debug`, binary `debug/chronicler_engine` are crate refs — untouched.
- `docs/` merge safe: root `docs/` has only `agents/`; engine `docs/` has `diataxis/specs/plans/external_applications/AGENTS.md/CHANGELOG.md`. No collisions. Verified by `ls`.
- `chronicler_engine/.pi/tasks/` is gitignored ephemeral — drop, not `git mv`. Root `.pi/tasks/` exists separately.
- `chronicler_engine/python` symlink unused by `build.py` — safe to drop. No script depends on it.
- `scripts/issue_tracker/` (referenced by `check_python_docstrings.py`) doesn't exist — dead reference. Task 3.4 drops it (YAGNI).
- Pre-commit hook uses `SCRIPT_DIR/../..` for repo root — currently broken (scripts/ not at root); the move *fixes* it. No hook edit, only reinstall (Task 7.4).
- `docs/README.md` is NOT generated by `generate_docs_index.py` (writes `docs/AGENTS.md`). Hook's `docs/README.md` check is a pre-existing no-op; move neither fixes nor breaks it. No action.
- Total ~34 SP across 7 phases; each phase ≤ 8 SP; no task > 3 SP. No further breakdown needed.
- Commit strategy (recommendation): two commits — (A) Phase 1+2+6 (move + sed + gitignore), (B) Phase 3+4+5+7 (code + prose + verify). Keeps each `git status` reviewable.
