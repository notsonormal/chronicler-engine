# Test Design Audit — HTTP and Browser Tiers (2026-09-29)

READ-ONLY review. Scope: every test function in `tests/http/*.rs` (excluding
`requires_migration/`), `tests/browser/*.rs`, `tests/browser/stub/*.rs`, plus
the helpers they call (`tests/test_utils/{browser,wait,stub_server,htmx_settle}.rs`,
`tests/http/support/*`). Every file was read in full.

**Reviewed: 151 test functions** (128 HTTP, 7 full-stack browser, 16 stub
browser — `test_invariants` counted once with its 9 internal checks) plus 8
helper files. Findings: 14 (1 tautological, 3 mechanism, 3 setup-bypass,
2 timing, 1 same-tier duplication, 1 stub fidelity, 3 weak assertions).

---

## Class 1 — Tautological tests

### T1 (HIGH) — `tests/browser/dashboard.rs:11-46` `test_form_stays_static_after_submission`

```rust
let form_id_before: String = page
    .evaluate::<(), String>("document.querySelector('#command-form')?.id || ''", None)
...
let form_id_after: String = page
    .evaluate::<(), String>("document.querySelector('#command-form')?.id || ''", None)
...
assert_eq!(
    form_id_before, form_id_after,
    "Form should stay in DOM (static shell)"
);
```

**Why it is a problem.** The template always renders `id="command-form"`
(`src/adapters/driving/http/templates.rs:76`), so both reads return the same
constant string unless the form is removed from the DOM entirely. The
regression the test name implies — htmx re-swapping and re-registering the
form node — replaces the node with a *fresh* form carrying the *same* id, and
this assertion passes. Only total disappearance can fail it, which no
submission path produces.

**Fix.** Assert node *identity*, not the id string: stash the node before
submitting and assert the same node is still live after:

```js
// before
window.__formBefore = document.querySelector('#command-form');
// after
document.querySelector('#command-form') === window.__formBefore
```

That is the observable meaning of "stays static": the node was never replaced.

---

## Class 2 — Mechanism tests

### M1 (MED) — `tests/browser/stub/dashboard.rs:44-72` `swap_action_area_via_restore` (used by 16.9, 16.10)

```rust
saveActionArea();
...
area.innerHTML =
    '<div class="text-check-preview"><p>preview</p></div>';
restoreActionArea();
```

**Why it is a problem.** The helper calls the shell's internal JS functions
(`saveActionArea()` / `restoreActionArea()`) by name and hand-writes
`area.innerHTML` to fake the swap, where a fully observable path exists: the
shipped flow swaps `#action-area` through `POST /action/confirm` (outerHTML —
`src/adapters/driving/http/templates.rs:126,129`), which is *why*
`saveActionArea` exists. Driving the real confirm flow exercises the real
htmx swap, the real save/restore invocation, and the real detach/reattach
of the page-load handles — the current test only exercises the two functions
in isolation.

**Observable path and cost.** Stub serves: `/action/check` → a canned
text-check preview (the real `TextCheckPreviewTemplate` render, as
`options_dock_html()` already does for the dock), `/action/confirm` →
`FIXTURE_ACTION_AREA`. The test then clicks the preview's Send-original
button through `click_and_settle`. Cost: ~30 lines in `stub_server.rs` plus
two `with_stub_page` outcome parameters; test runtime unchanged. This also
fixes stub-fidelity finding S1 below.

### M2 (MED) — `tests/http/retrigger.rs:286-315` and `tests/http/swipe_new.rs:715-763` (concurrent-generation "Still thinking..." tests)

```rust
let (_, _, claim) = state
    .generation_gate
    .try_claim(game_id, &mut game_state, state.message_service.as_ref())
    .expect("pre-claim should succeed");
```

**Why it is a problem.** The setup claims the generation gate directly
in-process — an internal seam at the HTTP tier, where the scenario
("POST while generating → Still thinking...") has an observable path.

