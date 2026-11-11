# 08 — Verification strategy for a TUI extension

Type: research
Status: open
Blocked by: (none)
Assignee: (unclaimed)

## Question

How do we know the extension works? What is testable, and what must a human
check by hand?

The destination is "installed and in use", so verification is part of the
deliverable. But a TUI extension resists ordinary testing: the panel, the
overlay, and the image protocol all need a real terminal.

## What to find out

1. Which parts are pure and unit-testable without a terminal. Candidates: spec
   to HTML, Mermaid wrapper generation, artifact path resolution, mode state
   parsing, the `before_agent_start` prompt section.
2. How pi's own extensions are tested. `@luan.sh/pi-libtui` and `pi-panels` ship
   `test/` directories that run under `bun test`. `bun` is **not** installed
   here. Find out what that costs.
3. Whether `setCapabilities()` and `setCapabilityOverrides()` from
   `@earendil-works/pi-tui` let a test force the image protocol, so the `Image`
   path is testable without a graphics-capable terminal.
4. What a manual smoke checklist must cover: panel appears, tab switches, image
   renders, no-host fallback, non-TUI mode, `.explain/` artifact written.
5. Whether `PI_TUI_WRITE_LOG` gives usable evidence for rendering problems.

## Deliverable

A markdown summary linked as an asset, plus a proposed test layout and a manual
smoke checklist.

## Background

- pi extensions run inside the pi process, loaded with `jiti`. TypeScript needs
  no separate compile step for a local extension.
- The repository this package lives in has its own gate, which is Rust. The
  extension's tests must not depend on it. Ticket 10 covers the interaction.
- `docs/tui.md` warns to test narrow widths, wide characters, resize, theme
  changes, and both regular and fullscreen modes.

## Recommendation

Split verification in two: unit tests for the pure parts, and a written manual
checklist for the panel. Do not attempt an automated TUI test in v1.
