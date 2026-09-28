# Retro Follow-ups: Test-Harness Guardrails and Failure Visibility

## Summary

Four environment improvements from the session retrospective (v2 after plan review — placements corrected, glue work and failure modes added). Two mechanical guardrails (template raw strings, Form-field deserialization), one navigation fix (harness gotchas as co-located docs), one information-access fix (engine stdout surfaced on browser-test failure plus TraceLayer request logging — approved).

Not in scope: HTTP-tier test failure dumps (capture_failure_state is browser-tier only); LLM provider traffic capture; TraceLayer tuning (latency thresholds, custom spans); an inline-suppression mechanism for the guardrail suite; cleanup tooling for tmp/ accumulation.

Metrics: candidates 3/2/4 unchanged (viability 5, cost 1–3 SP). Candidate 1 viability 4, usefulness 5, cost 5 SP.

## Key Changes

- **Candidate 3** — `tests/infrastructure/guardrails/style.rs`: new `check_template_raw_strings` (`pub fn` with `///` doc comment, feeding the generated syn table). Flags the 3-byte sequence `r#"` in files under `templates/` directories or named `templates.rs`, skipping `*_tests.rs`. Message states the fix and why: `hx-target="#id"` terminates `r#"..."#` with `error: prefix ... is unknown`. Wrapper test in `mod.rs`. Converts 3 offenders: `games/templates/games.rs:23`, `prompt_presets/templates/prompt_presets.rs:10`, `worlds/templates/worlds.rs:165`.
- **Candidate 2** — `tests/infrastructure/guardrails/layers.rs`: new `check_form_fields_urlencoded_safe` beside `check_handler_return_type`. Flags `pub <field>: Vec<|HashMap<|BTreeMap<|HashSet<` lines under `src/adapters/driving/http/**/handlers/`, skipping `*_tests.rs`. Message: serde_urlencoded cannot decode collections from urlencoded bodies (single checked checkbox → scalar → extractor 422); use per-value boolean fields with `#[serde(default)]`. Wrapper in `mod.rs`. Plus one rule under **Implementation** in `CODING_STANDARDS.md`.
- **Candidate 4** — append `//!` caveats after line-1 summaries (generator reads line 1/2 only — zero index drift). `wait.rs`: strict-mode visibility, `attempts × 50ms` budget, empty elements never visible, count-based alternative. `browser.rs`: locator re-resolution after htmx swaps, unwaitable status spans, failure-dump locations. One pointer line in the hand-written head of `tests/AGENTS.md`.
- **Candidate 1 (harness)** — `server.rs`: retain buffer Arcs on `TestServer`, register in `LOG_REGISTRY: OnceLock<Mutex<HashMap<u16, ServerLogBuffers>>>` (mirrors `PORT_PIDS`, `server.rs:17`), deregister in `Drop`; drain threads also tee — best-effort — to `tmp/test_server_logs/<port>_{stdout,stderr}.log`. Extract a shared buffer-snapshot helper from the existing startup-failure dump (server.rs:425+) instead of duplicating it. `browser.rs`: `capture_failure_state` writes per-port dump files to `tmp/test_diagnostics/` + ~30-line tails; lock poisoning recovered via `into_inner()`. No signature changes — zero call-site churn.
- **Candidate 1 (request logging)** — `Cargo.toml` tower-http features `["fs", "trace"]`; `router.rs:210` `.layer(tower_http::trace::TraceLayer::new_for_http())` before `.with_state`. No WebSocket routes exist; blast radius is plain HTTP plus ServeDir static logging noise at debug.

Glue work: `python scripts/generate_guardrails_doc.py` after the new `check_*` fns land (freshness check gates the build).

Complexity note: 11 files touched; 8 are 1–3-line edits. Substantive code only in `server.rs`, `browser.rs` (test_utils), `style.rs`, `layers.rs`.

## Implementation

### Phase 1: Mechanical guardrails and docs

