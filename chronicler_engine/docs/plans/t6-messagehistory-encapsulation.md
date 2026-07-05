# T6: MessageHistory Encapsulation

**Parent Plan:** [abstraction-fixes-followup-superplan.md](./abstraction-fixes-followup-superplan.md)
**Status:** Planning — ready
**Date:** 2026-06-28
**Depends on:** none (isolated; can run parallel to other tracks)
**Blocks:** none
**Priority:** P2
**Findings owned:** A5, N15

---

## Summary

`model/message_history.rs` exposes 6 methods that bypass the encapsulated MAX_MESSAGES cap:

- `pub fn replace(&mut self, messages: Vec<Message>)` — bypasses MAX_MESSAGES
- `pub fn retain(&mut self, f)`
- `pub fn iter_mut(&mut self)`
- `pub fn as_slice(&self)`
- `pub fn clear(&mut self)`
- `pub fn from_messages(messages: Vec<Message>) -> Self` — bypasses MAX_MESSAGES cap (N15)

The struct promises encapsulation ("Callers cannot bypass rules with direct `.push()`") but multiple bypasses remain. Only `append` enforces the 1000 cap.

## Architecture-Lens Reframe

Classic "tests want to test past the interface → module is wrong shape" (candidate #5). `replace`/`retain`/`iter_mut`/`as_slice` exist as implementation exposure for test setup. Deletion test on `replace`: complexity reappears as `clear + append loop` at callers — concentrates the MAX_MESSAGES cap-bypass bug (N15) into one place. Earns keep *only by accident* (current callers exploit the bypass). Frame removal as **interface correction**, not encapsulation cleanup.

## Key Changes

1. Remove `replace`, `retain`, `iter_mut`, `as_slice`, `clear` from public API. Replace callers with the existing controlled API: `iter()`, `last()`, `len()`, `append()`, `delete_last()`, `edit()`.
2. `from_messages`: enforce the 1000-message cap (truncate), OR rename to `from_messages_trusted` + `#[doc(hidden)]` for storage loaders only.
3. Audit callers — `ArrivalTaskContext::run()` (`init_game.rs:135` area; `history.replace(msgs)`), `context.rs:180`, `retry.rs:75`, plus storage loaders + tests.

## Decisions to Lock

- `from_messages` for storage loaders: enforce cap, or add `from_messages_trusted`?

## Blast Radius

`model/message_history.rs` + callers.

## Verification

- `python build.py` — fmt + clippy + tests + coverage must pass clean.
- After public API shrink, search for remaining uses of `replace`/`retain`/`iter_mut`/`as_slice`/`clear` on `MessageHistory` — should be zero outside the module.
- Add a test: `from_messages` with > 1000 messages → verifies cap enforcement (or rename + `#[doc(hidden)]`).
- Add a test: `replace` no longer accessible from outside the module (compile-fail test or grep test).

## Pre-Implementation Checklist

- [ ] Grep all `\.replace\(`, `\.retain\(`, `\.iter_mut\(`, `\.as_slice\(`, `\.clear\(` call sites on `MessageHistory` to enumerate migration targets.
- [ ] Grep all `MessageHistory::from_messages\(` call sites — distinguish storage loaders (trusted) from test fixtures.
