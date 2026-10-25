# Grill the proposed test-to-spec assignments

Type: grilling (HITL)
Status: resolved
Blocked by: 01

## Question

Review the research asset from ticket 01 and decide, for each untagged `tests/http/` test, whether it:

1. belongs under an existing `docs/specs/*.md` scenario,
2. needs a new spec (confirm the filename and scope), or
3. should be explicitly exempt from SCENARIO tagging (confirm the reason).

Also confirm whether any existing spec scenario is missing a covering test after the migration, and how to close that gap.

## Output

- A final decision list: for every untagged test, either (a) the spec/scenario it tags, (b) the new spec title and filename, or (c) the exemption reason.
- A short backlog of new specs to create, with filenames and owning behaviour.
- Any updates needed to the map's Decisions-so-far and Out-of-scope sections.

## Out of scope

- Writing the specs or tests.
- Covering non-HTTP tiers.

## Answer

All three decision groups accepted as proposed.

- **Existing spec assignments:** 11 tests mapped to existing specs with new or existing scenario IDs:
  - `reset.md` 7.1, 7.3
  - `actions.md` 1.1, 1.5, 1.9
  - `story_log.md` 8.1, 8.4, 8.5, 8.6
  - `games_create.md` 17.1, 17.2
- **New specs to create:** 8 specs covering 70 tests:
  - `connections.md` (16 tests)
  - `dashboard_fragments.md` (8 tests)
  - `debug.md` (5 tests)
  - `games_list.md` (8 tests)
  - `status.md` (3 tests)
  - `swipe_switch.md` (3 tests)
  - `text_check.md` (14 tests)
  - `worlds.md` (13 tests)
- **Exemptions:** 2 `server_impl_wiring.rs` tests exempt from SCENARIO tagging (server binding tests, not endpoint behaviour).

The catalog is complete and assignments are grilled. No further decisions needed before a writing effort tags the tests and writes the specs.
