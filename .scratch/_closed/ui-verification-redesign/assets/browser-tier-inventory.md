# Browser tier inventory — as of 2026-09-13

Facts gathered this session about what the browser tier actually is today.

## The 26 tests, by surface

- **games.rs** (3): games panel posture fragment render, tense-change
  autosave + rerender, mode switch retargets + nudges.
- **prompt_presets.rs** (1): preset editor mode-flags roundtrip.
- **options.rs** (3): Use-click submits option, options dock survives reload,
  Edit fills without submitting.
- **invariants.rs** (1 `#[test]` hosting 9 run_subtest checks): CSS computed
  styles, layout measurements, text-wrap — shares ONE server + ONE browser;
  the only fixture-sharing test.
- **worlds.rs** (2): world edit form renders posture selects, posture change
  autosaves status (the unresolved flake).
- **story_log.rs** (4): edit mode activates on click, edit cancel restores,
  polling pauses during edit, delete removes message.
- **dashboard.rs** (3): form stays static after submission, status updates
  during generation, error toast on action failure.
- **slash_menu.rs** (9): opens on `/`, filters by prefix, arrow keys, enter,
  escape, click, reopens after action-area rerender, impersonate → input
  entry, guide does not persist input entry.

## Fixture shape

`with_test_page` (tests/test_utils/browser.rs:72) per test:
- allocates a port (3010–3050 via file locking),
- spawns `TestServer::new_with_mock` (full engine, mock LLM),
- launches fresh Chromium (`launch_chrome`),
- `goto_with_connection_check`,
- waits for `#story-log .log-entry` ≥ 1 — the shared readiness gate
  (line 87) every test passes through,
- runs the test fn, closes the browser.

invariants.rs is the only exception (shared server+browser across subtests
via `run_subtest`).

## Waits available (tests/test_utils/wait.rs)

All polling-based with hardcoded timeouts; none are event-driven:

- `wait_for_llm_idle(port, timeout)` — polls a status source until LLM settles.
- `wait_for_element_children(page, selector, min_count)` — 10s cap, 200ms poll.
- `wait_until_visible / wait_until_hidden(page, selector, timeout)` — caller-set.
- `wait_for_status_ready` (12s), `wait_for_status_generating` (5s),
  `wait_for_status_ready_or_error` (15s).
- `wait_for_element_persist` (no-change stability), `wait_for_condition_async`.

Nothing waits on htmx lifecycle events, network idleness, or an explicit
"application ready" signal.

## Tier rules (tests/STRATEGY.md)

Browser tier placement rule: a test belongs in the browser tier **only if it
asserts something only a browser can see** (DOM, CSS, JS, rendering). SCENARIO
tags map tests/browser/<feature>.rs ↔ docs/specs/browser_<feature>.md, enforced
by `scripts/validate_feature_spec.py`; invariants.rs is a named tag exemption.

Overlap worth scrutinizing in ticket 02: the posture *was* the behaviour under
test in tests/browser/worlds.rs, and there is ALSO an http-tier contract test
(tests/http/worlds.rs — posture merge contract). The current design tests
server-observable autosave behaviour through a full Chromium round-trip where
an HTTP test could assert the same contract. This is likely the main
"tests assert through the wrong tier" smell.

## Runtime cost

- Browser binary serialized: ~127s at 8 cores (was ~154s at 2 cores pre-WSL
  bump). Two wait-shape barriers: the ~30s server-ready window and per-test
  server+Chromium spawn.
- The migration-transaction fix earlier this session cut cold engine boot
  ~5.9s → ~1.7s, the dominant per-test fixture cost.
- Browser tier ≈ 87% of total gate wall time for 1.7% of the tests.

## Spec/docs surfaces that read this tier

- `docs/specs/browser_*.md` ↔ `tests/browser/*.rs` (tag-enforced).
- `docs/diataxis/reference/frontend/dashboard.md`,
  `docs/diataxis/reference/frontend/http_routes.md` (extract_http_routes.py).
- `docs/diataxis/explanation/dashboard_design.md`,
  `docs/diataxis/explanation/two-state-channels.md` — the *design intent*
  these tests should be measuring against.

## nextest knobs currently in play (`.config/nextest.toml`)

- `test-threads = 4`, `retries = 1`, `slow-timeout 60s`.
- Browser whole-binary override: `threads-required = "num-test-threads"`
  (serialize browser tests; added for the old 2-core box).
- architecture binary gets `slow-timeout 180s`.
