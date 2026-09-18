# Decide the UI verification design: what the browser tier is for, and what the app exposes for it

Type: grilling
Status: resolved
Blocked by: 01, 02

## Question

**HITL.** Grill the user — do not infer an answer from the evidence alone.
Consume the resolutions of "Audit the current browser-test design",
"Research how mature projects verify HTMX apps", and "Root-cause the hx-post
change-event no-fire" and settle, from first principles:

1. **What the browser tier is for.** After the tier audit, which behaviours
   genuinely require a real browser, and which should verify at request level?
   What happens to the current 26 tests (keep / move to HTTP / delete)?
2. **Readiness.** How does a test know the UI is ready to act on — the
   no-fire race (resolved in ticket 03: `change` dispatches inside htmx's
   20 ms `defaultSettleDelay` window, before trigger listeners attach) must
   be structurally impossible in the chosen design, not polled around.
3. **App-side vs harness-side.** Should the dashboard emit explicit readiness
   signals (htmx lifecycle hooks, a ready flag), should the harness interpret
   htmx state, or should verification skip the browser where possible so the
   question dissolves? Weigh app-changes against test-changes honestly per the
   user's "nothing is sacred, prefer simplification" standing preference.
4. **Fixture shape.** Per-test server+browser vs shared fixture (invariants.rs
   already shares one) — and whether the shared `#story-log .log-entry` gate
   at browser.rs:87 survives, and as what.
5. **What gets deleted.** Harness surface, wait primitives, duplicative tests —
   the design should name what it removes, not only what it adds.

The map's destination is a shipped redesign, so this decision is the last
planning step: the verdict on each of the five above, plus the concrete
implementation tickets it graduates (name them, create them as child issues,
wire their blocking). The remaining map is executing those tickets.

Record the decision under `## Answer`, create the implementation tickets the
decision specifies, and append a one-line gist + link to the map's
Decisions-so-far.

## Answer

Decided with the user (grilling, 2026-09-16). The two problems with the browser tests are
(a) flake and (b) speed; the answer is a three-tier decomposition ("D2"), executed
prototype-first.

**Bloodline of the decision.** The research asset's ranking was not trusted wholesale
the user's distrust was specific to the settle-gate's call-site discipline and its
target-scoping being unsourced. The design therefore shrinks the suite's exposure to
that primitive to the handful of tests that genuinely need a full app, and verifies it
empirically before multiplying it.

**Tier spine (the placement rule, ratified).** A test is placed by one rule, in this
order: "Could curl observe this?" → tier 1 (HTTP contract). "If the server behind this
were fake, does the behaviour change?" No → tier 2 (quick browser). Otherwise → tier 3
(full-stack). Tie-breaker: when in doubt, file down. Tier 3 is the exception list. The
rule is written into `tests/STRATEGY.md` with the 26-test redistribution as worked
examples.

- **Tier 1 — HTTP contract.** Takes the 12 class-2 assertions (server-derived content),
  plus two new tests covering the orphaned contracts the audit exposed
  (`POST /worlds/:key/posture` and the `#game-posture-controls` fragment render), and
  deletes the one class-3 duplicate (`test_games_mode_switch_retargets_and_nudges`,
  identical twin of an existing HTTP test).
- **Tier 2 — quick browser (new).** A small stub server in `tests/test_utils/` serves
  the real `assets/index.html` shell plus canned fragments (the five pollers'
  responses); one shared browser process, fresh page per test. Takes ~10–11 of the 13
  class-1 tests (slash menu ×7, options edit-fill, story-log edit/cancel, polling
  pause, error toast, invariants). Accepted tax: canned fragments can drift from real
  templates; updating the fixture is fixture maintenance, done in the same change as
  the template change.
- **Tier 3 — full-stack browser.** Keeps what needs the app's real logic: tests 14/15
  (prior test-strategy-execution ruling: their value is client wiring),
  `test_options_dock_survives_reload`, and ~3–4 parameterised wiring smoke guards
  (posture change in worlds + games, preset chain, options Use) asserting real server-
  side state change. Per-test `TestServer` + browser unchanged.

**Readiness (tier 3 only).** Settle-gate inside the harness interaction wrappers: an
`add_init_script` `htmx:afterSettle` counter, baseline armed before the interaction,
polled over `evaluate_value` (playwright-rs 0.9.0 has no `wait_for_function`). Test
authors keep writing plain `select_option`/`click`; enforcement lives inside the
helper, so discipline is not opt-in. `hx-preserve` is the ratified fallback for any
surface that resists wrapping.

**Verification bar (option C).** Before the acceptance runs, delete the nextest
`retries = 1` override so the gate cannot silently retry a surviving race green. Then:
5 consecutive green full-gate runs with zero legacy flake signatures, plus a ~50-run
stress loop on the posture flow counting old signatures (acceptance = zero). The
browser-binary serialization override is deleted once the stress loop is green, with
wall times measured before/after.

**What gets deleted (ratified).** `wait_for_condition_sync`, `extract_port_from_url`,
`GET /status/ready` (never requested by the UI), the duplicated server-ready probe in
`goto_with_connection_check`, a suite convention banning `networkidle`, and the shared
`#story-log .log-entry` precondition — retired into an explicit `wait_for_story_log()`
called only by tier-3 tests that need it, so an infra hiccup is attributable to the
test that needed the log.

**Sequencing — prototype first (user decision).** One vertical slice on the worlds
surface (the ticket-03 flake home) proves: the settle-gate primitive, the tier-2 stub
pattern on 1–2 tests, the demoted posture contract, and the 50-run stress loop.
Go/no-go is HITL: the full rollout proceeds only if the slice holds on the machine.

**Graduated implementation tickets.** 06 (vertical-slice prototype) blocks 07–11
(tier rollouts + hygiene); 12 (acceptance gate) blocks on 07–11.

- [06 — Prototype the vertical slice: worlds posture flow across all three tiers](06-vertical-slice-prototype.md)
- [07 — Tier-2 rollout: move the remaining class-1 tests onto the stub-server tier](07-tier-2-rollout.md)
- [08 — Tier-1 rollout: demote the class-2 assertions to HTTP contract tests](08-tier-1-http-demotion.md)
- [08b — Tier-1 rollout: write the two orphaned HTTP contracts](08b-tier-1-orphaned-contracts.md)
- [09 — Tier-3 conversion: settle-gate wrappers and the keeper set](09-tier-3-conversion.md)
- [10 — Harness deletion pass: the ratified removal list](10-harness-deletion-pass.md)
- [11 — Record the placement rule and reconcile spec/test bookkeeping](11-strategy-doc-and-spec-bookkeeping.md)
- [12 — Acceptance gate: retries off, 5 green runs, stress loop, serialization override deleted](12-acceptance-gate.md)
