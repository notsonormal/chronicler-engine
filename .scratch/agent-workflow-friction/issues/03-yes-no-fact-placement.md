# Yes/no on placing nine facts where they are read

Type: grilling (HITL)
Status: resolved

## Question

Each item puts one specific fact into text that is loaded at the moment it is needed. This is more reliable than advice, but not guaranteed. F02 shows the same rule failing in a skill body. Answer yes or no for each:

1. A pointer in `AGENTS.md` to `CODING_STANDARDS.md` (F01). Commit `62072d5a` removed it during a deliberate trim, and every session pays for each line in that file.
2. The no-build rule pasted into both `code-review` §4 sub-agent brief templates (F02). These are pasted verbatim into the reviewer's prompt.
3. `session_search` and the session-JSONL directory named in `retro` step 2 (F03).
4. "Scenario IDs are scoped per spec file" in `tests/STRATEGY.md § SCENARIO tags` (F04).
5. Trigger phrases in the `tdd` description for adding, strengthening, or de-tautologizing tests (F05).
6. Usage-evidence pruning in the `chronicler-docs-hygiene` description (F06).
7. The `--cleanup` scope stated in the `build.py` docstring and in `ENVIRONMENT.md` (F14).
8. "Return the screenshot inline, never a URL or path" in `chronicler-ui-investigator` § Capture (F27).
9. "Repo-authored skills get no `skills-lock.json` entry" in `writing-for-agents` (F35).

## Context

- Findings F01–F06, F14, F27 and F35 are in [findings.md](../assets/findings.md). The ✔ marks were checked on 2026-10-04, so re-check before editing.
- If the answer is no, the finding is dropped. Example reasons: "AGENTS.md stays trimmed", or "the rule already failed in a skill body".
- Use `/writing-for-agents` for each yes.

## Done when

- All nine have a yes or no in the `## Answer`.
- The yeses are applied in one change, in this session or in one graduated ticket, cut case by case (ticket 01 set no sizing rule).

## Answer

Grilled item by item on 2026-10-05. Facts re-checked the same day: all nine gaps still hold.

1. **F01 — yes.** One line under `AGENTS.md § Documentation Index` pointing at `CODING_STANDARDS.md` (implementation, comment, review, testing rules).
2. **F02 — no.** The §4 briefs stay as they are, even though the sub-agents never load the line-15 rule; dropped by user decision.
3. **F03 — no.** `retro` step 2 stays as is; dropped by user decision.
4. **F04 — no.** `tests/STRATEGY.md § SCENARIO tags` stays as is; dropped by user decision.
5. **F05 — no.** The `tdd` description stays as is; dropped by user decision.
6. **F06 — no.** A description trigger would promise a check the body lacks, and a new phase is skill-body advice (ruled out by Q7).
7. **F14 — yes.** The `build.py` docstring and `--help` state the real scope: `--cleanup` deletes this checkout's whole target dir and the machine-wide `<tmp>/chronicler_test_ports` lock dir that every checkout shares, so don't run it while another checkout is testing. No behaviour change; no `ENVIRONMENT.md` edit.
8. **F27 — no.** The look-at-the-shot rule in § Capture already covers it; dropped by user decision.
9. **F35 — no.** `writing-for-agents` is upstream (`mattpocock/skills`); dropped by user decision.

**Applied (one change, this session):**
- `AGENTS.md § Documentation Index`: "`CODING_STANDARDS.md` holds the implementation, code-comment, code-review and testing rules. Read it before writing or reviewing code."
- `build.py` module docstring and `--cleanup` help: the real scope as in item 7. The old "tests are already concurrency-safe" sentence is removed, because `--cleanup` breaks it. `python build.py py-tests` passes.

Nothing graduates: no new tickets. A behaviour fix for `--cleanup` (delete only dead-PID locks) was considered and not ticketed, since no failure has been seen.
