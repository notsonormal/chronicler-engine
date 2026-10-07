# Run the full build gate once per change set

Type: grilling (HITL)
Status: open
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
