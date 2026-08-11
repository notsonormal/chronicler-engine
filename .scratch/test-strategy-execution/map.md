# Map: chronicler_engine test strategy — execution

Labels: wayfinder:map

## Destination

The chronicler_engine component tier dissolved — tests moved to their
proper tiers (unit / HTTP E2E), specs grown alongside the E2E tests, the
three retry-spec code changes landed, browser tests moved per the audit,
nextest tuned to the new suite. In parallel, a codebase-wide investigation
maps spec and tier coverage gaps across all components (~950 tests, 2
specs) — producing fog for a future effort, not implementing fixes. The
way is clear when the component tier is gone, every moved test has a home,
and the investigation has mapped the landscape.

## Notes

- **Execution-carrying** (overrides planning-default): this map carries
  execution into the map itself, not just decisions. But execution is
  scoped to dissolving the component tier; the rest of the codebase is
  investigation only.
- Domain: Rust, hexagonal-ish architecture (`adapters/application/domain`),
  cargo-nextest, test binaries under `tests/`.
- **Predecessor map:** [test-strategy/map.md](../test-strategy/map.md) —
  the planning map. All its decisions are input context. Key inputs:
  - Ticket 04's test disposition for action_pipeline + retry (9 delete,
    6 async→unit, 22 flow→HTTP E2E, ~16 port down, 4 drift→spec):
    [asset](../test-strategy/assets/pipeline-suite-diff.md)
  - Tickets 07/13's tier rules, codified in
    `tests/STRATEGY.md`
  - Ticket 05's retry spec (`docs/specs/retry.md`) —
    3 code changes required
  - Ticket 14's browser audit: 17 tests → HTTP E2E, 13 stay:
    [asset](../test-strategy/assets/browser-tier-audit.md)
- **Grilling decisions (this session):**
  1. Specs are living documents, grown alongside HTTP E2E tests, derived
     from component tests. Existing specs (action_pipeline, retry) are
     reference material, not templates to translate.
  2. The consolidation splits by component (action pipeline, retry,
     lifecycle/sequencing/arrival) and by track (unit vs HTTP E2E).
  3. Code changes (3 retry/retrigger fixes) are a separate ticket — they
     block retry work but not action pipeline work.
  4. The lifecycle, sequencing, and arrival tests weren't in the diff
     asset — their classification and spec-worthiness need research (ticket
     06) before implementation tickets can graduate.
  5. The browser tier needs a forward-looking design (ticket 07, grilling)
     before the mechanical moves (ticket 08) — the audit answered what
     moves down, not what the tier should look like.
  6. The codebase-wide audit is investigation only — it maps gaps but
     doesn't fix them. A grilling ticket (10) decides how to partition it;
     research tickets graduate from that.
  7. Both specs (action_pipeline, retry) are updated during E2E test
     writing, not treated as fixed inputs.
- Standing preferences: unit tests are branch-exhaustive (every branch,
  sync or async, fakes at driven ports); HTTP E2E validates specs
  end-to-end through the real driving adapter; browser tests are
  presentation-layer only; driven-adapter tests cover the storage seam.
  No mockall — hand-written fakes at trait ports only.
- Skills to consult: `/grilling`, `/domain-modeling`, `/to-spec` (when
  growing docs/specs).

## Decisions so far

<!-- one line per closed ticket: gist + link -->

