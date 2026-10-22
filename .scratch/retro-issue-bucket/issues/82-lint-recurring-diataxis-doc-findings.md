# Lint the mechanical part of the recurring `docs/diataxis/` review findings

Type: grilling (HITL)
Status: open
Blocked by: —

## Question

All 8 slice reviews of the response build found the same doc findings in `docs/diataxis/`: negative explaining, code-indexing (signature and field lists), and edge cases or copy text that belong in a spec ("Where a fact lives", `docs/AGENTS.md`). From R4 on, the implementer brief named these rules, and they still came back; each cost a fix round. Which part should a check catch?

- **`validate_docs.py` rule**: flag a backticked signature (`-> ` or `(name: Type`) in `docs/diataxis/`, outside fenced code.
- **Vale rule** (`scripts/vale_lint.py`): flag "because" in a doc with `diataxis: reference` (a Reference doc gives no reasons).
- **Both.**
- **Leave it**: the review is the intended place for these judgement calls.

## Context

- Source: lead session `01a15b99` (2026-10-21); reviewers `d871f601` (R1: 5, 6, 13), `ce884bd5` (R2/R3: 3, 6, 7), `94f8e7f0` (R4: 1, 3, 4), `dc7b25f3` (R5: 2, 7), `dd001baa` (R6: 5, 7, 10), `77cb5e82` (R7: 10, 11), `8c9df275` (R8: 2, 3), `7790d891` (R9: 5, 7).
- Example (R4, finding 1): `game_flow.md` §Game Rules listed the four parameters of `game_rules::apply` and the fields of `RulesOutcome`.
- Example (R7, finding 11): `game_flow.md` held "because a stop or a reset ended it" in a Reference doc.
- More evidence, the Autoplay build (2026-10-21, `docs/plans/autoplay-plan.md`): both slice reviews found the same kinds in `game_flow.md` again, after one fix round each.
  - Slice 1, reviewer `c1a376e4`, finding 2: the "The Job Entry and Busy" section listed driver mechanics (one blocking task, the cycle count, "the run hold goes in the same hold that counts the last cycle") and a `GameWriter::config_write` mechanics sentence. That is code-indexing. Its finding 3: "hold" named two things (one critical section of the registry, and the run hold), against the STE rule "same term for the same thing".
  - Slice 2, reviewer `9900936f`, finding 4: a run-hold sentence carried a colon clause inside a list ("A run hold names the run, the cycles of the run that ended: each ended when …, and the budget."). That breaks STE.
  - Both were fixed in the implementer session `f3f67647` / `26838475`. Neither kind is caught by the two checks in the Question: a signature pattern or "because" in a Reference doc.
