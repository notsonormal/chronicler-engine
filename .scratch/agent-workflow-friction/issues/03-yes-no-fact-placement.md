# Yes/no on placing nine facts where they are read

Type: grilling (HITL)
Status: open

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
