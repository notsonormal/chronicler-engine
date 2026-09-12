# Ticket 20: Per-Surface Spec + Test-File Reorganization

## Summary

Resolve map ticket 20 ([Per-surface spec + test-file reorganization](.scratch/narrator-modes-and-options/issues/20-implement-per-surface-spec-test-reorg.md)) with the scope refined in this session: decompose `docs/specs/browser.md` fully. DOM-observed scenarios get per-surface spec files `docs/specs/browser_<feature>.md`; the residual chrome scenarios land in `browser_dashboard.md`. Test side mirrors: `tests/browser/behaviour.rs` splits into per-feature files plus `dashboard.rs`. The validator gains the surface-consistency rule. End state: full gate green, validator reports the same declared/covered counts (139/139) with the surface rule active and zero violations.

## Key Changes

- **Spec side (10 files):** rename `browser.md` → `browser_dashboard.md` (H1 `# Feature Spec: Browser Dashboard`; keeps only 16.5–16.7, ids unchanged). Create six files (house header: `# Feature Spec: Browser <Name>`, `Endpoint: browser DOM.`, `## Scenarios`): `browser_story_log.md` 30.1–30.4 (from 16.1–16.4), `browser_slash_menu.md` 31.1–31.9 (from 17.1–17.9), `browser_options.md` 26.1–26.3 (from options.md 24.3/24.8/24.9), `browser_games.md` 27.1–27.3 (from games.md 20.1–20.3), `browser_prompt_presets.md` 28.1 (from 21.27), `browser_worlds.md` 29.1–29.2 (from 18.1/18.2). Donor specs lose the moved blocks; stay-behind ids keep gaps.
- **Test side (8 files):** cut `tests/browser/` files mirroring the specs — `story_log.rs` (~lines 9–176, 4 tests, needs `expect` + `Duration`), `slash_menu.rs` (~329–663, 9 tests + the `type_into_command` helper), `options.rs` (tail 1019–1232, 3 tests), `games.rs` (~665–810, 3 tests), `worlds.rs` (~812–891, 2 tests + `open_world_edit`), `prompt_presets.rs` (~893–994, 1 test); rename residual `behaviour.rs` → `dashboard.rs` (16.5–16.7 + the stdout-tee exemption). Retag: 3 dashboard tags to the new path; 22 moved tests get new path + new id. Register all modules in `mod.rs`; rewrite both `//!` headers; every new file gets its own `//!` header; prune imports per file.
- **Enforcement:** STRATEGY.md "SCENARIO tags" rewrite (per-surface rule, spec↔test mirroring, exemptions now point at `dashboard.rs`); `validate_feature_spec.py` gains the surface-consistency check (`browser_*.md` ⇔ `tests/browser/` only, and vice versa) as a new violation category in the summary line and exit code; `TAG_EXEMPT_TESTS` path updated to `dashboard.rs`.
- **Prose refs:** `dashboard.md:82` → link + cite to `browser_slash_menu.md` 31.1–31.9, with the two→three slash-command fix (`/impersonate`, `/guide`, `/options`); `ui_design.md:10` → glob phrasing (`docs/specs/browser_*.md`); regenerate `tests/AGENTS.md`.
- **Tracker:** amend ticket 20's body Scope to this approved shape before starting; at resolution, amend ticket 12's `## Answer` + the map's ticket-12 line, record the refinement rationale in ticket 20's `## Answer`, and graduate the Documentation fog into ticket 21 (`Type: task`, `Blocked by: 20`).

## Implementation

### Phase 1: Tracker amendment + spec files

- [ ] #### Task 1.1: Amend ticket 20's body and rename the dashboard spec (1 SP)
  - Rewrite the ticket's Scope/Notes bullets to the approved shape (dashboard/story-log/slash-menu split, name choices, id ranges); `git mv docs/specs/browser.md docs/specs/browser_dashboard.md` (plain `mv` if git is permission-blocked); H1 → `# Feature Spec: Browser Dashboard`; delete 16.1–16.4 and 17.1–17.9 blocks.
- [ ] #### Task 1.2: Create the six surface specs and migrate 22 scenario texts (5 SP)
  - Verbatim gherkin moves with renumbering per the id map above; delete moved blocks from donors `options.md`, `games.md`, `prompt_presets.md`, `browser_dashboard.md`.

### Phase 2: Test files

