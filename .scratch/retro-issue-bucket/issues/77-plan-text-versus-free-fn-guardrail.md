# Plan text asks for free functions that the free-fn guardrail refuses

Type: grilling (HITL)
Status: open
Blocked by: —

## Question

The build plan of the response asks for "a pure function" or "a free helper fn" in several slices. The free-fn-location guardrail refuses a free fn outside its allowed places, so each implementer found out at gate time and made an associated fn instead. What should change?

- **Plan rule**: the plan template (or `CODING_STANDARDS.md`) says that a new helper is an associated fn unless its module is on the guardrail's allowlist, and names the allowlist.
- **Guardrail**: the guardrail allows free pure fns under `domain/model/`, so the plan text and the code agree.
- **Leave it**: the guardrail failure message is clear enough, and the fix costs one gate run.

## Context

- Source: lead session `01a15b99` (2026-10-21), build of R1 to R9 of `docs/plans/rust-rebuild-response-build-plan.md`.
- R3: `StoryWindow::room_headers` is an associated fn "because a free fn failed the free-fn-location guardrail" (implementer `2858e33d`).
- R6/R9: `EngineError::count_of_games` is associated for the same reason (implementer `41d0d6c8`).
- R8: plan §10 and ticket 20 Q12 allow "a free helper fn that builds an input from values"; the guardrail bans free fns in `pipeline/` (implementer `f08bc2e1`).