**Observable path and cost.** Hold a generation open with
`MockBackend::default().with_delay(2000)`: POST `/action` (or the seeded
anchor flow), then POST `/retrigger` / `/swipe/new` inside the delay window
and assert the "Still thinking..." body. Cost: a bounded timing window
(deterministic today, racy-by-construction with the delay approach), plus
`wait_idle` cleanup after. Trade-off: `try_claim` is deterministic and
race-free; the delay path is more shipped but time-dependent. Alternative
that keeps determinism: move the gate-claim setup to the unit tier (STRATEGY.md
already routes "mid-flight observation" there) and keep the HTTP test on the
response-shape assertion via the delay path. Either is better than an
in-process seam silently living at the HTTP tier.

### M3 (LOW) — `tests/browser/stub/options.rs:32-41` and `tests/browser/stub/story_log.rs:15-24, 85-94`

```rust
const btn = document.querySelector('#options-dock .option-item .mini-btn');
if (!btn) return false;
btn.click();
```

**Why it is a problem.** `evaluate`-based `btn.click()` fires only the
element's `onclick` handler; a Playwright `locator.click()` dispatches real
pointer events through the real hit-test path. Minor, and inconsistent within
the same file — `story_log.rs` test 30.2 already uses `page.locator(...).click(None)`.

**Fix.** Use `locator.click(None)` for all real-element interactions (the
mini-btn, the edit-btn); keep `evaluate` for assertions only. Cost: none —
the locators already exist.

---

## Class 3 — Setup that bypasses the shipped path

### B1 (HIGH) — `tests/browser/stub/dashboard.rs:12-34` `dispatch_error_toast` (used by 16.7 `:121-139`, 16.8 `:141-162`)

```rust
const evt = new CustomEvent('htmx:beforeSwap', {{
    bubbles: true,
    cancelable: true,
    detail: {{ isError: true, serverResponse: '<p>{message}</p>', ... }},
}});
document.body.dispatchEvent(evt);
```

**Why it is a problem.** The module header justifies the synthesis with "No
engine endpoint produces `htmx:beforeSwap` with `isError`" — but the stub
server itself defines `StubActionOutcome::Error` (a 500 from
`POST /action/check`, `tests/test_utils/stub_server.rs:218-224`) and **no test
anywhere uses it** (verified: every `with_stub_page`/`StubServer::start` call
passes `Pending`). A real form POST against the Error outcome makes htmx
itself fire `htmx:beforeSwap` with `isError` (the shipped shell listens for
exactly that at `assets/index.html:202-203`), so the whole event-synthesis
layer duplicates a real path the stub already implements.

**Fix.** Start the stub with `StubActionOutcome::Error` and drive
`send_action(&page, "anything")`; assert the toast shows the stub's 500 body
("Stub action failure" — parameterize the stub's error body if the
tag-stripping leg needs markup). For 16.8's two-error sequence, two
`send_action` calls replace the two synthetic dispatches; the inter-error
sleep stays (it is behavior timing, see P1). Cost: deletes ~20 lines of
synthesis code; the unused-variant problem disappears with it.

### B2 (MED) — `tests/browser/stub/dashboard.rs:97-115` `inject_status_html` (used by 16.10)

```rust
const display = document.getElementById('status-display');
if (!display) throw new Error('no live #status-display');
display.innerHTML = html;
```

**Why it is a problem.** Raw `innerHTML` writes stand in for the shipped
delivery path. The real `/status/generating` endpoint returns exactly the
injected markup on a failed generation —
`src/adapters/driving/http/layout/handlers/endpoints.rs:67-69`:

```rust
if let Some(err) = status.error_message() {
    Html(format!("<span class=\"status error\">Error: {err}</span>"))
}
```

and the shell's own poller (`hx-get="/status/generating" hx-trigger="load,
every 5s"` on `#status-display`) swaps that response in through htmx. The
stub hardcodes `"idle"` (`tests/test_utils/stub_server.rs:140`), so the test
writes by hand what the stub could serve verbatim — through the real poller
and the real swap, which would additionally exercise the swap wiring the
raw write skips.

