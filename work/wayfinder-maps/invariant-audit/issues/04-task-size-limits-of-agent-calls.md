# Decide and set the size limits of the Agent calls

Type: task (AFK, decide and do)
Status: open
Blocked by: —

## Question

Give the verdict for each of these three findings under the enforcement order of the map Notes, and put each verdict in place.

- **Agent prompts with no size limit (llm-05).** Only the narrator has a `TokenBudget`. The Quantifier and Options prompts hold four uncut Messages and every Character and Room id of the World. When the prompt overflows the context window, the Quantifier falls back and the scene silently freezes.
- **The token estimate (llm-06).** The estimate is a fixed 4 characters per token. Non-Latin text then overruns the window. When a Connection has no `max_context_tokens`, the provider default is used, and nothing ties it to the model.
- **A `max_tokens` of 0 (llm-10).** The narrator clamps with `.max(1)`. The Quantifier and Options calls send a preset value of 0 unchanged.

The candidate fixes, from the report: a `TokenBudget` and the narrator's `HistoryCut` for the Agent calls; an estimate corrected from the `usage.prompt_tokens` that the provider reports; one place that keeps `max_tokens ≥ 1` (refuse 0 when a preset is saved, a `NonZeroU32`, or one clamp that every call uses).

## Context

- Findings in full: llm-05, llm-06, llm-10 in [the llm report](../assets/llm.md). The coordinator did not read these in the code.
- Do not decide again what other maps own. [Decide the parameters of each call](../../prompt-flexibility/issues/07-call-parameters.md) put `max_tokens` on each preset. [Long-term memory and state](../../long-term-memory-and-state/map.md) decides the size and the point of the history cut.
- An "accept" verdict (likely for llm-06, because the count is an estimate) stops the task and asks the owner (map Notes, decide-and-do rule).
- The six end conditions of the map Notes apply.

## Answer
