# Map: build the Rust rebuild

Labels: wayfinder:map

## Destination

[Target design](../turn-state-model/assets/target-design.md) version 2 runs in the current Rust crate: the six preparation slices and the core step of [the build plan](../../docs/plans/rust-rebuild-build-plan.md) are done and merged to `main`.

## Notes

- **Do, not only plan.** This map overrides "plan, don't do" of `/wayfinder`: each ticket is a `task` that builds one slice of the plan (build plan §10; ticket 09 Q9 of [the Rust rebuild](../platform-choice/map.md)).
- The plan is the source of truth. A gap that needs a decision stops the slice and goes to the owner as a new ticket, numbered from 08.
- One implementer at a time; the slices run in order. Every commit passes the commit gate of plan §2.
- **Standing approval** (owner, 2026-10-17): the owner approves the commits and merges of the slices in advance. The lead commits after a green commit gate, merges a done slice to `main`, and starts the next slice. The lead stops only for a gap that needs a decision.
- **Seam-case form** (owner, 2026-10-17, option B): `// CASE <id>: <description>` on the tag above the test; no module-doc list. Amends ticket 26 R4. [The seam-case form: the tag carries the description](issues/08-seam-case-tag-only-form.md) is built and merged.
- Terms: `CONTEXT.md`. "Turn" and "Action Pipeline" are deprecated.

## Decisions so far

- [P0: the test rule and the guardrail machinery](issues/01-p0-test-rule-and-guardrail-machinery.md) — built and merged; the walker tables are empty until their rows land; `unit_test_standards.md` is deleted (owner), so read `tests/STRATEGY.md` where the plan names it; the `Narrator` joins the box lists in P4.
- [P1: `ActiveGame` and `CurrentGame`](issues/02-p1-active-game-and-current-game.md) — built and merged; `GameId` is a newtype (lead); rule 2's row allows only `catalogue.rs`, and `CurrentGame` reads through `GameCatalogue`; call-site rows take a `callees` list.
- [P2: delete the arrival opening](issues/03-p2-delete-the-arrival-opening.md) — built and merged; the boot-time `load_game_state` call stays until the core step; `with_max_tokens` waits for P4.
- [P3: `SwipeState` and the game rules](issues/04-p3-swipe-state-and-game-rules.md) — built and merged; `apply_game_rules` takes a `GameRuleRequest` and returns `Result<GameRulesOutcome>`; the wrappers keep the story writes and the old error-path order.
- [P4: the Agent edge on values](issues/05-p4-agent-edge-on-values.md) — built and merged in two commits; the preset and budget reads stay in the pipeline (rule 11); the Options agent struct is `OptionsGeneratorAgent`; 15 prompting helper tests stay legacy (owner item).
- [P5: the storage `Tx`](issues/06-p5-storage-tx.md) — built and merged in two commits; tier 1 runs on SQLite `:memory:`; 5 target tests are red and ignored until C5; the Job-task tracker moves to the Job runner in the core step.

## Not yet specified

## Out of scope

- The efforts of plan §10 ("Past the build").
