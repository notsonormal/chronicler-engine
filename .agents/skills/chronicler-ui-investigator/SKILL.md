---
name: chronicler-ui-investigator
description: "Chronicler UI investigation — drive the running dashboard through Chrome DevTools and capture shot, DOM and endpoint evidence for ad-hoc checks, gameplay debugging, and post-plan review. Triggers on: /chronicler-ui-investigator, chronicler ui, dashboard review, ui screenshot"
argument-hint: "[port] [world]"
---

# Chronicler UI Investigator

Drive the running Chronicler dashboard and capture evidence for an expectation the caller supplies. This skill captures; the caller judges. Defaults: port 3000, world `redmist_estate`.

## Prerequisites

- Browser tools: `chrome_devtools_navigate`, `chrome_devtools_evaluate`, `chrome_devtools_screenshot`, `chrome_devtools_list_pages`, `chrome_devtools_select_page`, plus the `chrome_devtools_load` loader. If a capability is missing, call the loader with a task query (`/chrome-devtools enable` activates everything too); if the loader itself is missing, `/reload`.
- Chrome must already be running. `~/.bashrc` starts it via `/home/node/.pi/start-browser.sh`; run that script and retry when a tool reports an unreachable endpoint, or when a CDP call times out (e.g. `Page.enable` on a wedged page).
- No browser tools at all: drive `node scripts/cdp.mjs` (`list`, `shot`, `snap`, `html`, `eval`, `nav`, `click`, `type`) from `bash`, or use the Playwright harness below.
- [ENVIRONMENT.md](ENVIRONMENT.md) has the rest of the machinery: the extension, viewports, delegated sweeps, and what breaks in this container.

## Workflow

### 1. Serve

```bash
cargo run -- --world redmist_estate --port 3000
curl -s -o /dev/null -w "%{http_code}" http://127.0.0.1:3000/   # 200 → reuse this server
```

Done when `/` returns 200. A dead session can leave the port bound — check before starting a second engine. Raw `cargo run` in the shared `build.py` target dir recompiles the dependency tree (~2 min); the repo's `ENVIRONMENT.md` has the measurement.

### 2. Drive

```javascript
chrome_devtools_navigate(url="http://127.0.0.1:3000")
```

`chrome_devtools_evaluate` awaits the promise it is given, so poll inside it for dynamic content:

```javascript
chrome_devtools_evaluate(expression="(async () => { for (let i = 0; i < 40; i++) { if (document.querySelector('#story-log .log-entry')) return 'ready'; await new Promise(r => setTimeout(r, 250)); } return 'timeout'; })()")
```

For a turn in flight, poll `#status-display` until it reads "Ready" — during generation it reads "Thinking", "Narrating", "Generating" or "Quantifying" (spec 16.6). `/status/generating` returns a bare phase name or `idle`, and `onStatusPoll` maps `idle` to "Ready" in the DOM, so poll the element, not the endpoint.

Done when the surface under test is on screen.

### 3. Capture

```javascript
// DOM structure — replaces accessibility-tree snapshots
chrome_devtools_evaluate(expression="(() => ({ title: document.title, sections: [...document.querySelectorAll('[id]')].map(e => e.id).slice(0, 30), storyLogEntries: document.querySelectorAll('#story-log .log-entry').length, actionArea: !!document.querySelector('#action-area'), connectionStatus: document.querySelector('#connection-status')?.textContent }))()")

// Fragments and status respond from page context, leaving the dashboard loaded
chrome_devtools_evaluate(expression="(async () => { const paths = ['/fragment/action-area', '/fragment/character-headshots', '/status/generating']; const out = {}; for (const p of paths) { out[p] = (await fetch(p)).status; } return out; })()")

// Shot — returns the image inline and writes it to savePath (a temp file if omitted)
chrome_devtools_screenshot(savePath="tmp/chronicler-ui.png")
```

For a post-plan pass, fetch the full dashboard set instead — `header`, `story-log`, `visual-sidebar`, `options-dock`, `action-area`, `character-headshots`, `settings`, `prompt-presets`, `games`, `worlds`, `llm-messages` — plus `/status/generating`, and confirm each is 200.

`#connection-status` renders server-side and always reads "Connected"; it is not a live socket indicator.

**Done only when you have looked at a shot.** A green test, a DOM dump, a clean engine log and a delegated agent's report are each narrower evidence, and none of them stands in for a shot of the rendered page that you have personally compared against expectations. Write findings to the ticket as they are established, so a dead session costs the shots rather than the analysis. Console messages have no tool equivalent: read the engine log tee, `tmp/test_server_logs/{port}_{stream}.log` (see `tests/AGENTS.md`), or the terminal for a hand-started server.

## Reproducible checks: the Playwright harness

When the browser tools are unavailable, or the check should ship as coverage: real Chromium against a real server, driven by Rust tests in `tests/browser/`. Reuse `send_action` / `wait_for_status_ready` / `capture_failure_state` from `tests/test_utils/` — failure dumps write a shot and a DOM dump under `tmp/`. `SLOW_MO=500 python build.py test-pattern <name>` slows a run down for watching; `HEADED=1` cannot work here (no display). Register a throwaway test in `tests/browser/mod.rs` and delete it afterwards; shipped coverage belongs to a spec ticket.

## Delegated sweeps

For a broad sweep, delegate to a `generalist` managed agent with the exact `node scripts/cdp.mjs` commands in the task — viewports, method and the review's P1–P3 scale live in ENVIRONMENT.md's *Delegated screenshot sweeps*. Its report does not discharge the look-at-the-shot rule.

## Driving gameplay

Everything below is **POST**: a GET returns 405, a bodyless POST 415, so send a JSON body with the right content type.

- `/action` submit a player command · `/action/check` text check before submitting · `/action/confirm` confirm a corrected command
- `/check-text` standalone text check · `/swipe/new` new swipe · `/message/:id/swipe/:index` switch swipe
- `/retrigger` retrigger the last event · `/history/:id` edit a history entry · `/history/delete` delete the last one

A fetch proves an endpoint responds, not that the UI updates — drive the real DOM from `chrome_devtools_evaluate` for click Send, switch swipe and retrigger, and for the slash flows (`/impersonate`, `/guide`, `/options`) that submit through `#slash-menu` (spec `browser_slash_menu.md`). Full route list: `docs/diataxis/reference/frontend/http_routes.md`, generated from `router.rs`.

Selector vocabulary: `docs/specs/browser_*.md`, enforced in `tests/browser/` (`dashboard.rs`, `games.rs`, `options.rs`, `prompt_presets.rs`, `worlds.rs`, plus `stub/`). Tabs switch through `.tab[data-tab="<name>"]`; the `#<name>-tab` id is the hidden content panel, so clicking the id does nothing. Common handles: `.log-entry`, `.edit-btn` / `#edit-textarea` / `.cancel-btn` (edit mode), `.delete-btn`, `#command-form input[name="command"]`, `#status-display`, `#error-notification.visible`, `#slash-menu` / `.slash-suggestion`, `#world-posture-status`.

## Troubleshooting

| Symptom | Check |
|---|---|
| Status display frozen | `/status/generating`; `#status-display` swaps on its own 5s poll |
| Capability tool missing | `chrome_devtools_load`; loader missing → `/reload` |
| Endpoint unreachable, or a CDP call times out | `/home/node/.pi/start-browser.sh`, then retry |
| Engine won't start | Port already bound by a dead session; run `cargo run` in the foreground for the error |