**Fix.** Add a stub status outcome serving the error span (and the Ready
span for the dedupe-reset leg) from `/status/generating`. Cost: one more
`StubServer` parameter and a ~5-line route arm; the test then waits on the
poll interval (up to ~5s per injection) instead of injecting instantly —
slower, but it observes the shipped chain end to end.

### B3 (MED) — `tests/browser/stub/slash_menu.rs:212-218` `test_slash_menu_reopens_after_action_area_rerender`

```js
const area = document.getElementById('action-area');
const productionMarkup = area.innerHTML;
area.innerHTML = productionMarkup;
```

**Why it is a problem.** Self-assigning `innerHTML` re-parses the form nodes
but is not a swap: it skips htmx entirely (no settle lifecycle, no swap
events) and reads no served content. The in-file comment defends it as "a
real DOM replacement", which is true of the DOM mutation but not of the
*swap* the scenario describes ("replacing the action area" — 31.7).

**Fix.** Same observable path as M1: preview → confirm, with the confirm
response swapping `#action-area` through real htmx. The `/`-typing legs
afterward stay unchanged. Cost: shared with M1's stub work.

---

## Class 4 — Timing

### P1 (MED-LOW) — `tests/browser/stub/dashboard.rs:141-162` `test_newer_error_keeps_toast_visible`

```rust
dispatch_error_toast(&page, "First failure").await;
tokio::time::sleep(Duration::from_millis(2500)).await;
dispatch_error_toast(&page, "Second failure").await;
tokio::time::sleep(Duration::from_millis(3500)).await;
```

**Why it is a problem.** The sleeps are deliberately coupled to the shipped
5s toast-hide timer: check lands at ~6s, the second toast's own timer expires
at ~7.5s — a **1.5s margin** covering two `evaluate` round-trips. Under
parallel test load a delayed read can cross 7.5s and fail a healthy build.
The sleeps themselves are unavoidable (the behavior is time-based); the
margin is the design choice that can be widened.

**Fix.** Dispatch the second error *earlier* (e.g. at 1.5s → its timer at
6.5s is irrelevant; what matters is checking as soon as the first timer
(5s) has certainly fired while the second is safely far away — check at
5.6s with the second dispatched at 2.5s gives ~1.9s of margin; better:
first at 0s, second at 2.5s, check at 5.6s). Or read the toast state
repeatedly over a 200ms-poll window centered at 6s and require
*visible throughout* — which is the actual invariant ("still up at 6s").

### P2 (LOW) — `tests/browser/stub/dashboard.rs:229` `test_status_error_reaches_observer_after_action_area_swap`

```rust
tokio::time::sleep(Duration::from_millis(5500)).await;
```

**Why it is a problem.** 500ms past the 5s hide timer; the subsequent
visibility assertion races nothing except evaluate latency, but the margin
is thin in the same way as P1.

**Fix.** Poll for the toast's hidden state with a bounded deadline
(`wait_for_condition_async` already exists) instead of a fixed 5.5s sleep —
"hidden by now" is a pollable condition, unlike P1's "still visible at t".

All other waits in scope are poll-based (`wait.rs`, `htmx_settle.rs`,
`wait_for_condition_async`) with explicit bounded timeouts and
diagnostics-on-panic; no unbounded waits and no `networkidle` found.

---

## Class 5 — Same-tier duplication (STRATEGY.md 'Overlap rule')

### D1 (MED) — `tests/http/settings.rs:92-139` — 20.2 / 20.3 / 20.4

```rust
let req = post_form_request(
    "/settings",
    "narration_connection_id=openrouter-gpt-4o-mini&quantifier_connection_id=ollama-gemma-4-26B",
);
...
assert_eq!(body, "Settings saved!");
```

