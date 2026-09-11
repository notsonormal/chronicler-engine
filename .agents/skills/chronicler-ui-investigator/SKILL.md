---
name: chronicler-ui-investigator
description: "Investigation workflow for Chronicler Engine UI - used for testing, debugging, and post-plan verification. Triggers on: /chronicler-ui, /ui-investigate, investigate chronicler ui, test chronicler ui, debug ui"
argument-hint: "<action> [port] [world]"
---

# Chronicler UI Investigator

Investigation workflow for Chronicler Engine UI - used for testing, debugging, and post-plan verification.

## Overview

Provides browser automation to:
1. Launch Chronicler server (or connect to running instance)
2. Navigate to UI
3. Capture state (DOM state via evaluate, screenshots, HTTP endpoint checks; console errors via engine logs)
4. Return findings for caller to interpret

**Does NOT include specific expectations** - caller provides what to look for.

## Prerequisites

- Chronicler Engine project at the repo root
- Browser automation via the `@narumitw/pi-chrome-devtools` pi extension: `chrome_devtools_navigate`, `chrome_devtools_evaluate`, `chrome_devtools_screenshot`, `chrome_devtools_list_pages`, `chrome_devtools_select_page`. Verified working under WSL. It attaches to CDP on `127.0.0.1:9222` or launches its own Chromium; `/chrome-devtools status` shows which. This is the path for ad-hoc interactive checks — no throwaway test needed. If the tools are missing from the toolset, the user runs `/chrome-devtools enable` or `/reload`.
- The repo's Playwright harness (`tests/browser/`): the path for reproducible checks and shipped coverage (spec tickets).

**No tool equivalent exists for:** console-message capture (use the engine log tee, see Step 3) and accessibility-tree snapshots (use a DOM-dump `chrome_devtools_evaluate` expression instead).

## Reproducible checks: the Playwright harness

Use the repo's headless Playwright harness (`tests/browser/`) — real Chromium against a real server, driven by Rust tests. Key pieces: `send_action` / `wait_for_status_ready` / `capture_failure_state` in `tests/test_utils/` (failure dumps write a screenshot + DOM dump under `tmp/`); `HEADED=1 SLOW_MO=500 python build.py nextest <name>` runs one test interactively. Engine stdout/stderr tees to `tmp/test_server_logs/{port}_{stream}.log` (see `tests/AGENTS.md`). For ad-hoc verification prefer the extension (above). Write a throwaway test in `tests/browser/` (register it in `mod.rs`), run it, view the screenshots, then DELETE it only when the extension is unavailable — shipped coverage belongs to a spec ticket. The Mandatory Screenshot Verification rule below still applies: the failure dumps are screenshots; look at them.

## Usage Patterns

### For Testing
```
/chronicler-ui test [port]
```
- Starts fresh server
- Captures baseline UI state
- Returns DOM state + screenshot for verification

### For Debugging
```
/chronicler-ui debug <port> <world>
```
- Connects to running server
- Focus on engine-log errors and DOM state
- Useful when issue is already reproduced

### For Post-Plan Verification
```
/chronicler-ui verify [port]
```
- After plan implementation
- Captures full state for comparison
- Returns complete DOM dump + screenshot

---

## Parameters

| Parameter | Default | Description |
|-----------|---------|-------------|
| `port` | 3000 | Server port |
| `world` | redmist_estate | World to load |
| `action` | test | test, debug, or verify |

---

## Workflow Commands

### Step 1: Ensure Server Running

**Option A: Start new server**
```bash
cargo run -- --world <world> --port <port>
```

**Option B: Use existing**
```bash
# Skip if already running: curl -s -o /dev/null -w "%{http_code}" http://127.0.0.1:<port>/
```

### Step 2: Navigate

```javascript
chrome_devtools_navigate(url="http://127.0.0.1:3000")
```

The navigation call returns after page load. To wait for dynamic content (htmx swaps, generation status), poll inside `chrome_devtools_evaluate` — the tool awaits the returned promise:

```javascript
chrome_devtools_evaluate(expression="(async () => { for (let i = 0; i < 40; i++) { if (document.querySelector('#story-log .log-entry')) return 'ready'; await new Promise(r => setTimeout(r, 250)); } return 'timeout'; })()")
```

Waiting for a generation to finish: poll `#status-display` until it reads "Ready". During generation it shows one of "Thinking", "Narrating", "Generating", "Quantifying" (spec scenario 16.6).

### Step 3: Capture State

```javascript
// DOM structure (replaces accessibility-tree snapshots)
chrome_devtools_evaluate(expression="(() => ({ title: document.title, sections: [...document.querySelectorAll('[id]')].map(e => e.id).slice(0, 30), storyLogEntries: document.querySelectorAll('#story-log .log-entry').length, actionArea: !!document.querySelector('#action-area'), connectionStatus: document.querySelector('#connection-status')?.textContent }))()")

// HTTP endpoints respond — fetch from the page context; keeps the dashboard loaded
chrome_devtools_evaluate(expression="(async () => { const paths = ['/fragment/action-area', '/fragment/character-headshots', '/status/ready', '/status/generating']; const out = {}; for (const p of paths) { out[p] = (await fetch(p)).status; } return out; })()")

// Screenshot (for visual analysis) — returns inline AND saves to disk
chrome_devtools_screenshot(savePath="tmp/chronicler-ui.png")
```

Console errors have no tool equivalent. For test-spawned servers read the engine log tee; for a server you launched by hand, read its terminal output:

```bash
grep -iE "error|panic" tmp/test_server_logs/3000_*.log | tail -20
```

---

## Return Format

The skill returns raw data - caller interprets:

