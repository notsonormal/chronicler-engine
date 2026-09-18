# Tier-1 rollout: demote the class-2 assertions that already have HTTP siblings

Type: task
Status: resolved
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

## Answer

Seven browser tests demoted or deleted. Four HTTP tests added first, then the
browser copies removed in the same change as their spec reconciliation. The
wiring residue (click → POST hops) was not touched: it stays in tier 3 for
ticket 09.

### Per-test disposition

| # | Browser test (scenario) | Disposition | Covering HTTP test |
|---|---|---|---|
| 16 | `test_options_use_click_submits_option` (browser_options 26.1) | delete | `tests/http/options.rs:347` `test_using_offered_option_submits_as_input_http` (options.md 24.13) |
| 18 | `test_delete_removes_message` (browser_story_log 30.4) | delete | `tests/http/story_log.rs:128` `test_delete_removes_entry_from_fragment_http` (story_log.md 8.4) |
| 19 | `test_slash_impersonate_produces_input_entry` (browser_slash_menu 31.8) | delete | existing `tests/http/actions.rs:683` SCENARIO 1.9 |
| 20 | `test_slash_guide_does_not_persist_input_entry` (browser_slash_menu 31.9) | delete | existing `tests/http/actions.rs:730` SCENARIO 1.10 |
| 22 | `test_games_tense_change_autosaves_and_rerenders` (browser_games 27.2) | delete | `tests/http/games_config.rs:99` `test_posture_autosave_rerenders_fragment_with_new_tense_http` (games.md 20.7) |
| 23 | `test_preset_editor_mode_flags_roundtrip` (browser_prompt_presets 28.1) | delete | `tests/http/prompt_presets.rs:810` `test_allowed_modes_duplicate_edit_save_chain_http` (prompt_presets.md 21.27) |
| 25 | `test_games_mode_switch_retargets_and_nudges` (browser_games 27.3, audit class 3) | delete | existing `tests/http/narrator_mode.rs:144` SCENARIO 23.3 |

### Coverage check, per test

- **16** — the browser test clicked `.option-btn` and counted log entries. The
  Use handler is `useOption` in `assets/index.html`, which fills
  `#command-form input[name=command]` with the button's `textContent` and calls
  `requestSubmit()` on a form whose `hx-post` is `/action/check`. The new test
  POSTs `/action/check` with that text, so it performs the same request. It
  asserts the persisted `Input` entry and a following `Narration`, which is
  stronger than the browser copy's DOM counts.
- **18** — the browser test clicked `.delete-btn`. `deleteMessage()`
  (`assets/index.html:336`) does `fetch("/history/delete", {method:"POST"})`
  then re-fetches `/fragment/story-log`. The new test performs both hops and
  asserts exactly one fewer rendered `.log-entry`. The pre-existing 8.1 asserts
  message-store facts but never the rendered count, so it did not cover this.
- **19/20** — SCENARIO 1.9/1.10 assert the message store and input-buffer
  status (`imBuffer.status is Idle`), which the browser copies never checked.
  Strictly stronger; clean delete.
- **22** — the audit expected `20.4` plus `23.4` to carry this. Neither did:
  `20.4` is the storage-failure path (500 error span) and `23.4` is about a
  *deliberate* perspective surviving a mode switch. No test asserted the
  success-path re-render of the tensed fragment, so the new 20.7 was needed.
- **23** — the audit pointed at `prompt_presets.rs:167/389/745`. Those cover
  the edit form, a save, and the panel's mode-gated buttons as separate
  scenarios; none ran the duplicate → edit-form-flags → save chain, and none
  asserted the post-save card offers both activation buttons. New 21.27 covers it.
- **25** — `narrator_mode.rs:144` (23.3) asserts both facts the browser copy
  asserted: the perspective nudge (`value="second" selected`) and the system
  preset retarget (`value="system_if_default" selected`), plus that the next
  prompt uses the IF preset. Twin confirmed; deleted.

### HTTP tests added

| File | Test | Scenario |
|---|---|---|
| `tests/http/options.rs` | `test_using_offered_option_submits_as_input_http` | options.md 24.13 |
| `tests/http/story_log.rs` | `test_delete_removes_entry_from_fragment_http` | story_log.md 8.4 |
| `tests/http/games_config.rs` | `test_posture_autosave_rerenders_fragment_with_new_tense_http` | games.md 20.7 |
| `tests/http/prompt_presets.rs` | `test_allowed_modes_duplicate_edit_save_chain_http` | prompt_presets.md 21.27 |

### Spec reconciliation

Each deleted browser scenario was re-expressed in the HTTP spec that owns its
behaviour and retired from the `browser_*` file. This is required in both
directions: a `browser_*` tag outside `tests/browser/` is a surface violation,
and a declared scenario with no covering test is a gap.

| Browser spec scenario | Action | New location |
|---|---|---|
| `browser_options.md` 26.1 | retire (re-expressed) | options.md 24.13 |
| `browser_story_log.md` 30.4 | retire (re-expressed) | story_log.md 8.4 |
| `browser_slash_menu.md` 31.8, 31.9 | retire (already in actions.md 1.9/1.10) | — |
| `browser_games.md` 27.2 | retire (re-expressed) | games.md 20.7 |
| `browser_games.md` 27.3 | retire (already in narrator_mode.md 23.3) | — |
| `browser_prompt_presets.md` 28.1 | retire (re-expressed) | prompt_presets.md 21.27 |

`docs/specs/browser_prompt_presets.md` became empty and was deleted with its
browser module. `tests/browser/{slash_menu,story_log,prompt_presets}.rs` became
empty and were deleted with their `mod.rs` entries.

### Validator, before and after

Before (at base `c9f9a96`): `141 declared, 141 covered, 0 gap(s), 0 orphan(s),
0 untagged, 0 surface mismatch(es), quarantine 86/86`.

After: `138 declared, 138 covered, 0 gap(s), 0 orphan(s), 0 untagged, 0 surface
mismatch(es), quarantine 86/86`.

141 + 4 added − 7 retired = 138. The validator was not weakened.

### Verification

Full gate green: clippy OK, **1596 integration passed / 0 failed / 2 skipped**,
**19 browser passed / 0 failed**, architecture 1 passed, guardrails 129 passed.

Browser tier went 26 → 19 (−7), matching the seven deletions exactly.

### Findings

1. **Millisecond id collision in `generate_preset_id()`.** The function
   (`src/adapters/driving/http/utils/handler_helpers.rs:29`) derives a preset id
   from `SystemTime::now().as_millis()`. A create followed by a duplicate inside
   one millisecond produces the same id, so the copy overwrites the source
   instead of adding a preset. I hit this in the new 21.27 test. The test now
   seeds its source with a fixed id to stay deterministic. **The underlying bug
   is real and unfixed** — it affects any two preset writes in the same
   millisecond, including a double-click on Create or Duplicate. Out of scope
   here; recommend a separate ticket.
2. **The audit's sibling pointers for tests 22 and 23 were optimistic.** For 22
   it named `20.4`/`23.4`; neither asserts the success-path re-render. For 23 it
   named three scenarios; none runs the chain or asserts the post-save card. I
   added the missing coverage rather than deleting blind, as the ticket
   instructed. Both are recorded above.