**Why it is a problem.** Three tests at the same tier POST different form
combinations and make the *identical* assertion — `body == "Settings
saved!"`. 20.3 (quantifier switch) and 20.4 (both) are observably the same
test: nothing read back distinguishes them, so 20.4's incremental value
(narrator id also changing) is asserted nowhere. The switch actually taking
effect is only proven elsewhere (20.5's stored read-back; 20.8's
`/debug/backend`). Cross-tier overlap is fine; same-tier assertions that
cannot tell the scenarios apart is the violation.

**Fix.** Keep all three (spec coverage is mandatory), but after each POST
read back the stored settings (`app_state.settings()`, as 20.5 does) and
assert both connection ids match the form — then each scenario's distinct
Given is observably distinct. Cost: one assertion block per test.

---

## Class 6 — Stub fidelity

### S1 (MED) — `tests/test_utils/stub_server.rs:212-217` — `StubActionOutcome::Idle` on `/action/check`

```rust
StubActionOutcome::Idle => (
    StatusCode::OK,
    [(header::CONTENT_TYPE, "text/html; charset=utf-8")],
    FIXTURE_ACTION_AREA,
)
```

**Why it is a problem.** The real `POST /action/check` never returns a full
action area — it returns the "Thinking..." span (with status-swap headers),
a text-check preview, or an error (`src/adapters/driving/http/action/handlers/actions.rs:30-127`).
The full `#action-area` document is served by `POST /action/confirm`, which
the preview form targets with `hx-swap="outerHTML"` (`templates.rs:126`).
Serving `FIXTURE_ACTION_AREA` from `/action/check` therefore diverges from
the route's real response set — and since the command form targets
`#action-area` with `hx-swap="innerHTML"`, using the variant as-is would
nest a second `#action-area` inside itself. The variant is also dead code:
never referenced by any test.

**Fix.** Move the canned action area to a stub `/action/confirm` route
(outerHTML — faithful to the shipped flow), serve a canned
`TextCheckPreviewTemplate` render from `/action/check` when the confirm flow
is requested, and drop the misleading `Idle`-on-`/action/check` arm. This is
the same stub work that fixes M1 and B3. The `Pending` arm's fidelity was
verified as correct: same span, same `add_status_swap_headers` builder as
the real handler. The story-log fixture's structural hooks (`showEditForm`,
swipe-controls, `data-id`/`data-raw-text`) match the real
`NarrativeLogTemplate` (`templates.rs:25`).

---

## Class 7 — Weak assertions

### W1 (LOW-MED) — `tests/http/actions.rs:456-496` `test_empty_narrator_response_sets_error_no_narration_http`

```rust
let has_new_narration_for_this_action = messages
    .iter()
    .filter(|m| m.message_type == MessageType::Narration)
    .any(|m| !m.text().is_empty() && m.text() != "Welcome to the Test World, Test Player! You find yourself in a cozy room with wooden beams and a warm fire. ...");
```

**Why it is a problem.** "No new narration" is asserted by comparing every
narration against an inline verbatim copy of the fixture's opening-narration
string. If the fixture text ever changes, the opening narration itself is
misclassified as "new" and the test fails for a setup reason; conversely a
regression that persists any other canned text fails only by luck of the
string. The test's own comment ("count narrations before and after to be
sure") names the right technique and then doesn't use it.

**Fix.** Snapshot the narration count after setup (before the action), then
assert the count is unchanged after `wait_idle` — precise, non-brittle, and
it catches a persisted empty-response narration whatever text it carries.

### W2 (LOW) — `tests/http/actions.rs:392-424` `test_nonexistent_room_sets_error_status_http`

```rust
let new_narrations: Vec<_> = messages
    .iter()
    .filter(|m| m.message_type == MessageType::Narration)
    .collect();
// The fresh-game opening narration exists, but no new narration from this action.
// We assert none of the narrations mention "look" output (the action failed).
let _ = new_narrations; // not asserting count here — the opening narration may exist
```

