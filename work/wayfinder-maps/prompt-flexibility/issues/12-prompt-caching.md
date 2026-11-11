# Find how prompt caching works for our providers and our layout

Type: research
Status: resolved
Blocked by: —

## Question

Does our layout break the provider cache on every call today? If yes, what are the few cheap rules that stop it, and what do they save in cost and in latency for our connections?

## Context

- **Owner's position (this session).** Caching is a balance, not a goal. A roleplay call uses far fewer tokens than a coding agent, and ST and ME do not put much work into caching. But the engine should not break the cache on every call. So this ticket looks for cheap rules, not for the best possible hit rate.
- **Why it touches the layout.** A provider cache reads only a prefix: the first changed byte ends the hit. [Decide which parts of a Narrator-role call an author can change](06-the-layout-model.md) decides the order of the parts, so it is where a cheap rule would go.
- **Our current layout (known, from [the answer of ticket 01](../assets/what-reaches-the-model.md)).** One system message (`<role>`, `<instructions>`, `<global_rules>`) and one user message. The user message starts with the parts that change often: Game State (the current room), `KnownCharacters` with the `(in room)` / `(elsewhere)` marks, and `CharactersPresent`. Then come Persona, World Lore and the history. The history is cut from the oldest end to 16000 estimated tokens by a character cut (`src/application/prompting/token_budget.rs`, `utils/context.rs`). **inferred** After the history reaches the cut, its first bytes change on every call.
- **Our engine today (known).** No request sets a cache field. `src/adapters/driven/llm/transport/utils/response.rs` reads `completion_tokens` and `reasoning_tokens` from `usage`, but no cached-token count.
- **Our connections (`data/settings.json`).** OpenRouter (`openai/gpt-4o-mini`, `sao10k/l3.3-euryale-70b`, `sao10k/l3.1-70b-hanami-x1`, `mistralai/mistral-nemo`, `deepseek/deepseek-v4-flash`), OpenCode Go (`deepseek-v4.1-flash`) and local Ollama (Gemma 4). All go through the one chat-completions provider (`src/adapters/driven/llm/providers/chat_completions.rs`).
- **ST and ME (known).** ST has opt-in Claude cache settings (`cachingAtDepth` in `src/prompt-converters.js`). ME has cache fields on a connection (`packages/server/src/db/schema/connections.ts`). One line each is enough: what they offer, and whether it is on by default.

Points to cover:

1. **Today, measured (the outside check).** Use the dev DB (see `ENVIRONMENT.md` for its place). Take one stored Autoplay Game and the stored request of each call. For each pair of consecutive calls of the same kind (Narration, Impersonate, the Quantifier, Options), find the length of the shared prefix. Name the part where the first difference falls.
2. **The stake.** From the same Game, give the input tokens per cycle and per 100-cycle trial. Give the cost of a trial with no cache hit and with the hits that the cheap rules would give, at the current prices of our connections. For Ollama, give the time to process the prompt again on each call, measured or from a source.
3. **Automatic or opt-in.** For each of our connections, say only whether a prefix cache works with no change to the request, and how a hit shows in `usage`. Cover OpenRouter (and whether its routing to different upstream hosts loses the cache), DeepSeek, and Ollama or llama.cpp (prefix reuse of the KV cache, `keep_alive`, `num_ctx`). Say "I don't know" for OpenCode Go if it has no docs. Do not build a full provider survey.
4. **The cheap rules.** Name the smallest changes that stop a break on every call, with an estimate of the shared prefix after each change. Candidates: the parts that change often move after the stable parts; the history cut moves in large steps, not on every call; the system message stays the same for a Game. Say what each rule would cost an author, if the layout lets an author order the parts.

Record the date and the URL of each provider fact.

## Output

A short asset `assets/prompt-caching.md` with the measured shared prefix for each call kind, the stake in cost and latency, a table of our connections (automatic or not, and the `usage` field), and the cheap rules with their effect. Add facts for [Decide which parts of a Narrator-role call an author can change](06-the-layout-model.md) (an order rule with its reason) and [Shape the test setup and the trial](08-test-setup-and-trial.md) (whether to record cached tokens per call).

This ticket finds facts. It does not decide the layout, and it does not change the engine.

## Answer

Asset: [Prompt caching: the measured prefix, the stake and the cheap rules](../assets/prompt-caching.md).

