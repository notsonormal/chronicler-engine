# Map: Pi Explain Panel

## Destination

A working **pi-explain** v1, installed and in use. `/explain` returns an
explanation as Markdown, a Mermaid diagram, or a self-contained HTML page; the
artifact lands in the project's `.explain/`; and it appears in a side panel in
pi's TUI. The map is done when the package is installed, accepted against a
checklist, and no decision remains open.

This is a **build-in-the-map** effort. The charting answer to Q1 was Option B, so
tickets resolve decisions *and* carry the execution that reaches the
destination. This overrides wayfinder's plan-don't-do default.

## Notes

- Tracker: local markdown (`.scratch/pi-explain/`). See
  `docs/agents/issue-tracker.md` and `docs/agents/triage-labels.md`.
- Domain: this extension is for **pi**, the coding-agent harness. It is not part
  of Chronicler Engine. Nothing under `src/` is involved. It lives in this
  repository because the tracker and the working checkout are here.
- Skills every session should consult: `/grilling`, `/domain-modeling`.
- **Settled during charting.** These are not ticket resolutions. Treat them as
  standing constraints.
  - Name: **pi-explain**, map title **Pi Explain Panel**.
  - Independent package. No runtime dependency on `visual-explainer`. Borrow its
    renderer approach as vendored assets if that helps.
  - The markdown viewer is our own panel tab inside `pi-explain`, not
    `pandi-mdview`.
  - v1 formats: Markdown, Mermaid, self-contained HTML. Video is out of scope.
  - Artifacts are project-local under `.explain/`. The extension never writes
    outside the project root.
  - Panel: depend on `@luan.sh/pi-panels` (MIT) and load its extension from
    `node_modules` through our own `pi` manifest. Do not vendor it.
  - Isolation invariant: a few seconds of extra startup is acceptable, provided
    the artifact body never reaches the main transcript.
- Environment facts, verified 2026-10-09:
  - `google-chrome` at `/usr/bin/google-chrome`. node 24.21.0. npm 11.19.0.
  - **Absent:** `bun`, `mmdc`, `ffmpeg`.
  - pi 1.0.0 at `/home/node/.npm-global/bin/pi`.
  - On npm: `@luan.sh/pi-panels` 0.3.5, `@luan.sh/pi-libtui` 0.3.15,
    `@luan.sh/pi-libactions` 0.3.4.
- pi primitives an extension can use:
  - `registerSidePanelProvider` from `@luan.sh/pi-libtui`, keyed on
    `Symbol.for("pi-panels/registry/v1")`. Contributors never import
    `pi-panels` itself.
  - The `Image` component from `@earendil-works/pi-tui`. Kitty and iTerm2
    graphics, text fallback. In fullscreen mode iTerm2 degrades to text.
  - `ctx.ui.setWidget()` for a band near the editor. `ctx.ui.custom()` for an
    overlay. `pi.appendEntry()` for session state that stays out of context.
- Cost warning. Inlining pi-panels is about **24,700 vendored lines**: 766 for
  pi-panels, 23,681 for `pi-libtui` (`host/` alone is 4,340), 229 for
  `pi-libactions`. Charting rejected this.
- pi packages may depend on other pi packages, and the manifest may point at the
  dependency's `node_modules` path (`docs/packages.md`). The user does not
  install pi-panels as a separate pi package. But pi does not install the
  dependencies of a local package, so each checkout needs one `npm install` in
  `.pi/extensions/pi-explain/`. See ticket 01.

## Decisions so far

<!-- One line per closed ticket. Empty until the first ticket resolves. -->

- [01 — Where pi-explain lives in this repository](issues/01-where-pi-explain-lives.md)
  — `.pi/extensions/pi-explain/`, listed under `packages` in
  `.pi/settings.json`. Entry at `src/index.ts`, no root `index.ts`. Manual
  `npm install` per checkout.

## Not yet specified

- The implementation tickets themselves: the format and mode layer, the panel
  tab, the local renderer, the isolation runner, the install wiring. Their shape
  depends on tickets 03, 04, and 05. Expect them to graduate as those close.
- Whether the picker band is a `ctx.ui.setWidget()` widget or lives in the panel
  header.
- Whether `/explain` mode state persists per session or per project.
- How the extension reports failure to the user: no Chrome, no image protocol,
  renderer error, subagent failure.

## Out of scope

- Animated video with canvas and text-to-speech. Needs ffmpeg, a TTS path, and a
  canvas driver. Ruled out during charting.
- Publishing to npm. v1 is a local package.
- Forking or patching `visual-explainer`. We borrow its approach, not its code
  path.
