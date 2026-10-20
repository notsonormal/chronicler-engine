# Run the architecture review of the rebuilt code on Opus 5.5

Type: research
Status: resolved
Blocked by: —

## Question

Run steps 1 and 2 of `/improve-codebase-architecture` (`.agents/skills/improve-codebase-architecture/SKILL.md`) on the code after the Rust rebuild, as one of three independent runs (tickets 14, 15 and 16). What are the deepening opportunities in that code, and which one comes first?

- **Model:** Opus 5.5 (delegate role `reviewer`).
- **Tree:** `wf/rebuild-core` at `2885df2c` (the rebuild with the fixes of the whole-rebuild review). All three runs read this commit. Record `git rev-parse HEAD` and `git status --short -- src tests | wc -l` at the start; if `src/` or `tests/` differ from `2885df2c`, stop and report.
- **Scope:** all of `src/`, weighted to the parts the rebuild made: `application/generation/`, `application/pipeline/`, `application/narration/`, `application/agents/`, `application/games/`, `adapters/driven/storage/`, `adapters/driving/http/`, `domain/model/`, `test_support/`.
- **Blind run:** do not read `.scratch/rust-rebuild-build/assets/whole-rebuild-review/` or ticket 13. The synthesis (ticket 17) compares the runs with that review, so a run that read it would not be independent.
- **Explore yourself.** The run has no subagent tool, so step 1's scout dispatch becomes your own exploration.
- **Stop after step 2.** No grilling and no interface proposals: step 3 is HITL and belongs to ticket 13.
- **Read-only** on the code. Do not run `python build.py`.

## Done when

- The HTML report of step 2 is written to the OS temp dir, and its absolute path is in the Answer.
- `[../assets/architecture-review/opus.md](../assets/architecture-review/opus.md)` holds the same candidates in Markdown: for each candidate, the files, the problem, the solution, the benefits (locality, leverage, tests) and the recommendation strength (`Strong`, `Worth exploring`, `Speculative`), then the top recommendation. Use the `/codebase-design` terms and the `CONTEXT.md` terms.
- The Answer gives the tree (commit, and the dirty-file count under `src/` and `tests/`), the candidate count by strength, the top recommendation in one line, and the asset link.

## Answer

- Tree: `d7cf3eb0`; `git diff 2885df2c HEAD -- src tests` is empty; 0 dirty files under `src/` and `tests/`. `docs/adr/` does not exist; the run checked candidates against `arch-lint.toml` and `tests/STRATEGY.md`.
- Candidates: 8 (Strong 3, Worth exploring 4, Speculative 1): 1 deepen the Game rules, one call per generation; 2 delete the in-memory Storage adapter, SQLite `:memory:` is the one stand-in; 3 deepen the Narrator, the narration prompt rules move behind its interface; 4 move the Job plan rules from `gate.rs` to the Tail; 5 one preset module; 6 one analysis output type from the analysis hook to the Game rules; 7 the story window, one read for the story log with the Retrigger rule from the Tail; 8 one composition root, the LLM port as the only test input.
- Top recommendation: candidate 1. The Game rules take one generation and return the analysed Swipe state, the room report and the rendered event record. This moves the event, room report and event record rules out of the Job and the writer, and removes the Retrigger-direction divergence.
- Possible bugs found by reading (not confirmed by a test): the Retrigger direction is not rendered (`gate.rs:587` against `game_rules.rs:119`); `render_story_log` keeps the 50 oldest Messages; the template shows the Retrigger button on a Narration only, but `Tail::retrigger_allowed` also allows an Impersonate Input.
- Asset: [../assets/architecture-review/opus.md](../assets/architecture-review/opus.md). HTML report: `/tmp/architecture-review-arch-opus-20261019T232823Z.html` (session `959d0d4d`, 8.2 min).
