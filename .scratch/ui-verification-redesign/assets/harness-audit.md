# Browser-test harness audit — 26 tests, 8 files, and the harness itself

Audit asset for "Audit the current browser-test design: what does each test
actually verify, and through which layer?". Read-only; no code changes.

Sources: [browser-tier-inventory.md](./browser-tier-inventory.md),
[flake-investigation-2026-09-13.md](./flake-investigation-2026-09-13.md),
`tests/STRATEGY.md`, the eight files under `tests/browser/`, the harness under
`tests/test_utils/`, the resolved prior-art tickets in
`.scratch/test-strategy-execution/` (07, 08, 14, 15), a route inventory of
`src/adapters/driving/http/builders/router.rs`, and a readiness-hook inventory
of `src/adapters/driving/http/` plus `assets/index.html`.

## 1. The 26 tests, classified

Classification key (from the ticket):

1. **Browser-only by nature** — asserts rendering/layout/client-JS behaviour.
   Correct tier.
2. **Browser-observable behaviour that is really the app's wiring** — the
   application contract is the HTTP exchange; the browser is one client of it.
3. **Already covered at the HTTP tier** — an equivalent request-level contract
   test exists.

| # | Test | File:line | What it actually asserts | Class |
|---|---|---|---|---|
| 1 | `test_invariants` (9 subtests) | `invariants.rs:63` | computed styles, `getBoundingClientRect`, design tokens, `@media` at 500px | 1 |
| 2 | `test_slash_menu_opens_on_slash` | `slash_menu.rs:19` | `/` opens menu, 3 commands in order | 1 |
| 3 | `test_slash_menu_filters_by_prefix` | `slash_menu.rs:51` | `/g` filters to `/guide` | 1 |
| 4 | `test_slash_menu_arrow_keys_move_active` | `slash_menu.rs:84` | ArrowDown/Up move `.active` | 1 |
| 5 | `test_slash_menu_enter_populates_input` | `slash_menu.rs:134` | Enter fills input, menu closes | 1 |
| 6 | `test_slash_menu_escape_closes` | `slash_menu.rs:161` | Escape closes, value unchanged | 1 |
| 7 | `test_slash_menu_click_populates_input` | `slash_menu.rs:184` | click fills input, menu closes | 1 |
| 8 | `test_slash_menu_reopens_after_action_area_rerender` | `slash_menu.rs:219` | re-render re-arms the menu | 1 |
| 9 | `test_options_dock_edit_fills_without_submitting` | `options.rs:140` | fill + focus, entry count unchanged | 1 |
| 10 | `test_edit_mode_activates_on_click` | `story_log.rs:9` | clicking `.edit-btn` reveals `#edit-textarea` | 1 |
| 11 | `test_edit_cancel_restores_original` | `story_log.rs:39` | cancel hides textarea, restores text | 1 |
| 12 | `test_polling_pauses_during_edit` | `story_log.rs:84` | textarea survives 3 s of polling | 1 |
| 13 | `test_error_toast_on_action_failure` | `dashboard.rs:87` | synthetic `htmx:beforeSwap` → toast text | 1 |
| 14 | `test_form_stays_static_after_submission` | `dashboard.rs:11` | `#command-form` id survives the swap | 2 |
| 15 | `test_status_updates_during_generation` | `dashboard.rs:52` | polling JS moves `#status-display` off "Ready" | 2 |
| 16 | `test_options_use_click_submits_option` | `options.rs:7` | Use click → 1 Input entry + narration | 2 |
| 17 | `test_options_dock_survives_reload` | `options.rs:99` | same option set after `page.reload` | 2 |
| 18 | `test_delete_removes_message` | `story_log.rs:116` | delete click → entry leaves the DOM | 2 |
| 19 | `test_slash_impersonate_produces_input_entry` | `slash_menu.rs:261` | Impersonate does not persist as plain Input | 2 |
| 20 | `test_slash_guide_does_not_persist_input_entry` | `slash_menu.rs:301` | `/guide` adds no Input entry, adds narration | 2 |
| 21 | `test_games_panel_renders_posture_fragment` | `games.rs:11` | fragment renders selects with current values | 2 |
| 22 | `test_games_tense_change_autosaves_and_rerenders` | `games.rs:68` | change → fragment re-renders with new value | 2 |
| 23 | `test_preset_editor_mode_flags_roundtrip` | `prompt_presets.rs:11` | duplicate → edit → check → save → new button | 2 |
| 24 | `test_world_edit_form_renders_posture_selects` | `worlds.rs:42` | edit form renders 3 selects | 2 |
| 25 | `test_games_mode_switch_retargets_and_nudges` | `games.rs:106` | mode change nudges perspective + system preset | 3 |
| 26 | `test_world_posture_change_autosaves_status` | `worlds.rs:66` | change → `#world-posture-status` contains "Saved" | 2 |

