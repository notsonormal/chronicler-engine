# Run the full build gate once per change set

Type: grilling (HITL)
Status: resolved
Blocked by: —

## Question

The session for [Split Settings into Connections and Text Check sub-tabs](../dashboard-ui-review/issues/68-split-settings-sub-tabs.md) ran `python build.py` — the full 19-step gate, browser suite included — five times, three of them after a comment removal or a docs-only tweak. The session for [Remove the story-log ✓ and `POST /check-text`](../dashboard-ui-review/issues/70-remove-story-log-check.md) ran it twice, the second time only to re-verify a `ui_design.md` edit.

Two pieces of steering pull toward the repeat. `AGENTS.md`'s build-loop text is negative-framed and never says "once". The map's per-ticket rule — "End of each execution ticket: `python build.py` is green" — reads as "re-run it after every edit".

What wording, and where, makes the full gate the last step?

- **Rewrite the `AGENTS.md` build-loop guidance positively.** "Run the full gate once, as the last step. Verify each focused change with `python build.py check` or `test-pattern`."
- **Ease the map's end-of-ticket criterion.** A docs-only edit needs `python build.py validate-docs`, not the gate.
- **Both.**

## Context

- `AGENTS.md` → **Development Loop** → **Final Validation**.
- `.scratch/dashboard-ui-review/map.md` → Notes → "End of each execution ticket".
- The t68 session ran the full gate at entry idx 313, 341, 353, 379, 403; wall time 42 min.
- The t70 session re-ran the gate at entry idx 204 after the `ui_design.md` edit, reasoning: "to report an honest final result, the map says `python build.py` is green".

## Answer

Decided 2026-10-08 with the user. Implemented in [30 — Add the `--no-browser` gate tier and a checkable gate verdict](30-gate-no-browser-and-verdict-record.md).

- **A cheaper tier, not only wording.** `python build.py --no-browser` runs the full gate minus the browser tier. Measured warm: 34 s of step time, against about 170 s for the full gate. Bare `python build.py` stays the full gate. Inverting the default (bare = fast) was rejected: it drops browser coverage silently and changes the meaning of every earlier "`python build.py` green" claim.
- **The ladder in `AGENTS.md`.** Heading "Final Validation (once, at the end)". "Green means the full gate ran on the tree you report. While you iterate, run one step (`clippy`, `test-pattern <pattern>`) or `python build.py --no-browser`." The old sentence that named bare `build.py` as the efficient move is gone.
- **The integration step names its content.** "(~20s, unit tests included)". The integration step already runs the 1220 lib unit tests, so a separate unit tier buys nothing.
- **Evidence.** `logs/build_history.txt`: 114 full-gate runs since 2026-09-25, 317.5 min, 63% of all build wall time. Session `01a11ce3` ran four complete gates in 25 min on 2026-10-08.
- **Triage basis does not hold.** `triage.md` row 03 called the premise "partly stale" because `AGENTS.md:388` already said "run once before considering done". The sentence existed and did not bind: the journal data above postdates it.
- **Not changed.** The map's per-ticket rule in `.scratch/dashboard-ui-review/map.md` is that effort's text; with "green" now defined in `AGENTS.md`, it reads as one full gate per ticket.
