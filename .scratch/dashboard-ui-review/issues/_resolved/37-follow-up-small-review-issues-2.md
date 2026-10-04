# Follow up on small issues found during review, round 2

Type: task (AFK)
Status: resolved
Blocked by: —

## Question

Ticket 36 is resolved. Code reviews still find small non-blocking issues. Which are still true, and what fixes them? Same rules as 36: check each item first, drop what no longer holds, and promote anything bigger to its own ticket (add it to the final re-review's `Blocked by:`).

## Items

From [Copy sweep](20-copy-sweep.md):
- Scenario 20.8 (`docs/specs/games.md`) also holds the saved-games assertions; move them into their own scenario.
- The games test asserts the class `game-badge` is absent, where the spec says "no status badge"; assert the visible text.
- "No other saved games." has no referent when no game is active (unreachable in production).
- The "Interactive Fiction" literal is repeated at about 10 sites.
- `tests/http/games_fragment.rs` has a comment citing a `games_tests` module that does not exist.

From [Decide whether duplicate connection and preset names are allowed](28-decide-duplicate-names.md):
- `PromptPresetService::save_preset` checks siblings and writes under two lock acquisitions (connections check inside one lock).
- `connection_name_available` and `preset_name_available` are the same trim-and-lowercase scan; one shared helper.
- Preset uniqueness is service-only, so seeding could still introduce a duplicate.

From [Follow up on small issues found during review](36-follow-up-small-review-issues.md):
- `setStatus(label, cls)`: `cls` is an abbreviation; rename or pass one status value.
- The escaping test in `tests/http/visual_sidebar.rs` no longer pins the escaped form; also assert it.
- 32.1 counts `.image-label`, while the spec says one portrait per Character; count the portrait element.
- InMemory `delete_world` leaves the world's `characters` rows behind (SQLite cascades them).
- Nothing drives `submitRetrigger()`'s URL in a browser test.

From [Fix the weak dashboard and settings tests](34-fix-weak-dashboard-and-settings-tests.md):
- `connection_card` in `tests/http/settings.rs` duplicates `preset_card_html_slice`; generalise the helper to take a card class.
- `submit_command` (`tests/browser/stub/dashboard.rs`) duplicates the fill script in `send_action`; share the JS only.
- `StubStatus`'s doc promises an idle/phase/error triple but serves two; the `Arc<Mutex<_>>` inside `Arc<StubState>` is redundant.
- 16.8's 1.5s timer margin and its single read at 3.5s are load-sensitive: poll visibility across the window.
- 16.10 waits up to ~10s of real polling; set `StubStatus::Error` before the confirm swap so the fresh node's first poll serves it.
- The stub's Error arm omits `add_status_swap_headers`, which the real failing `/action/check` adds.
- Spec 16.6 prose names the `send_action("wait")` helper.
- A 16.9 acknowledgement race failed one gate run and passed on re-run; with the earlier stub-browser timing flake, check whether the stub browser tests share a timing assumption.

From the code review of the merged `43` / `50` / `59` / `61` batch — checked against the code and all five fixed in that batch, nothing open:
- `tests/browser/stub/swipes.rs` hand-rolled its two poll loops; both now use `wait_for_condition_async`.
- `tests/browser/stub/options.rs`'s bare 500ms sleep is now the named `SUBMIT_SETTLE` window.
- The `<span class="status error">Error: ` parse duplicated in `tests/http/actions.rs` and `tests/http/swipe_new.rs` is one `status_error_message` in `tests/http/support/http_assertions.rs`.
- The display-name default was derived at five sites, and `NewGame` carried a `display_name` beside the `name` it is derived from; the field is gone, both insert paths derive it, the v25 backfill keeps its own call, and the function is `default_display_name` — `display_name_from_key` read like `world_key`.
- `announceSwipeRestore` repeated `showError`'s pending-clear timer shape; both now use one `scheduleNoticeClear`.

## Done when

- Each item is fixed, dropped with a reason, or promoted to a ticket.
- `python build.py` is green.

## Answer

Every item was checked against the code first.

**Copy sweep (20):** scenario 20.8 split, saved-games assertions moved into a new 20.9 with its own test; the badge assertion checks the visible text; the stale comment fixed. The "Interactive Fiction" literal now lives in one `NarratorMode::display_label()`, with one shared select-options builder for the games and worlds templates and template helpers for the preset badges. *Dropped:* "No other saved games." with no active game, because it is unreachable in production (a game is always loaded or created at startup and the active game can't be deleted).
**Duplicate names (28):** one `name_is_available` (new `src/application/utils/name_uniqueness.rs`) serves both services. *Deferred:* the preset check-then-write across two lock acquisitions (storage has no single-lock check-and-write) and service-only preset uniqueness (needs a storage constraint or migration) — one candidate ticket: storage-level preset name uniqueness.
**Round 1 leftovers (36):** `setStatus` parameter is now `statusClass`; the escaping test asserts the escaped form again; 32.1 counts portrait containers; InMemory `delete_world` drops the world's characters, with an InMemory/SQLite test pair. *Deferred:* driving `submitRetrigger()` in a browser test needs a new scenario, a stub route and a button in the fixture — candidate ticket.
**Test stubs (34):** shared `card_html_slice`, shared `fill_command_input`, `StubStatus` gained a `Phase` and lost its redundant `Arc`, 16.8 polls visibility across its window, 16.10 sets the error before the confirm swap, the stub's Error arm adds the status-swap headers, 16.6 prose fixed. **Flake:** the 16.9 acknowledgement race was a late `load` poll on the fresh `#status-display` reading the stub's constant "idle" and unlocking Send before the test looked; the stub now serves a phase while sending, then idle.

**Gate:** worktree on `26c8ad94`, nothing merged since: `nextest: 1665 passed, 0 failed, 2 skipped`, browser 30 passed (`build_20261001_213503.log`). After the review rename on main: browser 30 passed (`build_20261001_220049.log`).

**Code review** (`/code-review`, ISSUES): the one must-fix was the snake_case `status_class` in a camelCase script, renamed to `statusClass`. Reviewer disagreed with a scout that called the `>Current<` assertion tautological (it matches the historical badge text exactly).

**Not fixed (judgement):** the doc comment on the shared select helper lists its consumers; `connection_name_available` is now a one-line wrapper; `card_html_slice` uses `unwrap_or(0)` where it should return `None`; a dead `Ben &#` disjunct in the escaping test; 20.9's Then says "no status badge" while the test pins the text `Current`; the InMemory `delete_world` test seeds one world, so clearing all characters would pass; 16.9 now pre-sets the phase status, so its lock reads no longer observe the acknowledgement as the spec words it.