Totals: **13 class 1**, **12 class 2**, **1 class 3** (26 tests).

### The single class-3 test, and why only one

Test 25 duplicates `tests/http/narrator_mode.rs:144`
(`test_mode_switch_renders_if_bundle_and_nudges_perspective_http`,
SCENARIO 23.3), which posts `/games/:id/mode` and asserts the response body
contains `value="second" selected` and `value="system_if_default" selected`.
The browser test asserts the same two facts — computed on the same server
render — by reading the live select values. The browser hop adds nothing.

### The class-2 tests: the real smell

Class 2 is the deep-thin-brittle-layer smell. Each test's *assertions* are on
server-derived content (posture values, option texts, preset flags, entry
counts). Only the *mechanism* is browser-side. Each has HTTP siblings asserting
adjacent facts one hop earlier — but, importantly, **not** the whole contract.
The gaps matter for the design decision:

| Browser test | HTTP sibling coverage | What is genuinely browser-only |
|---|---|---|
| 16 options Use click | `options.rs:87` (24.1) renders the dock | the click → Input-entry hop |
| 17 dock survives reload | `options.rs:87` (24.1) dock render | the reload hop |
| 18 delete removes message | `story_log.rs:12` (8.1) delete contract | click → DOM removal |
| 19 `impersonate` Input entry | `actions.rs:730` region (1.10) | none material |
| 20 `/guide` no Input entry | `actions.rs:730` (SCENARIO 1.10), same assertions | the click hop |
| 21 games fragment render | **none** — no HTTP test reads `#game-posture-controls` | the whole fragment render |
| 22 tense change autosave | `games_config.rs:21` (20.4, failure path); `narrator_mode.rs:184` (23.4) posts posture | the change hop + success-path re-render |
| 23 preset mode-flags roundtrip | `prompt_presets.rs:389` (21.14) update; `:167` (21.4) edit form; `:745` activation gating | the duplicate → check → save click chain |
| 24 world edit form renders | `requires_migration/worlds_fragment_handlers.rs:240` covers only *not-found* | the form render |
| 26 world posture autosave | **none** — `tests/http/worlds.rs` covers `POST /worlds/:key` (full form), not `POST /worlds/:key/posture` | the whole autosave contract |

**Two findings the ticket did not anticipate:**

- **`POST /worlds/:key/posture` has no HTTP test at all.** Tests 24 and 26 are
  the only coverage of the endpoint registered at `router.rs:124`. The handler
  (`worlds/handlers/worlds.rs:240`) returns the `Saved` span at line 286.
  Ticket 26's implied classification ("already covered at HTTP") is therefore
  wrong — the endpoint is browser-only today, which is *why* the flake is
  load-bearing.
- **`#game-posture-controls` has no HTTP test.** Test 21 is the only assertion
  that the games posture fragment renders populated selects.

That inverts part of the ticket's premise. The tier is not simply re-testing the
HTTP tier; it is the *sole* coverage for two HTTP contracts, and it reaches them
through a Chromium round-trip. Demoting those two contracts to the HTTP tier
*adds* coverage while removing the race.

## 2. The harness, piece by piece

