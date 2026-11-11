# Assessment B — Detector + Browser Evidence

Target: the Game tab, `http://127.0.0.1:3001/?critique=detector` (local probe server, already running).
Tab: CDP target id `E6F45AC6` (only this tab was used).
Detector: `impeccable` CLI **4.8.0** (`.pi/skills/impeccable/scripts/VERSION` = 0.1.18), `IMPECCABLE_NO_TELEMETRY=1` on every call.
Viewport: 1280×860 CSS px, DPR 1. Story window: `#story-log` (1018 px wide, 707 px tall, 41 entries `#entry-282` … `#entry-322`).

Raw outputs, next to this file: `b-scan-default.json`, `b-scan-top.json`, `b-index-html.json`, plus two determinism re-runs (`b-scan-default-2.json`, `b-scan-top-2.json`).

---

## 1. CDP scans of the rendered Game tab

Command (identical for all four runs):

```
IMPECCABLE_NO_TELEMETRY=1 .pi/skills/impeccable/scripts/impeccable detect --json \
  --cdp 9222 --target critique=detector
```

Exit code **2** on every run (= findings), stderr empty.

| Run | Scroll position of `#story-log` | `line-length` | `straggler` | Total | Exit |
|---|---|---|---|---|---|
| default (as found) | bottom, `scrollTop` 9272 = max 9272 | 78 | 7 | **85** | 2 |
| top (`scrollTop = 0`) | top | 78 | **0** | **78** | 2 |
| default, repeated after reload | bottom | 78 | 7 | **85** | 2 |
| top, repeated | top | 78 | **0** | **78** | 2 |

Severities present: `warning` 78, `advisory` 7. Categories present: `quality` only. **No `critical`/`error` findings, no colour, contrast, typography, spacing or accessibility rule fired.**

### Rule: `line-length` (warning, quality) — 78 findings, 78 unique selectors

Same 78 selectors in all four runs. The scanned element set is **not** scroll-window dependent: both scans cover entries 282–322 (41 entries), so the detector reads the whole DOM, not only the viewport.

Example selectors and snippets:

| Selector | Snippet |
|---|---|
| `#entry-282 > span.text > p` | ~132 chars on 4 of 5 rendered lines (aim for <80) |
| `#entry-283 > span.text > p` | ~134 chars on 3 of 3 rendered lines |
| `#entry-284 > span.text > p:nth-of-type(2)` | ~131 chars on 3 of 4 rendered lines |
| `#entry-285 > span.text > p:nth-of-type(2)` | ~127 chars on 2 of 3 rendered lines |
| `#entry-322 …` | (last entry in range) |

All 78 selectors are `.log-entry > span.text > p`, i.e. story paragraphs. The full selector list is in the JSON files.

**Measured in the page and confirmed real.** `cdp.mjs eval` over every paragraph:

| Measurement | Value |
|---|---|
| Paragraphs in the story window | 127 |
| Width of every paragraph | 810.09 px (min = median = max) |
| Category split of those paragraphs | 60 `narration`, 48 `input`, 9 `narration location`, 10 `input location` |
| Font | 14 px `-apple-system, BlinkMacSystemFont, "Segoe UI", Roboto, sans-serif`, `line-height` 21 px |
| `#story-log` client width | 1018 px |

810 px at 14 px in that stack is ≈ 130–135 characters per line — exactly what the snippets report. Cause, read from source: `assets/styles.css:288` gives `.log-entry { max-width: 85%; }` and `assets/styles.css:330` `.log-entry .text p { margin-bottom: var(--spacing-sm); }` sets no measure. No `max-width` anywhere on the prose column, so the bubble stretches to 85 % of a 953 px content box. This is a **true positive**, and a layout defect: the column is ~1.8× the recommended 65–75 ch.

### Rule: `straggler` (advisory, quality) — 7 findings at bottom scroll, 0 at top scroll

