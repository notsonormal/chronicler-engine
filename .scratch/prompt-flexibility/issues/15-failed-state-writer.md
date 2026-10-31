# Decide what the state does when a state writer fails

Type: grilling
Status: open
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