| Piece | Flake it prevents | Flake it causes | Survives a simpler design? |
|---|---|---|---|
| `with_test_page` per-test `TestServer` (`browser.rs:72`) | cross-test state bleed — fresh port + game per test | ~5.8 s/test; port contention in 3010–3050 | **Yes, the server half.** Not the fresh-`Browser` half: `invariants.rs` runs 9 checks on one browser via `run_subtest` (`invariants.rs:21`). |
| Shared story-log gate (`browser.rs:87`) | acting before the first story-log render | **The single point of failure.** One CDN hiccup became 6 different "flaky tests" (flake-investigation §Fixed) | **Only as an explicit readiness contract.** Today it is a bare count-poll whose result is discarded (`let _ =`), duplicated at `invariants.rs:35`. |
| `wait_until_visible` / `wait_until_hidden` (`wait.rs:71`, `:84`) | acting before a fragment renders | **The ticket-03 race.** Visibility precedes htmx listener attachment; `worlds.rs:31` waits on `select[name="narrator_mode"]`, then test 26 immediately `select_option`s | **No, not as a readiness primitive.** Visibility is a rendering fact, not an htmx-readiness fact. |
| `wait_for_element_children` (`wait.rs:41`) | acting before N entries exist | 10 s hard cap, 200 ms poll; panics *inside* the helper via `capture_failure_state`, so the reported name is the helper's, not the test's | Yes — but only if made event-driven. |
| `wait_for_status_ready` (`wait.rs:113`, 12 s) | reading status mid-generation | polls text, not state; can pass on a stale "Ready" | Yes, with the ack guard below. |
| `wait_for_status_generating` (`wait.rs:145`, 5 s) | the stale-"Ready" race | burns budget per action | **Yes — load-bearing.** `send_action` (`browser.rs:95`) calls it at line 120, before any `wait_for_status_ready`; the doc comment names the real race. |
| `wait_for_status_ready_or_error` (`wait.rs:168`, 15 s) | — | — | Yes; only `tests/llm/flow_llm_tests.rs` uses it (3 uses). |
| `wait_for_element_persist` (`wait.rs:187`) | polling destroying edit state | 1 use (`story_log.rs:104`); encodes "wait 3 s and hope" rather than asserting the poll is paused | Yes. |
| `wait_for_condition_async` (`wait.rs:214`) | reflow / async settle | none | Yes (3 uses, incl. the responsive invariant). |
| `wait_for_llm_idle` (`wait.rs:9`) | reading state mid-generation | on timeout it POSTs `/status/reset-generating` (`wait.rs:29`) — a *test* mutating app state | Yes; 4 uses. |
| `wait_for_condition_sync` (`wait.rs:239`) | — | — | **No uses. Dead.** |
| `extract_port_from_url` (`wait.rs:105`) | — | — | **No uses. Dead private fn.** |
| `goto_with_connection_check` (`browser.rs:14`) | navigating to a not-yet-listening server | re-probes `wait_for_server(port, 100)` after `TestServer` already probed with 300 | Yes, folded into one server-ready check. |
| `capture_failure_state` (`browser.rs:154`) | silent failures | writes to `tmp/`, overlapping the engine-log tee | **Yes — earns keep.** Failure-only, 9 uses; it produced the CDN evidence. |
| `send_action` + `dismiss_text_check_if_present` (`browser.rs:95`, `:128`) | the ack race; the text-check dialog replacing `#status-display` | `dismiss_*` polls `.btn-original` after every action | **Yes — load-bearing.** Both encode a real, non-obvious hazard. |
| nextest `retries = 1` + browser `threads-required = "num-test-threads"` | surfaces nothing; it *hides* flakes (build.py's flaky epilogue was added to compensate) | serializes the whole binary for a 2-core box that is now 8-core | **No.** A stabilization knob on a stale box size. |

## 3. Readiness-hook inventory: what the app exposes

Request-level (from `src/adapters/driving/http/`):

| Hook | Contract | Observed by tests? |
|---|---|---|
| `GET /status/generating` (`endpoints.rs:49`) | `Error: {msg}` span, or bare `narrating`/`quantifying`/`generating-event`/`options`, or `idle` | Yes — `wait_for_llm_idle`, `requires_migration/fragment.rs:186`, and `#status-display` polling |
| `GET /status/ready` (`endpoints.rs:45`) | static `Ready` span, reads no state | Only `fragment.rs:167`. **The UI never requests it.** |
| `POST /status/reset-generating` (`endpoints.rs:82`) | `reset`/`failed` | Yes — `wait.rs:29` escape hatch, `fragment.rs:217` |
| `GET /debug/state` (`debug.rs:15`) | full `DebugStateView` JSON incl. `generation_phase` | Only `requires_migration/debug.rs`. **No browser test.** |
| `GET /debug/is_generating` (`debug.rs:25`) | `"true"`/`"false"` | Only `requires_migration/debug.rs`. **No browser test.** |
| `GET /debug/backend` (`debug.rs:33`) | `{backend_name, model_name}` JSON | Only `requires_migration/debug.rs`. |
| `HX-Retarget: #status-display` + `HX-Reswap: innerHTML` (`headers.rs:21`, `:24`) | dispatch responses target the status span | Yes — `actions.rs:787` |
| `HX-Refresh: true` (`response.rs:24`) | empty 200, client reloads | Yes — `requires_migration/fragment.rs:341` |

Client-side (from `assets/index.html`, served via `include_str!`):

- Status poll: `hx-get="/status/generating" hx-trigger="load, every 5s"` +
  `hx-on::after-swap="onStatusPoll(this)"` (`index.html:76-82`).
- Five concurrent pollers: `every 2s` (`#story-log` `:36`, `#options-dock`
  `:52`), `every 4s` (llm-messages `:119`), `every 5s` (visual sidebar `:42`,
  status `:79`). **This is why `networkidle` is unsatisfiable.**
- `htmx:beforeSwap` body listener → error toast (`index.html:196-203`).
- Programmatic re-fetch: `htmx.trigger(el, "htmx:refresh")` (`index.html:278,
  292,303,312,327,328,666`) and `htmx.ajax` (`:341` story-log, `:632`
  action-area).
- `MutationObserver` on `#status-display` (`index.html:608-627`).
- `hx-sync="this:drop"` on the command form (`index.html:61`) — drops duplicate
  in-flight `/action/check` submissions.

Absent — the raw material ticket 04 will find missing:

- **No `HX-Trigger` header anywhere** in `src/`. No server-emitted htmx event
  exists to subscribe to. (A test asserts its absence:
  `requires_migration/fragment.rs:374`.)
- **No `htmx:afterSettle` / `htmx:afterSwap` / `htmx:load` listener in app
  code.** Ticket 02's sourced readiness event is unused.
- **No `hx-preserve`.** The command form survives swaps structurally (it sits
  outside the swap target, `index.html:56`), not by preservation.
- **No WebSocket code in `src/`.** `AGENTS.md`'s "HTTP/WebSocket server" line
  is stale. `two-state-channels.md` documents persisted vs process-local
  generation state, not sockets. The dashboard is polling-only.
- **No readiness marker after a swap** — no class, flag, or event set once htmx
  has processed swapped content.
- `#connection-status` is a hardcoded `Connected` literal (`templates.rs:16`);
  nothing computes it.

## 4. Prior art, re-read through the robustness lens

`.scratch/test-strategy-execution/issues/07-browser-tier-design.md` settled
placement: `behaviour.rs` (6 specced) + `invariants.rs` (7 exempt), and 17
move-down tests. 08 executed the moves; 14 grilled missing tests and graduated
the responsive-layout + error-toast tests.

The per-surface re-split (ticket 20) then reorganized `behaviour.rs` into 7
files. Three consequences:

- **Deliberate growth** (tickets 14/15): the responsive-layout invariant
  (`invariants.rs:366`) and the error-toast behaviour (`dashboard.rs:87`) were
  *added on purpose*. They are not drift.
- **Real drift**: the tier now files tests by *surface* (worlds, games), where
  ticket 07 filed them by *assertion shape*. That is why class-2 and class-3
  tests sit in per-surface files beside class-1 tests, and why a browser-only
  HTTP-contract test (`worlds.rs:66`) shares a file with a pure-JS interaction
  test (`worlds.rs:9`).
- **Placement precedent for class 2**: ticket 07 already ruled that
  `test_status_updates_during_generation` and
  `test_form_stays_static_after_submission` stay browser-only even though their
  static aspects are HTTP-covered — because their value is the client wiring.
  That ruling is the anchor for judging class 2 today.

## 5. Verdict: load-bearing or accidental?

**Both.** The core is load-bearing; the shape is accidental, and that shape is
what makes the flake class structural.

**Load-bearing.** The per-test `TestServer` (state isolation),
`send_action`'s `wait_for_status_generating` ack guard (`browser.rs:120` — it
documents a genuine stale-"Ready" race), `dismiss_text_check_if_present` (a
dialog that removes `#status-display`), and `capture_failure_state` (it produced
the CDN evidence). Removing any of those loses real protection.