- **Yes, our layout breaks the cache on every call.** In Game 6 of `tmp/stage6/debug/chronicler_3060.db` the shared prefix of two calls of the same kind is the system message plus at most 460 bytes of the user message. That is 23.5% to 38.7% of a Narration call (mean 27.8%) and 16.9% to 40.0% of an Impersonate call (mean 26.8%). The first difference always falls inside `GameState` (byte 20 to 30, a new Room) or inside `KnownCharacters` (byte 13 to 454).
- **Three causes, and one is a bug.** The card list order is random, because both the Narrator (`job.rs:255`) and the Quantifier (`orchestration.rs:30`) build their card list from a `HashMap`. The `(in room)` / `(elsewhere)` marks change whenever a Character moves and duplicate `CharactersPresent`. And the volatile `GameState` block sits first, so a new Room discards the 6000 to 43000 chars of history after it.
- **The provider agrees.** `usage.prompt_tokens_details.cached_tokens` is 512, 1024 or 1280 on 7 of the 19 successful Narrator calls and 0 on the other 12. That is a mean hit share of 7.4% of the input tokens.
- **The stake is small in money and large in relative terms.** A Cycle sends 7439 input tokens at cycle 1 and adds about 1350 per cycle. At the Wafer price of $0.700 per 1M a 100-cycle trial costs $1.99 with no cache, $1.86 as we run today, and $0.35 with the cheap rules. A cache hit costs 0.1 of the input price, measured.
- **Routing matters as much as the cache.** OpenRouter served the one model id from Wafer, Azure and Cloudflare at $0.700, $0.210 and $0.440 per 1M input tokens. Its sticky key is a hash of the first system message and the first non-system message, so our changing user prompt defeats it. A `session_id` pins it.
- **The cheap rules, in value order.** (1) Sort the card list by id. (2) Drop the presence marks from `KnownCharacters`. (3) Order the blocks stable-first and the history last. (4) Quantize the history cut so that it moves in steps. (5) Send a stable `session_id` per Game to OpenRouter.
- **Rules 1 to 3 lift the Narration prefix from a mean of 27.8% to 84.6%** (95.5% to 96.7% in the three current-tag pairs, where no Room changed). An author pays nothing for any of them. Rule 3 becomes an author-facing order rule with a written reason if ticket 06 chooses an author-ordered layout.
- **Rule 4 decides a long trial.** At the engine's 32768 context the cap binds at about cycle 18 of a 100-cycle trial. Above the cap the history cut slides on every call, which returns the hit to 10%. The cut rule keeps it near 91%.
- **ST and ME treat caching as an optional provider feature, both opt-in and both Claude-only.** ST defaults `claude.cachingAtDepth` to `-1` and `claude.enableSystemPromptCache` to false. ME defaults `enable_caching` to false with `caching_at_depth` 5.

Coordinator notes (after review):

- **Rule 3 conflicts with a written engine rule (known).** Rule 3 puts `<writing_style>` and `<output_format>` before the history. `docs/diataxis/explanation/prompt_system_design.md` puts them after the history on purpose, for recency. **inferred** Both rules fit: keep the style and format sections after the history. Then only those sections and the Player Input are new on each call, and they are small.
- **A better place for GameState (inferred, not measured).** Rule 3 keeps `GameState` before the history, so a Room change still loses the history (38.5% kept). A `GameState` block after the history keeps the history cached on a Room change too. ME puts its state block near the newest history (see [the long-term state asset](../assets/long-term-state.md)).
- **The random card order is also a reproducibility fault (inferred).** Two trials of one test setup send the cards in different orders. That matters for [Shape the test setup and the trial](08-test-setup-and-trial.md), not only for the cache.

Facts the named later tickets need:

- For [06](../issues/06-the-layout-model.md): the order rule "the parts that change less often come first, and the new story text comes last", with the reason that a provider cache reads only a prefix and the history is the largest part of a long call. A layout model that cannot order the blocks (B and C) must fix this order in code. A part that must stay engine-fixed: the system half is one message for the whole Game.
- For [08](../issues/08-test-setup-and-trial.md): record `usage.prompt_tokens_details.cached_tokens` per call. `raw_response_json` already holds it, so no schema change is needed. Report the upstream host and the context cap with the trial, because the same model cost 3.3 times more on one host than on another, and a trial past the cap measures the cut. Read `usage.opencode.allowance` for the current narration connection, because its `usage.cost` is 0.
