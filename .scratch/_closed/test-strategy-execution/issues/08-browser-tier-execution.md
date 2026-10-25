# Execute the browser tier changes

Type: task (AFK execution of 07's decisions)
Status: resolved
Assigned to: wayfinder session 2026-08-06
Blocked by: 07 (resolved)

## Question

Execute the browser tier changes per the target-state design (ticket 07).
This is the mechanical work that graduates from the grilling.

## Resolution

Ticket 08 executed in 5 phases against the plan in
`docs/plans/ticket-8-browser-tier-execution.md`. All acceptance criteria
met. Final state: **52 declared / 52 covered, 0 gaps, 0 orphans, 0
format violations; 1356 pass / 2 skipped; 101/101 guardrails; 0 build
warnings.**

### Track 1 — Delete 17 browser tests

Deleted 17 move-down tests across 4 files, then deleted the files:
`tests/browser/trigger.rs` (-176), `editing.rs` (-350), `structure.rs`
(-367), `interaction.rs` (-36).

### Track 2 — Write new HTTP tests

6 new HTTP tests landed (not 9 — see Deviations):

- `tests/http/actions.rs`:
  - S1.7 `test_no_trigger_npc_produces_narration_no_event_http` —
    no-trigger NPC produces Narration with `event_header() == None`.
  - S1.8 `test_trigger_does_not_refire_on_second_encounter_http` —
    repeat action → trigger narration count ≤ 1.
- `tests/http/fragment.rs`:
  - `test_story_log_fragment_renders_action_buttons_after_actions` —
    `edit-btn` present, `delete-btn` present (≤2 after 2 actions),
    `retry-btn` absent.
  - `test_header_fragment_displays_game_title` — "Chronicler Engine"
    inside `.game-title` on `/fragment/header`.
  - `test_header_fragment_has_connection_status` —
    `id="connection-status"` on `/fragment/header`.
- (4th fragment test `test_action_area_input_has_no_required` was
  dropped — see Deviations.)

### Track 3 — Reorganize 13 keep-tests

- `tests/browser/behaviour.rs` — 6 tagged tests (16.1–16.6):
  edit-mode activate, edit-cancel restore, polling-pause, delete,
  form-static, status-update.
- `tests/browser/invariants.rs` — 7 exempt tests: scrollable,
  no-horizontal-overflow, text-wrap, element-positioning, portrait
  horizontal, portrait fixed-width, edit-textarea-height.
- `tests/browser/mod.rs` rewired: `mod behaviour; mod invariants;`.

### Track 4 — Spec + STRATEGY.md + validator

- New `docs/specs/browser.md` — 6 Given/When/Then scenarios (16.1–16.6).
- `tests/STRATEGY.md` amended: SCENARIO tags allowed in `tests/http/` +
  `tests/browser/behaviour.rs`; named exemption for `invariants.rs`.
- `scripts/validate_feature_spec.py` `TEST_DIRS` amended: added
  `tests/browser/` so `behaviour.rs` tags count as coverage.

### Track 5 — Cleanup

Removed 5 dead helpers (only used by deleted tests):
`element_exists`, `element_count`, `get_status`, `wait_for_log_entries`
(from `tests/test_utils/browser.rs`); `wait_for_non_loading_value`
(from `tests/test_utils/wait.rs`).

### Deviations (user-approved)

1. **6 new HTTP tests instead of 9.** 7 fragment assertions consolidated
   into 3 tests by setup/endpoint shape (story-log bundle = 3 assertions
   in one test; header = 2 tests; action-area = 0 — see #3). The audit
   said "9 gaps"; the consolidation covers all 7 fragment assertions
   except the dropped `no-required` one, just in fewer test functions.
2. **Optional partials #11/#14 skipped.** Ticket 07 marked them
   optional; not needed for coverage.
3. **`no-required` fragment test dropped.** Browser test
   `test_input_no_required_attribute` was inverted/buggy — asserted
   `!has_required` but the action-area template renders
   `required minlength="1"`. Git history shows commit `8e4acf5`
   flipped the assertion without updating the template; the test only
   "passed" because browser tests skip without Chrome. No HTTP
   replacement — asserting a false property would propagate the bug.
   Flagged as plan deviation per AGENTS.md; user chose option (a) drop
   it. Template `required` may itself be wrong (empty command triggers
   continuation, S1.5 — `required` would break that UX) but that's a
   separate bug, out of scope for ticket 08.
4. **No guardrail in `tests/infrastructure/guardrails/` enforces
   SCENARIO placement.** Ticket 07 assumed one exists. Enforcement is
   `validate_feature_spec.py` `TEST_DIRS` — amended that instead.
   STRATEGY.md text updated to match.

### Two-axis code review (post-implementation)

Standards + Spec review run as parallel sub-agents. Standards agent
died on 503 backend queue errors; re-spawned. Findings + fixes:

- **Standards hard violation:** `test_index_page_has_connection_status`
  was misnamed + misplaced (fetched `/fragment/header`, not `/`).
  Fixed: renamed `test_header_fragment_has_connection_status`, moved
  from `index_handler.rs` to `fragment.rs`.
- **Standards smell (Duplicated Code):** S1.7 built a raw `NpcCard`
  literal when `TestNpc::named("bartender", "Bartender")` produces the
  same shape. Fixed: use `TestNpc::named`.
- **Standards smell (naming inconsistency):** S1.7 bound
  `quantifier_provider`, S1.6/S1.8 bound `quantifier` for the same
  role. Fixed: renamed to `quantifier_provider` everywhere.
- **Spec scope creep:** `.pi/extensions/pi-permission-system/config.json`
  adds `git commit` deny rules — pre-dates ticket 8 (already in working
  tree at session start), not introduced by this work. Flagged to user;
  not a ticket-8 defect.

### Asset

- Plan: `docs/plans/ticket-8-browser-tier-execution.md`
  (and duplicate `ticket-8-execute-the-browser-tier-changes.md` —
  planning artifact, can be deduped later).

### Acceptance check

- [x] 17 browser tests deleted; 4 source files deleted.
- [x] 6 new HTTP tests landed (7 fragment assertions consolidated into
      3; 1 dropped as buggy; 2 I.5 restorations). Deviation from 9 → 6
      user-approved.
- [x] `actions.md` has 2 new scenarios (1.7 negative no-trigger, 1.8
      no-refire).
- [x] `tests/browser/` = `behaviour.rs` (6) + `invariants.rs` (7) +
      `mod.rs`.
- [x] `docs/specs/browser.md` exists with 6 scenarios (16.1–16.6).
- [x] STRATEGY.md + validator amended; `invariants.rs` has named
      exemption.
- [x] `validate_feature_spec.py` green (52/52/0/0/0).
- [x] Guardrail suite green (101/101).
- [x] Full suite green (1356 pass, 2 skipped, 0 warnings).

### Fog graduates

Ticket 14 (grilling "What browser tests are missing?") was blocked by
07, not 08 — now unblocked. No new fog surfaced from this execution.
One out-of-scope observation: the `required minlength="1"` template
attribute may be a bug (conflicts with empty-command continuation,
S1.5) — not ticketed here; belongs to a future UI/template audit.

### Context

- [Ticket 07 resolution](07-browser-tier-design.md) — the target-state
  design this ticket executes.
- [Browser tier audit asset](../test-strategy/assets/browser-tier-audit.md) —
  the 17/13 classification with per-test destinations.
- Tier rules in `tests/STRATEGY.md`.
- Plan: `docs/plans/ticket-8-browser-tier-execution.md`.