**Accidental.** Every readiness primitive is a visibility or count poll with a
hardcoded timeout, and none reads an htmx-readiness fact. `wait_until_visible`
returning on a rendered-but-unwired select is exactly ticket 03's mechanism. The
harness therefore cannot be fixed by tuning timeouts — **the app exposes no
htmx-readiness signal to poll.** The shared story-log gate then makes one
infrastructure hiccup fail N unrelated tests, because it is the only gate and
every test passes through it.

Accumulated surface that does not earn its keep: `wait_for_condition_sync` (no
uses), `extract_port_from_url` (no uses), the never-requested `/status/ready`,
the duplicated server-ready probe, and a binary-wide serialization override for
a 2-core box.

**The three design facts ticket 04 inherits:**

1. The harness has no app-side readiness signal to consume. "Make the race
   structurally impossible" requires adding one (`HX-Trigger`, an `afterSettle`
   marker, `hx-preserve`) or removing the browser hop.
2. Two HTTP contracts — `POST /worlds/:key/posture` and the games posture
   fragment — are covered *only* through Chromium. Demoting them to the HTTP
   tier adds coverage; it does not remove any.
3. 13 class-1 tests keep the browser tier alive regardless of what happens to
   class 2: 7 slash-menu interactions, 1 options Edit interaction, 3 story-log
   JS behaviours, 1 error toast, and 1 invariants test hosting 9 sub-checks.
   The tier's floor is ~13 tests, not 26.
