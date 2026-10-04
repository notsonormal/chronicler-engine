# 15 — One fragment error policy for the HTTP adapter

Type: grilling
Status: open
Blocked by: (none)
Assignee: (unclaimed)

## Question

Do we commit to one error-policy module in the HTTP adapter that maps
`(error, fragment context)` to status code, disclosure markup and `HX-Reswap`
— with handlers returning only a body or an `ApplicationError` — and if so,
what is that module's interface?

## Background

This is **candidate G** of the 2026-10-04 review, rated **Worth exploring**.
See `assets/architecture-review-2026-10-04.html`, card G.

The friction: error policy is decided per handler, across three helper homes
and four handler return conventions.

- **Helper homes.** `http/error.rs` (`IntoResponse for ApplicationError`, 4
  arms), `utils/error.rs` (`error_fragment`, `render_error`,
  `error_disclosure`, `raw_error_detail`, `generation_error_summary`,
  `error_fragment_response`, `error_response`), `utils/response.rs` (6 pub fns
  plus `html_escape`), and `utils/fragment.rs` (500 + `render_error` +
  `HX-Reswap: none`).
- **Return conventions.** `Result<_, ApplicationError>` (~10 handlers),
  `Response<Body>` (~24), `Html<String>` (~10), and `String`/`Json` in
  `debug/handlers/debug.rs`.
- **Conflicting disclosure.** `utils/error.rs` maps `Validation → 400`, but
  `games/handlers/games.rs` renders `is_user_displayable()` errors as a 200
  fragment, and `worlds/handlers/worlds.rs` inlines its own
  `<li class="world-item">` error wrapper.
- **Mislabelled errors.** `AppState`'s 7 `render_*` methods wrap failed read
  queries in `EngineError::Config` (9 sites in `app_state.rs`), via a
  `render_error_context` helper that is called 15 times.

Deletion test: deleting `utils/error.rs` brings `error_fragment(...)` back
into about 10 handlers, so the helpers earn their keep. The *policy*
duplication does not.

## What this ticket resolves

- **Commit or reject.**
- **Policy.** Which `ApplicationError` kinds map to which status, swap
  behaviour and markup. In particular: is a user-displayable Validation error
  a 400 or a 200 fragment?
- **Interface.** One mapping function, an extractor, or a response type;
  whether handlers converge on one return convention.
- **Read-failure kind.** What replaces `EngineError::Config` for failed
  fragment reads.
- **What survives.** Handler unit tests and `tests/http/**` that assert on
  status codes.

## Constraints

- Status codes and `HX-Reswap` behaviour are observable HTTP contract. Check
  `docs/specs` and the browser tests before changing any mapping.
- Decision ticket, no implementation.

## Notes

- Resolution uses `/grilling`.
- Related: ticket 11 (status strip uses `error_disclosure`) and the map's fog
  "HTTP adapter cohesion" (`AppState`'s 21 public items).
- Extra friction seen in passing, not in scope here: `worlds.rs` repeats a
  5-line "list worlds + games + render panel" block twice and re-parses
  `MapDef` / `StartingScenario` JSON in the adapter.
