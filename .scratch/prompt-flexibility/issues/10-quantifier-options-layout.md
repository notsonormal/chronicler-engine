# Decide which parts of a Quantifier or Options call an author can change

Type: grilling
Status: open
Blocked by: 01, 02, 06, 13, 14

## Question

For each fixed part of a Quantifier call and an Options call that [List what reaches the model in each call](01-what-reaches-the-model.md) found: does it become changeable by an author, or does it stay an engine rule with a written reason? Does the layout model of [Decide which parts of a Narrator-role call an author can change](06-the-layout-model.md) also fit these calls?

## Context

- Both calls have their own prompt code (`application/agents/quantifier/prompt.rs`, `application/agents/options/prompt.rs`) with their own tags (`<available_rooms>`, `<RecentHistory>`, `<NpcsInArea>`…).
- Both parse the model's answer (`agents/quantifier/utils/parser.rs`, `agents/options/utils/parser.rs`). The parts that a parser depends on may need to stay engine rules.
- The Quantifier failed every cycle in the first Autoplay run (`content_filter` 400), and the game state stayed stale with no visible sign. Which model it uses is authoring and out of scope; how a failure shows is in [Shape the test setup and the trial](08-test-setup-and-trial.md).
- The Quantifier is today the only Agent that writes state. A memory writer or a state writer would be another Agent at a Hook: see [Find how ST, ME and other tools keep long-term memory](13-long-term-memory.md) and [Find how ST, ME and other tools keep long-term state](14-long-term-state.md).
