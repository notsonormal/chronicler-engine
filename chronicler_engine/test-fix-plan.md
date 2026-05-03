# Implementation Plan: Test Suite Improvements

## Overview
Address the gaps and inconsistencies identified in the Test Police review. The work is scoped to chronicler_engine tests — no production code changes except for comment cleanup.

## Architecture Decisions
- **Keep E2E tests in e2e_tests.rs** — don't move behavior tests, only pure CSS/HTTP assertions
- **Extract shared setup via closure-based helper** — matches existing `with_isolated_settings` pattern
- **Use `launch_chrome()` consistently** — already exists in test_utils.rs, e2e_tests.rs should use it
- **Add missing component tests to existing modules** — fits current `settings_tests` and `llm` test structure

---

## Task List

### Phase 1: Foundation — Extract E2E Helper & Normalize Patterns

#### Task 1: Extract `with_test_page` helper in test_utils.rs
**Description:** Add a helper that wraps the repeated e2e setup (get port, start mock server, launch browser, goto page, wait for entries) and passes the `Page` + `port` to a closure.

**Acceptance criteria:**
- [ ] `with_test_page` helper exists in `tests/test_utils.rs`
- [ ] Helper launches browser via `launch_chrome()`
- [ ] Helper waits for `#story-log .log-entry` to have at least 1 child
- [ ] Helper cleans up browser on completion
- [ ] Existing `flow_mock_tests.rs` behavior is unchanged

**Verification:**
- [ ] `cargo test --test flow_mock_tests` passes
- [ ] `cargo test --test e2e_tests` passes (before using helper — just confirming no breakage)

**Dependencies:** None
**Files likely touched:**
- `tests/test_utils.rs`

**Estimated scope:** Small (1 file, 1 function)

---

#### Task 2: Standardize browser launch and close patterns in e2e_tests.rs
**Description:** Replace all manual `Playwright::launch()` / `browser.close()` boilerplate in `e2e_tests.rs` with `launch_chrome()` + `with_test_page` (if ready) or at minimum standardize on `launch_chrome()` and `let _ = browser.close().await;`.

**Acceptance criteria:**
- [ ] All 23 e2e tests use `launch_chrome()` instead of manual launch
- [ ] All browser close calls use `let _ = browser.close().await;`
- [ ] No remaining `browser.close().await.unwrap();` calls in e2e_tests.rs

**Verification:**
- [ ] `cargo test --test e2e_tests` passes
- [ ] `grep -n "browser.close().await.unwrap()" tests/e2e_tests.rs` returns nothing

**Dependencies:** Task 1 (for `with_test_page`), or can be done in parallel if we standardize on `launch_chrome()` first
**Files likely touched:**
- `tests/e2e_tests.rs`

**Estimated scope:** Small (1 file, pattern replacement)

---

### Checkpoint: Foundation
- [ ] `cargo test --tests` passes
- [ ] No manual Playwright launch boilerplate remains in e2e_tests.rs
- [ ] `cargo clippy --tests` clean

---

### Phase 2: Missing Test Coverage

#### Task 3: Add `single_user_message` component tests
**Description:** Add tests to `tests/component_tests.rs` in the `settings_tests` module covering the new checkbox field.

**Acceptance criteria:**
- [ ] `test_settings_panel_has_single_user_message_checkbox` — verifies `single_user_message` input exists in `/fragment/settings`
- [ ] `test_add_connection_with_single_user_message` — POSTs to `/connections/add` with `single_user_message=true`, verifies connection is created with flag set
- [ ] `test_edit_connection_preserves_single_user_message` — POSTs to `/connections/{id}/edit` with `single_user_message=true`, verifies updated connection has flag set
- [ ] `test_connection_card_shows_single_user_message` — verifies connection card HTML includes indicator when flag is true

**Verification:**
- [ ] `cargo test --test component_tests settings_tests` passes
- [ ] `cargo test --test component_tests` full suite passes

