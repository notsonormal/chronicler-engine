# Tier-2 rollout: move the remaining class-1 tests onto the stub-server tier

Type: task
Status: resolved
Blocked by: 06

## Question

Roll out the tier-2 fixture validated by ticket 06 to the rest of the class-1
set, per the ratified placement rule ("if the server behind this were fake,
does the behaviour change?" — no → tier 2). The audit
(`assets/harness-audit.md` §1) is the source of per-test facts.

Move, keeping every assertion identical:

- `slash_menu.rs` tests 2–7 (opens / filters / arrows / enter / escape /
  click) — pure client JS on the shell.
- `options.rs` test 9 (`options_dock_edit_fills_without_submitting`).
- `story_log.rs` tests 10–12 (edit activates, cancel restores, polling pauses
  during edit) — stories as canned entries; the poll hits the stub's canned
  response.
- `dashboard.rs` test 13 (error toast — synthetic `htmx:beforeSwap`).
- `invariants.rs` test 1 (9 sub-checks) — move onto the tier-2 fixture, one
  shared browser with fresh pages via `run_subtest`.

Judge individually — these two are not automatic:

- `slash_menu.rs` test 8 (`reopens_after_action_area_rerender`): port via a
  synthetic `#action-area` innerHTML transplant + re-arm check, **or** leave in
  tier 3 if the transplant reads as faking the thing under test. Whichever,
  say why in `## Answer`. Ticket 09 expects the answer.
- Any test where the canned fragment visibly diverges from the real template:
  stop, note the drift, and ask whether it belongs in tier 3 instead. This is
  the accepted drift tax (ticket 04 Answer) — pay it with a fixture update,
  not by stretching the assertion.

The tier-2 fixture itself (stub server + shared browser, fresh page per test)
is extended, not rebuilt, from ticket 06. Keep it serverless from the app's
perspective: no `TestServer`, no engine boot.

Ticket 06's spike ported 2 tests onto the fixture
(`tier2::test_slash_menu_opens_on_slash_tier2`,
`tier2::test_error_toast_on_action_failure_tier2`) as mirrors of their tier-3
twins. Reuse them; do not move them twice. The duplicates are deleted here,
not in ticket 06 — state the deletion in `## Answer`.

**Exit check (ratified with ticket 06's verdict):** both spike tests are
server-independent, so the slice never measured the drift tax for a test that
*reads* served content. The `story_log.rs` ports (tests 10–12: canned entries
in the story log, polling against the stub's canned response) are the first
fragment-reading ports. Characterise the drift there — does the test still
exercise behaviour, or has the canned fragment quietly become the assertion? —
before porting anything else. Record the finding in `## Answer`; if any port
reads as faking the thing under test, stop and say so rather than multiplying
it.

Record under `## Answer`: the per-test placement table (moved → which fixture
mode / or left in tier 3 + why), the wall time of the tier-2 binary, and the
drift items found.

## Answer

Resolved 2026-09-18. Implementation written by an implementer subagent, killed
mid-flight by a WSL restart before it could compile; verified, fixed and
finished in the orchestrator session. Landed on `ui-verif/ticket-07` as
`b05fbe5` (checkpoint) + the verification commit.

### Per-test placement table

Placement rule applied per test: "if the server behind this were fake, does the
behaviour change?" — no → tier 2.

| # | Test | From | Disposition | Fixture mode |
|---|---|---|---|---|
| 2 | `test_slash_menu_opens_on_slash` | `slash_menu.rs` | moved → `tier2.rs` | stub, canned fragments |
| 3 | `test_slash_menu_filters_by_prefix` | `slash_menu.rs` | moved → `tier2.rs` | stub |
| 4 | `test_slash_menu_arrow_keys_move_active` | `slash_menu.rs` | moved → `tier2.rs` | stub |
| 5 | `test_slash_menu_enter_populates_input` | `slash_menu.rs` | moved → `tier2.rs` | stub |
| 6 | `test_slash_menu_escape_closes` | `slash_menu.rs` | moved → `tier2.rs` | stub |
| 7 | `test_slash_menu_click_populates_input` | `slash_menu.rs` | moved → `tier2.rs` | stub |
| 8 | `test_slash_menu_reopens_after_action_area_rerender` | `slash_menu.rs` | **moved → `tier2.rs`** (see judgement below) | stub + synthetic DOM transplant |
| 9 | `test_options_dock_edit_fills_without_submitting` | `options.rs` | moved → `tier2.rs` | stub + new `options_dock.html` fixture |
| 10 | `test_edit_mode_activates_on_click` | `story_log.rs` | moved → `tier2.rs` | stub + `story_log.html` fixture |
| 11 | `test_edit_cancel_restores_original` | `story_log.rs` | moved → `tier2.rs` | stub + `story_log.html` fixture |
| 12 | `test_polling_pauses_during_edit` | `story_log.rs` | moved → `tier2.rs` | stub + `story_log.html` fixture |
| 13 | `test_error_toast_on_action_failure` | `dashboard.rs` | moved → `tier2.rs` | stub, synthetic `htmx:beforeSwap` |
| 1 | `test_invariants` (9 sub-checks) | `invariants.rs` | moved → tier-2 fixture, `run_subtest` shape preserved | stub + shared browser |

Left in tier 3 (not this ticket's scope): `dashboard.rs` 14/15,
`options.rs` 17, `slash_menu.rs` 19/20, `story_log.rs` 18, `games.rs` 21/22/25,
`worlds.rs` 24/26, `prompt_presets.rs` 23. Ticket 08 demotes the class-2
assertions among these; ticket 09 owns the surviving wiring guards.

### Duplicates deleted

Ticket 06 shipped two tier-2 tests as mirrors of tier-3 twins
(`test_slash_menu_opens_on_slash_tier2`, `test_error_toast_on_action_failure_tier2`).
Both are now the **only** copy: the tier-3 originals were deleted in this
ticket, and the `_tier2` suffix was dropped so each test has one canonical
name. `grep -rn "_tier2" tests/` returns nothing.

### Test 8 judgement — moved to tier 2

The re-render scenario is pure client behaviour and does not read served
content. The slash palette's `input` listener is a **document-level
delegation** (`assets/index.html:537`), and the menu is built under
`document.body` (`assets/index.html:490-509`). The port transplants
`#action-area`'s own markup back into itself, which destroys and recreates the
form nodes and fires `focusout` — a real DOM replacement, not a mock of the
behaviour. The assertion (typing `/` into the fresh input reopens the menu with
3 commands) is therefore identical in meaning to the tier-3 original.
`hx-trigger` re-arming is not what this scenario tests, so the settle gate is
not implicated.

### Exit check — the drift tax on the first fragment-reading ports

**Finding: the drift is real but bounded, and the tests still exercise
behaviour.** For tests 10–12 the canned `story_log.html` supplies the *input
fixture* (one entry with `.text`, `data-id`, `data-raw-text`, `.edit-btn`),
while the behaviour under test is client JS shipped in the real
`assets/index.html`:

- `showEditForm` (`assets/index.html:232`) swaps `.text`'s innerHTML for a
  `#edit-textarea` seeded from `data-raw-text`, and rebuilds `.message-actions`
  into save/cancel buttons.
- `cancelEdit` (`assets/index.html:268`) restores the captured
  `originalText` and resumes polling.
- `pausePolling` (`assets/index.html:209`) nulls `hx-trigger` on `#story-log`.

The assertions are about those functions' effects, not about the fixture's
content, so a *content* drift in the fixture cannot make a test pass falsely —
and the shell is `include_str!`-ed from the real `assets/index.html`, so JS
drift is impossible by construction.

**Where the tax does land:** the fixture must keep the *structural hooks* the
JS addresses — the `.log-entry` / `data-id` / `.text` / `data-raw-text` /
`.edit-btn` / `.message-actions` shape. If the real Askama template
(`src/adapters/driving/http/templates.rs:25`) changed those names, the tier-2
tests would fail loudly (element not found), not silently pass. That is the
acceptable direction of failure.

**One genuine gap found and recorded, not fixed here:** the fixture is a
hand-shaped snapshot, and the only remaining non-quarantined test that pins the
*real* story-log fragment's DOM is `invariants.rs:321` (`.edit-btn` presence).
The real-template assertions for `edit-btn` / `delete-btn` now live only in
`tests/http/requires_migration/fragment.rs:84-95`, which is the quarantine
(`REQUIRES_MIGRATION_TEST_COUNT`, untagged by design). The tier-2 fixture is
therefore the sole *gated* structural check on the edit path. This is drift tax
paid knowingly per ticket 04's ratchet: updating the fixture in the same change
as the template it mirrors is the cost, and the failure mode is loud. Recorded
for ticket 11's bookkeeping.

**No port read as faking the thing under test, so the rollout continued.**

### Verification actually run

On `ui-verif/ticket-07`, `--target-dir target/agent7`:

- `python build.py --target-dir target/agent7 check` — **OK**, 17.33s.
- `python build.py --target-dir target/agent7 clippy` — **OK**, 0 warnings.
- `python build.py --target-dir target/agent7 browser` —
  **`nextest: 26 passed, 0 failed`**, 490.79s (nextest applies the browser
  binary's `threads-required = "num-test-threads"` serialization override).
- `python scripts/validate_feature_spec.py` —
  **`141 declared, 141 covered, 0 gap(s), 0 orphan(s), 0 untagged, 0 surface
  mismatch(es), quarantine 86/86`**, exit 0.

### Wall times (measured directly through the binary)

`CARGO_TARGET_DIR=target/agent7` must be set when invoking the test binary
directly, or the harness falls back to `cargo run` and deadlocks on the package
cache lock — see the note below.

| Scope | Tests | Wall |
|---|---|---|
| tier-2 module (`tier2::`) | 12 | **4.70 s** |
| invariants (now tier 2) | 1 (9 sub-checks) | 2.56 s |
| full browser binary (direct) | 26 | **20.23 s** |
| tier-3 `worlds::` only | 2 | 7.03 s |

The full binary at 20.23 s direct versus 490 s under nextest is a
serialization-override artifact, not test cost — relevant to ticket 12's
step 4, which deletes `threads-required = "num-test-threads"` and measures the
difference. Tier-2's 12 tests in 4.70 s confirms ticket 06's per-test finding
(engine boot is a constant, ~2 s, that tier 2 avoids entirely).

Note for whoever runs these next: invoking the browser binary directly
**requires** `CARGO_TARGET_DIR` to match the build, because
`tests/test_utils/server.rs:212` resolves the engine binary from that env var.
Without it the harness shells out to `cargo run` per test and stalls on
"Blocking waiting for file lock on package cache" — which presents as a
30 s server-start timeout, not as an obvious misconfiguration.
