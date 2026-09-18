# Tier-1 rollout: write the two orphaned HTTP contracts

Type: task
Status:
Blocked by: 06

> Naming: split out of the original ticket 08 on 2026-09-18 (the two orphaned
> contracts are net-new authorship, not demotion). Suffixed `08b` rather than
> renumbering 09–12, so the pairing stays legible; this is the tracker's first
> non-`NN` filename.

## Question

Write the two HTTP contract tests the audit named as **orphaned** — contracts
that today have no HTTP coverage at all and are reached *only* through a
Chromium round-trip. This is the coverage **gain** the audit identified: moving
them to the HTTP tier adds coverage while removing the race. This ticket
**deletes nothing** — it only adds.

Per `assets/harness-audit.md` §1's two findings:

1. **`#game-posture-controls` has no HTTP test.** Test 21
   (`test_games_panel_renders_posture_fragment`, `games.rs:11`) is the only
   assertion that the games posture fragment renders populated selects. Write
   an HTTP test that GETs the games fragment and asserts the selects render with
   the active game's current posture and preset ids. Template:
   `src/adapters/driving/http/games/templates/games.rs:124`; route:
   `/fragment/games` (`builders/router.rs:116`).
2. **The world edit-form render has no HTTP coverage.** Test 24
   (`test_world_edit_form_renders_posture_selects`, `worlds.rs:42`) is the only
   coverage of the full edit form; the quarantine
   (`requires_migration/worlds_fragment_handlers.rs:240`) covers only
   *not-found*. Write an HTTP test that GETs `/worlds/:key/edit`
   (`builders/router.rs:132`) and asserts the three posture selects render with
   the world's current values.

**The posture POST contract itself is NOT this ticket** — it already landed in
ticket 06's slice (4 tests, SCENARIO 25.5, `tests/http/worlds.rs`). Extend from
there rather than duplicating.

**Do not re-create the surviving browser wiring checks.** Tests 24 and 26 have
browser-side wiring residue whose final form is owned by ticket 09 (seeded by
ticket 06's slice). This ticket adds HTTP assertions; it does not delete or
rewrite the browser copies. Any deletion of test 21/24 browser copies belongs to
the ticket that can prove the HTTP coverage carries the facts — decide that in
`## Answer`: if the new HTTP test fully carries the asserted content, say so and
note the deletion as a follow-up for ticket 09's keeper-set pass rather than
deleting here.

## Spec reconciliation is part of this ticket

New HTTP tests need scenarios in a non-`browser_*` spec, because
`scripts/validate_feature_spec.py` rejects a `browser_*.md` tag outside
`tests/browser/`. So each new HTTP test needs either:

- an existing scenario in the HTTP spec that already describes the fact (check
  `docs/specs/games.md` and `docs/specs/worlds.md` first — `worlds.md` 25.5 may
  already cover the edit-form surface), or
- a new scenario added to the HTTP spec, with the test tagged to match.

Let the validator be the oracle: `0 gap(s)`, `0 orphan(s)`, `0 untagged`,
`0 surface mismatch(es)`.

**Do not weaken the validator to make a move pass.** Surface it in `## Answer`
if a scenario can't be cleanly expressed.

Record under `## Answer`: the HTTP tests added with paths and their SCENARIO
tags, the spec scenarios added or reused, whether the new HTTP coverage fully
carries the browser assertions (and therefore whether the browser copies are
now deletable by ticket 09), and `validate_feature_spec.py` output before and
after.
