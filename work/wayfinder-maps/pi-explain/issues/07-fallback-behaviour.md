# 07 — Fallback behaviour

Type: grilling
Status: open
Blocked by: (none)
Assignee: (unclaimed)

## Question

What does the extension do when the panel cannot exist, or cannot show a
picture?

Four failure modes:

1. No panel host is loaded.
2. pi runs in print, JSON, or RPC mode, with no UI at all.
3. The terminal has no Kitty or iTerm2 graphics protocol, so `Image` renders a
   text placeholder. In fullscreen mode, iTerm2 also degrades to text.
4. Rendering fails: no Chrome, a render timeout, or invalid Mermaid.

## Options

- **Option A — overlay fallback.** Use `ctx.ui.custom({ overlay: true })`,
  anchored right, with focus released. Covers mode 1. Modes 2 to 4 fall back to
  text plus a browser link.
- **Option B — browser only.** No panel when the host is missing. The artifact
  always opens in the browser.
- **Option C — text always.** The panel shows Markdown, and pictures are an
  extra. Every failure degrades to text with a path the user can open.

## Background

- In print, JSON, and RPC modes the extension must still work. It writes the
  artifact and returns the path. Nothing may depend on rendering.
- `ctx.ui.custom()` with `overlay: true` is the only pi primitive that gives a
  custom screen without a third-party host. The doom-overlay example shows a
  continuously rendered overlay, so persistence is possible.
- The map's fog already carries the question of how failures are reported.

## Recommendation

Option A, with Option C as the floor under it. The artifact path is always the
guaranteed output.