| Selector | Snippet |
|---|---|
| `#entry-287 > span.text > p:nth-of-type(2)` | "unpacked?" alone on the last of 4 lines (9 % of the measure) |
| `#entry-288 > span.text > p:nth-of-type(1)` | "with." alone on the last of 4 lines (4 %) |
| `#entry-289 > span.text > p:nth-of-type(3)` | "step." alone on the last of 2 lines (4 %) |
| `#entry-295 > span.text > p:nth-of-type(2)` | "drive." alone on the last of 3 lines (4 %) |
| `#entry-306 > span.text > p:nth-of-type(1)` | "jacket." alone on the last of 2 lines (5 %) |
| `#entry-307 > span.text > p:nth-of-type(2)` | "it." alone on the last of 3 lines (1 %) |
| `#entry-308 > span.text > p:nth-of-type(1)` | "runner." alone on the last of 3 lines (5 %) |

Five of the seven are the same paragraphs that also carry `line-length` (duplicate selectors in `b-scan-default.json`); only `#entry-289 > span.text > p:nth-of-type(3)` and `#entry-306 > span.text > p:nth-of-type(1)` are straggler-only.

**Observed and reproducible: the rule fires only when `#story-log` is scrolled to the bottom.** Two bottom runs give 7, two top runs give 0, with no change to the DOM in between (41 entries before and after; `nav` reload kept the count at 41). I did not find why. *(guessed: the rule measures rendered line boxes of in-viewport text, and the bottom position lays out a different set of paragraphs than the top position — the seven selectors are not contiguous with either viewport, so this explanation is unproven.)* Treat the 7 as a real but scroll-dependent sample, not as the complete set for the story.

---

## 2. CLI scan of markup: `assets/index.html`

```
IMPECCABLE_NO_TELEMETRY=1 .pi/skills/impeccable/scripts/impeccable detect --json assets/index.html
```

**0 findings, exit code 0, stderr empty.** Saved as `b-index-html.json` (`[]`).

Reason this is near-meaningless for this app, and why I did not widen it: the HTML is not in markup files. Only a static shell lives in `assets/index.html`; every fragment that carries the story, the controls and the forms is rendered by Askama templates inside Rust string literals under `src/adapters/driving/http/`. A file scan of that tree would read Rust source as markup and produce noise, so the rendered CDP scan above is the real evidence for this target. I did not scan `assets/styles.css`: `critique.md` says not to pass CSS-only files.

---

## 3. Overlay flow (critique.md steps 2–4)

| Step | Result | Evidence |
|---|---|---|
| Preflight mutation (mutation available?) | **Available** | `document.title` set to `CDP-PREFLIGHT-OK` and read back; an injected `<script>` set `window.__preflightScript = 1`; both confirmed, then cleaned up |
| Console capture installed before injection | Done | `console.log/info/warn/error/debug` wrapped to push any message whose text contains `impeccable` into `window.__imp`; verified still wrapped after injection (`String(console.log)` contains `__imp`) |
| `impeccable live-server --background` | Started | `{"pid":3558801,"port":8400,...}`; `curl http://localhost:8400/detect.js` returned **200** |
| Inject `http://localhost:8400/detect.js` into the tab | **Succeeded** | no `onerror`; globals `impeccableDetect`, `impeccableDetectAsync`, `impeccableScan`, `impeccableAnalyzeVisualContrast`, `impeccableSnapshotCapture` and others present on `window` |
| Wait, read console messages | **78 overlays, 0 console messages** | see below |
| Overlay screenshot | `tmp/impeccable/critique/b-overlay.png` | orange boxes on each long paragraph, each labelled `line length too long` |
| `live-server stop --keep-inject` | Stopped | `Stopped live server on port 8400.`; no repo file touched |
| Reload to remove overlay | Done | `nav …/?critique=detector`, then `.impeccable-overlay` count **0**, `window.__imp` `undefined` |

**Overlay content.** `document.querySelectorAll('.impeccable-overlay')` returned **78** elements at injection time (the tab was scrolled to the top then). Every `.impeccable-label` text was `line length too long` — one label value only, 78 counts. No straggler overlay appeared, consistent with the top-scroll scan returning 0 stragglers. Boxes are absolutely positioned at the paragraph rects (`width: 814.09px`), so the overlay agrees with the measured 810 px paragraph width.

**Console finding: none.** `window.__imp` stayed empty after 3 s, and the wrap was still in place. The injected bundle therefore logs nothing for this page (`impeccableDetect()` is callable directly and returns an array of **81** scanned element records with keys `selector`, `tagName`, `rect`, `isPageHidden`-adjacent flags `isHidden`, and `findings` — not a finding list). So: the overlay is real and user-visible; there is **no console text to quote**, and any claim of console output from this run would be invented.

