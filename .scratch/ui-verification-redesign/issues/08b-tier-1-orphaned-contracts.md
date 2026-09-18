# Tier-1 rollout: write the two orphaned HTTP contracts

Type: task
Status: resolved
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

## Answer

Two HTTP tests added, nothing deleted. Both fragments are plain server renders
that previously had no HTTP coverage — the only assertion of either went
through a Chromium round-trip.

### HTTP tests added

| File | Test | SCENARIO |
|---|---|---|
| `tests/http/games_fragment.rs` (new) | `test_games_fragment_renders_posture_controls_http` | games.md 20.8 |
| `tests/http/worlds.rs` | `test_world_edit_form_renders_posture_selects_http` | worlds.md 25.6 |

`tests/http/games_fragment.rs` is new and registered in `tests/http/mod.rs`;
`tests/http/worlds.rs` gained the test alongside the existing 25.1–25.5.

### What each asserts

**Games fragment (20.8)** — `GET /fragment/games`. The fragment renders
`#game-posture-controls` with the active game's *stored* posture selected
(`novel` / `third` / `past`), all six selects present (`narrator_mode`,
`narrative_perspective`, `narrative_tense`, `system_preset_id`,
`quantifier_preset_id`, `impersonate_preset_id`), the auto-save routes pinned
to the active game id (`/games/{id}/mode`, `/posture`, `/presets`), and no
`Presets unavailable` degradation.

**World edit form (25.6)** — `GET /worlds/posture_world/edit`. The edit form
(not the create form) renders `.posture-group` with the world's stored values
selected (`interactive_fiction` / `second` / `past`), the
`#world-posture-status` target, and the world-scoped auto-save route.

### Spec scenarios added

| Spec | Scenario | Rationale |
|---|---|---|
| `docs/specs/games.md` | 20.8 (new) | `games.md` had only failure paths for posture (20.4) and no fragment-render scenario. |
| `docs/specs/worlds.md` | 25.6 (new) | 25.5 covers the posture *POST* endpoint's outcomes, not the edit-form render. 25.5 was not reusable here. |

### Does the new HTTP coverage fully carry the browser assertions?

**Yes for both**, so the browser copies are deletable — but that deletion
belongs to ticket 09's keeper-set pass, not here. Per-test:

| Browser test | Browser assertions | Carried by the HTTP test? |
|---|---|---|
| 21 (`test_games_panel_renders_posture_fragment`, browser_games 27.1) | mode/perspective/tense select values; three preset selects visible | **Yes, and stronger** — the HTTP test asserts the same values *and* the auto-save routes and the no-degradation path. |
| 24 (`test_world_edit_form_renders_posture_selects`, browser_worlds 29.1) | three posture selects visible after the Edit click | **Yes, and stronger for content** — asserts the stored values are selected. |

**One caveat, stated rather than buried.** The HTTP test hits
`/worlds/:key/edit` directly. The browser copy reached it through a click hop
(worlds tab → Edit button → htmx swap). That click→POST-or-GET wiring is
exactly the residue ticket 09 owns ("wiring does not demote"), so this ticket
correctly does **not** claim it. The same applies to browser test 21's
tab-switch hop. Deleting those browser copies is only safe once ticket 09
supplies the wiring guard.

### Validator, before and after

Before (at 08b base, `2abf7b9`): `138 declared, 138 covered, 0 gap(s), 0
orphan(s), 0 untagged, 0 surface mismatch(es), quarantine 86/86`.

After: `140 declared, 140 covered, 0 gap(s), 0 orphan(s), 0 untagged, 0 surface
mismatch(es), quarantine 86/86`.

138 + 2 added = 140. The validator was not weakened.

### Verification

Full gate green: clippy OK, **1598 integration passed / 0 failed / 2 skipped**
(1596 + 2 new), **19 browser passed / 0 failed**, architecture 1 passed,
guardrails 129 passed.

Both tests were mutation-checked to confirm the assertions are load-bearing,
not accidentally matching:

| Mutation | Result |
|---|---|
| Rename the `system_preset_id` select in the games template | games test fails |
| Force the tense option to never mark `past` selected in the world template | world test fails |

Both templates were restored immediately, and both tests pass again.
