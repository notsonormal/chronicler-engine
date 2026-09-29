# Stop the story-log fragment nesting a second #story-log

Type: task (AFK)
Status: open
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
