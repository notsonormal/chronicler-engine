# Delegation

Prefer single long running subagents over multiple smaller subagents because it is more expensive to load up twice the context. This is a preference; some workflows and skills deliberately use multiple subagents, such as some code review workflows or when implementing multiple wayfinder tickets at once.

Only create `scout` subagents if the current model is Anthropic (e.g. Opus or Sonnet). With a non-Anthropic model, read all the information you need in the current session. This rule is specific to `scout` subagents only.
