# `ScriptedModel` does not fail on a scripted reply that no call uses

Type: grilling (HITL)
Status: open
Blocked by: —

## Question

`ScriptedModel` fails a test on a call that it has no reply for, but not on a reply that no call uses. In the R5 review, the reviewer read a second `.failed(Role::Trigger)` reply as unused and asked to delete it; only a trial deletion showed that a call uses it. Should a leftover reply fail the test?

- **Strict**: `ScriptedModel::drop` fails on leftover replies (as it fails on a stray call). Tests that script spare replies on purpose change.
- **Opt-in**: a method such as `expect_all_used()` that a test calls at its end.
- **Leave it.**

## Context

- Source: lead session `01a15b99` (2026-10-21); reviewer `dc7b25f3` finding 3; implementer `9d3f9cf1`, fix run, finding 3 ("the test did not script a trigger call; its calls so far: …").
- R5 implementer note: "A leftover scripted reply does not fail a test. Only a stray call does."
- File: `src/test_support/scripted_model.rs`.
