# The free-fn guardrail fails only in the full gate

Type: grilling (HITL)
Status: open
Blocked by: —

## Question

A new free function in `src/adapters/` broke `guardrails_free_fn_location`. `clippy` and `test-pattern` passed, and only the full gate found it, so the gate ran twice (about 3 minutes each). Must the narrow loop catch a guardrail failure earlier?

## Context

Source: `/retro` 2026-11-02, the session of layout-trials ticket 11.

- **known** `logs/build_20261102_194337.log`: `adapters/driven/llm/providers/chat_completions.rs:160 [ERROR] free fn \`game_session\` outside allowed category folder`. The fix made it an associated function, and the second full gate passed.
- **known** `AGENTS.md`, Development Loop: `python build.py guardrails` costs about 10 s. The table does not say when to run it. My loop was `test-pattern … --test-target lib`, then `clippy`.
- **known** Neither reviewer reported it. `/code-review` tells the Standards reviewer to skip what tooling enforces, which is correct.
- **inferred** The guardrails are a mechanical check, so a rule in `CODING_STANDARDS.md` is the wrong fix (`/retro`, "Coding standards").

## Options to decide

- **A.** `python build.py clippy` also runs the guardrails binary (about +10 s on each run).
- **B.** One line in the Development Loop table: run `guardrails` before the full gate when a change adds a type or a function.
- **C.** Nothing: a second full gate run is a rare cost.
