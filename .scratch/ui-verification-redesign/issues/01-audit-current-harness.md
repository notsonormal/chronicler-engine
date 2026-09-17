# Audit the current browser-test design: what does each test actually verify, and through which layer?

Type: research
Status: resolved
Blocked by:

## Question

Read [browser-tier-inventory.md](../assets/browser-tier-inventory.md) and
[flake-investigation-2026-09-13.md](../assets/flake-investigation-2026-09-13.md)
first. Then audit the 26 tests in `tests/browser/` against
`tests/STRATEGY.md`'s placement rule (browser tier only for assertions only a
browser can observe: DOM, CSS, JS, rendering).

Classify each test into one of:

1. **Browser-only by nature** — asserts rendering/layout/client JS behaviour
   (e.g. slash menu keyboard nav, CSS invariants). Correct tier.
2. **Browser-observable behaviour that is really the app's wiring** — e.g.
   an `hx-trigger="change"` select producing a POST and a status fragment.
   The *application* contract is the HTTP exchange; the browser is one client
   of it.
3. **Already covered at the HTTP tier** — an equivalent request-level contract
   test exists (e.g. http-tier posture merge in tests/http/worlds.rs vs
   browser posture autosave in tests/browser/worlds.rs).

Also audit the harness itself: `with_test_page`, the shared story-log gate at
browser.rs:87, `wait.rs` primitives, `goto_with_connection_check`. Ask of each
piece: what flake does it prevent, what flake does it cause, and would it exist
in a simpler design? Note that the shared gate is what turned one CDN hiccup
into 6 different "flaky tests".

Two further things to inventory while auditing:

1. **Prior art.** Read `.scratch/test-strategy-execution/issues/07-browser-tier-design.md`
   (Answer section), `08-browser-tier-execution.md`, and
   `14-browser-missing-tests.md`. Tier placement was already grilled out
   (13 keep / 17 move-down; behaviour.rs specced + invariants.rs exempt). Do not
   re-litigate placement from scratch — your audit re-reads the current tier
   through a *robustness* lens and notes where the tier has drifted since the
   per-surface split (9 files, 26 tests).
2. **The app's existing readiness surface.** Inventory every readiness /
   observability hook the dashboard already exposes, since ticket 04 (Decide
   the UI verification design) will want them as raw material: `/status/generating`,
   `/debug/state`, `/debug/is_generating`, `HX-Trigger` response headers,
   htmx lifecycle events already plumbed, WebSocket state channels (see
   `docs/diataxis/explanation/two-state-channels.md`). Say which are observed
   by tests today and which go unused.

Deliverable: a linked markdown asset — a per-test classification table, the
readiness-hook inventory, + a short verdict paragraph on whether the harness
complexity is load-bearing or accidental. Make no code changes.

## Answer

Audit asset: [harness-audit.md](../assets/harness-audit.md). No code changes.

### The 26 tests: 13 / 12 / 1

Of the 26 tests, **13 are browser-only by nature** (7 slash-menu interactions,
1 options Edit interaction, 3 story-log JS behaviours, 1 error toast, and 1
invariants test hosting 9 sub-checks), **12 are browser-observable app wiring**
(class 2), and **1 is already covered at HTTP** (class 3).

- Class 3 is exactly one test:
  `test_games_mode_switch_retargets_and_nudges` (worlds-era name;
  `tests/browser/games.rs:106`) asserts the same two facts as
  `tests/http/narrator_mode.rs:144` (SCENARIO 23.3) on the same server render.
- Class 2 is the deep-thin-brittle-layer smell: assertions are on
  server-derived content (posture values, option texts, preset flags, entry
  counts); only the mechanism (click, `change`, reload, swap) is browser-side.

### Two findings the ticket did not anticipate

**`POST /worlds/:key/posture` has no HTTP test at all.** The endpoint
(`src/adapters/driving/http/builders/router.rs:124`, handler
`worlds/handlers/worlds.rs:240`) is covered only through the two browser worlds
tests. `tests/http/worlds.rs` covers `POST /worlds/:key` (full form), not the
posture autosave endpoint. So the flaky test is not re-testing HTTP — it is the
sole coverage of a real HTTP contract, reached through a Chromium round-trip.

**`#game-posture-controls` has no HTTP test.** `tests/browser/games.rs:11` is
the only assertion that the games posture fragment renders populated selects.

This inverts part of the ticket's premise: demoting those two contracts to the
HTTP tier *adds* coverage while removing the race, rather than merely relocating
it.

### Readiness surface: no htmx-readiness signal exists

Observed by tests today: `GET /status/generating` (the `#status-display` poll
source), `POST /status/reset-generating`, `HX-Retarget`/`HX-Reswap`,
`HX-Refresh`, and the browser-side `htmx:beforeSwap` listener.

Unused: `GET /status/ready` (registered, tested, never requested by the UI),
`GET /debug/state`, `GET /debug/is_generating`, `GET /debug/backend`.

Absent entirely: **no `HX-Trigger` header in `src/`**, **no `htmx:afterSettle`
listener**, **no `hx-preserve`**, **no WebSocket code** (the `AGENTS.md`
"HTTP/WebSocket server" line is stale; `two-state-channels.md` is about
persisted vs process-local generation state). The dashboard is polling-only,
with five concurrent pollers — which is why `networkidle` is unsatisfiable.

### Verdict: both load-bearing and accidental

Load-bearing: the per-test `TestServer` (state isolation), `send_action`'s
`wait_for_status_generating` ack guard (`tests/test_utils/browser.rs:120`,
which documents a genuine stale-"Ready" race), `dismiss_text_check_if_present`,
and `capture_failure_state` (it produced the CDN evidence).

Accidental: every readiness primitive is a visibility or count poll with a
hardcoded timeout, and none reads an htmx-readiness fact. `wait_until_visible`
returning on a rendered-but-unwired select is exactly ticket 03's mechanism — so
the harness cannot be fixed by tuning timeouts, because **the app exposes no
htmx-readiness signal to poll.** The shared story-log gate then makes one
infrastructure hiccup fail N unrelated tests. Dead or stale surface:
`wait_for_condition_sync` and `extract_port_from_url` (no uses),
`/status/ready`, the duplicated server-ready probe, and the binary-wide
serialization override for a 2-core box.

### Consequence for ticket 04

1. The design must **add** a readiness signal (`HX-Trigger`, an `afterSettle`
   marker, `hx-preserve`) or **remove** the browser hop — nothing in the app can
   be polled for htmx-readiness today.
2. Demoting the two orphaned HTTP contracts adds coverage; it is not a
   coverage trade.
3. The browser tier's floor is ~13 class-1 tests regardless of what happens to
   class 2.

No fog graduates from this answer beyond ticket 04's own design decision. No
new tickets surfaced.

