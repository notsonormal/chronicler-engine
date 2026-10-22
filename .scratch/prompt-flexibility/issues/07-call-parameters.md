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
