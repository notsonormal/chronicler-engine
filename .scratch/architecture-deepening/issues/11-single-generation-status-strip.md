# 11 — One generation-status strip for the action area and the poll

Type: grilling
Status: resolved
Blocked by: (none)
Assignee: (unclaimed)

## Question

Do we commit to one status-strip module in the HTTP adapter that both the
action area and the `GET /status/generating` poll render through — and if so,
what is its interface, and where does it read generation status from?

## Background

This is **candidate C** of the 2026-10-04 review. See
`assets/architecture-review-2026-10-04.html`, card C.

The friction: the same three-way branch (error → in-progress phase → idle) is
implemented twice and copied a third time:

- `src/adapters/driving/http/layout/handlers/endpoints.rs` (poll): loads state
  through `state.message_service.load_expecting_valid_state()`, then
  `error_disclosure(... generation_error_summary ...)`, else
  `phase.as_endpoint_str()`, else `"idle"`.
- `src/adapters/driving/http/view_models.rs` (`ActionAreaViewModel::new`): the
  same `error_disclosure` call, else
  `<span class="status thinking">{phase.display_text()}</span>`, else
  `<span class="status ready">Ready</span>`. The view model builds raw markup
  in Rust and passes it to the template as `SafeHtml`.
- `tests/test_utils/stub_server.rs`: restates the poll shape.

The poll also skips `GameViewQuery`. `AppState` reads generating status
through `game_view_query.get_generating_status()`, while the poll goes to
`message_service` directly.

Deletion test: delete the strip from either site and the same branch
reappears in the other and in the stub.

## What this ticket resolves

- **Commit or reject.**
- **Interface.** A status view value plus one render function? A template?
  Do the poll and the action area return the same markup, or one value
  rendered two ways (`as_endpoint_str` vs `display_text`)?
- **Read path.** Whether the poll moves onto `GameViewQuery`.
- **Stub.** Whether `stub_server.rs` imports the production rendering, and
  whether that is allowed from `tests/test_utils`.
- **What survives.** Which `tests/http/**` and view-model tests cross the new
  interface unchanged. `tests/http/support/http_requests.rs` names
  `/status/generating` as the status read seam.

## Constraints

- `arch-lint.toml`: the HTTP layer must not import storage.
- The poll response is part of the HTTP contract the browser tests drive. Any
  markup change needs a spec check in `docs/specs`.
- Decision ticket, no implementation.

## Notes

- Resolution uses `/grilling`.
- Related: ticket 09 (status/phase transitions) and ticket 15 (error policy:
  `error_disclosure` is one of its helpers).
- Explanation context: `docs/diataxis/explanation/two-state-channels.md`
  ("UI display reads the persisted status").

## Comments

**2026-10-14, from [Decide where the new design of the game-state lifecycle makes sense](../../turn-state-model/issues/24-decide-fix-or-rewrite.md).** In the new design, `GameViewQuery::status` holds the one status rule ([sketch 25](../../turn-state-model/assets/sketch-25-new-design.md) §3.4). The strip that renders it is in the HTTP adapter, outside that design, so this ticket's rendering question stays open.

## Answer

Closed 2026-10-16, superseded by [the Rust rebuild](../../platform-choice/map.md). Ruling of [Research: sort the open architecture-deepening tickets against the new design](../../platform-choice/issues/04-research-sort-architecture-deepening.md), [asset](../../platform-choice/assets/research-04-architecture-deepening-sort.md) §2.5: **settled.** One status rule in `GameViewQuery::status` (target design §2.3, §6; sketch 25 §3.4); the poll moves onto it, and checks busy before the failure line. The poll and the action area render one `Status` value into two HTTP contracts, so the rest is not a deepening.
