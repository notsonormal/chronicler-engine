# Follow up on small issues found during review, round 2

Type: task (AFK)
Status: open
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

## Done when

- Each item is fixed, dropped with a reason, or promoted to a ticket.
- `python build.py` is green.
