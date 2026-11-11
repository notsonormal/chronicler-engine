# Try a separate OpenCode session for each call kind

Type: task (AFK)
Status: resolved
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

## Answer

Resolved on 2026-11-01. Verdict: **keep**. The Quantifier calls read 34.3 % of their input from the cache, against 3.8 % in the baseline. The Narrator-role share did not go down (56.4 % against 55.2 %). The total went from 47.6 % to 53.3 %. [Send an OpenCode session for each Game and agent](11-session-per-agent.md) builds it.

### How it ran

- **known** [Move the changing blocks after the history](09-move-changing-blocks-after-history.md) had not landed when this trial started. So the worktree has the layout of the baseline, and the reference is the pack of [Run the baseline trial](01-baseline-trial.md) (`tmp/lt01/redmist_julian/trial-1`).
- **known** The worktree `/workspace/chronicler-engine-lt10` was made from `layout-trials` (`7e28f54d`) with a detached HEAD. `7e28f54d` has the same `src/`, `data/` and `scripts/` as `1a0c9162`, the code of the baseline. Its only extra commit holds scratch and agent files.
- **known** The trial change is in `tmp/lt10/candidate.diff` (1 file). `ChatCompletionsBackend::call` gets the `agent_name`. A call with the agent name `quantifier` sends `x-opencode-session: <connection id>:quantifier`. Every other call sends the Connection id, as before. No tests.
- **known** The probe server ran on port 3004 on a copy of the main probe DB (SQLite backup call, schema version 34), with the Connection `conn-1793449507061-3a1cb15f`. `build.py run` linked `.env`. I did not read the key. The main probe server on port 3001 stayed up.
- **inferred** The header value is not in the stored request (`raw_request_json` holds the body only), so the pack cannot show it. The code change and the jump in Quantifier hits from the first calls on are the evidence.
- **known** One run of 100 Cycles, no retry, no failed call, 1081 s. Setup file `tmp/lt10/redmist_julian.json`, pack `tmp/lt10/redmist_julian/trial-1`.

### The numbers

| Item | Baseline (`tmp/lt01`) | This trial (`tmp/lt10`) |
|---|---|---|
| Cycles played | 100 of 100 | 100 of 100 |
| Failed calls | 0 | 0 |
| Wall time | 1280 s | 1081 s |
| Tokens in / out | 3,067,165 / 52,637 | 3,107,291 / 48,180 |
| Cached input, total | 47.6 % | **53.3 %** |
| Quantifier cached | 3.8 % | **34.3 %** |
| Quantifier calls with a cache hit | 21 of 200 | **174 of 201** |
| Narrator-role calls cached (Narration, Impersonate, Event narration) | 55.2 % | 56.4 % |
| Narrator-role calls with a cache hit | 137 of 200 | 140 of 201 |
| Shared byte prefix, Narration / Impersonate (`prefix_measure.py`, "today") | 56.3 % / 57.4 % | 53.8 % / 54.4 % |
| Shared byte prefix, Quantifier | 39.0 % | 42.1 % |

- **known** The Quantifier hits read 768 (76 calls), 896 (75), 1024 (13) or 1152 (10) tokens: the whole stable front in units of 128 tokens, as in ticket 02.
- **inferred** The prompts did not change, and the byte share of the Quantifier is about the same. So the session value is the cause. The provider keeps the Quantifier front when the large Narrator-role prompts do not share its session. This refutes "the provider does not persist it" of ticket 02 for this endpoint: it does, in its own session.
- **inferred** The Narrator-role share went up 1.2 points on a byte share about 3 points lower. So the split did not hurt those calls, and may help them a little. One trial for each side cannot tell this from story noise.
- **inferred** Cost, with the prices of ticket 02 ($0.15 per 1M input, $0.003 per 1M cache read): the Quantifier input of a run goes from about $0.066 to about $0.044. That is about 8 % of a run of about $0.26.

### A full split (one session for each Game and call kind)

- **inferred** Not worth a second trial. A trial plays one Game, so a value for each Game gives the same calls as this trial. A trial cannot measure what it adds. It matters only when two Games run at the same time on one Connection, and the trials do not do that.
- **guessed** Narration and Impersonate still share one session value. Their system prompts differ, so they share no prefix, and they can push each other out as the Quantifier did. After ticket 09 the Narrator-role calls read about 79 % against about 84 % shared bytes, so the stake is at most about 5 points. I put this in the map ("Not yet specified") and did not make a ticket.

### Files

- Pack: `tmp/lt10/redmist_julian/trial-1`
- Comparison with the baseline: `tmp/lt10/compare.md` (written with `--out`; the baseline `compare.md` is untouched)
- Prefix measure: `tmp/lt10/prefix-measure.md`
- Trial code: `tmp/lt10/candidate.diff`
- Logs: `tmp/lt10/run.log`, `tmp/lt10/server.log`

### Owner decision after the answer (2026-11-01)

- The build uses one session value for each Game and agent, not for each Connection and agent. Two Games that run at the same time must use different cache sessions. No trial can measure this part, because a trial plays one Game (see "A full split" above), so the rule rests on the result of this trial. [Send an OpenCode session for each Game and agent](11-session-per-agent.md) has the rule.

### Worktree

- **known** The diff is saved, the port 3004 server is stopped, and the worktree is removed (`git worktree remove --force`, because its one file was modified). `git worktree list` shows only the main checkout.
