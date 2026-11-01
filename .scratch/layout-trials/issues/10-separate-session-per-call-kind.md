# Try a separate OpenCode session for each call kind

Type: task (AFK)
Status: open
Blocked by: —

## Question

Does the cached share go up when the Quantifier calls send their own OpenCode session value, apart from the Narrator-role calls?

## Context

- Target part: the cache. The cached share alone decides (map Notes). Also look at the Quantifier share and the Narrator-role share separately: the gain can show in one and a loss in the other.
- Known, `src/adapters/driven/llm/providers/chat_completions.rs`: an OpenCode Go call sends the header `x-opencode-session` with the Connection id ("one session per connection"). The trials use one Connection for every call kind, so the Narration, Impersonate and Quantifier calls of every Game share one session value. `LlmBackendType::session_header` in `src/domain/model/llm_backend.rs` names the header. OpenRouter gets a `session_id` in the body, one for each Game (`sends_game_session`).
- Known, pi (`@earendil-works/pi-ai/dist/providers/opencode-headers.js`): pi sends the same header with its session id, one for each conversation. Its comment calls it "OpenCode's required per-conversation routing header".
- Inferred (owner, 2026-11-01): if the session value pins calls to one upstream server, the large Narrator-role prompts (3000 to 18 000 tokens) can push the Quantifier front out of that server's cache between two Quantifier calls. That fits [Find why the Quantifier calls read so little from the cache](02-quantifier-cache.md): an identical front hits on only 15.3 % of calls, and the time gap does not matter. Not known: OpenCode Go has no documentation for the header, so it may only route.
- `prefix_measure.py` cannot screen this candidate, because the bytes of the prompts do not change. So the trial is the measure. This is an exception to the map note "A cache candidate is measured before its trial".
- The stake is small. A perfect Quantifier hit rate would cut 9.1 % of the cost of a run (ticket 02). A gain on the Narrator-role calls (79 % cached against about 84 % shared bytes in [Try the changing blocks after the history](03-changing-blocks-after-history.md)) would count too.

## Steps

1. Make a linked worktree with a detached HEAD from `1a0c9162` (the code of [Run the baseline trial](01-baseline-trial.md)). If [Move the changing blocks after the history](09-move-changing-blocks-after-history.md) has landed, use `layout-trials` instead, and use the pack of ticket 03 (`tmp/lt03/redmist_julian/trial-1`) as the reference: its code is the same layout.
2. Trial-only change: the Quantifier calls send `x-opencode-session: <connection id>:quantifier`. The `agent_name` of `LlmProvider::complete` is `"quantifier"` for these calls. The Narrator-role calls keep the Connection id. No tests.
3. Start a probe server on its own port with a copy of the main probe DB, and run one trial of 100 Cycles under `tmp/lt10/` (map Notes, "How a trial runs" and "Worktree limits").
4. `trial.py compare` writes beside the first pack. Give `--out tmp/lt10/compare.md` so that it does not overwrite the reference pack's `compare.md`.
5. Remove the worktree.

Done when the answer gives the cached share of both trials (in total, for the Quantifier calls and for the Narrator-role calls), the verdict (keep or drop), and whether a full split (one session for each Game and call kind, as pi does for each conversation) is worth a second trial. A kept candidate gets a new task ticket (map Notes).
