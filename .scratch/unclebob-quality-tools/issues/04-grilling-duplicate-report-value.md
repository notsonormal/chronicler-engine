# Grilling: does the duplicate report have value here, which settings and ignore rules, and what happens to jscpd?

Type: grilling (HITL)
Status: open
Blocked by: 02

## Question

Does a structural duplicate report (dryer) earn a place in the dev loop of Chronicler Engine, which settings and ignore rules does it run with, and does jscpd stay?

## Context

Evaluation: [Research: what a structural duplicate report shows beside jscpd](./02-research-duplicate-report.md), asset [02-duplicate-report.md](../assets/02-duplicate-report.md). At the defaults, 52 production pairs: 10 real, 42 idiom. Three real pairs are ones the gate's jscpd misses (A1, A2, B8). A reader of the gate's printed summary sees only 1 of the 10 real pairs: the summary drops same-file clones, and test clones fill its top 25 (see the addendum in ticket 02). Seven structural rules (R1–R7) remove the 42 and cost one real pair (`from_config`). Test code yields 409 (`*_tests.rs` + `src/test_support/`) + 1,016 (`tests/`) pairs, and 92.5% of the gate's jscpd report is test clones. dryer has no ignore mechanism, so every rule is a port or patch feature. dryer costs 7.8 s on production against 0.2 s for jscpd.

If every jscpd clone were fixed, dryer would still report 43 production pairs: 3 real (A1, A2, B8) and 40 idiom. Up to 3 more real pairs (B5, B6, B7) have only a partial jscpd clone. See the asset, section *What dryer still finds after every jscpd clone is fixed*.

## Decisions wanted

1. Value: are the three pairs jscpd misses (the `assembler.rs` renderer pair, the two agent `Debug` impls, `list_games`/`list_personas`) enough to carry a tool step? Or does a change to the jscpd summary filter (same-file clones, a production-first ranking) give most of the gain without dryer?
2. Rules: adopt R1–R7? They hide `from_config`, which is real.
3. Settings: defaults plus rules, or `--min-nodes 100` plus rules?
4. Scope: production only, or test code too (it needs its own rule for `page.evaluate` wrappers)?
5. Place: on-demand subcommand, report-only full-gate step, or a gate?
6. jscpd: keep as is, narrow it, or replace it?

## Out of scope

The build of the tool. This ticket decides value and shape; a build ticket follows.