---

## 4. False positives and context

| Finding | Class | Reason |
|---|---|---|
| 78 × `line-length` on story paragraphs | **True positive** | Measured: every paragraph is 810 px at 14 px ≈ 130+ chars. Fix is CSS, not text: cap the prose measure (`max-width: ~72ch` on `.text p`, or `min(85%, 72ch)` on `.log-entry` at `assets/styles.css:288`). |
| 5 of 7 `straggler` findings | **True but largely redundant** | Same paragraphs as `line-length`. A `text-wrap: pretty` on `.log-entry .text p` (`assets/styles.css:330`) addresses widows and the long measure together. |
| 2 `straggler`-only findings (`#entry-289`, `#entry-306`) | **True positive, not actionable by hand** | The copy is LLM-written narrative stored in the Game record. The advice "edit the last sentence" cannot be applied to authored output; only the CSS half of the remedy applies. |
| `straggler` absent at top scroll | **Not a false positive — a coverage gap** | Scroll-position-dependent, reproduced twice each way. Do not read the top-scroll result as "no widows in the story". |
| Game display name `Redmist Estate — 31 Oct 2026 (7)` (em-dash, by design) | **Not flagged** | The detector raised nothing against the header in any run. No suppression was needed. Recorded so the parent does not attribute an em-dash finding to this run. |
| Story prose content, names, LLM wording | **Not flagged** | No `smart-quotes`, `em-dash` or copy rule fired on narrative text, so there is nothing to suppress as a false positive here. |

`assets/index.html` "clean" (0 findings) is **not** a true negative for the Game tab. The file it scans holds a shell only; the audited surface is template output.

---

## 5. Skipped or failed steps

| Step | Status | Concrete reason |
|---|---|---|
| Plain URL scan `impeccable detect <url>` | **Skipped** | Chrome cannot start its own sandbox in this container, so the launcher's browser fails to start. Used the running Chrome with `--cdp 9222 --target critique=detector` instead, as the brief directs. |
| File/directory scan of the template sources | **Skipped on purpose** | The markup lives in Askama templates inside Rust `&str` literals under `src/adapters/driving/http/`. Scanning Rust source as markup would report noise, not the rendered page. |
| CSS-only scan of `assets/styles.css` | **Skipped** | `critique.md` forbids passing CSS-only files to `detect`. |
| Console readout from `cdp.mjs` | **Worked around, and empty anyway** | `cdp.mjs` cannot read the console, so messages were captured through the in-page `window.__imp` array as the brief instructs. The array stayed empty because the bundle logs nothing on this page. |
| Root-cause of the `straggler` scroll dependency | **Not determined** | Two runs per position confirm the dependency; the mechanism is a guess. Reported as an observation, not a conclusion. |
| `impeccableSnapshotCapture`, `impeccableCollectVisualContrastCandidates` | **Not exercised** | They exist on `window` but `critique.md`'s flow does not call them, and a contrast sweep may report against LLM prose. Left untouched to keep the run to the documented flow. |

## 6. Read-only compliance and left state

- No Action sent, no Send / Autoplay / Retry / swipe / edit / delete / Save clicked, **no POST**.
- `cdp.mjs` was used only for `eval`, `shot`, `nav`, `list`. The `nav` reload does not create a turn.
- No repo file edited. `git status --porcelain` after the run shows only the pre-existing `M .gitignore`, `?? .pi/skills/` and `?? PRODUCT.md` — none of them mine.
- Files written: `assessment-b.md`, `b-scan-default.json`, `b-scan-top.json`, `b-index-html.json`, `b-scan-default-2.json`, `b-scan-top-2.json`, `b-before.png`, `b-overlay.png`. There is no `b-scan-default.err`/`b-scan-top.err` content: both stderr files are empty; `b-index-html.err` too.
- `pgrep -af "impeccable"` after the stop shows no `live-server` process. The live server is **not** running; the `detect.js` tag and the 78 overlays were removed by a reload of the tab.
- Tab state restored: `#story-log` back at the default **bottom** position (`scrollTop` 9272 = max 9272), `document.title` back to `Chronicler Engine`, 41 entries intact.
