# Send an OpenCode session for each Game and agent

Type: task (AFK)
Status: open
Blocked by: 10

## Question

Make the kept candidate of [Try a separate OpenCode session for each call kind](10-separate-session-per-call-kind.md) a part of the engine, on the branch `layout-trials`, test first.

## Context

- The trial code is in `tmp/lt10/candidate.diff` (1 file, no tests). Use it as a guide, not as the change. The trial sent `<connection id>:quantifier` for the Quantifier calls and the plain Connection id for every other call.
- Known: `ChatCompletionsBackend` in `src/adapters/driven/llm/providers/chat_completions.rs` builds the header value. `LlmProvider::complete` already gets the `agent_name` (`"narrator"`, `"quantifier"`, `"options"`). `LlmBackendType::session_header` in `src/domain/model/llm_backend.rs` names the header (OpenCode Go only).
- Rule (owner, 2026-11-01): one session value for each Game and agent. Two Games that run at the same time on one Connection must not share a session, or the prompts of one Game can push the other out of the cache, as the Narrator-role prompts did to the Quantifier (ticket 10).
  - With a Game (`CallSettings::game`): `<game session>:<agent_name>`. `<game session>` is the value that `game_session` already makes for OpenRouter (a UUID v5 of the `GameId`, fixed across restarts). Make it for OpenCode Go too, but keep the OpenRouter body field `session_id` as it is.
  - With no Game (the connection test, and any call with `game: None`): `<connection id>:<agent_name>`, as today plus the agent name.
  - Every agent gets its own value, the Options calls too. Narration and Impersonate both send `narrator`, so they keep one value (map, "Not yet specified").
- Trade-off (inferred): two Games of one World no longer share a cache session, so they cannot read the cards and the World lore that the other Game wrote to the cache. The trials play one Game, so no trial can measure the per-Game part. The rule rests on ticket 10: one session that mixes prompts lost the cache of the smaller ones. Not known: whether OpenCode Go has a limit on the header length or on the number of sessions.
- Not in scope: a split of Narration from Impersonate (ticket 10, "A full split").
- Tests: the seam contract tests in `src/adapters/driven/llm/providers/chat_completions_tests.rs` pin the header. `chat-completions-opencode-go-session-header` and `chat-completions-unsaved-connection-session` expect the plain Connection id today, so they change. Add cases for a Quantifier call, for a call with a Game, and for two Games that get different values. Each new test needs a `CASE` tag (`scripts/validate_seam_cases.py`).
- The field comment "one session per connection" on `connection_id` becomes wrong. Change the docs that the change makes wrong (`docs/AGENTS.md`, "Direction of edits"). Known: no doc under `docs/diataxis/` names `x-opencode-session` today.
- No probe trial is necessary: ticket 10 measured the effect. A short probe check that a Quantifier call still gets cache hits is enough.
- End with `/simple-after-plan` (map Notes, "Decide, then build").
