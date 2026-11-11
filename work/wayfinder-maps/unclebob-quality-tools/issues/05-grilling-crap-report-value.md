# Grilling: does the CRAP report have value here, and which CC rule?

Type: grilling (HITL)
Status: open
Blocked by: 01

## Question

Does a CRAP report earn a place in `build.py`, and if so with which CC rule: `?` counted (upstream) or `?` dropped? Where does its coverage come from?

## Context

Evaluation: [Research: what a CRAP report shows on this codebase](./01-research-crap-report.md), asset [01-crap-report.md](../assets/01-crap-report.md).

- Upstream rule (`?` counted): 7 of the top 20 are `from_row` column lists. 9 of the 20 are artefacts or unreachable code. 6 point at a coverage gap (inferred).
- `?`-free rule: no artefacts. 18 rows at CRAP ≥ 10, and 6 of them are complexity-only rows at 100 % coverage.
- 85.5 % of functions have 100 % line coverage, so CRAP = CC for them. The report is mostly a complexity list.
- The CC rule changes 14 of the top 20. Line vs region coverage changes 1. Branch coverage is not available (0 BRDA records).
- Cost: CRAP 4 s; LCOV export 12 s; a coverage run 487.6 s and a 6.1 GiB target dir.

## Decisions wanted

1. Value: is the list worth a place, given that the upstream rule needs a seven-entry ignore list (`from_row`, `story_row`, `migrate_up_to`, `init_logging`, `start_server`, `source`, `provider_label`)?
2. Rule: count `?` with an ignore list, or drop `?` and accept a complexity list?
3. Threshold: report-only, or a CRAP bar? At 30, 2 rows exceed it today; at 10, 18–46 rows do.
4. Coverage source: reuse the last `coverage.json` (12 s + 4 s), or run coverage on demand (~8m42s, 6.1 GiB)? This settles the map fog item *Coverage freshness for CRAP*.
5. Line or region coverage?

## Out of scope

The build of the tool. This ticket decides value and shape; a build ticket follows.