**Dependencies:** None (component tests don't depend on e2e refactor)
**Files likely touched:**
- `tests/component_tests.rs`

**Estimated scope:** Small-Medium (1 file, 4 test functions)

---

#### Task 4: Add edge case tests for `merge_single_user_message`
**Description:** Add unit tests in `src/narrative/llm.rs` covering empty inputs for the merge helper.

**Acceptance criteria:**
- [ ] `test_merge_single_user_message_empty_system` — empty system, non-empty user
- [ ] `test_merge_single_user_message_empty_user` — non-empty system, empty user
- [ ] `test_merge_single_user_message_both_empty` — both empty

**Verification:**
- [ ] `cargo test --lib merge_single_user_message` passes
- [ ] `cargo test --lib` full suite passes

**Dependencies:** None
**Files likely touched:**
- `src/narrative/llm.rs` (in the `#[cfg(test)]` module)

**Estimated scope:** Small (1 file, 3 test functions)

---

#### Task 5: Move CSS-only E2E tests to component_tests.rs
**Description:** Convert `test_css_valid` and `test_scrollbar_styled` from browser-based E2E tests to HTTP-based component tests. Also evaluate `test_story_log_scrollable` — it checks computed CSS, so it may need to stay E2E or become a simpler CSS file content check.

**Acceptance criteria:**
- [ ] `test_css_valid` moved to component_tests.rs — fetches `/assets/styles.css` via HTTP and asserts content
- [ ] `test_scrollbar_styled` moved to component_tests.rs — same pattern
- [ ] Original tests removed from e2e_tests.rs
- [ ] E2E test count drops from 23 to 21

**Verification:**
- [ ] `cargo test --test component_tests` passes
- [ ] `cargo test --test e2e_tests` passes
- [ ] Total test count unchanged (moved, not deleted)

**Dependencies:** Task 2 (e2e_tests.rs should be stable before removing tests from it)
**Files likely touched:**
- `tests/component_tests.rs`
- `tests/e2e_tests.rs`

**Estimated scope:** Small (2 files, move 2-3 tests)

---

### Checkpoint: Core Coverage
- [ ] All new component tests pass
- [ ] E2E test count reduced by 2 (moved to component)
- [ ] `single_user_message` has integration test coverage
- [ ] `cargo test --tests` passes

---

### Phase 3: Polish

#### Task 6: Clean up outdated/historical comments in test files
**Description:** Fix comments that describe past changes rather than current behavior.

**Acceptance criteria:**
- [ ] `flow_mock_tests.rs:31` comment updated to explain *why* location is checked in story log, not *that* it was moved
- [ ] Scan e2e_tests.rs for similar "used to be" / "now is" comments and rephrase or remove

**Verification:**
- [ ] `cargo test --tests` passes (no code changes, only comments)
- [ ] `cargo clippy --tests` clean

**Dependencies:** None
**Files likely touched:**
- `tests/flow_mock_tests.rs`
- `tests/e2e_tests.rs`

**Estimated scope:** XS (comment edits only)

---

#### Task 7: Reduce E2E test duplication with `with_test_page` helper
**Description:** Apply the `with_test_page` helper from Task 1 to as many e2e tests as practical. This is a gradual refactor — convert tests one at a time, running the suite after each batch.

**Acceptance criteria:**
- [ ] At least 10 of the 21 remaining e2e tests use `with_test_page`
- [ ] Each converted test is individually verified before proceeding to the next
- [ ] No test behavior changes (same assertions, same page interactions)

**Verification:**
- [ ] `cargo test --test e2e_tests` passes after each batch
- [ ] Line count of e2e_tests.rs reduced measurably

**Dependencies:** Task 1 and Task 2
**Files likely touched:**
- `tests/e2e_tests.rs`

**Estimated scope:** Medium (1 file, ~10 test refactors)

---

### Checkpoint: Complete
- [ ] All tests pass (`cargo test --tests`)
- [ ] Clippy clean (`cargo clippy --all-targets -- -D warnings`)
- [ ] Coverage maintained or improved
- [ ] Plan review: verify each issue from Test Police review is addressed

---

## Risks and Mitigations

| Risk | Impact | Mitigation |
|------|--------|------------|
| `with_test_page` helper doesn't play well with async closures | Medium | Use standard async fn + callback pattern; test with 1-2 e2e tests first |
| Moving CSS tests breaks because component test HTTP client doesn't serve static files | Medium | Verify `create_app_for_testing` serves `/assets/*` before moving tests |
| Settings test lock contention when adding more settings_tests | Low | Existing `SETTINGS_TEST_LOCK` handles this; new tests follow same pattern |
| E2E refactoring introduces subtle timing differences | Medium | Run full e2e suite after each batch; no changes to assertions or wait logic |

## Open Questions

1. **Should `test_story_log_scrollable` stay E2E or move?** It checks `overflowY` computed style. This requires a real browser (or very good CSS parser). Recommend keeping it E2E unless we want to just check the CSS file contains the rule.
2. **Should we also deduplicate `flow_mock_tests.rs` and `flow_llm_tests.rs` setup?** They also repeat server startup patterns. Could be a follow-up plan.
3. **Is 10 converted e2e tests enough for Task 7, or should we convert all 21?** Converting all 21 is safe but tedious. 10+ gives us the pattern established; the rest can be done opportunistically.
