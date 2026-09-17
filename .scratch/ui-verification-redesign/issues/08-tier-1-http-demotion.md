# Tier-1 rollout: demote the class-2 assertions to HTTP contract tests

Type: task
Status:
Blocked by: 06

## Question

Move the 12 class-2 tests' server-derived assertions to the HTTP tier, per the
ratified placement rule ("could curl observe this?"). Assertions move;
**wiring does not demote** — the click→POST hop residue stays in tier 3 as the
wiring guards (ticket 09). Demoting a test before its assertions are covered
at HTTP is forbidden: coverage stays connected throughout.

Per the audit (`assets/harness-audit.md` §1's class-2 table), each test falls
into one of three cases; for each, extend the existing HTTP siblings or add the
missing test:

**(a) Assert-and-delete candidates — HTTP coverage already exists or is
one sibling away.** Verify the coverage actually carries the asserted facts
before deleting the browser copy (`requires_migration/` hits do not count —
assert content, not just status):

- Test 16 (`options_use_click_submits_option`) — sibling `options.rs:87`
  (24.1) renders the dock; assert the Input-entry hop at HTTP level (POST the
  form `hx-headers` style the browser would).
- Test 18 (`test_delete_removes_message`) — sibling `story_log.rs:12` (8.1).
- Tests 19 (`impersonate`), 20 (`/guide`) — sibling `actions.rs:730` region
  (1.10).
- Test 22 (`games_tense_change_autosaves`) — siblings `games_config.rs:21`
  (20.4) and `narrator_mode.rs:184` (23.4); add the *success-path* re-render
  assertion (23.4 covers the post; the re-render fact is asserted browser-side
  today).
- Test 23 (`preset_editor_mode_flags_roundtrip`) — siblings
  `prompt_presets.rs:167/389/745`; the duplicate→check→save request chain at
  HTTP level, minus the click handling.
- Test 14 (`test_form_stays_static_after_submission`) — **no**: per the audit
  §4, rule 07 already placed this browser-side for wiring reasons. Leave for
  ticket 09.
- Test 15 (`test_status_updates_during_generation`) — **no**: same ruling.
  Leave for ticket 09.

**(b) The two orphaned contracts — new HTTP tests (this is the coverage gain
the audit named):**

- Test 21 (`#game-posture-controls` renders populated selects) — GET the games
  posture fragment, assert the selects render with current values.
- Tests 24/26's server-derived assertions — the full edit-form render and the
  `Saved` fragment; the posture POST contract itself lands in ticket 06's slice
  — extend from there rather than duplicating. The surviving browser wiring
  check's final form is owned by ticket 09 (seeded by ticket 06's slice); do
  not re-create it here.

**(c) `test_games_mode_switch_retargets_and_nudges` (test 25, audit class 3)**
— delete; `tests/http/narrator_mode.rs:144` (SCENARIO 23.3) is its twin.

Browser files: after each move, the moved test's browser copy is deleted **in
the same change**; a browser test left asserting server content "temporarily"
is a new flake in waiting.

Record under `## Answer`: the per-test disposition table, HTTP tests added with
paths, and any test where coverage turned out thinner than the audit claimed
(stop and note it rather than deleting blind).
