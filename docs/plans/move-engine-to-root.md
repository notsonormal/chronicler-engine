# Plan: Move `chronicler_engine/` contents to repo root

## Summary
The repo was extracted from `mrn-general/` with git history but still nests the engine under `chronicler_engine/`. Make it a standalone monorepo: move the folder's *contents* to the repo root via `git mv`, then rewrite every path reference that breaks. Root `AGENTS.md` already migrated (previous task). The crate name `chronicler_engine` stays unchanged — only filesystem *paths* change.

## Key Changes
- Physical move: `chronicler_engine/{src,tests,scripts,docs,assets,data,styles,old-docs,CONTEXT.md,Cargo.toml,Cargo.lock,build.py,arch-lint.toml,clippy.toml,rust-toolchain.toml,rustfmt.toml,.vale.ini,.cargo,.config}` → repo root. `docs/` merges into existing root `docs/` (no collision: root `docs/` has only `agents/`; engine `docs/` has `diataxis/specs/plans/external_applications/AGENTS.md/CHANGELOG.md`).
- Drop `chronicler_engine/.pi/tasks/` (gitignored ephemeral) and the `python` symlink (build.py uses `sys.executable`).
- Load-bearing code rewrites (build/tests break without these): `scripts/validate_docs.py`, `tests/infrastructure/guardrails/structure.rs` + `structure_tests.rs`, `scripts/install_git_hooks.py`, `scripts/check_python_docstrings.py`, `scripts/tests/test_validate_docs.py`, `scripts/tests/test_extract_http_routes.py`.
- Mechanical rewrite: all `src/**/*.rs` line-1 DOC anchors `//! [DOC: chronicler_engine/docs/diataxis/reference/...]` → `//! [DOC: docs/diataxis/reference/...]` (~150 files, one sed pass). Also `arch-lint.toml` line 2.
- Prose path fixes: root `AGENTS.md`, `docs/AGENTS.md` preamble, `docs/agents/domain.md`, `tests/STRATEGY.md`, `tests/browser/behaviour.rs` scenario comments, guardrail comment prose.
- `.gitignore` prefix cleanup (drop `chronicler_engine/` from ~10 rules, dedupe dups).
- KEEP unchanged: crate name `chronicler_engine` everywhere (`use chronicler_engine::...`, `cargo -p chronicler_engine`, `RUST_LOG=chronicler_engine=debug`, binary name `debug/chronicler_engine`, `Cargo.toml name=`, `arch-lint.toml root="./src"`, pre-commit hook's `SCRIPT_DIR/../..` logic — which the move actually *fixes*).

## Implementation

### Phase 1: Physical move (git mv, preserve history)

- [ ] #### Task 1.1: Move directories via git mv (3 SP)
  - `git mv chronicler_engine/{src,tests,scripts,assets,data,styles,old-docs} ./` for each
  - `git mv chronicler_engine/.cargo chronicler_engine/.config ./` (dot-dirs)
  - Move `docs/` children individually into existing root `docs/` (git mv can't dir-into-dir atomically): `git mv chronicler_engine/docs/diataxis chronicler_engine/docs/specs chronicler_engine/docs/plans chronicler_engine/docs/external_applications chronicler_engine/docs/AGENTS.md chronicler_engine/docs/CHANGELOG.md docs/`
- [ ] #### Task 1.2: Move root files via git mv (1 SP)
  - `git mv chronicler_engine/{CONTEXT.md,Cargo.toml,Cargo.lock,build.py,arch-lint.toml,clippy.toml,rust-toolchain.toml,rustfmt.toml,.vale.ini} ./`
- [ ] #### Task 1.3: Clean up residual folder (1 SP)
  - Remove `chronicler_engine/.pi/tasks/` (gitignored, ephemeral) and `chronicler_engine/python` symlink
  - `rmdir chronicler_engine` (confirm empty)
  - `git status` — confirm all moves tracked, nothing untracked-unexpected

### Phase 2: Path-anchor + scenario-comment mechanical rewrite

The substring `\[chronicler_engine/docs/` (opening bracket) appears in three forms — `//! [DOC: chronicler_engine/docs/diataxis/...]` (src, ~150), `// [chronicler_engine/docs/specs/...]` (tests, 92 matches across 11 files: browser/behaviour.rs, http/actions.rs, http/prompt_presets.rs, http/retrigger.rs, http/settings.rs, http/games_*.rs, http/reset.rs, …), and `[DOC: chronicler_engine/docs/...]` (arch-lint.toml line 2). It never appears in crate refs (`use chronicler_engine::`, `cargo -p chronicler_engine`, `chronicler_engine=debug`, `chronicler_engine.exe`), so a blanket sed is safe.

- [ ] #### Task 2.1: Rewrite all `\[chronicler_engine/docs/` refs via one sed (2 SP)
  - From repo root: `grep -rl '\[chronicler_engine/docs/' src tests *.toml | xargs sed -i 's|\[chronicler_engine/docs/|[docs/|g'`
  - Covers src DOC anchors, tests `// [chronicler_engine/docs/specs/...]` scenario comments, and `arch-lint.toml` line 2 in one pass
  - Spot-check: `head -2 src/lib.rs`; `grep -rc '\[chronicler_engine/docs/' src tests` → 0 for every file

### Phase 3: Load-bearing code rewrites (build/tests break without these)

- [ ] #### Task 3.1: Fix `scripts/validate_docs.py` repo_root logic (3 SP)
  - L850-855: `repo_root = engine_root.parent` → `repo_root = engine_root`; `reference_root = (repo_root / "chronicler_engine" / "docs" / ...)` → `reference_root = repo_root / "docs" / "diataxis" / "reference"`; `resolved = (repo_root / target)` stays but `target` no longer has `chronicler_engine/` prefix so path resolves under repo root directly. Verify against new anchor format `docs/...`.
  - L855 `resolved = (repo_root / target).resolve()` — with `target = "docs/diataxis/reference/foo.md"`, `repo_root / target` works. Keep.
  - L947 docstring "engine_root is the chronicler_engine/ directory" → update prose (Phase 5)
  - Self-test: `python scripts/validate_docs.py` runs clean against moved layout
- [ ] #### Task 3.2: Fix guardrail anchor-prefix check (3 SP)
  - `tests/infrastructure/guardrails/structure.rs` L86,94,100: `starts_with("chronicler_engine/docs/diataxis/reference/")` → `starts_with("docs/diataxis/reference/")`; update the 2 violation message strings identically
  - `tests/infrastructure/guardrails/structure_tests.rs` L18,53,283: fixture anchor strings `chronicler_engine/docs/...` → `docs/...`
  - Validate: `cargo nextest run --test architecture -- structure` (guardrails) passes
- [ ] #### Task 3.3: Fix `scripts/install_git_hooks.py` (1 SP)
  - L19: `repo_root / "chronicler_engine" / "scripts" / "git-hooks"` → `repo_root / "scripts" / "git-hooks"`
- [ ] #### Task 3.4: Fix `scripts/check_python_docstrings.py` (1 SP)
  - L86,90: `chronicler_engine = Path(__file__).parent.parent` then `chronicler_engine / "scripts"` and `chronicler_engine.parent / "scripts" / "issue_tracker"`. Post-move `Path(__file__).parent.parent` = repo root. Simplify: `root = Path(__file__).parent.parent; dirs = [root / "scripts"]`. Drop the `issue_tracker` line (dir doesn't exist) — or keep with a guard. Prefer drop (YAGNI).
- [ ] #### Task 3.5: Fix `scripts/tests/test_validate_docs.py` (3 SP)
  - L20: `REPO_ROOT = Path(__file__).resolve().parents[3]` → `parents[2]` (file moves from `chronicler_engine/scripts/tests/` to `scripts/tests/`, one less parent)
  - L20: `sys.path.insert(0, str(REPO_ROOT / "chronicler_engine" / "scripts"))` → `REPO_ROOT / "scripts"`
  - L155: `self.engine = self.root / "chronicler_engine"` → `self.engine = self.root` (fake layout drops the subfolder)
  - L185,190,199,242,265,324,340,371: anchor fixture strings `chronicler_engine/docs/...` → `docs/...`
  - L215,329: assertion strings `"must resolve under chronicler_engine/docs/..."` → `"must resolve under docs/..."`
  - L294: `engine_root = REPO_ROOT / "chronicler_engine"` → `engine_root = REPO_ROOT`
  - L317: comment only
  - Run: `python -m unittest scripts.tests.test_validate_docs` (or via build.py)
- [ ] #### Task 3.6: Fix `scripts/tests/test_extract_http_routes.py` (2 SP)
  - L15: `parents[3]` → `parents[2]`; `REPO_ROOT / "chronicler_engine" / "scripts"` → `REPO_ROOT / "scripts"`
  - L19: `ENGINE_ROOT = REPO_ROOT / "chronicler_engine"` → `ENGINE_ROOT = REPO_ROOT`
  - L260,293,298: drop `"chronicler_engine"` segment in path joins / fake-layout mkdir
  - Run: `python -m unittest scripts.tests.test_extract_http_routes`

### Phase 4: Root AGENTS.md path references

- [ ] #### Task 4.1: Update root AGENTS.md (1 SP)
  - DOCUMENTATION INDEX: `chronicler_engine/docs/AGENTS.md`→`docs/AGENTS.md`, `chronicler_engine/tests/AGENTS.md`→`tests/AGENTS.md`, `python chronicler_engine/scripts/generate_docs_index.py`→`python scripts/generate_docs_index.py`
  - DEVELOPMENT LOOP: `chronicler_engine/logs`→`logs`, `chronicler_engine/tmp`→`tmp`
  - DOC STRATEGY Core Principle 2: anchor format `//! [DOC: chronicler_engine/docs/diataxis/reference/<area>/<name>.md]` → `//! [DOC: docs/diataxis/reference/<area>/<name>.md]`
  - STRUCTURE auto-index: regenerated by `generate_structure_index.py` (run in Phase 6) — self-corrects

### Phase 5: Other prose path references

- [ ] #### Task 5.1: Fix docs/AGENTS.md preamble (1 SP)
  - Preamble refs `chronicler_engine/AGENTS.md`→`AGENTS.md` (the general-principles pointer). AUTO-INDEX block regenerated separately.
- [ ] #### Task 5.2: Fix docs/agents/domain.md (1 SP)
  - L8: `chronicler_engine/CONTEXT.md`→`CONTEXT.md`; layout diagram L21-22 `chronicler_engine/CONTEXT.md`→root `CONTEXT.md`
- [ ] #### Task 5.3: Fix guardrail comment prose (1 SP)
  - `tests/infrastructure/guardrails/layers.rs` L7, `location.rs` L45: `chronicler_engine/src/...` → `src/...` (comment-only)
  - `tests/STRATEGY.md` L3: `for chronicler_engine` → `for the engine` or drop the path; `tests/AGENTS.md` auto-regenerated
- [ ] #### Task 5.4: Fix script docstrings (non-functional but stale) (1 SP)
  - `scripts/vale_lint.py` L4,33,34; `scripts/generate_docs_index.py` L1; `scripts/extract_http_routes.py` L31,226; `scripts/validate_feature_spec.py` L3-5,20; `scripts/validate_docs.py` L1,50,831,863,947,1017; `scripts/healthcheck.py` L79; `scripts/install_git_hooks.py` L1; `scripts/coverage_summary.py` L31, `scripts/parse_coverage.py` L88 — drop `chronicler_engine/` prefix in prose. Path logic self-corrects via `Path(__file__).parent.parent`.

### Phase 6: .gitignore cleanup

- [ ] #### Task 6.1: Drop chronicler_engine/ prefixes (1 SP)
  - L214 `/chronicler_engine/.sisyphus` → `/.sisyphus`
  - L222 `/chronicler_engine/tmp` → drop (L213 already has bare `tmp`)
  - L224-225 `chronicler_engine/coverage.json`/`coverage_full.json` → `/coverage.json`, `/coverage_full.json`
  - L226 `/chronicler_engine/.pi-lens/cache` → `/.pi-lens/cache`
  - L231 `/chronicler_engine/jscpd-report` → `/jscpd-report`
  - L232 `chronicler_engine/report/jscpd-report.json` → `/report/jscpd-report.json`
  - L233,248 `chronicler_engine/.pi/task[s]` → covered by L230 `/.pi/tasks` + L247 `.pi/tasks/`; drop the redundant lines
  - Dedupe: L213 `tmp` + L212 `logs/` already cover the moved dirs; remove now-duplicate L222

### Phase 7: Regenerate auto-indexes + verify

- [ ] #### Task 7.1: Regenerate indexes (1 SP)
  - `python scripts/generate_docs_index.py` (regenerates `docs/AGENTS.md` AUTO-INDEX)
  - `python scripts/generate_structure_index.py` (regenerates root `AGENTS.md` STRUCTURE)
  - `python scripts/generate_tests_structure_index.py` (regenerates `tests/AGENTS.md`)
  - `python scripts/extract_http_routes.py` (regenerates `docs/diataxis/reference/frontend/http_routes.md`)
- [ ] #### Task 7.2: Run cargo gate (2 SP)
  - `cargo fmt`
  - `cargo clippy --all-targets -- -D warnings`
  - `cargo test --lib`
  - `cargo nextest run --test architecture` (canary — validates new anchor prefix in Phase 3.2)
  - `cargo nextest run --test guardrails`
- [ ] #### Task 7.3: Run python + final gate (2 SP)
  - `python -m unittest discover scripts/tests`
  - `python scripts/validate_docs.py`
  - `python scripts/validate_feature_spec.py`
  - Final whole-repo sweep (catches anything missed): `grep -rn 'chronicler_engine/' --include='*.rs' --include='*.py' --include='*.md' --include='*.toml' . | grep -v 'chronicler_engine::' | grep -v 'chronicler_engine=debug' | grep -v 'chronicler_engine.exe' | grep -v 'name = "chronicler_engine"' | grep -v 'cargo -p chronicler_engine' | grep -vE '(docs/CHANGELOG|docs/old-docs|docs/plans)/'` → expect 0 lines. Any hit is a missed rewrite.
- [ ] #### Task 7.4: Install + smoke-test pre-commit hook (1 SP)
  - `python scripts/install_git_hooks.py`
  - Verify `.git/hooks/pre-commit` exists and `SCRIPT_DIR/../..` resolves to repo root (the move fixes the previously-broken hook)

## Test Plan
- **Architecture test** (`cargo nextest run --test architecture`): canary for the DOC-anchor prefix rewrite in Phase 3.2 — fails if any `src/` anchor still has `chronicler_engine/` prefix or `structure.rs` check is half-updated.
- **Guardrails test**: validates structure_tests.rs fixtures match new `starts_with("docs/...")` check.
- **`python -m unittest scripts.tests.test_validate_docs`**: canary for `validate_docs.py` repo_root logic (Phase 3.1) — the `parents[3]→parents[2]` and fake-layout changes both break silently if wrong.
- **`python scripts/validate_docs.py`**: end-to-end check that real anchors resolve under `docs/diataxis/reference/` at root.
- **`python build.py`**: full gate — fmt + clippy + guardrails + tests. If green, the move is complete.
- **Pre-commit hook smoke**: stage a trivial change, run the hook, confirm index-regeneration fires (or no-ops cleanly).

## Per Task/Sub Task Validation Steps
- Task 1.3: `git status` shows all moves as renames, `rmdir chronicler_engine` succeeds (empty), no untracked files appear unexpectedly.
- Task 2.1: `grep -rc '\[DOC: chronicler_engine/' src` returns 0 for every file; `head -2 src/lib.rs` shows `docs/diataxis/...`.
- Task 3.1: `python scripts/validate_docs.py` exits 0 with no BROKEN_DOC_ANCHOR violations.
- Task 3.2: `cargo nextest run --test architecture` green; `cargo nextest run --test guardrails` green.
- Task 3.5/3.6: `python -m unittest scripts.tests.test_validate_docs scripts.tests.test_extract_http_routes` green.
- Task 4.1: `grep -c 'chronicler_engine/' AGENTS.md` returns 0 (after the rewrite — confirm no stale path refs remain).
- Task 7.3: `python build.py` exits 0; check `logs/build_*.log` for the Step Timing Summary showing all steps pass.

## NOT in scope
- `docs/CHANGELOG.md` (126 KB) — historical entries reference `chronicler_engine/src/...` from the nested era. Not rewritten (historical record). Excluded from the Phase 7 final sweep.
- `docs/old-docs/reviews/` — archived review docs with stale path refs. Not rewritten (archived). Excluded from the sweep.
- `docs/plans/*.md` (including this plan) — historical/archival plans reference old paths. Not rewritten. Excluded from the sweep.
- Crate-name references (`use chronicler_engine::`, `cargo -p chronicler_engine`, `RUST_LOG=chronicler_engine=debug`, binary `debug/chronicler_engine`, `Cargo.toml name=`) — crate refs, not paths. Untouched.
- The pre-commit hook's `docs/README.md` staleness check — pre-existing no-op (no generator produces `docs/README.md`). Not a move issue.

## Assumptions
- The crate name `chronicler_engine` is intentionally kept (only filesystem paths change). Confirmed: `Cargo.toml name=`, all `use chronicler_engine::`, `cargo -p chronicler_engine`, `RUST_LOG=chronicler_engine=debug`, binary `debug/chronicler_engine` are crate references, not paths — untouched.
- `docs/` merge is safe: root `docs/` has only `agents/` (3 files); engine `docs/` has `diataxis/specs/plans/external_applications/AGENTS.md/CHANGELOG.md`. No filename collisions. Verified by `ls`.
- `chronicler_engine/.pi/tasks/` is gitignored ephemeral (`.gitignore` L248) — safe to drop, not `git mv`. The root `.pi/tasks/` already exists and is separately gitignored.
- The `python` symlink at `chronicler_engine/python` is not used by `build.py` (uses `sys.executable`); safe to drop. Re-create at root only if a script is found that depends on it (none found).
- `scripts/issue_tracker/` referenced by `check_python_docstrings.py` does not exist — the reference is dead. Task 3.4 drops it (YAGNI) rather than preserving a path to nothing.
- The pre-commit hook (`scripts/git-hooks/pre-commit`) uses `SCRIPT_DIR/../..` to find repo root. Currently broken (scripts/ not at root). The move *fixes* it — no hook edit needed, only reinstall (Task 7.4).
- `docs/README.md` (pre-commit staleness check target) is NOT generated by `generate_docs_index.py` (which writes `docs/AGENTS.md`). The hook's `docs/README.md` check is a pre-existing no-op; the move neither fixes nor breaks it. No action.
- Story points total: ~34 SP across 7 phases. Each phase ≤ 8 SP; no single task exceeds 3 SP. No further breakdown needed.
- Commit strategy (not a task — recommendation): split into two commits — (A) Phase 1+2+6 (move + sed + gitignore), (B) Phase 3+4+5+7 (code + prose + verify). Keeps each commit's `git status` reviewable.
