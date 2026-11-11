# Decide the link check, the broken links of today and the old path

Type: grilling
Status: resolved
Blocked by: 08

## Question

Does the link check become part of the gate (for example in `scripts/validate_docs.py`), or does it stay a step of the move script? Does the gate also check backticked `work/` paths in docs and skills? Does the move fix the broken relative links of today, or leave them? Does `.scratch` keep working for a while after the move (for example as a symlink), or does it go at once?

## Context

- **The outside check (map Notes).** A link check and the full `python build.py` after each move. Broken links stay silent, so the link check is required.
- **Citations in skills (ticket 08).** [Decide where the names are written and how skills find the folder](08-names-in-docs-and-skills.md) kept paths to specific records in skills and docs, as citations, and the move rewrites them. Today `scripts/validate_docs.py` (`check_source_paths`) checks only backticked `src/`, `tests/`, `scripts/` and `assets/` paths. So a stale `work/` citation fails no check.
- **The broken links of today.** More than 100 relative links in live maps do not resolve ([the charting facts](../assets/charting-facts.md), "Links").
- **The old path.** Ticket 08 puts a line in the tracker doc: "Upstream skills and older records say `.scratch/`. In this repo, that means `work/`." Backlog.md wrote the old folder name into the config of old installs (`back-105`, from ticket 03). Old sessions and `old-docs/` name `.scratch/` and stay as history.

## Answer

Grilled with the owner on 2026-11-11. The owner chose the recommended option for Q1 to Q4, and rejected the check in Q5.

1. **The link check is part of the gate.** A new script, `scripts/validate_work_records.py`, runs in the doc step of `build.py` and in `build.py validate-docs`. The move script calls it too. It is separate from `validate_docs.py`, which holds the diataxis rules. The computed status view (fog item "The review after use") could later live in this script.
2. **What the gate checks, as errors:**
   - (a) Relative markdown links in the live records under `work/`: `workstream.md`, `map.md`, tickets, one-off issues, `wiki/` and `plans/`. It skips HTML comments and fenced code, so the `(link)` placeholder of the wayfinder template passes. It skips `assets/` and `_closed/`.
   - (b) Backticked `work/` paths in `AGENTS.md`, `docs/` (including `docs/agents/`) and `.agents/skills/**`. Only `work/` tokens are checked in skills, so upstream skills that say `.scratch/` do not fail.
3. **Broken links of today.** The move fixes the 17 in live maps and tickets: the `tmp/` links become backticked text (`tmp/` is git-ignored), and `fable-audit` ticket 02's `../invariant-audit/map.md` points at the closed map. The 120 in live assets and the 95 in `_closed/` stay as history, outside the gate check. The move script also runs a strict check over every file, assets and `_closed/` included: no link that resolved before the move may break after it.
4. **`.scratch` goes at once.** No symlink: the layout changes the shape of each path (`.scratch/<map>/` becomes `work/workstreams/<ws>/wayfinder-maps/<map>/`), so a symlink would not make old paths resolve. The move script writes an old-to-new path table to `work/workstreams/agent-workflow/wayfinder-maps/work-records/assets/path-moves.md`. The tracker doc's old-path line (ticket 08, point 4) links to it.

Rejected:

- **The check only in the move script.** It goes silent after the move.
- **Fix all 232 broken links**, or none.
- **A gate error when a `.scratch/` folder exists** (owner: "a pointless validation check").

Facts for later tickets:

- **Counts on 2026-11-11.** 7,379 relative links in `.scratch/` resolve. Broken: 17 in live maps and tickets (2 placeholders in HTML comments, 14 into `tmp/`, 1 stale), 120 in live assets (94 in `weekly-retro-2026-11-04`), 95 in `_closed/`. The scan script is `tmp/linkscan.py` (git-ignored, may be gone).
- **Not covered by the gate.** The comment in `.agents/skills/verify-gate/mise.toml` line 2 names `work/wayfinder-maps/statechart-models/` without backticks. The move script rewrites it (ticket 08), but no check guards it later.
- **`validate_docs.py` today** reads only `docs/diataxis/` and its `ROOT_DOCS` list. It does not read `docs/agents/` or `.agents/skills/`.

Changed later (2026-11-11): [Decide what a workstream is for, or drop it](11-workstream-purpose.md) dropped the workstream tier. The path table is `work/wayfinder-maps/work-records/assets/path-moves.md`, and `.scratch/<map>/` becomes `work/wayfinder-maps/<map>/`. The live records of check (a) are `map.md`, tickets, one-off issues, `wiki/` and `plans/`. The other points hold.
