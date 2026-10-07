# Decide how a test reads a response body

Type: grilling (HITL)
Status: open
Blocked by: —

## Question

The quarantined tests under `tests/http/requires_migration/` read a body with a raw byte literal, `axum::body::to_bytes(response.into_body(), 1024)`. [Split Settings into Connections and Text Check sub-tabs](../dashboard-ui-review/issues/68-split-settings-sub-tabs.md) changed the delete refusal to return the whole Settings panel; three of those literals then truncated the body and panicked, and the agent bumped them to 16384 by hand.

The non-quarantined HTTP tests use a shared helper instead. Twenty-plus raw literals remain in `tests/`.

How should a test read a body?

- **Ban the literal.** A guardrail flags `to_bytes(..., <literal>)` in `tests/` and requires the shared helper; migrate the quarantine files in the same ticket.
- **One shared constant.** Keep `to_bytes`, but source the limit from one named constant.
- **Leave it.** The truncation panic names the limit, so the fix is local and cheap.

## Context

- `tests/http/requires_migration/connections.rs:243`, `:289` — `to_bytes(..., 1024)`.
- `tests/http/requires_migration/text_check.rs:63` — `to_bytes(..., 1024)`.
- The t68 session bumped three limits to `16384` after `LengthLimitError` at `connections.rs:164`.
- `tests/infrastructure/guardrails/` — the guardrail suite, whose rules are `pub fn check_<name>`.
