# Map: UI verification redesign — from per-flake patching to robust-by-design

Labels: wayfinder:map

## Destination

The redesign is **shipped**: the browser tier verifies the UI the way the design
decision settles it, the flake mechanisms are structurally removed (not merely
diagnosed), the harness surface that doesn't earn its keep is deleted, and the
full gate runs green with the relevant races impossible rather than retried
away. The application and its browser tests work today; the way is clear when
they work *robustly by construction*, the code shows it — and **5 consecutive
green full-gate runs with zero legacy flake signatures** (story-log CDN gate
timeout, posture-change no-fire) confirm it on the machine.

## Notes

- Domain: the dashboard is HTMX fragments over axum; htmx 1.9.10 is now vendored
  at `assets/htmx.min.js` (was fetched from unpkg per page load — a flake root
  cause, fixed 2026-09-13).
- Uncommitted tree state to know about: 10 modified files + `assets/htmx.min.js`
  (build-speedup + vendoring work) are sitting uncommitted as of chart date.
- Skills to consult: `research` for tickets 01–03; `/grilling` +
  `/domain-modeling` for ticket 04 (Decide the UI verification design).
- Related prior work: the [test-strategy-execution map](../test-strategy-execution/map.md)
  already designed and executed a browser-tier consolidation — its
  [ticket 07](../test-strategy-execution/issues/07-browser-tier-design.md)
  (target state: `behaviour.rs` specced + `invariants.rs` code-is-definition)
  and [ticket 08](../test-strategy-execution/issues/08-browser-tier-execution.md)
  settled tier *placement* (13 keep / 17 move-down), and
  [ticket 14](../test-strategy-execution/issues/14-browser-missing-tests.md)
  grilled missing behaviours. This map does NOT re-litigate placement; it
  answers the robustness question that effort never asked: how a test knows
  the UI is ready. The tier has since re-split per-surface (9 files, 26
  tests), so the prior answers need re-reading in that light.
- Assets from the flake investigation (read these first):
  - [Flake investigation evidence](./assets/flake-investigation-2026-09-13.md)
  - [Browser tier inventory](./assets/browser-tier-inventory.md)
- Provided external research: a user-run Gemini deep-search survey,
  [HTMX Verification Testing Patterns.txt](./assets/HTMX%20Verification%20Testing%20Patterns.txt)
  (23K, 24 sources). Ticket 05 (Analyze the Gemini deep-search survey)
  validates it against the repo; ticket 02 is blocked on that analysis so it
  extends rather than redoes it.
