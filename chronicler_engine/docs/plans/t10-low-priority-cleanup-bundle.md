# T10: Low-priority Cleanup Bundle

**Parent Plan:** [abstraction-fixes-followup-superplan.md](./abstraction-fixes-followup-superplan.md)
**Status:** Planning — opportunistic (pick 3-5 per sprint)
**Date:** 2026-06-28
**Depends on:** none
**Blocks:** none
**Priority:** P3
**Findings owned:** A9, A11, D2, D7, D9, D11, N13, N16, M3, M4, B12

---

## Summary

12 untouched + cosmetic items from the original 47-finding investigation + 8 NEW cosmetic issues. All low severity, mechanical. Pick 3-5 during a sprint; each <1 hour. No structural risk.

## Items

- **A9 `push_section`** — `narrative/prompt/assembler.rs:52`; 6 callers now (was 3). Keep-or-inline decision.
- **A11 `MessageEntry` DTO mirroring** — `state/message_types.rs:16-32`. Collapse or `impl From<&Message>`.
- **D2 `empty_to_none`** — `storage/backend/helpers.rs:5`; 5 callers. Inline or `String` extension trait.
- **D7 `ActionForm` reused for `check_text_handler`** — `server/fragments/misc/text_check.rs:18`. Split `CheckTextForm`.
- **D9 `add_status_swap_headers`** — 1 caller. Inline.
- **D11 `from_row` consistency** — 4 of 9 Db* models have `from_row`. Low priority.
- **N13 `Ok(_) => unreachable!()` arms** — 4 remain (`history.rs:23,36`, `misc/swipe.rs:17,30`). Invert to `let Ok(_) = ... else { ... }`.
- **N14 `Confidence` derive `Ord`** — collapses `StatePatch::merge` 4-arm match to `min()`. Covered by T5 if A3 collapse happens.
- **N16 `list_personas` 3-line passthrough** — `application_service.rs:434-439`. Move to direct storage calls (matches `list_worlds`).
- **M3 `response_length: Option<&str>` stringly typed** — `narrative/prompt/assembler.rs:11`. Enum or token count.
- **M4 `QuantifierParseResult::is_high()` only** — add `is_low()`/`is_medium()` or replace with derived `Ord` + comparison.
- **B12 `trigger_eval.rs` cohesion** — `evaluate_triggers` + `NpcEncounterLog` CRUD helpers in same file.

## Verification

- `python build.py` — fmt + clippy + tests + coverage must pass clean for each picked item.
- Each item is <1 hour mechanical work. Verify per-item: code compiles, no new clippy warnings, existing tests pass.

## Pre-Implementation Checklist

- [ ] Pick 3-5 items per sprint. Each independently shippable.
- [ ] No structural dependencies between items — bundle / unbundle freely.
