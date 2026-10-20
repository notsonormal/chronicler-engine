# Map: build the Rust rebuild

Labels: wayfinder:map

## Destination

[Target design](../turn-state-model/assets/target-design.md) version 2 runs in the current Rust crate: the six preparation slices and the core step of [the build plan](../../docs/plans/rust-rebuild-build-plan.md) are done, the response to the reviews of the whole rebuild is built, and all of it is merged to `main`. (Redrawn by the owner on 2026-10-19 to take in the response.)

## Notes

- **Do, not only plan.** This map overrides "plan, don't do" of `/wayfinder`: each ticket is a `task` that builds one slice of the plan (build plan §10; ticket 09 Q9 of [the Rust rebuild](../platform-choice/map.md)).
- The plan is the source of truth. A gap that needs a decision stops the slice and goes to the owner as a new ticket, numbered from 08.
- One implementer at a time; the slices run in order. Every commit passes the commit gate of plan §2.
- **Standing approval** (owner, 2026-10-17): the owner approves the commits and merges of the slices in advance. The lead commits after a green commit gate, merges a done slice to `main`, and starts the next slice. The lead stops only for a gap that needs a decision.
- **Seam-case form** (owner, 2026-10-17, option B): `// CASE <id>: <description>` on the tag above the test; no module-doc list. Amends ticket 26 R4. [The seam-case form: the tag carries the description](issues/08-seam-case-tag-only-form.md) is built and merged.
- **The response to the reviews** (owner, 2026-10-19): its design questions are grilling tickets (19 to 23); once they are answered, a build plan for the response gives the task tickets. The standing approval covers the commits of those tasks.
- Terms: `CONTEXT.md`. "Turn" and "Action Pipeline" are deprecated.

## Decisions so far

- [P0: the test rule and the guardrail machinery](issues/01-p0-test-rule-and-guardrail-machinery.md) — built and merged; the walker tables are empty until their rows land; `unit_test_standards.md` is deleted (owner), so read `tests/STRATEGY.md` where the plan names it; the `Narrator` joins the box lists in P4.
- [P1: `ActiveGame` and `CurrentGame`](issues/02-p1-active-game-and-current-game.md) — built and merged; `GameId` is a newtype (lead); rule 2's row allows only `catalogue.rs`, and `CurrentGame` reads through `GameCatalogue`; call-site rows take a `callees` list.
- [P2: delete the arrival opening](issues/03-p2-delete-the-arrival-opening.md) — built and merged; the boot-time `load_game_state` call stays until the core step; `with_max_tokens` waits for P4.
- [P3: `SwipeState` and the game rules](issues/04-p3-swipe-state-and-game-rules.md) — built and merged; `apply_game_rules` takes a `GameRuleRequest` and returns `Result<GameRulesOutcome>`; the wrappers keep the story writes and the old error-path order.
- [P4: the Agent edge on values](issues/05-p4-agent-edge-on-values.md) — built and merged in two commits; the preset and budget reads stay in the pipeline (rule 11); the Options agent struct is `OptionsGeneratorAgent`; 15 prompting helper tests stay legacy (owner item).
- [P5: the storage `Tx`](issues/06-p5-storage-tx.md) — built and merged in two commits; tier 1 runs on SQLite `:memory:`; 5 target tests are red and ignored until C5; the Job-task tracker moves to the Job runner in the core step.
- [The core step (C1 to C7)](issues/07-core-step.md) — built in seven commits and resolved before the merge to `main` (owner); merged into `wf/rebuild-review` for a review of the whole rebuild; migration v28 deletes saved Games; follow-ups are tickets 09 to 12.
- [Move or delete the 15 legacy prompting tests](issues/09-legacy-prompting-tests.md) — option D (owner): prompting is inside the Narrator box; the 15 tests are deleted and the new case `narrator-history-cap` pins the history cap.
- [Decide if the Connection form sets the context window and the answer budget](issues/10-connection-form-budget-fields.md) — option B (owner): the form sets the context window only; empty means the provider default, any positive integer is valid, a provider change keeps an explicit value.
- [Close or fix: `build.py test-pattern` cannot run ignored tests](issues/11-test-pattern-ignored-tests.md) — option A (owner): closed; no ignored test is left outside `tests/llm/`; the next plan that parks red tests adds the flag.
- [Decide if a stop cancels a running model call](issues/12-stop-cancels-the-model-call.md) — option C (owner): a call that runs ends on its own, and the id check refuses its answer; `game_flow.md` states the limit.
- [Decide how to respond to the review of the whole rebuild](issues/13-respond-to-the-whole-rebuild-review.md) — widened option B (owner): fix before the merge; the architecture candidates are problems to resolve now; the UI findings are fixed; graduated tickets 19 to 24.
- [Run the architecture review of the rebuilt code on Opus 5.5](issues/14-architecture-review-opus.md) — 8 candidates (3 Strong); top: deepen the Game rules, one call per generation.
- [Run the architecture review of the rebuilt code on GLM 5.3](issues/15-architecture-review-glm.md) — 5 candidates (1 Strong); top: one domain module for the room of a Swipe state.
- [Run the architecture review of the rebuilt code on Qwen 3.8](issues/16-architecture-review-qwen.md) — 5 candidates (1 Strong); top: one type family for the analysis hook.
- [Synthesize the three architecture reviews of the rebuilt code](issues/17-synthesize-the-architecture-reviews.md) — 6 merged and 5 single-model candidates, none rejected; they touch review findings 4, 5, 7, 12, 15, 17, 19, not 2, 3, 6; order: analysis type, Game rules, plan rules to the Tail, narrator.
- [Review the dashboard after the Rust rebuild](issues/18-review-the-ui-after-the-rebuild.md) — 1 P1, 1 P2, 3 P3. New in the rebuild: a bare `/impersonate` answers 500, and `/retrigger` accepts an Impersonate Input. Older: past 50 Messages, the story log hides the newest Messages and its controls act on a hidden Message.
- [Decide the shape of a generation: its kind, its analysis output and the Game rules](issues/19-shape-of-a-generation.md) — a data-carrying `Generation` enum (bare `/impersonate` allowed; ids and texts only); pure plan fn `JobPlan::for_request` in the domain; `QuantifierResult` of two facts, no merge; one `game_rules::apply` per generation on a `PreAnalysisState`; the trigger context holds the event record; hooks named by position; the room report moved to ticket 25.

## Not yet specified

- **The small items of the review.** Candidates S2 (one preset module, with the gap that a preset delete does not check the preset slots of each Game) and M6 (the shallow catalogues), and the medium and low findings that tickets 19 to 24 do not take ([FINDINGS.md](assets/whole-rebuild-review/FINDINGS.md), [synthesis.md](assets/architecture-review/synthesis.md) §3). Triage them once the design tickets have placed their neighbours.
- **The plan of the response, then the merge.** After the design tickets (19 to 23, and 25 for the room report): one build plan for the response, its slices as task tickets, and which branch merges to `main` (`wf/rebuild-core` holds tickets 09 to 13 and the review fixes `2885df2c`; `wf/rebuild-review` at `fa1a25e1` lacks them).

## Out of scope

- The efforts of plan §10 ("Past the build").
- A streaming LLM transport with a real cancel of a running call. A non-streaming call cannot save tokens on a stop with every provider ([Decide if a stop cancels a running model call](issues/12-stop-cancels-the-model-call.md)).
