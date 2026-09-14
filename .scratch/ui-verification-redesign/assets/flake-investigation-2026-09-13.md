# Flake investigation evidence — 2026-09-13

Session investigation into browser-tier flakiness. Two distinct root-cause
classes found; only the first is fixed.

## Fixed: htmx fetched from unpkg CDN per page load

**Root cause.** `assets/index.html` loaded htmx from
`https://unpkg.com/htmx.org@1.9.10`. Every browser test launches a fresh
Chromium with no shared HTTP cache, so each of the 26 tests re-fetched the 47KB
script cross-network. When the fetch stalled, htmx never initialized, no
`hx-get="/fragment/story-log"` request was ever made, and the shared
precondition `wait_for_element_children("#story-log .log-entry", 1)`
(tests/test_utils/browser.rs:87, hard 10s cap) timed out.

**Evidence.**

- Failing run's DOM dump (tmp/test_diagnostics): `#story-log` present but empty
  with unprocessed `hx-*` attributes still on it. 24K dump vs ~48K normal.
- Failing run's teed engine stdout (tmp/test_server_logs/{port}_stdout.log):
  ZERO `/fragment/story-log` requests. Passing runs: 2–18. Server healthy
  throughout (served `/`, CSS, 200s) — it was simply never asked.
- `grep -oE 'https?://' assets/index.html` returned exactly one external URL:
  the unpkg script tag.
- 7 distinct tests had flaked across session logs; 6 of them failed on
  `#story-log .log-entry`, 1 on `#options-dock .option-btn` — because
  *every* `with_test_page` test passes through the shared line-87 gate. One CDN
  hiccup failed whichever test happened to be there, which is why "which test
  is flaky" had no single answer.

**Fix (done).** Downloaded `htmx.org@1.9.10` dist to `assets/htmx.min.js`
(47,755 bytes, md5 `7e9c374d75c23ad1d811715d41ac89f1`, byte-identical to CDN).
`assets/index.html:7` now uses `/assets/htmx.min.js?v=1` (existing
cache-busting convention). Verified: engine serves the vendored file and all
fragments including `/fragment/story-log` fire. Uncommitted as of map date.

## Unfixed: `worlds::test_world_posture_change_autosaves_status` — hx-post never leaves the browser

**Behaviour.** Test selects an option in the autosaving world-posture form
(`hx-post` + `hx-trigger="change"`) and asserts `#world-posture-status`
contains "Saved" within a hard 5s. In failing runs the POST is never sent at
all.

**Reproduction (serialized, retries=0, 8 cores).** 3 of 8 solo runs failed
(runs 5, 7, 8). EVERY failure had **zero** posture requests in the teed engine
log; every pass had 2. The form's `GET /worlds/redmist_estate/edit` succeeds in
failing runs too — so the form loads and is visible, but the `change` event
never becomes an HTTP request.

**In-page probe at the failure point** (temporary edit, since reverted),
immediately after `wait_until_visible` on the narrator_mode select and before
`select_option`:

```json
{"htmxLoaded":true,"htmxVersion":"1.9.10","selHasHxPost":true,"selInDoc":true,"panels":1}
```

i.e., htmx is loaded, the select carries its `hx-post` attribute, the select
is in the document, and there is exactly one `.worlds-panel` (not a
stale/duplicate swap target). Yet the POST still didn't fire. Probe runs 1–2
failed while 3–6 passed — the DOM looks identical.

**Leading hypothesis (unverified).** htmx attaches its `change` listener to
swapped-in content asynchronously-ish after the swap; `wait_until_visible`
returns as soon as the element is visible, which can precede listener
attachment. The harness has no "htmx has processed this element" wait — all
waits in tests/test_utils/wait.rs are visibility/text polling. This is a
*test-harness readiness* gap, but the redesign should also ask whether the
*application* makes readiness observable in a cleaner way (e.g. htmx lifecycle
events, explicit ready flags) rather than every test re-solving timing.

## Supporting facts

- Each browser test = fresh `TestServer` (its own port from 3010–3050) + fresh
  Chromium. Failures flush-node-idle timing, so nothing can be assumed about
  event ordering between htmx swap and the next Playwright action.
- nextest overrides serialize the entire browser binary
  (`threads-required = num-test-threads`), added when the box had 2 cores. The
  box is now an 8-core WSL2 guest (.wslconfig processors=8, applied and
  restarted). Serialization kept for now — it's a stabilization knob, not a
  correctness fix.
- Engine log tee: `tmp/test_server_logs/{port}_{stream}.log` persists after
  failure. Failure DOM dumps: `tmp/test_diagnostics/`.
- build.py now surfaces `(N flaky)` in its epilogue (added this session) — the
  detection gap where nextest retries masked flakes as silence is closed.

## Verbatim failure

```
world posture auto-save should report Saved: Assertion timeout: Expected
element '#world-posture-status' to contain text 'Saved', but had '' after 5s
```
panics at `tests/browser/worlds.rs:86:17`.
