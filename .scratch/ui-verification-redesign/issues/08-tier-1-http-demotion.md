# Tier-1 rollout: demote the class-2 assertions that already have HTTP siblings

Type: task
Status: claimed
Blocked by: 06

## Question

Demote the class-2 tests whose server-derived assertions **already have, or are
one sibling away from**, HTTP coverage — per the ratified placement rule ("could
curl observe this?"). Assertions move; **wiring does not demote** — the
click→POST hop residue stays in tier 3 as the wiring guards (ticket 09).
Demoting a test before its assertions are covered at HTTP is forbidden:
coverage stays connected throughout.

Per the audit (`assets/harness-audit.md` §1's class-2 table):

**(a) Assert-and-delete candidates — verify the coverage actually carries the
asserted facts before deleting the browser copy.** `requires_migration/` hits do
not count — assert content, not just status:

- Test 16 (`options_use_click_submits_option`) — sibling `options.rs:87`
  (24.1) renders the dock; assert the Input-entry hop at HTTP level (POST the
  form `hx-headers` style the browser would).
- Test 18 (`test_delete_removes_message`) — sibling `story_log.rs:12` (8.1).
- Tests 19 (`impersonate`), 20 (`/guide`) — siblings `actions.rs` SCENARIO 1.9
  and 1.10, which already assert the same facts (`actions.rs:681`, `:731`).
  Verify, then delete the browser copies.
- Test 22 (`games_tense_change_autosaves`) — siblings `games_config.rs:21`
  (20.4) and `narrator_mode.rs:184` (23.4); add the *success-path* re-render
  assertion (23.4 covers the post; the re-render fact is asserted browser-side
  today).
- Test 23 (`preset_editor_mode_flags_roundtrip`) — siblings
  `prompt_presets.rs:167/389/745`; the duplicate→check→save request chain at
  HTTP level, minus the click handling.

**(c) `test_games_mode_switch_retargets_and_nudges` (test 25, audit class 3)**
— delete; `tests/http/narrator_mode.rs:144` (SCENARIO 23.3) is its twin.
Confirm the twin asserts both facts (perspective nudge + system-preset
retarget) before deleting.

**Not this ticket** — leave for ticket 09, per the audit §4 rule-07 ruling:

- Test 14 (`test_form_stays_static_after_submission`)
- Test 15 (`test_status_updates_during_generation`)

**Not this ticket** — the two orphaned contracts (games posture-fragment
render; world edit-form render) are **`08b`**, which writes new HTTP tests and
deletes nothing.

## Spec reconciliation is part of this ticket

Deleting a browser test does **not** by itself keep the gate green. Verified
against `scripts/validate_feature_spec.py`:

- `find_surface_violations` rejects a `browser_*.md` tag outside
  `tests/browser/` — so re-tagging the deleted test's scenario to an HTTP spec
  but leaving it in a `browser_*.md` file fails.
- `main` returns `1` if a declared scenario has no covering test — so deleting
  the browser test without re-expressing its scenario elsewhere also fails.

Both directions fail, so each demotion must reconcile the **spec** as well:
re-express the scenario in the appropriate non-`browser_*` spec (with an id
consistent with that file's scheme) and update the test tag, or retire the
scenario if its behaviour is genuinely browser-only. Let
`python scripts/validate_feature_spec.py` be the oracle; it must report
`0 gap(s)`, `0 orphan(s)`, `0 untagged`, `0 surface mismatch(es)` on exit.

Scenario→spec mapping to reconcile (browser spec on the left): 26.1
(`browser_options.md`), 30.4 (`browser_story_log.md`), 31.8/31.9
(`browser_slash_menu.md`), 27.2/27.3 (`browser_games.md`), 28.1
(`browser_prompt_presets.md`).

**Do not weaken the validator to make a move pass.** If a scenario genuinely
belongs to two tiers now, the spec says so; if it can't be asserted, surface
that in `## Answer`.

Browser files: after each move, the moved test's browser copy is deleted **in
the same change**; a browser test left asserting server content "temporarily"
is a new flake in waiting.

Record under `## Answer`: the per-test disposition table, HTTP tests added with
paths, the scenario→spec reconciliation table, `validate_feature_spec.py` output
before and after, and any test where coverage turned out thinner than the audit
claimed (stop and note it rather than deleting blind).
