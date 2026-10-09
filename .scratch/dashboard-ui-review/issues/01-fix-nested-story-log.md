# Stop the story-log fragment nesting a second #story-log

Type: task (AFK)
Status: resolved
Blocked by: —

## Question

The story-log fragment (`src/adapters/driving/http/templates.rs:25`) wraps its entries in `<div class="story-log" id="story-log">`. The shell (`assets/index.html:34`) swaps that fragment into the existing `#story-log` with `innerHTML`. The result is two elements with the same id and a bordered box inside a bordered box. What change leaves exactly one `#story-log`?

## Context

- Finding 2.3 (P1). Screenshots 01, 19.
- Before you choose between removing the wrapper and changing the swap (for example `outerHTML`), find every user of the id and class: CSS, client JS, tests, and stub fixtures (`tests/test_utils/stub_server.rs`).

## Done when

- After polls, the page has one element with id `story-log`. There is no nested box at 1280px or 480px width.
- A test is placed by `tests/STRATEGY.md`. Likely tier 1, because the fragment's shape is `curl`-observable.
- `python build.py` is green. Commit after user approval.

## Answer

**Approach: killed the wrapper in the fragment** (`src/adapters/driving/http/templates.rs`,
`NarrativeLogTemplate`). The fragment now ships bare `.log-entry` markup only; the
shell's `#story-log` div keeps sole ownership of the id, class, border, scroll
container, and polling behavior.

Why not change the swap: every delivery path swaps the fragment into the shell
div by innerHTML — the htmx poll (`hx-swap="innerHTML"`), `switchSwipe()`
(`innerHTML = html`), and `deleteMessage()` (`htmx.ajax ... swap: "innerHTML"`).
With the wrapper gone, entries land exactly where the old wrapper used to sit,
so all three flows are fixed with no JS change. `outerHTML` would have needed
client-JS edits in three places inside htmx's swap lifecycle (and `htmx.ajax`
has no outerHTML-friendly target form), so it was neither minimal nor safe.

Consumer sweep (every user of the id/class checked before editing):

- `assets/styles.css` `.story-log` rules — match the shell div only, untouched.
- `assets/index.html` client JS (`pausePolling`, `resumePolling`, triggers) —
  address `#story-log` via getElementById/htmx; with one element in the DOM
  they now bind to the right (only) element.
- Copy-consumer audit confirmed nothing depends on the fragment carrying a
  wrapper.
- `src/adapters/driving/http/templates_tests.rs` — empty-log assertion now
  asserts the render is empty (the wrapper was the only unconditional output).
- `tests/http/requires_migration/fragment.rs` — needle updated to
  `class="log-entry"` on a seeded entry.
- `tests/test_utils/stub_fixtures/story_log.html` — wrapper removed so the
  tier-2 stub matches the real fragment's shape (stub-tier tax; fixture
  regenerated).
- `src/adapters/driving/http/layout/handlers/endpoints_tests.rs` — truthy
  smoke test seeds an entry (a fresh default app renders an empty fragment).

Test placement: **tier 1 (HTTP E2E)** per `tests/STRATEGY.md` — the fragment's
shape is curl-observable. Test: `test_story_log_fragment_declares_no_log_container`
in `tests/http/story_log.rs`, tagged
`[docs/specs/story_log.md] SCENARIO: 8.5` (scenario added to the spec).

Verification: full `python build.py` gate green — fmt, clippy, guardrails,
spec-coverage, architecture, guardrail, 1634 integration, 21 browser tests,
0 failures (logs/build_20260929_210307.log).