- Execution override: this effort **carries execution into the map** (overrides
  plan-don't-do). Tickets 01–04 still resolve in order — research the evidence,
  then grill the design — and the design's Answer graduates implementation
  tickets, which subsequent sessions resolve one at a time until the
  destination is shipped.
- Standing preferences (from the user):
  - First-principles robustness over per-flake patching. Fixing one flaky test
    while the design invites the next one is failure.
  - Either the application design or the browser-test design may change; nothing
    is sacred. The project is deliberately not complicated — prefer
    simplification over added machinery.
  - Be suspicious of accumulated diagnostics and harness structure; judge
    whether each piece earns its keep rather than assuming it does.

## Decisions so far

<!-- the index — one line per closed ticket: enough to judge relevance, then zoom the link for the detail the ticket holds -->

- [Analyze the Gemini deep-search survey on HTMX verification patterns](issues/05-analyze-htmx-deep-search.md) — the survey's patterns are worth keeping (three-tier model, polling-vs-lifecycle diagnosis, targetId-scoped readiness protocol); its repo facts are a stale snapshot and need re-verifying. The htmx#2787 newline claim is a fixed 2.0 regression that does not apply at 1.9.10, and the Stack Overflow source cited for the readiness protocol does not support it.
- [Research how mature projects verify HTMX apps: what do the good patterns look like?](issues/02-research-how-to-verify-htmx-apps.md) — event-driven readiness on `htmx:afterSettle` is the sourced pattern (listeners attach in the settle task; confirmed in the vendored 1.9.10 source); only `hx-preserve` makes the no-fire structurally impossible; request-level is the maintainer-endorsed default tier; JSDOM tier and `networkidle` ruled out; playwright-rs 0.9.0 forces the readiness helper to be `add_init_script` + `evaluate_value` polling. Six-approach shortlist handed to the design decision, with per-option performance/reliability/maintainability assessment (speed levers: tier demotion, then de-serialization; the settle-gate buys correctness, not speed).
- [Root-cause the hx-post change-event no-fire: what actually stops the POST?](issues/03-root-cause-the-hx-post-no-fire.md) — racy by construction, fix class is a harness-side readiness gap. Its specific mechanism (the `change` lost inside htmx's 20 ms `defaultSettleDelay` window) was **refuted by ticket 06's experiments** — correction appended to that ticket's Answer: the operative race is interacting before the *registering* swap settles, and the delay setting is neither necessary nor sufficient.
- [Audit the current browser-test design: what does each test actually verify, and through which layer?](issues/01-audit-current-harness.md) — the 26 tests split 13 browser-only / 12 browser-observable app wiring / 1 HTTP-covered; the tier files by surface where ticket 07 filed by assertion shape, and two HTTP contracts (`POST /worlds/:key/posture` and the games posture fragment) are covered *only* through Chromium, so demotion adds coverage rather than trading it. Every readiness primitive is a visibility or count poll; the app exposes **no** htmx-readiness signal (`HX-Trigger`, `afterSettle` listener, `hx-preserve` are all absent), so tuning timeouts cannot fix the race. Harness verdict: core load-bearing, shape accidental; dead surface named.
- [Decide the UI verification design: what the browser tier is for, and what the app exposes for it](issues/04-decide-ui-verification-design.md) — three tiers (D2): HTTP contracts absorb the 12 class-2 assertions + the two orphaned contracts + delete the 1 duplicate; a new quick-browser tier (stub server, real shell, canned fragments, shared browser process) absorbs ~11 of the 13 class-1 tests; tier 3 keeps the 5-rule-07/wiring tests + ~4 smoke guards, all behind a settle-gate *in the harness wrappers* (afterSettle counter, `hx-preserve` as fallback). Placement is one rule with worked examples ("could curl observe this?" / "if the server were fake?"; in doubt, file down); `networkidle` banned, dead wait surface + `retries = 1` + `/status/ready` deleted; verification is 5 green gate runs + ~50-run posture stress loop after retries go. Graduates tickets 06–12, starting with **06 — the worlds-posture vertical slice** (prototype-first, HITL go/no-go) which blocks 07–11; 12 is the acceptance gate.
- [Prototype the vertical slice: worlds posture flow across all three tiers](issues/06-vertical-slice-prototype.md) — **go** (HITL, 2026-09-17). The slice refuted ticket 03's 20 ms mechanism (44/50 with the delay zeroed) and proved the corrected design: `detail.elt` target-scoping is required (pollers satisfy a plain counter) and every swap-triggering helper must await that swap's settle before returning (registering-swap rule, structural in helpers). Shipped with the verdict: 4 HTTP contract tests for `POST /worlds/:key/posture` (SCENARIO 25.5), the tier-2 stub + 2 ported tests (~2 s saved per test = engine boot, a constant not a ratio), 50/50 stress runs with retry masking off at stock htmx behaviour, full gate green. Attached rulings: ticket 07 gets a drift-tax exit check on its first fragment-reading ports; ticket 09 owns the registering-swap rule. Full per-run tables and the wrong-hypothesis record in the ticket's Answer.
- [Tier-2 rollout: move the remaining class-1 tests onto the stub-server tier](issues/07-tier-2-rollout.md) — 12 class-1 tests moved onto the tier-2 stub (slash menu ×7 incl. the re-render port, options edit-fill, story-log ×3, error toast, invariants ×9); the two ticket-06 mirrors are now the only copies. Drift-tax exit check: real but bounded — tests 10–12 exercise client JS in the real shell against a canned structural fixture, so content drift fails loudly; the one gap is that the gated structural check on the edit path now lives only in the fixture (recorded for ticket 11). Tier-2: 12 tests in 4.70 s; full browser binary 26 passed in 20.23 s direct vs 490 s under nextest's serialization override (input for ticket 12).

- [Tier-1 rollout: demote the class-2 assertions that already have HTTP siblings](issues/08-tier-1-http-demotion.md) — 7 browser tests demoted/deleted after adding 4 HTTP twins first (options use-click via `POST /action/check`, story-log delete via `/history/delete` + fragment re-fetch, games posture success-path re-render, preset duplicate→edit-flags→save chain); the 26→19 browser drop matches the deletions exactly. Demotion is a spec restructuring act, not a delete: a `browser_*` tag outside `tests/browser/` is a surface violation and an uncovered declared scenario is a gap, so each scenario was re-expressed in its HTTP spec (24.13, 8.4, 20.7, 21.27) and retired from the browser file; `browser_prompt_presets.md` and 3 browser modules became empty and were deleted. Validator 141/141 → 138/138, 0 gaps, not weakened. Two audit sibling pointers were optimistic — for tests 22 and 23 neither named sibling asserted the fact, so coverage was added rather than deleting blind. **Unfixed bug found:** `generate_preset_id()` is millisecond-based, so a create+duplicate in the same millisecond collides and the copy overwrites the source (double-click on Create/Duplicate); the new test seeds a fixed id, and the bug warrants its own ticket.

- [Tier-1 rollout: write the two orphaned HTTP contracts](issues/08b-tier-1-orphaned-contracts.md) — the coverage *gain* half of the original ticket 08 split: 2 HTTP tests added, nothing deleted. `GET /fragment/games` (games.md 20.8, new `tests/http/games_fragment.rs`) asserts the games posture fragment renders the active game's stored values selected plus its auto-save routes; `GET /worlds/:key/edit` (worlds.md 25.6, appended to `tests/http/worlds.rs`) asserts the edit form renders the world's stored posture selected. Both fragments were previously reachable only via Chromium, so this adds coverage while removing the race. HTTP coverage fully carries both browser assertions (21 and 24), so their copies are deletable — but the click/tab-switch wiring hop is ticket 09's residue, so deletion is deferred to 09's keeper-set pass rather than claimed here. Validator 138/138 -> 140/140, not weakened; both tests mutation-checked.

- [Tier-3 conversion: settle-gate wrappers and the keeper set](issues/09-tier-3-conversion.md) — tier 3 now holds **7 tests**, every interaction behind a settle-gate helper, and the gate **cannot be bypassed**: a new guardrail fails the build on any raw `.click(`/`.select_option(` in a `with_test_page` file (`tier2.rs`/`invariants.rs` exempt by scope, not names). `settle_gate.rs` gained **`await_panel_ready`**, a baseline-free wait — every panel is `hx-trigger="load"`, so its swap lands before any baseline is armed and a baseline-relative wait can never see it; this closed a latent gap in `open_world_edit`. Tests 21/24 deleted, both wiring hops folded into the games/worlds guards (HTTP twins 20.8/25.6 carry their content). Four guards added (worlds, games, preset chain, options Use), `send_action` reused rather than duplicated, story-log precondition split (no tier-3 test needs it). Six plan-mechanics corrections recorded, incl. that `/action/check` retargets to `#status-display` (not `#action-area`) and that strict-mode multi-match needs `nth`/`:has` scoping. Stress loop **50/50, 0 lost interactions** on the final guard form; full gate green (1604 integration / 135 guardrails / 20 browser); validator 141/141.

- [Harness deletion pass: the ratified removal list](issues/10-harness-deletion-pass.md) — every item of ticket 04's ratified list is now deleted, folded, or declined with evidence. `/status/ready` is gone end to end (route, handler, re-export, both coverage-only tests) after confirming the boot probe hits `GET /` and `wait_for_status_ready` polls the DOM, not the route; `REQUIRES_MIGRATION_TEST_COUNT` 86→85 and the routes doc regenerated (the gate's `http-routes-check` makes that mandatory). `wait_for_condition_sync` + `extract_port_from_url` deleted (zero call sites; every other `wait.rs` primitive has a live caller). The duplicated boot probe folded into `TestServer`'s own 30 s probe. Two items closed against their original text: the `networkidle` ban text belongs to ticket 11 (zero uses found, evidence handed over), and the `invariants.rs` story-log twin no longer exists since ticket 07's restructure. **One gate failure surfaced, and the offending test was deleted**: `test_extract_http_routes.py` hardcoded 57 route rows — after lowering it to 56 the whole test went, since a count you must hand-edit on every route change is a maintenance trap and its useful assertion already lives in `test_table_row_count_matches_routes` (relative, not pinned). Python tests 125→124. `wait.rs` 260→236, `browser.rs` 394→392; full gate green (1604 integration / 137 guardrails / 20 browser / 1 architecture, 0 failed); validator 141/141, quarantine 85/85.

- [Record the placement rule and reconcile spec/test bookkeeping](issues/11-strategy-doc-and-spec-bookkeeping.md) — the placement rule is now the decision of record in `tests/STRATEGY.md`: the old 4-line "Browser placement test" became `## Placement rule: which of the three UI tiers` (tier table grown to five rows with a *Quick browser (tier 2)* row, the three-step rule, the "when in doubt, file down" tie-breaker, a 5-row worked-example table citing real test paths from the finished suite, tier 2's loud-failure tax, and the readiness gates), with ticket 10's `networkidle` ban folded in (zero uses re-verified). **The sweep found zero scenario drift to fix** — the validator was green before any edit (141/141, 0 gaps/orphans/untagged/surface mismatches) and identical after, so no test and no scenario changed and the validator was not weakened. Two doc-drift corrections the validator cannot see: `STRATEGY.md`'s mirror example named the deleted `slash_menu.rs`, now restated as the tier-2 consolidation exception (`tier2.rs` hosts class-1 scenarios from four specs, mirror is per-test); `tests/AGENTS.md`'s preamble said four tiers, now the three-tier placement rule. **Flagged for the user, not assumed**: if the mirror convention itself should change (split `tier2.rs` per spec), that is a rule change, not a bookkeeping fix. Ticket 07's recorded drift-tax gap was **corrected, not carried**: its "edit/delete assertions live only in the quarantine" claim is wrong (the unit tier pins both via `test_story_log_template_has_message_actions`); the real gap — no gated test pinned the edit-JS hooks `data-raw-text`/`data-id`/`class="text"` on the real template, mutation-proven silent (renamed hook, gate stayed green, Edit opens an empty textarea) — graduated as ticket 14 and is closed. Full gate green (1604 integration / 137 guardrails / 20 browser / 1 architecture); docs-only change (`STRATEGY.md` +89/−16, `AGENTS.md` 1 line).

- [Pin the message-edit template hooks the edit JavaScript reads](issues/14-pin-message-edit-template-hooks.md) — one unit test (`test_story_log_template_edit_path_hooks`, `src/adapters/driving/http/templates_tests.rs`) pins the real template's `data-id`, `data-raw-text`, `class="text"`, the `showEditForm(1)` onclick wiring, and the `| escape` wire form (`&#34;`). Five mutation runs, five catches — four renames plus the `| safe` escape bypass, whose diagnostic shows the broken wire form directly. Review corrections recorded: the draft's "remove `| escape`" mutation was a no-op (askama auto-escapes `ext = "html"` templates) and was replaced with the `| safe` bypass; the nesting residual (`.text` inside `.log-entry`) accepted, since a substring assert cannot pin nesting and a parser/browser check is machinery the project does not want for one assertion. Full gate green (1605 integration / 137 guardrails / 20 browser / 1 architecture); validator 141/141 unchanged; production source untouched.

- [Acceptance gate: retries off, 5 green full-gate runs, stress loop, serialization override deleted](issues/12-acceptance-gate.md) — **the destination is satisfied.** `retries = 1` removed, then 10 green full-gate runs with single attempts (5 with the browser serialization override, 5 in the shipped config): every run `1605 integration / 137 guardrails / 20 browser / 1 architecture`, 0 failed, zero real `FLAKY` lines, zero legacy signatures. Two independent 50/50 posture stress loops (before and after the override deletion) with 0 lost interactions. The browser `threads-required = "num-test-threads"` override is **deleted**: 54.2s → 19.2s (**2.8×**, measured over 12 direct runs + 17 total), no spawn-contention flake reproduced on the 8-core box; full gate 66–71s → 33.7s. Validator 141/141 unchanged (config-only change). Ticket 07's "490s under nextest's serialization override" observation is retroactively explained by this override. Only file changed: `.config/nextest.toml` (−9 lines). `tmp/acceptance/` and the two loom logs are left for ticket 13's sweep.

- [Branch-wide temporary-code and scratch-artifact sweep](issues/13-branch-temporary-code-sweep.md) — **the branch is clean, and the map is closed.** `scripts/measure_tiers.sh` deleted (spent; its table lives in ticket 07's Answer; the stale test list would have needed repair only to be deleted); `scripts/stress_posture.sh` **deleted in `7b69574`** — the ticket-13 Answer recorded it as a keeper first, and the correction is appended to that ticket: the settle gate's `expect_settled` panics after its 5 s timeout, so a broken gate fails loudly in one ordinary gate run, and the script's real value was ticket 12's one-time acceptance measurement rather than ongoing protection. **Worktrees ruled (a) not a supported workflow** — nothing creates one, no home is documented, and the sanctioned mechanism is `--target-dir`; no `run_cleanup` step added. `tmp/` 24 MB → 23 MB: ticket 12's `acceptance/` and `posture_stress/` plus `ticket14/` deleted; `clean_tmp_dirs` removed nothing (oldest file 29.1 days, under its 30-day threshold — correct behaviour, residual left to age out). `logs/` declined (gitignored, `build_*.log` trimmed at 3 days, `build_history.txt` self-bounded at 1000 lines); one nuance recorded, not fixed: `chronicler_*.log` (114 MB) sits outside `clean_old_logs`'s `build_` prefix. Stub fixtures, the stub server (`tier2_stub.rs` at resolve time; `stub_server.rs` after `e981b5b`), and the smoke-break extension confirmed keepers; the drift-tax comparator declined against ticket 07's loud-failure bound (and ticket 14 now pins the real template side). **Plan-archival rule recorded as ticket-tied** and applied: the two ticket-tied resolved plans archived (`ticket-10`, `ticket-17`), `docs/plans/` 31 → 29, `old-docs/archived-plans/` 17 → 19; `ticket-09-tier-3-conversion-...md` turned out never to have existed. Item 9 confirmed: zero untracked files, all deliverables committed. Full gate green on the final tree (1605 integration / 137 guardrails / 20 browser / 1 architecture; validator 141/141, quarantine 85/85; `Total: 33.60s`); no `src/` or test change.

## Not yet specified

**The map is complete.** Every ticket is resolved, the last one (ticket 13)
carried the final post-acceptance sweep, and the frontier is empty. Nothing
remains in the fog: there is no open question between here and the destination,
which ticket 12 met and ticket 13 left standing on a clean tree. Further work
beyond this point is a fresh effort, not a resumption.

(Sweep fog graduated 2026-09-19 as ticket 13 and is now closed: the branch's
temporary scaffolding — one-off measurement scripts, the two spent git
worktrees in `tmp/`, and the `docs/plans/` archival convention — was a single
post-acceptance sweep, since ticket 12 still needed `scripts/stress_posture.sh`
and the then-current `tmp/` state. Editing an instrument before it produced its
final evidence would have inverted the order. Both instruments ended up
deleted: `measure_tiers.sh` in the sweep itself, `stress_posture.sh` afterwards
— see ticket 13's appended correction.)

(Fog graduated by ticket 04's resolution: implementation became tickets 06–12;
the serialization override + `retries = 1` question is decided in ticket 12's
acceptance gate; spec/test-tag bookkeeping is ticket 11; the shared story-log
gate became an explicit `wait_for_story_log()` in tickets 09–10.)

(Fog graduated by ticket 06's verdict, 2026-09-17: poller-noise scoping is
**resolved required**, not fog — the gate is target-scoped by construction; the
drift-tax question graduated as ticket 07's exit check, not as a new ticket;
and the registering-swap rule graduated as a structural ticket-09 requirement
rather than a convention.)

- None currently.

(Fog graduated by ticket 07's resolution, 2026-09-18: the ticket-08 scope split
on **axis A** — spec-surface reconciliation folded into each rollout ticket so
each exits with a green `validate_feature_spec.py`, since the validator rejects
a `browser_*.md` tag from `tests/http/` *and* flags an uncovered declared
scenario, so a demotion is a spec restructuring act rather than a deletion.
Ticket 08 keeps the demotions-with-existing-siblings (16, 18, 19, 20, 22, 23 + the
class-3 delete 25); the newly created **08b** owns the two orphaned contracts
(games posture-fragment render, world edit-form render) and deletes nothing.
Ticket 11 therefore shrinks to the `STRATEGY.md` placement rule, the
`networkidle` ban, and a final sweep.)


## Out of scope

- The CDN-vendoring fix itself — already done (assets/htmx.min.js +
  `assets/index.html`), recorded in the investigation asset.
- A top-to-bottom re-litigation of tier *placement* — settled by
  test-strategy-execution tickets 07/08 (see Notes). Exception: the tier has
  drifted since that ruling (26 tests now, 13 kept then), so ticket 01 flags
  drift and ticket 04 asks the user explicitly what to do about it; the
  default is that the 07/08 ruling stands. Note the later growth was partly
  deliberate — test-strategy-execution tickets 14/15 grilled + implemented
  new browser tests (responsive layout, error toast) — and the per-surface
  re-split (ticket 20) reorganized behaviour.rs into files; ticket 01 notes
  provenance per file rather than treating all growth as drift.
- Build-pipeline work (gate steps, flaky epilogue, coverage) — separate effort,
  mostly completed 2026-09-13.
- Environment tuning (the move to an 8-core WSL guest) — done, recorded in the
  investigation asset.
- Narrative/LLM pipeline behaviour — verification of *what* the LLM generates,
  vs this map's concern: *that* the UI wires and renders it correctly.
