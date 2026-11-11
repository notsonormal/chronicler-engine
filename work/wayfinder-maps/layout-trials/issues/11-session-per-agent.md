# Send an OpenCode session for each Game and agent

Type: task (AFK)
Status: resolved
Blocked by: 10

## Question

Make the kept candidate of [Try a separate OpenCode session for each call kind](10-separate-session-per-call-kind.md) a part of the engine, on the branch `layout-trials`, test first.

## Context

- The trial code is in `tmp/lt10/candidate.diff` (1 file, no tests). Use it as a guide, not as the change. The trial sent `<connection id>:quantifier` for the Quantifier calls and the plain Connection id for every other call.
- Known: `ChatCompletionsBackend` in `src/adapters/driven/llm/providers/chat_completions.rs` builds the header value. `LlmProvider::complete` already gets the `agent_name` (`"narrator"`, `"quantifier"`, `"options"`). `LlmBackendType::session_header` in `src/domain/model/llm_backend.rs` names the header (OpenCode Go only).
- Rule (owner, 2026-11-01): one session value for each Game and agent. Two Games that run at the same time on one Connection must not share a session, or the prompts of one Game can push the other out of the cache, as the Narrator-role prompts did to the Quantifier (ticket 10).
  - With a Game (`CallSettings::game`): `<game session>:<agent_name>`. `<game session>` is the value that `game_session` already makes for OpenRouter (a UUID v5 of the `GameId`, fixed across restarts). Make it for OpenCode Go too, but keep the OpenRouter body field `session_id` as it is.
  - With no Game (the connection test, and any call with `game: None`): `<connection id>:<agent_name>`, as today plus the agent name.
  - Every agent gets its own value, the Options calls too. Narration and Impersonate both send `narrator`, so they keep one value (map, "Out of scope": they are one agent).
- Trade-off (inferred): two Games of one World no longer share a cache session, so they cannot read the cards and the World lore that the other Game wrote to the cache. The trials play one Game, so no trial can measure the per-Game part. The rule rests on ticket 10: one session that mixes prompts lost the cache of the smaller ones. Not known: whether OpenCode Go has a limit on the header length or on the number of sessions.
- Not in scope: a split of Narration from Impersonate (ticket 10, "A full split").
- Tests: the seam contract tests in `src/adapters/driven/llm/providers/chat_completions_tests.rs` pin the header. `chat-completions-opencode-go-session-header` and `chat-completions-unsaved-connection-session` expect the plain Connection id today, so they change. Add cases for a Quantifier call, for a call with a Game, and for two Games that get different values. Each new test needs a `CASE` tag (`scripts/validate_seam_cases.py`).
- The field comment "one session per connection" on `connection_id` becomes wrong. Change the docs that the change makes wrong (`docs/AGENTS.md`, "Direction of edits"). Known: no doc under `docs/diataxis/` names `x-opencode-session` today.
- No probe trial is necessary: ticket 10 measured the effect. A short probe check that a Quantifier call still gets cache hits is enough.
- End with `/simple-after-plan` (map Notes, "Decide, then build").

## Answer

Resolved on 2026-11-02. Built on `layout-trials`. An OpenCode Go call sends `x-opencode-session: <game session>:<agent name>`, or `<connection id>:<agent name>` with no Game. The OpenRouter body field `session_id` did not change.

### The change

- **known** `src/adapters/driven/llm/providers/chat_completions.rs`: `call` gets the `agent_name`. `session_header_value` builds the value. `game_session` (the UUID v5 of the `GameId`) is now an associated function that both providers use. The OpenRouter `session_id` still sends only when `sends_game_session` is true. The wrong field comment "one session per connection" is gone.
- **known** Tests first (`chat_completions_tests.rs`): the two old cases now expect `…:narrator`. Three new cases, each with a `CASE` tag: `chat-completions-opencode-go-session-per-agent` (a Quantifier call with no Game, and three agents of one Game get three values), `chat-completions-opencode-go-game-session` (Game 7 pins `7d30fc87-b5d6-50d4-a2a9-4e5d581dfeaf:quantifier`, the same across restarts), `chat-completions-opencode-go-session-per-game` (two Games, two values). The three new tests failed before the code change.
- **known** `docs/diataxis/reference/narrative/narration_system.md`: the session sentences are a separate paragraph. They state the rule for each Role. An Impersonate sends the value of the Narrator Role.
- **known** Each value is for each `agent_name`, so for each Role: `narrator` (Narration, Impersonate), `quantifier`, `options`, and `trigger` (an event Narration, `Kind::Event => Role::Trigger`). The connection test sends `<connection id>:connection_test`.

### The probe check

- **known** A probe server on port 3005 (`target/lt11`, a copy of the main probe DB made with the SQLite backup call) ran 10 Cycles of `tmp/lt11/redmist_julian.json`: no failed call, 120 s, 135,955 tokens. 19 of 22 Quantifier calls had a cache hit (896 or 1024 tokens), against 21 of 200 in the baseline. The server is stopped. Pack: `tmp/lt11/redmist_julian/trial-1`, log: `tmp/lt11/run.log`.

### `/simple-after-plan`

- **known** The scope was these 3 files only. Another session edited `data/` and ticket 12 at the same time, so the review bundle was built by hand (`tmp/review/lt11/`).
- Comment pass: one field comment deleted (accepted). Doc review: "agent" became "Role" (`CONTEXT.md`: an Agent is a Hook helper), the session sentences became their own paragraph (accepted).
- `/code-review`, Spec axis: no blocking finding. Rejected: "write agent, not Role, in the doc", because each production `agent_name` is a Role name. Standards axis: accepted the Role term in the method comment, the header value computed only for a provider with a header, and the stale test name. Rejected: the `dialogue` agent name (only the mock provider and the scripted model use it), and the split of the no-Game Quantifier assert (it is the Quantifier case that this ticket asks for).
- Full gate `python build.py`: the first run failed `guardrails_free_fn_location` (a new free function `game_session`). The second run passed: guardrails 309, integration 1628, browser 44.
- Retro: two findings filed, [The review bundle takes the whole tree](../../retro-issue-bucket/issues/100-review-bundle-takes-the-whole-tree.md) and [The free-fn guardrail fails only in the full gate](../../retro-issue-bucket/issues/101-free-fn-guardrail-only-in-the-full-gate.md).
