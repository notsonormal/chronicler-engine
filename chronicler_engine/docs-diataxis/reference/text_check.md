---
diataxis: reference
title: Text Check
---

> **Diátaxis mode:** Reference. The text-check feature as it is: pre-flight and manual entry points, settings gating and dictionary lifetime, the issues the checker returns (struct shape deferred to source), fail-open error handling, and the invariants the checker preserves. Reader problem: *look-up* — when does the checker run, what does it return, and what guarantees does it make.

## Overview

The text checker runs **before** player input reaches the LLM (pre-flight) and on demand from any UI button that opts in (manual). Both paths surface textual issues so the player can choose corrected, original, or cancel. The checker never rewrites input without the player's choice.

The engine reads mode and auto-check on each call; the ignored-words list is merged into the dictionary once at service construction. (Settings shape: `./startup.md`; JSON: `./data_schemas.md`.)

## Entry points

Pre-flight runs only when **both** `mode != Disabled` and `enable_auto_check = true`. The manual button bypasses auto-check and runs whenever mode is set. Both paths invoke the same port and return the same result shape.

## Issues

Each check returns zero or more issues; each carries an `IssueKind` category, a span into the player's text, a message, and an optional suggested replacement. The variant set and struct shape live at `src/application/ports/text_checker.rs` and are not restated here.

## Error handling

Fail-open on both paths. Lint failures log at the engine boundary and forward the original text; pre-flight is skipped. Settings load failures fall back to defaults and the engine continues.

## Boundaries

Three invariants the checker preserves:

- **Player text only.** The checker sees only the player's command text; it does not see the system prompt, conversation history, or state.
- **Player consent.** Corrections never apply without an explicit choice.
- **Dictionary stays local.** The in-process checker has no network path; ignored words never leave the host.

## Document References

- [ADR-011](../../docs/adr/adr-011-text-check-integration.md) — harper-core choice and the pre-flight + manual entry-point split.
- [`./startup.md`](./startup.md) — settings shape and load semantics.
- [`./data_schemas.md`](./data_schemas.md) — `text_check` settings block JSON.
