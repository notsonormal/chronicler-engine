# Map: invariants the engine relies on but does not enforce

Labels: wayfinder:map

## Destination

Each of the 53 invariants of [the audit](assets/REPORT.md) has a verdict, and the verdict is in place. It is enforced at a named point, or the code that relies on it is removed, or it is accepted with a written reason, or it is rejected as a false finding. `python build.py` passes. An independent run of the audit again confirms the result ([ticket 16](issues/16-research-outside-check.md)).

## Notes

- **Charted 2026-11-08** (owner, three grilling rounds), and cut to four grilling tickets in a fourth round. Tickets 01 to 16. New tickets number from 17.
- **Origin.** Six read-only scouts searched one theme each at commit `8b23151c`: story, gate, world, llm, http, storage. They gave 56 findings, which are 53 unique invariants. [The report](assets/REPORT.md) ranks them in four tiers. The theme reports next to it ([story](assets/story.md), [gate](assets/gate.md), [world](assets/world.md), [llm](assets/llm.md), [http](assets/http.md), [storage](assets/storage.md)) hold, for each finding, the reliance site, the write paths, the evidence and a suggested enforcement point. Every `file:line` is at `8b23151c`, so read the code again before you trust a line number. The coordinator read the code of 14 findings and confirmed 13. Every other finding has only the scout's status. [The scout brief](assets/scout-brief.md) is the exact text that the six scouts got.
- **Names.** A finding has a label `<theme>-NN`, for example llm-01. In text for the owner, give its name with its label, for example "the empty scene from a Quantifier answer with names (llm-01)". Use the terms of `CONTEXT.md`. "Invariant" is a general programming word and gets no glossary entry.
- **The tasks make things** (owner, charting, Q1 Option B). Each grilling ticket gives verdicts and graduates the task tickets that put them in place. The tasks change `src/`, `tests/`, `docs/specs/` and `docs/diataxis/`. If a task finds that its fix is not local, it stops and says so, and the fix goes back to a grilling (root `AGENTS.md`: do not bolt on).
- **Scope** (owner, charting, Q2 Option B). Tiers 1 to 3 (25 findings) get one verdict for each finding. Tier 4 (28 findings) gets one verdict for each group: story records, schema, gate locks, wiring and config, and the HTTP and JS contract. A group verdict can still treat one finding in its own way.
- **One owner** (owner, charting, Q3 Option C). This map owns every finding, also the ones that other maps name:
  - The byte-500 panic (llm-07) is R4 of [the weekly retro](../weekly-retro-2026-11-04/map.md). Its ticket [Confirm where each product fault goes](../weekly-retro-2026-11-04/issues/05-grilling-product-fault-routes.md) records R4 as routed here. The retro proposes the lint `clippy::string_slice`, and [ticket 06](issues/06-task-cut-provider-text-by-characters.md) decides between that and a helper.
  - The gate findings are in the area of [statechart-models](../statechart-models/map.md). A change to a function in `.agents/skills/verify-gate/coverage.toml` runs `/verify-gate` (or `/maintain-formal-verification-skill`) before the task ends. The `build.py` warning names those functions. Its tickets 14 and 15 (diagrams, `game_flow.md`) are still open, so tell their owner when a gate rule changes.
  - The code-concepts synthesis lists `update_world`, `from_configs` and `error_body` as open questions (Q15, Q24, Q57). That map is a catalogue and fixes nothing.
- **Four fixes go ahead in another effort** (owner, 2026-11-10, D7 e of [the audit of 2026-11-10](../fable-audit/decisions.md)). The random Trigger order (world-04), the byte cut of provider text (llm-07), the Agents that disappear at boot (llm-08) and the empty Message text of an edit (llm-04) are in [its engine ticket](../fable-audit/issues/02-task-engine-fixes.md). That ticket writes a line in tickets 01, 06, 09 and 14 here when a fix is done.
- **Kinds of verdict**: enforce (name the point), remove the reliance, accept (with a reason), or false finding.
- **Enforcement preference, in order** (owner, charting, Q5):
  1. Remove the code that relies on the invariant, when the invariant only protects dead or speculative design.
  2. The type system: an enum, a newtype, a non-empty type, or private fields.
  3. One validating constructor, or one write function that all paths must use.
  4. A database constraint in a new migration.
  5. A runtime check that returns an error at the write.
  6. A guardrail test, for contracts across languages (JS and Rust).

  A silent repair on the read side is never the fix. A migration that meets an existing row that breaks a new constraint fails loudly and names the row. It does not repair the row (`CODING_STANDARDS.md`: no backward compatibility unless the owner asks for it).
- **The outside check** (owner, charting, Q4: all three):
  - Each ticket first reads its findings in the code again and records "holds" or "does not hold" before it gives a verdict.
  - Each enforcement comes with a test that tries to break the invariant and expects a loud refusal. The test fails before the fix. A schema constraint gets a migration test with a bad row. Where no seam exists, the ticket says so.
  - Ticket 16 runs the audit again with agents that made no fix.
- **The end of a task ticket** (owner, charting round 2, Q4). All six must be true:
  1. The findings are read again and recorded as "holds".
  2. The breaking test fails first, then passes.
  3. The `/verify-gate` maintain pass runs when a covered function changed.
  4. The full `python build.py` gate passes.
  5. A `/code-review` runs against the commit before the ticket, and its findings are fixed.
  6. One commit for each ticket, after the owner approves it.
- **Grilling only for the owner's decisions** (owner, round 4, Q1 Option A). A grilling ticket holds only a decision that only the owner makes: a product rule, or a design that the owner asked to see. Four remain: Game rules (01), World validation (02), the small product rules (03), and the story record writes (11). Every other ticket is a **decide-and-do** task: it gives its verdicts under the enforcement order above, writes one line for each verdict with its reason in its Answer, and makes the fix.
- **When a decide-and-do task stops** (owner, round 4, Q2 Option C). The owner sees its verdicts at the commit approval (end condition 6). But the task stops and asks the owner when the enforcement order does not settle a choice, and always before one of these:
  - an "accept" verdict;
  - a choice that changes what the player sees, for example a new refusal, a crash in place of a hang, or a boot that fails on data that exists today.
- **Order** (owner, charting round 2, Q3 Option A, with the round-4 cut): the tier-1 grillings first (01 to 03), then the tasks of the local defects (04 to 10), then tier 4 (11 to 15). A ticket with no blocker can run in a parallel session next to the grillings. Tasks 09 and 10 wait for the rules of ticket 03. Task 12 waits for ticket 11, so one rebuild of the `messages` table serves both.
- **Skills:** `/grilling` and `/domain-modeling` for each grilling ticket. Read the theme report of each finding before the first question. Use `/verify-gate` for a gate change and `/code-review` at the end of each task. Ticket 16 uses [the scout brief](assets/scout-brief.md).

## Decisions so far

## Not yet specified

- **The task tickets of each grilling.** Each grilling ticket (01, 02, 03, 11) graduates its own tasks. The count is not known until its verdicts exist. Ticket 16 waits for all of them.
- **Where the reason of an accepted invariant lives.** A code comment at the reliance site, a sentence in `docs/diataxis/`, or a spec scenario. The first grilling that accepts an invariant meets this question.
- **A standing rule against new unenforced invariants** (owner, charting, Q1 Option C deferred). A guardrail or a review rule. It is clearer after the fixes show which mechanisms they use.
- **New invariants that ticket 16 finds.** Whether they go in this map or in a new effort.

## Out of scope