- [Storage integration spec decision](issues/16-storage-spec-decision.md) — storage seam stays spec-less; STRATEGY.md needs explicit exemption clause (test code is the definition). Physical position of storage tests deferred to ticket 17.
- [Land the three retry-spec code changes](issues/01-code-changes.md) — all three landed (R4.8 system-log preservation via `Some(&mut state)`; R4.3/R4.4 pre-spawn anchor check → 500; R5.3/R5.4 generation gate on retry/retrigger returning `ProcessActionResult`), 6 new tests, 1358 pass. Unblocks retry tracks (04, 05, 06).
- [Port action pipeline unit tests down from the component tier](issues/02-action-pipeline-unit.md) — component tier dissolved for action pipeline + collaborators + GameCatalogue. New `catalogue_tests.rs` (12) + `gate_tests.rs` (3); 14 assertions ported into `pipeline_tests.rs` (S1.2/1.3/1.4/1.5/2.1/2.3/3.1/3.4/4.1/5.1/5.2 + async 3.2/3.3/4.2/4.3); `collaborators.rs`, `action_pipeline/actions.rs`, `action_pipeline/pipeline.rs` deleted. 1358 pass, 26/26 guardrails. ~~S2.4 drift deferred to ticket 05~~ → RESOLVED by ticket 11 (spec corrected Idle→Error); mislabeled `test_pipeline_trigger_complete_failure` fog now tracked by ticket 04. reset() error-branch fog recorded for a possible future ticket.
- [Move action pipeline flow tests to HTTP E2E and grow the spec](issues/03-action-pipeline-http-e2e.md) — 8 `flow/sequence.rs` tests ported to `tests/http/flow_sequence.rs` (S6 sequencing / S7 reset / S8 delete, HTTP-framed via POST /action / /swipe/new / /history/delete / /reset); `flow/sequence.rs` deleted; `validate_feature_spec.py` scan dirs fixed (tests/http, src, tests/integration/storage). S6/S7/S8 scenarios moved to new `docs/specs/flow.md` (flow ≠ action pipeline; pilot dedups IDs across specs, flow owns 6.x–8.x, action_pipeline owns 1.x–5.x). 1358 pass, 101/101 guardrails. S1–S5 reframe + non-HTTP scenario removal + 14 tag removal split to ticket 11 (spec restructure: HTTP-observable only). 5 pre-existing pilot gaps (S1.1/1.2/1.4/2.2/2.4) deferred to ticket 11.
- [Spec restructure: HTTP-observable, endpoint-named](issues/11-spec-restructure-http-observable-only.md) — `action_pipeline.md` + `flow.md` dissolved into three endpoint-named specs: `actions.md` (S1–S6, 12 scenarios), `reset.md` (S7, 2), `story_log.md` (S8, 3). Three matching HTTP test files in `tests/http/`; `flow_sequence.rs` deleted; helpers relocated to `test_helpers.rs`. 3 handler unit tests added (`Err`/`ConcurrentGeneration`/`ShuttingDown`) via real pipeline. Shutdown fix: `is_shutting_down()` guard at top of `process_action` + `retry` + `retrigger` (consistent 503). S2.4 spec corrected (Idle→Error, resolves drift deferred from ticket 02). S1.3 dropped (room change not HTTP-observable — `pending_location` set after narration pushed; user-approved). S3.4 both cases leave spec (internal state + redundant with S1.1). 14 SCENARIO tags stripped from `pipeline_tests.rs`, replaced with `//` comments. `tests/STRATEGY.md` generalized (no hardcoded IDs, snapshot clause dropped, tag placement narrowed to `tests/http/` only). `validate_feature_spec.py` `TEST_DIRS` narrowed to `[tests/http/]` + Given/When/Then/And hard-line-break format check added. Final count: **17 declared, 17 covered, 0 gaps, 0 orphans, 0 format violations**. Build green (1362 pass). Review fixes landed: S1 (validator↔STRATEGY.md contradiction), S3 (`unit_test_standards.md` `.build_app_state()`→`.build_service()`), S4 (`rust_idioms.md` shutdown-gate doc updated for process-action now checking twice), P2 (14 `//` comments above detagged tests).
- [Port retry unit tests down from the component tier](issues/04-retry-unit.md) — component tier dissolved for retry. 4 port-down tests + 2 shutdown-guard tests + 1 replace-not-append history-invariant test added to `retry_tests.rs`; 2 existing tests strengthened (`test_retry_main_narration_happy_path` Idle+completion, `test_retry_no_input` noop-on-history). `test_pipeline_trigger_complete_failure` fixed: `with_fail()`→`with_trigger_narration_fail()` + S2.4 assertions (main preserved, System log, Error). Loose `test_retry_handler`/`test_retrigger_handler` ("any 2xx/4xx/5xx") replaced with 4 specific handler tests (503 shutdown + 400 validation, both endpoints). `tests/integration/application/action_pipeline/retry.rs` deleted (-10), `mod pipeline_retry` unwired. 1361 pass, 17/17 spec, 0 violations. `flow/retry_main.rs` + `flow/retry_event.rs` + `retry.md` scenario edits deferred to ticket 05.
- [Move retry tests to HTTP E2E, split spec into endpoint-named specs](issues/05-retry-http-e2e.md) — component tier dissolved for retry flow. `docs/specs/retry.md` deleted; split into `swipe_new.md` (17 scenarios 9.x–12.x, POST /swipe/new) + `retrigger.md` (9 scenarios 13.x–15.x, POST /retrigger) per ticket 11's endpoint-naming rule. R5.1/R5.2 (cancellation) dropped from specs — unit-only. `tests/http/swipe_new.rs` (17 tests) + `tests/http/retrigger.rs` (9 tests) created; 6 existing `fragment.rs` tests moved + retagged (concurrency tests kept `try_claim` pre-claim pattern, no flake). S1.6 added to `actions.md` + `actions.rs` (regression guard for deleted `test_trigger_continuation_runs_quantifier_and_detects_new_npc`). `flow/retry_main.rs` (10) + `flow/retry_event.rs` (3) deleted; unwired from `tests/integration/mod.rs`. One helper added (`app_with_narrator_and_quantifier`). 1369 pass, 44/44 spec, 0 violations, 101/101 guardrails. **Two-axis review done** (Standards + Spec, parallel sub-agents): 6 standards findings (1 hard self-ref comment, 5 smells — NpcCard dup, mysterious name, dead `_n` binding, restated-code box dividers; 1 suppressed by plan's "one helper only" decision, 1 pre-existing spec code-indexer → docs-hygiene) + 3 spec findings (10.1 missing active-swipe assertion + `>=2` should be `==2` per I.5; 11.7 seeds Input not Narration + missing swipes-unchanged assertion). P3 "pipeline_retry scope creep" was a false positive (ticket 04 work). **All 8 fixes applied** (7 from review + `>=2`→`==2`): self-ref comment dropped, NpcCard→`TestNpc::*` fixtures (S1.6+S9.5), `seed_event_flow` param renamed, dead binding dropped, 8 box dividers removed, 10.1 active-swipe assertions added, 11.7 seeds Narration + swipes-unchanged assertion. Suite green: 1369 pass, 44/44 spec, 101/101 guardrails, 0 build warnings.
- [Classify lifecycle, sequencing, and arrival tests](issues/06-lifecycle-sequencing-arrival.md) — 13 tests classified (5 port down, 8 delete, 1 new spec `games.md`, 0 drift). Sequencing portion already done by tickets 03+11; remaining 13 = `lifecycle.rs` (10) + `flow/arrival_persistence.rs` (3). Lifecycle: 8 delete as redundant with `catalogue_tests.rs`/`game_tests.rs`; 2 port down as net-new branches (`create_game_persists_scenario_message_and_swipe`, `delete_game_succeeds_silently_for_nonexistent_game`). Arrival: all 3 port down to new `src/application/arrival_service_tests.rs` (bootstrap-triggered, not HTTP-observable; no spec). New `games.md` spec sketched (7 scenarios: 9.1–9.3 create, 10.1/10.2 switch, 11.1–11.3 delete; one spec with three subsections recommended, 3 endpoint-named specs is the alternative). Asset: [lifecycle-arrival-disposition.md](assets/lifecycle-arrival-disposition.md). Graduates two tickets: lifecycle unit (AFK) + lifecycle HTTP E2E + spec (HITL).
- [Design the browser tier's target state](issues/07-browser-tier-design.md) — browser tier splits into `behaviour.rs` (6 tests, specced in new `docs/specs/browser.md`, tags allowed) + `invariants.rs` (7 tests, no spec, test code is the definition, named exemption). STRATEGY.md + guardrail amended: SCENARIO tags allowed in `tests/http/` + `tests/browser/behaviour.rs` only. Move-down list re-verified: all 17 still belong at HTTP, but 9 gaps need new HTTP tests in ticket 08 (7 fragment assertions + 2 I.5 restorations as new `actions.md` scenarios). S1.3 conflict (#6) dissolved into Invariant I.1 — delete, no replacement. No HTTP dual coverage for behaviour tests. Graduates grilling ticket "What browser tests are missing?" (blocked by 07).
- [Execute the browser tier changes](issues/08-browser-tier-execution.md) — 17 move-down browser tests deleted (4 files: `trigger/editing/structure/interaction.rs`); 13 keep-tests reorganized into `behaviour.rs` (6, tagged 16.1–16.6) + `invariants.rs` (7, named exemption); 6 new HTTP tests landed (7 fragment assertions consolidated into 3 by setup/endpoint shape; 1 dropped — `test_input_no_required_attribute` was inverted/buggy, asserted `!required` but template has `required minlength="1"`; 2 I.5 restorations as `actions.md` 1.7 + 1.8); new `docs/specs/browser.md` (6 scenarios 16.1–16.6); `STRATEGY.md` + `validate_feature_spec.py` `TEST_DIRS` amended (no SCENARIO-placement guardrail exists; validator is the enforcement). Dead helpers removed (`element_exists`, `element_count`, `get_status`, `wait_for_log_entries`, `wait_for_non_loading_value`). Two-axis review landed 3 fixes (misnamed `test_index_page_has_connection_status` → `test_header_fragment_has_connection_status` moved to `fragment.rs`; S1.7 raw `NpcCard` → `TestNpc::named`; `quantifier`→`quantifier_provider` consistency). Final: **52 declared, 52 covered, 0 gaps, 0 orphans, 0 format violations; 1356 pass, 2 skipped; 101/101 guardrails; 0 build warnings.** Deviations user-approved. Out-of-scope fog: `required minlength="1"` template may be a bug (conflicts with S1.5 empty-command continuation) — future UI/template audit, not this map.
- [Port lifecycle + arrival unit tests down from the component tier](issues/12-lifecycle-unit.md) — 5 unit tests landed (2 in `catalogue_tests.rs`, 3 in new `arrival_service_tests.rs`), 13 component-tier tests deleted (`lifecycle.rs` + `arrival_persistence.rs`), files unwired and empty dirs removed. `arrival_service_tests.rs` uses direct `MessageService::new` construction (no `WiredApp`/`AppState`/`pipeline` dead weight) and `TestDataBuilder::default_test()` so the scenario-inject branch is exercised (asserted `narrations.len() >= 2`). Final: **1348 pass, 2 skipped; 26/26 guardrails; 52/52 spec; 0 build warnings; 89.4% total coverage, `arrival_service.rs` 100%.**
- [Move lifecycle flow tests to HTTP E2E and grow the games spec](issues/13-lifecycle-http-e2e.md) — Option B: 3 endpoint-named specs (`games_create.md`, `games_switch.md`, `games_delete.md`, 8 scenarios) + 3 matching HTTP test files. 5 tests moved from `fragment.rs`, 2 ported from `games_fragment_handlers.rs` with tightened 400 body assertions, 1 new idempotent-delete test, 1 non-branch cross-world test deleted. Inline form-POST (no helper). Validator **52 declared, 52 covered, 0 gaps, 0 orphans**. Suite: **1348 pass, 2 skipped; 101/101 guardrails; 0 build warnings; 0 clippy issues**.
- [What browser tests are missing?](issues/14-browser-missing-tests.md) — forward-looking grilling of 6 candidates against the browser placement rule. **2 real gaps graduate** as one impl ticket (15): responsive layout at `< 768px` (invariant, no spec — `styles.css` declares `@media` rules nothing tests) + error-state toast rendering (`htmx:beforeSwap` isError → `showError` → `#error-notification.visible`, browser-only). **2 not gaps** (no behaviour to assert): htmx swap transitions (not used, cosmetic only) + connection-status transitions (hardcoded `connected`, no JS toggles, no disconnect detection). **2 fog**: accessibility (app claims no a11y — no ARIA / tabindex / focus management; tests would assert bugs → future UI/a11y effort, not this map) + empty-state rendering (new games always have scenario message per ticket 06; `#story-log` zero-entry state never occurs).
- [Add browser tests for responsive layout + error-state toast](issues/15-browser-missing-tests-impl.md) — 2 browser tests + spec scenario 16.7 landed. `invariants.rs::test_responsive_layout_under_768px` (set viewport 500x800, assert `.main-container` flex-direction column — proves `@media` rule wired). `behaviour.rs::test_error_toast_on_action_failure` (synthetic `htmx:beforeSwap` with `isError=true` → assert `#error-notification.visible` + non-empty text). **Deviation from ticket:** synthetic event dispatch instead of a real 500 — `route.fulfill` in playwright-rs 0.9.0 is broken for BOTH status and body (probe verified 500+body arrives as 200+empty), real server has no 500 path without production-code changes (out of scope), server-shutdown fires `htmx:responseError` not `beforeSwap`. App code under test is the body-level listener → showError → toast; htmx's 500→isError mapping is htmx's contract. Comment in test documents this. 15/15 browser tests pass, validator 53/53, build+clippy clean.
- [Decide how to partition the codebase-wide spec/tier audit](issues/10-partition-codebase-audit.md) — partition by component cluster, by intent. 4 graduating tickets: storage spec decision (16, grilling, blocks 17), integration migration (17, research, blocked by 16), missing-spec audit (18, research, HTTP E2E + browser only — domain/application/storage units have no spec per STRATEGY.md), branch-coverage audit (19, research, answers whether `ArrivalTaskContext::run` is isolated or a pattern). All investigation only — no fixes on this map.
- [Migrate existing integration tests to the right tier](issues/17-integration-migration.md) — 6 storage-driven-adapter files stay spec-less; 4 HTTP-level files port to `tests/http/` (settings, prompt-presets, CSS, visual-sidebar); 2 data-only tests in `model/world.rs` delete; LLM transport, bootstrap, and LLM E2E stay. Asset: [integration-migration-disposition.md](assets/integration-migration-disposition.md).
- [Review nextest configuration](issues/09-nextest-config.md) — retries dropped from 2 to 1 in the default `.config/nextest.toml`; `test-threads = 4` kept (sweet spot on 2-core host: -j 4 ~30s vs -j 2 ~42s vs -j 6 ~39s); default hard timeout lowered to 60s (terminate after 1 period) so hung tests fail in ~70s instead of 5 minutes; dedicated `[profile.llm]` with 300s timeout for LLM runs. Stale `tests/nextest.toml` deleted. `build.py` no longer hardcodes retries/jobs; `AGENTS.md` wall-clock expectation updated from 2-3 min to ~1 min. Six consecutive green nextest runs (1350 pass, 2 skipped, ~30s) plus `cargo llvm-cov nextest` green.

## Not yet specified

- **Browser accessibility audit.** No `aria-*` / `role=` / `tabindex` in
  any template; `showEditForm` doesn't focus `#edit-textarea`; `cancelEdit`
  restores no particular focus; no keyboard handlers. Real
  presentation-layer gaps but the app claims no a11y support — writing
  tests would assert bugs, which needs a "does the app claim a11y?"
  decision before a "where does the test go" decision. Future UI / a11y
  effort, not this map (destination = component tier dissolved).
- **Branch coverage pattern.** The consolidation may reveal that the
  component tier was masking coverage gaps beyond `GameCatalogue` (already
  identified). Per the tier rules, every branch needs unit tests. The open
  question is whether this is an isolated case or a pattern across other
  application-layer classes (`GameViewQuery`, `MessageService`,
  `GameCatalogue.reset()`). Graduates from the unit-track tickets' findings
  — if it turns out to be a pattern, it earns its own ticket. **Ticket 06
  added evidence:** `ArrivalTaskContext::run` has 5 uncovered branches
  (both-fail path, room_id-not-in-map early return, `arrival_preset`-None
  Config-error, recorder-Err status, `save_message_and_snapshot` failure
  path) — only 3 of ~10 branches covered by the ported tests. More
  evidence for the pattern; still not ticketed here (destination is
  component tier dissolved, not exhaustive branch coverage).
- **Codebase-wide spec/tier gaps.** ~950 tests across all tiers, 2 specs.
  The investigation (ticket 10 + graduating research tickets) maps which
  components have specs, which don't, and which tests are in the wrong
  tier. Each finding is fog that may graduate as a ticket on this map (if
  sharp enough and in scope) or stays as a signpost for a future effort.
  Implementing fixes for components outside the component tier is out of
  scope for this map.
- **Spec scope rule settled:** specs cover HTTP-observable behaviour
  only; unit tests cover branch coverage without spec scenarios. The
  reframe of S1–S5 and removal of non-HTTP scenarios (S3.1, S3.2, S4.x,
  S5.x, I.5) from `action_pipeline.md` graduated to
  [ticket 11](issues/11-spec-restructure-http-observable-only.md) during
  ticket 03 planning.
- **Spec/test naming rule settled:** specs and test files are named after
  the HTTP endpoint, not the internal component. `action_pipeline.md`
  (component name) and `flow.md` / `flow_sequence.rs` (dissolved
  component-tier directory name) leak internal names into the spec/test
  surface. Ticket 11 restructures to endpoint-named files: `actions.md` +
  `tests/http/actions.rs` (extend existing, POST /action, S1–S6), `reset.md` +
  `tests/http/reset.rs` (new, POST /reset, S7), `story_log.md` +
  `tests/http/story_log.rs` (new, POST /history/delete, S8). Ticket 03's
  `flow.md` / `flow_sequence.rs` are an interim state; ticket 11 dissolves
  them.
- **HTTP E2E coverage gap:** S1.x / S2.x / S3.3 / S3.4-success are
  HTTP-observable scenarios currently covered only by unit tests in
  `src/application/pipeline/pipeline_tests.rs` (via `src/` SCENARIO tags).
  STRATEGY.md requires HTTP-observable scenarios to have HTTP E2E tests in
  `tests/http/`. Ticket 11 adds the missing HTTP E2E tests and removes
  the `src/` tags (unit tests stay for branch coverage, lose the spec
  link).
- **Tier placement of handler failure tests:** `tests/http/actions.rs`
  had 8 failure-path tests using `Storage::with_failure()` injection.
  That fakes the driven port → unit tier (per STRATEGY.md: "both driven
  ports faked = unit"), but wearing an HTTP costume. Ticket 11 deletes
  all 8 and adds 3 proper unit tests in
  `src/adapters/driving/http/action/handlers/actions_tests.rs` for the
  3 missing `dispatch_action` branches (`Err` / `ShuttingDown` /
  `ConcurrentGeneration`), using pipeline stubs not storage-failure
  injection. Handler branch coverage stays at unit tier where it
  belongs; `tests/http/` becomes uniformly spec-covered.

## Out of scope

- Implementing spec/tier fixes for components outside the component tier.
  The investigation maps the gaps; fixing them is a future effort.
- Rewriting the pipeline architecture beyond testability-driven extraction.
- Coverage-percentage targets or coverage tooling changes (invariant-based
  approach is preferred over %).
- Adopting mockall or any expectation-based mocking framework (decided
  against; hand-written fakes at ports only).
- Reorganising the tests/ directory tree away from the existing
  mirror-convention (judged working; churn not worth it).
- Pure-function extraction as a pipeline-wide pattern (closed on planning
  map, tickets 03/10 — the pipeline is mostly wiring, not decision
  clusters).