**Why it is a problem.** Dead binding plus a comment narrating a
non-assertion — the collected value is immediately discarded. Either the
count assertion is needed (see W1's before/after technique) or the whole
block is noise that misleads a reader into thinking something is checked.

**Fix.** Delete the binding and both comments (the Error-status match
above is the scenario's assertion), or convert it to the W1-style
before/after count assertion.

### W3 (LOW) — `tests/http/worlds.rs:146-171` `test_world_posture_invalid_value_returns_error_span_http`

```rust
let body = response_body(resp).await;
assert!(
    body.contains("error"),
    "an invalid posture value must render an error span, got: {body:?}"
);
```

**Why it is a problem.** `contains("error")` matches the substring anywhere —
a class name, a route fragment, an unrelated message. The test name promises
an *error span*; the assertion can't tell a span from a stray word.

**Fix.** Assert the concrete shape, e.g. `body.contains(r#"class="error""#)`
or the error span's message text, mirroring the sibling test's
`assert!(body.contains("Saved") && body.contains("posture-status"))`.

### W4 (LOW, setup note) — `tests/http/actions.rs:397-400` same test as W2

```rust
gs.movement.current_room_id = "non_existent_room".to_string();
```

**Why it is a (mild) problem.** The broken state (room pointing at a
nonexistent id) is written directly through `message_service`, bypassing
every shipped path. A real path exists: the quantifier mock returning a
movement to a nonexistent destination
(`{"movement": {"type": "Entering", "destination": "non_existent_room"}}`)
drives the shipped movement code into the same state. Failure-injection
through direct writes is an accepted pattern in this repo (Pattern 5), so
this is a judgment call — but the quantifier route exercises the actual
movement validation the scenario is about.

**Fix.** Prefer the quantifier-response setup (one extra mock response); if
the direct write stays, the W2 cleanup still applies.

---

## Checked and clean

### tests/http/ (128 tests)
- `actions.rs` — `test_action_succeeds_one_narration_idle_http`,
  `test_input_persisted_before_narration_http`,
  `test_quantifier_npc_fires_trigger_http`,
  `test_no_trigger_npc_produces_narration_no_event_http`,
  `test_trigger_does_not_refire_on_second_encounter_http`,
  `test_empty_command_continuation_no_input_http`,
  `test_trigger_continuation_reruns_quantifier_detects_new_npc_http`,
  `test_failing_narrator_sets_error_status_http`,
  `test_trigger_narration_failure_preserves_main_sets_error_http`,
  `test_delayed_llm_completes_without_deadlock_http`,
  `test_three_actions_in_sequence_http`,
  `test_sequential_execute_retry_execute_http`,
  `test_async_action_sequence_then_retry_http`,
  `test_slash_impersonate_produces_input_http`,
  `test_slash_guide_does_not_persist_input_http`,
  `test_slash_command_bypasses_text_check_http`
  (findings: W1, W2, W4 on 2.1/2.3 tests)
- `games_config.rs` — all 4 (20.4-20.7): storage-failure 500s assert the
  named error; 20.7 asserts re-rendered fragment + persisted posture
- `games_create.rs` — all 3 (17.1-17.3)
- `games_delete.rs` — all 3 (19.1-19.3)
- `games_fragment.rs` — 20.8: selected-option proof that stored posture
  reached the render; per-game auto-save routes pinned
- `games_switch.rs` — both (18.1, 18.2)
- `narrator_mode.rs` — all 5 (23.1-23.5): prompt forensics read back through
  storage-backed recorder — strong
- `options.rs` — all 10 (24.1-24.13): 24.13 honestly scopes what the HTTP
  tier can observe about the Use-click hop
- `prompt_presets.rs` — all 28 (21.1-21.27): card-scoped assertions
  (21.23/21.27) avoid panel-wide false matches; ids read from storage, not
  scraped HTML
- `reset.rs` — both (7.1, 7.2)
- `retrigger.rs` — 8 of 9 clean (13.1, 13.2, 14.1-14.6); finding M2 on 15.1
- `settings.rs` — 5 of 8 clean (20.1, 20.5, 20.6, 20.7, 20.8);
  finding D1 on 20.2/20.3/20.4
- `story_log.rs` — all 5 (8.1-8.5): before/after counts, no-second-container
  invariant — exemplary
- `swipe_new.rs` — 20 of 21 clean (9.1-9.6, 10.1, 10.2, 11.1-11.8, 22.1-22.4);
  finding M2 on 12.1
- `worlds.rs` — 8 of 9 clean (25.1-25.4, 25.6, three of the 25.5 family);
  finding W3 on `test_world_posture_invalid_value_returns_error_span_http`

### tests/browser/ (7 tests)
- `dashboard.rs` — `test_status_updates_during_generation` clean;
  finding T1 on `test_form_stays_static_after_submission`
- `games.rs` — `test_games_posture_change_reaches_server` clean (change →
  settle → reload → re-render proof)
- `options.rs` — both clean: reads the *rendered* button text rather than a
  seed string — explicitly anti-tautological
- `prompt_presets.rs` — clean: seed via the engine's own HTTP API,
  pre-conditions asserted, reload before the post-save assertion
- `worlds.rs` — clean

### tests/browser/stub/ (16 tests)
- `dashboard.rs` — findings B1/B2/M1/P1/P2; the *assertions* themselves are
  strong (fresh-node marker, dedupe-reset sequence, label checks)
- `invariants.rs` — `test_invariants` + all 9 checks clean: computed styles,
  layout measurements, declared exemption, per-check panic isolation;
  `check_log_entry_text_wraps_within_bubble`'s synthetic node is the point
  of a CSS invariant test, not a bypass
- `options.rs` — clean apart from M3 (evaluate click); assertions on
  rendered text, entry-count invariance
- `slash_menu.rs` — 6 of 7 clean (31.1-31.6); finding B3 on 31.7
- `story_log.rs` — 2 of 3 clean (30.2, 30.3); M3 on 30.1/30.3's evaluate click

### Helpers (8 files)
- `tests/test_utils/browser.rs` — clean; `send_action` acknowledges the
  stale-Ready race and closes it; `capture_failure_state` diagnostics
- `tests/test_utils/wait.rs` — clean; all waits poll with bounded timeouts
  and panic with diagnostics (per Cross-cutting 5)
- `tests/test_utils/htmx_settle.rs` — clean; the settle-gate design directly
  addresses the lost-listener race
- `tests/test_utils/stub_server.rs` — findings S1 (Idle arm) + B1/B2's
  unused `Error` variant and hardcoded idle status; otherwise faithful
  (Pending arm consumes the engine's own `add_status_swap_headers` builder;
  options dock rendered through the engine's own template)
- `tests/http/support/*` — all 4 clean (`app_wiring`, `http_assertions`,
  `http_fixtures`, `http_requests`); `assert_option_selected` is the right
  primitive (selection is the only server-observable posture proof)

---

## Top findings summary

| # | Sev | Class | Location | One-line |
|---|-----|-------|----------|----------|
| 1 | HIGH | 1 (tautological) | `tests/browser/dashboard.rs:17-34` | id-string compare can't catch form re-registration — assert node identity |
| 2 | HIGH | 3 (bypass) | `tests/browser/stub/dashboard.rs:12-34` | synthetic beforeSwap event while `StubActionOutcome::Error` (unused) would produce a real 500 |
| 3 | MED | 2 (mechanism) | `tests/browser/stub/dashboard.rs:44-72` | `saveActionArea`/`restoreActionArea` called directly; real confirm-swap path exists via the stub |
| 4 | MED | 6 (fidelity) | `tests/test_utils/stub_server.rs:212-217` | Idle arm serves a full action area from `/action/check`, which the real route never returns (and it would nest) |
| 5 | MED | 5+7 (dup/weak) | `tests/http/settings.rs:92-139` | three POST /settings tests with identical, effect-blind assertions |
