# Decide the parameters of each call

Type: grilling
Status: open
Blocked by: 01, 02

## Question

Which parameters can an author set, and at which level: for each connection, each role, each call kind or each preset? This covers the connection, max tokens, samplers (temperature and others) and the response length.

## Context

- Today: `max_tokens` and `max_context_tokens` are set for each connection. Impersonate and Event Narration use the Narrator connection. No sampler is sent. One global `response_length` text reaches every Narrator-role call, including Impersonate.
- ME has an Impersonate connection and preset for each request. ST has max tokens and samplers in each preset (see [How ST and ME let an author change a call](02-st-me-prompt-configuration.md)).
- Open question from the first Autoplay run: should the global response length apply to the Persona's voice (Impersonate) at all?
- Caching is a low-effort balance (map Notes). OpenRouter served one model id from three hosts at $0.700, $0.210 and $0.440 per 1M input tokens, and its sticky routing breaks on our changing prompt. Decide whether a connection pins its host (a `session_id` per Game, or a provider order). An opt-in cache setting is only needed if a connection has no automatic cache: see [Find how prompt caching works for our providers and our layout](12-prompt-caching.md).
