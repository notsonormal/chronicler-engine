# 06 — Image rendering pipeline

Type: research
Status: open
Blocked by: (none)
Assignee: (unclaimed)

## Question

How does an artifact become a picture in the panel, given what this machine has
and does not have?

Two artifacts need rendering:

1. **Mermaid** source into an image.
2. **Self-contained HTML** into a preview image.

## Constraints

- `mmdc` is **not** installed. Neither is `bun`. npm and node are present.
- `google-chrome` **is** installed at `/usr/bin/google-chrome`.
- `visual-explainer` uses `playwright-core` (optional peer dependency) for its
  screenshot path. It is not installed here.
- The panel renders pictures through the `Image` component from
  `@earendil-works/pi-tui`, which needs a PNG, JPEG, GIF, or WebP as base64 plus
  a MIME type.
- A pure-JS Mermaid render needs a DOM. Options include a headless Chrome pass
  over the Mermaid source, or a bundled renderer such as `mermaid` plus a
  DOM shim.

## What to find out

1. The cheapest reliable Mermaid-to-PNG path that needs no new system binary.
2. Whether headless Chrome alone covers both cases: render the Mermaid or HTML
   in a page, screenshot it, done.
3. The screenshot parameters that match a side panel: what width and height, and
   whether a 3:2 shape fits the explain-claude-mod precedent.
4. The failure modes: no Chrome, Chrome crash, render timeout, invalid Mermaid.
5. Whether the same pipeline can produce the full-size image for the browser
   view, so the panel and the browser agree.

## Deliverable

A markdown summary linked as an asset. Include the exact command lines that
work on this machine, and the ones that need a fallback.

## Recommendation

Test the single-pipeline answer first: one headless Chrome pass renders both
Mermaid and HTML to PNG. Two pipelines mean two sets of failure modes.