| Data | Use Case |
|------|----------|
| `chrome_devtools_evaluate` (DOM dump) | Verify elements present, layout structure |
| `chrome_devtools_screenshot` | Visual regression, color checking (returns inline) |
| `chrome_devtools_evaluate` (fetch loop) | Fragment + status endpoints respond 200 |
| Engine log tee grep | Detect JS errors, panics, 404s |

---

## Mandatory Screenshot Verification

**Every UI investigation MUST end with a screenshot.** This is non-negotiable.

After making any changes and before claiming verification:
1. Navigate to the page
2. Take a screenshot: `chrome_devtools_screenshot(savePath="tmp/<name>.png")`
3. **Look at the screenshot** — visually confirm the layout is correct
4. Report what you see and whether it matches expectations

**Do NOT claim "verified" based on:**
- DOM dump alone (doesn't show visual layout)
- Engine logs alone (no errors ≠ correct rendering)
- Subagent reports alone (you must see it yourself)
- Test passes alone (CSS bugs don't fail tests)

**Only claim verified when you have:**
- A screenshot showing the actual rendered page
- Personally confirmed the visual result matches expectations

---

## Customization Guide

### Testing - Check element presence
```javascript
chrome_devtools_navigate(url="http://127.0.0.1:3000")
chrome_devtools_evaluate(expression="(() => ({ sections: [...document.querySelectorAll('[id]')].map(e => e.id) }))()")
// Caller verifies specific elements exist
```

### Debugging - Find what's broken
```bash
grep -iE "error|panic" tmp/test_server_logs/3000_*.log | tail -20
```
```javascript
chrome_devtools_evaluate(expression="(() => ({ title: document.title, storyLogEntries: document.querySelectorAll('#story-log .log-entry').length, actionArea: !!document.querySelector('#action-area') }))()")

// Check interactive endpoints respond
chrome_devtools_evaluate(expression="(async () => { const out = {}; for (const p of ['/fragment/action-area', '/status/ready']) { out[p] = (await fetch(p)).status; } return out; })()")
```

### Post-Plan - Full capture
```javascript
// Verify all dashboard fragments and status endpoints load (all verified 200)
chrome_devtools_evaluate(expression="(async () => { const paths = ['/fragment/header', '/fragment/story-log', '/fragment/visual-sidebar', '/fragment/options-dock', '/fragment/action-area', '/fragment/character-headshots', '/fragment/settings', '/fragment/prompt-presets', '/fragment/games', '/fragment/worlds', '/fragment/llm-messages', '/status/ready', '/status/generating']; const out = {}; for (const p of paths) { out[p] = (await fetch(p)).status; } return out; })()")

chrome_devtools_navigate(url="http://127.0.0.1:3000")
chrome_devtools_evaluate(expression="(() => ({ sections: [...document.querySelectorAll('[id]')].map(e => e.id), storyLogEntries: document.querySelectorAll('#story-log .log-entry').length }))()")
chrome_devtools_screenshot(savePath="tmp/post-plan-ui.png", fullPage=true)
```

---

## Error Handling

| Issue | Check |
|-------|-------|
| Server won't start | `cargo run` manually to see errors |
| Page not loading | Verify server started, check port |
| Elements missing | Check world loaded correctly |
| Console errors | Grep the engine log tee (Step 3) |
| Command form missing | Check `/fragment/action-area` |
| Status not updating | Check `/status/generating` and `/status/ready` |
| `chrome_devtools_*` tools missing | User runs `/chrome-devtools enable` or `/reload` |

---

## Interactive endpoints not covered by default

All **POST**. A GET returns 405 and a bodyless POST returns 415 — send a JSON body with the right content type. Route list: `docs/diataxis/reference/frontend/http_routes.md` (generated from `router.rs`). These are used during gameplay and should be tested separately when validating interactivity:

- `/action` — submit a player command
- `/action/check` — text-check before submitting
- `/action/confirm` — confirm a corrected command
- `/check-text` — standalone text check
- `/swipe/new` — retry/generate a new swipe
- `/message/:id/swipe/:index` — switch to a different swipe
- `/retrigger` — retrigger the last event
- `/history/:id` — edit a history entry
- `/history/delete` — delete the last history entry

For gameplay flows (click Send, switch swipe, retrigger), drive the real DOM from `chrome_devtools_evaluate` or fall back to the Rust harness — a fetch only proves the endpoint responds, not that the UI updates.

Selector vocabulary for DOM work comes from `docs/specs/browser.md`, enforced by `tests/browser/behaviour.rs`: `.log-entry`, `.edit-btn` / `#edit-textarea` / `.cancel-btn` (edit mode), `.delete-btn`, `#command-form input[name="command"]`, `#status-display`, `#error-notification.visible`, `#slash-menu` / `.slash-suggestion`, `#world-posture-status`. Slash commands (`/impersonate`, `/guide`, `/options`) are client-side flows through `#slash-menu` (spec 17.1–17.9) that submit via `/action`.

---

## Integration Points

- **In tests**: Capture baseline, compare post-change
- **In debugging**: Get state when issue occurs
- **In verification**: After plan implementation completes

---

## Notes

- No expected values hardcoded - caller provides assertions
- `chrome_devtools_screenshot` returns the image inline and saves it to disk; pass `savePath` (relative to the repo root works, e.g. `tmp/<name>.png`), otherwise it writes a temp file. Either way: look at it.
- Waiting for dynamic content: poll inside `chrome_devtools_evaluate` (Step 2 snippet)
- The `#connection-status` element is rendered server-side in the header fragment and shows "Connected" by default; it is not a live WebSocket state indicator
- Zero-dependency fallback when the extension is absent: `tmp/cdp_probe.mjs` pattern (Node built-in WebSocket against CDP on 9222), or the Playwright harness above