- [ ] #### Task 2.1: Split behaviour.rs into six per-feature files (5 SP)
  - [ ] ##### SubTask 2.1.1: `story_log.rs` — cut ~9–176, retag 30.1–30.4 (1 SP)
  - [ ] ##### SubTask 2.1.2: `slash_menu.rs` — cut ~329–663 incl. `type_into_command`, retag 31.1–31.9 (1 SP)
  - [ ] ##### SubTask 2.1.3: `games.rs`, `worlds.rs`, `prompt_presets.rs` — cuts ~665–810 / ~812–891 / ~893–994, retag 27.x / 29.x / 28.1 (1 SP)
  - [ ] ##### SubTask 2.1.4: `options.rs` — tail cut 1019–1232, retag 26.1–26.3 (1 SP)
  - [ ] ##### SubTask 2.1.5: Rename residual to `dashboard.rs`; retag 16.5–16.7; register all modules in `mod.rs`; rewrite `//!` headers; prune imports; verify helper dependencies per cut (1 SP)

### Phase 3: Convention + enforcement

- [ ] #### Task 3.1: STRATEGY.md rewrite + validator surface rule (3 SP)
  - [ ] ##### SubTask 3.1.1: Rewrite tests/STRATEGY.md "SCENARIO tags" — drop the `behaviour.rs` hardcode, state the per-surface rule + file mirroring, repoint the stdout-tee exemption at `dashboard.rs` (1 SP)
  - [ ] ##### SubTask 3.1.2: Add the surface-consistency check to `scripts/validate_feature_spec.py` + update `TAG_EXEMPT_TESTS` path (3 SP)

### Phase 4: References, bookkeeping, gate, resolution

- [ ] #### Task 4.1: Prose references and bookkeeping (3 SP)
  - `dashboard.md:82` (link → `browser_slash_menu.md`, cite 31.1–31.9, two→three commands fix); `ui_design.md:10` (glob phrasing); regenerate `tests/AGENTS.md`; amend ticket 12's `## Answer` + map ticket-12 line.
- [ ] #### Task 4.2: Full gate (3 SP)
  - `python build.py` green end-to-end; browser test count unchanged (26 across the split files).
- [ ] #### Task 4.3: Resolve ticket 20 and update the map (1 SP)
  - Fill `## Answer` (including the session's refinement of ticket 19's Q1=A: story-log interactions are the story-log feature, the slash palette is its own surface, residual chrome named `browser_dashboard`), set `Status: resolved`, append the Decisions-so-far line; create ticket 21 (Documentation: CONTEXT.md terms, diataxis reference for IF mode + options, DOC anchors; `Blocked by: 20`); clear the Documentation bullet from Not-yet-specified. The two polish fog bullets (arrival-path always-on options, per-game always-on override UI) stay.

## Test Plan

Move-only: no test logic changes, so the browser suite must hold at identical count (26 tests across the six files + residual) and pass. `validate_feature_spec.py` must report the same declared/covered counts as before the reorg (139/139 per the ticket-18 close) with 0 gaps / 0 orphans / 0 untagged / 0 surface violations. Surface-rule spot-check: temporarily tag one browser test against `story_log.md` and one HTTP test against `browser_dashboard.md`, confirm the validator fails both, revert. Compile-level check that no module registration was forgotten: validator gaps would fire, but `python build.py check` + the browser binary run confirm directly.

## Per Task/Sub Task Validation Steps

- Task 1.1/1.2: `python build.py validate-docs`; `python scripts/validate_feature_spec.py` (expect exactly the moved-tag orphans until Phase 2 lands).
- Each 2.1.x cut: re-read the live line range immediately before cutting; then `python build.py check` and `cargo nextest run --test browser` (or `python build.py nextest`) until the split binary passes.
- After 2.1.5: validator reports 0 gaps / 0 orphans / 0 untagged.
- Task 3.1.2: validator clean on the real tree + the manual mis-tag spot-check above.
- Task 4.2: full `python build.py`; confirm via `tail -n 10 "$(ls -t logs/build_*.log | head -1)"`.

## Assumptions

- Fresh id ranges continue ticket 19's migration order: options = 26, games = 27, prompt_presets = 28, worlds = 29, story_log = 30, slash_menu = 31. `browser_dashboard.md` keeps 16.5–16.7 (rename ≠ new file); gaps tolerated everywhere.
- The residual test file renames with its spec: `behaviour.rs` → `dashboard.rs`, and the exemption path follows.
- Cut line ranges verified 2026-09-12 and re-read against the live file before every edit; per-file imports are re-derived from actual usage in the cut block, not carried blindly.
- `type_into_command` moves with `slash_menu.rs` (its only users are the 17.x tests — re-verified at cut time); `open_world_edit` moves with `worlds.rs`.
- The `dashboard.md:82` two→three slash-command correction ships with the link retarget (session decision; flagged for veto).
- Historical records (`docs/plans/*`, CHANGELOG, closed ticket bodies) keep old paths/ids; only ticket 12's `## Answer` and the map's ticket-12 line are amended.
- `TAG_EXEMPT_TESTS` keeps its single entry (path updated); `invariants.rs` exemption and the quarantine ratchet (86) are untouched.