- [ ] #### Task 1.1: `r##"` raw-string guardrail for template sources (1 SP)
  - [ ] ##### SubTask 1.1.1: Convert the 3 offenders to `r##"` delimiters (1 SP)
  - [ ] ##### SubTask 1.1.2: Add `check_template_raw_strings` to style.rs + wrapper in mod.rs (1 SP)
- [ ] #### Task 1.2: Form-field deserialization guardrail + reviewer rule + doc regen (3 SP)
  - [ ] ##### SubTask 1.2.1: Add `check_form_fields_urlencoded_safe` to layers.rs + wrapper in mod.rs (1 SP)
  - [ ] ##### SubTask 1.2.2: Rule line under Implementation in CODING_STANDARDS.md; run `generate_guardrails_doc.py` (1 SP)
- [ ] #### Task 1.3: Browser harness gotchas as co-located docs (1 SP)
  - [ ] ##### SubTask 1.3.1: Caveats in wait.rs/browser.rs module docs + pointer line in tests/AGENTS.md head (1 SP)

### Phase 2: Engine stdout visibility

- [ ] #### Task 2.1: Server stdout registry, tee, failure dump (3 SP)
  - [ ] ##### SubTask 2.1.1: Arcs on TestServer; LOG_REGISTRY + Drop deregistration; best-effort tee to tmp/test_server_logs/; shared snapshot helper (1 SP)
  - [ ] ##### SubTask 2.1.2: capture_failure_state dump files + ~30-line tails, poison recovery (1 SP)
  - [ ] ##### SubTask 2.1.3: Drain-thread lifecycle check (writes never panic; buffers valid after kill) (1 SP)
- [ ] #### Task 2.2: TraceLayer + end-to-end verification (3 SP)
  - [ ] ##### SubTask 2.2.1: tower-http `trace` feature + router `.layer()` (1 SP)
  - [ ] ##### SubTask 2.2.2: Permanent tee assertion test (no SCENARIO tag; asserts `tmp/test_server_logs/<port>_stdout.log` exists and is non-empty from inside a `with_test_page` closure) + forced-failure probe + full browser suite (3 SP)

## Test Plan

- Guardrails: `python build.py guardrails` green; negative probes (plant `r#"` and a `Vec<` Form field, confirm each wrapper fails with its message, revert). Doc freshness: `python scripts/generate_guardrails_doc.py` diff committed so build.py's check passes.
- Candidate 4: `python scripts/generate_tests_structure_index.py` shows no drift.
- Phase 2: permanent tee assertion test passes; one-off probe (bogus selector, confirm dump files + tail + TraceLayer `finished processing request` line with status, revert); `cargo test --test browser` (24 tests) green.
- Convention check: `style.rs`/`layers.rs` have no `*_tests.rs` files today — no new unit-test files added, matching existing granularity.
- Final: `python build.py` full gate.

## Per Task/Sub Task Validation Steps

- 1.1/1.2: `python build.py guardrails` + negative probe + regen diff present.
- 1.3: index regen no-drift; docs steps green.
- 2.1: `python build.py unit`; one browser test run produces tee files.
- 2.2: probe + full suite + `python build.py` full gate.

## Assumptions

- `tmp/` is gitignored scratch (verify during 2.1).
- The Form-check regex has a false-positive failure mode (a future legit non-Form collection field under handlers/): no inline suppression exists in the guardrail suite; the remedy is moving the type out of the handler module, and the violation message says so. Accepted for v1.
- Untagged tests in `tests/browser/behaviour.rs` are legal per the validator's declared-scenario ↔ tag coupling (only invariants.rs is a named exemption; untagged tests are simply not coverage). Inferred from parser semantics — validated in the final gate.
- TraceLayer default DEBUG verbosity acceptable; prod filter stays `info` (`src/bootstrap/logging.rs:41`).
- worlds.rs:165 placeholder string contains no `"##` sequence.
