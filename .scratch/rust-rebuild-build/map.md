# Map: build the Rust rebuild

Labels: wayfinder:map

## Destination

[Target design](../turn-state-model/assets/target-design.md) version 2 runs in the current Rust crate: the six preparation slices and the core step of [the build plan](../../docs/plans/rust-rebuild-build-plan.md) are done and merged to `main`.

## Notes

- **Do, not only plan.** This map overrides "plan, don't do" of `/wayfinder`: each ticket is a `task` that builds one slice of the plan (build plan §10; ticket 09 Q9 of [the Rust rebuild](../platform-choice/map.md)).
- The plan is the source of truth. A gap that needs a decision stops the slice and goes to the owner as a new ticket, numbered from 08.
- One implementer at a time; the slices run in order. Every commit passes the commit gate of plan §2.
- **Standing approval** (owner, 2026-10-17): the owner approves the commits and merges of the slices in advance. The lead commits after a green commit gate, merges a done slice to `main`, and starts the next slice. The lead stops only for a gap that needs a decision.
- Terms: `CONTEXT.md`. "Turn" and "Action Pipeline" are deprecated.

## Decisions so far

- [P0: the test rule and the guardrail machinery](issues/01-p0-test-rule-and-guardrail-machinery.md) — built and merged; the walker tables are empty until their rows land; `unit_test_standards.md` is deleted (owner), so read `tests/STRATEGY.md` where the plan names it; the `Narrator` joins the box lists in P4.

## Not yet specified

## Out of scope

- The efforts of plan §10 ("Past the build").
