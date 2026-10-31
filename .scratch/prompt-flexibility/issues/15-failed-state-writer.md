# Decide what the state does when a state writer fails

Type: grilling
Status: resolved
Blocked by: —

## Question

When a state writer fails (today the Quantifier, later perhaps a memory or state Agent), what does the Swipe state of the new Swipe hold: the previous state, an empty result, or nothing because the Job stops? How does the failure show to the player, and does the engine try again?

## Context

- **Today (known, checked by the coordinator).** `AgentRunner::after_narration` logs a failed Agent and returns `QuantifierResult::default()` (`src/application/agents/runner.rs:69-79`). Its doc comment says this is intended. `game_rules::apply` then sets `characters_present` from the empty result, so a failure clears the Characters present and keeps the Room (`src/domain/model/utils/game_rules.rs:67-71`).
- **The banner says the opposite (known).** The failure banner reads "Quantifier failed — keeping the Characters present" (`role_failure_effect` in `src/adapters/driving/http/builders/headers.rs`). A Settings row shows the Quantifier role health.
- **The size of it (known, from [Find how ST, ME and other tools keep long-term state](14-long-term-state.md)).** In the stored Game 6, 24 of 30 Quantifier calls failed with `HTTP 400` (`content_filter`). The state went stale, and a Character could drop out of the scene.
- **How ME does it (known, same asset).** A failed tracker is logged and dropped, and the previous state row stays. A person can edit a value and lock it.
- **Related tickets.** [Shape the test setup and the trial](08-test-setup-and-trial.md) owns how a failure shows in a reading pack. [Decide which parts of a Quantifier or Options call an author can change](10-quantifier-options-layout.md) owns the Quantifier prompt. This ticket owns the engine rule for the state.
- **A choice for the owner first.** This is an engine defect, not a part of a call. The map Notes say known defects wait for the build plan. The owner may instead fix it now, test-first, outside this map.

## Output

A resolution comment with the rule (keep, clear, or stop; retry or not), the banner text that matches it, and whether the rule is the same for every future state writer.
- **A trial observation (observed, not checked against the code).** In trial B of [the trial prototype](../assets/trial-prototype.md), finding 5, each failed Quantifier entry tried twice. The Swipe state of each entry still showed the Characters present of the entry before, not an empty list. Check this against `game_rules::apply` before deciding.

## Answer

Resolved with the owner, 2026-10-31.

**The premise was wrong (known, checked in the code and a unit test).** A failed model call does not clear the Characters present. After 2 failed attempts, `quantify_room_with_llm_call` returns a `Low` confidence reading (`src/application/agents/quantifier/utils/orchestration.rs`). `process_quantifier_result` then falls back to `static_character_result`, which keeps the previous Characters present and has no destination. The unit test `test_determine_characters_backend_error_fallback` asserts this. The banner "keeping the Characters present" is thus correct. Only the `Err` path of `AgentRunner::after_narration` (`src/application/agents/runner.rs`) clears them, through `QuantifierResult::default()`. `run` returns `Err` only when the hook input has no saved text or no Room, which is a wiring fault. The doc comment of `after_narration` misleads. The trial B observation (finding 5 of [the trial prototype](../assets/trial-prototype.md)) is correct.

**The rule (owner):**

1. **Keep.** A failed state writer leaves the previous state: the Room and the Characters present stay. The Job continues. A move that the Narration described is lost until a later call succeeds. This is today's rule for a model failure, and ME's rule for a tracker.
2. **Retry.** Keep 2 attempts at once, as today. The next entry repairs a stale state.
3. **The `Err` path.** Same rule as a model failure: keep the previous state and show the banner. It does not clear the Characters present. The log warning still names the fault.
4. **Banner text.** "Quantifier failed — keeping the Room and the Characters present".
5. **Every future state writer** (memory, state Agents) uses the same rule. A failed writer leaves its own fields of the previous Swipe state as they were. Its role shows in the banner, and the Job continues. Each writer sets its own number of attempts.
6. **When.** In the build plan, not now (map Notes: known defects wait). The build plan changes the `Err` path, the doc comment of `after_narration`, and the banner text (`role_failure_effect` in `src/adapters/driving/http/builders/headers.rs`), test-first.

**The `content_filter` failures (facts for later tickets):**

- **known** Game 6 (`tmp/stage6/debug/chronicler_3060.db`, 2026-10-21 to 10-26) ran the Quantifier on OpenRouter `openai/gpt-4o-mini` and the Narrator on `deepseek/deepseek-v4-flash`. Azure's content filter rejected the Quantifier prompt (`HTTP 400`, `ResponsibleAIPolicyViolation`) in 24 of 30 calls. The stored error is cut off, so the filter category is unknown.
- **known** The OpenCode Go trials (`tmp/pf08/`) ran every role on `deepseek-v4.1-flash`. Quantifier failures: 0 of 69 in trial A, 4 of 178 in trial B. A filtered call reports `finish_reason: content_filter` and `prompt_tokens: 0`.
- **inferred** `prompt_tokens: 0` means the input was rejected before the model ran, so a Narration that passed says nothing about the next input. Narration call 503 held the text of failed Quantifier call 501 in a longer prompt and passed. The text of call 501 is harmless.
- **unknown** What triggers the OpenCode Go filter. No replay now (owner). The map's fog holds the question.
