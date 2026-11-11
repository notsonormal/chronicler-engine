# Run the architecture review of the rebuilt code on Qwen 3.8

Type: research
Status: resolved
Blocked by: —

## Question

Run steps 1 and 2 of `/improve-codebase-architecture` (`.agents/skills/improve-codebase-architecture/SKILL.md`) on the code after the Rust rebuild, as one of three independent runs (tickets 14, 15 and 16). What are the deepening opportunities in that code, and which one comes first?

- **Model:** Qwen 3.8 (delegate role `judge-qwen`).
- **Tree:** `wf/rebuild-core` at `2885df2c` (the rebuild with the fixes of the whole-rebuild review). All three runs read this commit. Record `git rev-parse HEAD` and `git status --short -- src tests | wc -l` at the start; if `src/` or `tests/` differ from `2885df2c`, stop and report.
- **Scope:** all of `src/`, weighted to the parts the rebuild made: `application/generation/`, `application/pipeline/`, `application/narration/`, `application/agents/`, `application/games/`, `adapters/driven/storage/`, `adapters/driving/http/`, `domain/model/`, `test_support/`.
- **Blind run:** do not read `.scratch/_closed/rust-rebuild-build/assets/whole-rebuild-review/` or ticket 13. The synthesis (ticket 17) compares the runs with that review, so a run that read it would not be independent.
- **Explore yourself.** The run has no subagent tool, so step 1's scout dispatch becomes your own exploration.
- **Stop after step 2.** No grilling and no interface proposals: step 3 is HITL and belongs to ticket 13.
- **Read-only** on the code. Do not run `python build.py`.

## Done when

- The HTML report of step 2 is written to the OS temp dir, and its absolute path is in the Answer.
- `[../assets/architecture-review/qwen.md](../assets/architecture-review/qwen.md)` holds the same candidates in Markdown: for each candidate, the files, the problem, the solution, the benefits (locality, leverage, tests) and the recommendation strength (`Strong`, `Worth exploring`, `Speculative`), then the top recommendation. Use the `/codebase-design` terms and the `CONTEXT.md` terms.
- The Answer gives the tree (commit, and the dirty-file count under `src/` and `tests/`), the candidate count by strength, the top recommendation in one line, and the asset link.

## Answer

- Tree: `d7cf3eb0`; `src/` and `tests/` identical to `2885df2c`; 0 dirty files under `src/` and `tests/`. `docs/adr/` does not exist.
- Candidates: 5 (Strong 1, Worth exploring 3, Speculative 1): 1 one type family for the analysis hook; 2 the storage record seam has one real adapter (the `no_in_memory` arms; string-keyed read failure injection against the typed `TxWrite`); 3 three projections of one story entry (`StoryRow`, `MessageEntry`, `StoryEntry`); 4 prompt wording (`continuation_user_message`) leaks into the Job across the Narrator seam; 5 `PersonaCatalogue` and `WorldCatalogue` are shallow pass-throughs, kept by a guardrail (deepen or leave).
- Top recommendation: candidate 1. The Quantifier's answer goes `QuantifierResult` to `StatePatch` to a new `QuantifierResult` with loss; `Confidence` and `QuantifierConfidence` are two enums with `From` both ways; `SceneState.quantifier_confidence: Option<String>` is a third, dead form. One type family deletes them, and the Quantifier seam cases pin the change.
- Asset: [../assets/architecture-review/qwen.md](../assets/architecture-review/qwen.md). HTML report: `/tmp/architecture-review-arch-qwen-20261019-233248.html` (session `727c691e`, 16.6 min).
