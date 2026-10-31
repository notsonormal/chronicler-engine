# Decide the parameters of each call

Type: grilling
Status: resolved
Blocked by: 01, 02

## Question

Which parameters can an author set, and at which level: for each connection, each role, each call kind or each preset? This covers the connection, max tokens, samplers (temperature and others) and the response length.

## Context

- Today: `max_tokens` and `max_context_tokens` are set for each connection. Impersonate and Event Narration use the Narrator connection. No sampler is sent. One global `response_length` text reaches every Narrator-role call, including Impersonate.
- ME has an Impersonate connection and preset for each request. ST has max tokens and samplers in each preset (see [How ST and ME let an author change a call](02-st-me-prompt-configuration.md)).
- Open question from the first Autoplay run: should the global response length apply to the Persona's voice (Impersonate) at all?
- Caching is a low-effort balance (map Notes). OpenRouter served one model id from three hosts at $0.700, $0.210 and $0.440 per 1M input tokens, and its sticky routing breaks on our changing prompt. Decide whether a connection pins its host (a `session_id` per Game, or a provider order). An opt-in cache setting is only needed if a connection has no automatic cache: see [Find how prompt caching works for our providers and our layout](12-prompt-caching.md).

## Progress (grilling, 2026-10-31)

Round 1. The owner chose "match ME" (Marinara Engine) for Q1 to Q5. Mapping:

- Q1: generation parameters (`max_tokens`, samplers) live on the preset. The connection keeps model, context window, `reasoning_effort`, routing and a `max_tokens` ceiling (ME `maxTokensOverride`).
- Q2: four connection slots: Narration (with Event Narration and the regenerate), Impersonate, Quantifier, Options. Impersonate has its own connection and preset (ME `impersonateConnectionId`, `impersonatePresetId`).
- Q3: remove the global `response_length`. The length words are preset text (ME `{{length}}` in Output Format). A per-Game value waits for preset variables (ticket 06).
- Q4: typed samplers plus a free object (ME `customParameters`). A sampler that is not set is not sent.
- Q5: decided in round 2 (Option B, ST's rule).

Round 2: Q5 B, Q6 B, Q7 A, Q8 A, Q9 B without a written rule. Round 3: the owner confirmed points A, B and D, and corrected C.

## Answer

The parameters of a call follow Marinara Engine (owner, 2026-10-31). The preset holds the values for each call kind. The connection holds the facts about the model and the host. Two choices go past ME, and the reasons come from this map.

- **Generation parameters go on the preset.** These are `max_tokens`, typed `temperature` and `top_p`, and a free object for other samplers (ME `customParameters`). This holds for all four preset kinds: system, impersonate, quantifier and options. A value that is not set is not sent, so the provider default applies. Event Narration and the regenerate of a user Input use the parameters of the system preset.
- **The connection keeps the model and the host.** These are the model, the context window, `reasoning_effort`, `provider_order`, `pin_provider_order` and a `max_tokens` ceiling (ME `maxTokensOverride`). The engine sends the lower `max_tokens` of the preset and the ceiling. If neither sets it, the engine sends 2048. The Narrator-role clamp to the free context space stays an engine rule. The build plan does not need to keep today's connection `max_tokens` values: it can rename or reset the field.
- **There are four connection slots:** Narration (Event Narration and the regenerate use it too), Impersonate, Quantifier and Options. If Impersonate or Options is empty, it uses the narration connection. The magic `options` connection id and the dead `agents[].backend` (`BackendSelector`) are removed. ME has `impersonateConnectionId` and a connection for each agent.
- **The global `response_length` is removed.** The engine adds no `Response Length:` line. The length words are preset text, as ME's `{{length}}` in its Output Format. A value for each Game waits for preset variables in [Decide which parts of a Narrator-role call an author can change](06-the-layout-model.md). This also fixes the known defect: the global length no longer reaches Impersonate.
- **The free object has a guard (past ME).** On load or save the engine refuses a free object that holds `model`, `messages`, `stream` or `max_tokens`, and the error names the key. Reason: such a preset would silently change the model or drop the history, and that would make a trial meaningless. ME blocks only `__proto__`, `constructor` and `prototype`.
- **OpenRouter host pin (past ME, as ST does).** Every OpenRouter call sends `session_id` = a hash of the `GameId`, with no setting. Reason: one model id cost $0.700, $0.210 or $0.440 per 1M input tokens depending on the host, and a change of host loses the cache ([Find how prompt caching works for our providers and our layout](12-prompt-caching.md)). ST sends `x-session-affinity`, an HMAC of the chat id. The value comes from the `GameId` of the Job, not from `ActiveGame`.
- **No override for each Game.** To change a parameter for one Game, choose another preset (ME has chat `customParameters`).
- **No cache setting.** OpenRouter caches our models automatically, and no connection of ours needs `cache_control`.

For the build plan:

- The new fields can be set through `data/` JSON. The Impersonate and Options connection pickers and the preset parameter fields in the existing Settings and preset forms belong to the map's fog item "A UI for the new changeable fields".
- The test setup file of [Shape the test setup and the trial](08-test-setup-and-trial.md) lists `response_length`. That field goes away, and the presets carry the length.
- `docs/diataxis/reference/narrative/prompt_system.md` and the Settings reference must describe the new parameter places.

Sources: [ST and ME prompt configuration](../assets/st-me-prompt-configuration.md), [prompt caching](../assets/prompt-caching.md), [what reaches the model](../assets/what-reaches-the-model.md), and the pinned clones: ME `connection-default-provider.ts` (`mergeCustomParameters`), `openai.provider.ts` (`provider.order`), ST `chat-completions.js:2474` (`x-session-affinity`).
