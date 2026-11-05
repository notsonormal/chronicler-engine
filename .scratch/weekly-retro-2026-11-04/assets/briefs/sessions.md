# Weekly retro 2026-11-04: session-mining brief

## What this is

A weekly retrospective of how coding agents worked on Chronicler Engine from 2026-10-28 to 2026-11-04: 255 pi sessions, about $503. The goal: find the friction that repeats, so the environment (skills, `AGENTS.md`, checks, scripts, extensions) can change. One-off model slips are not interesting. Patterns are.

## Data (already extracted, all under `tmp/retro-2026-11-04/data/`)

- `sessions_with_effort.json` — one record per session: `file`, `name` (delegate sessions are `delegate · <role> · <label>`), `skill`, `ask` (the user's request), `n_user`, `tools` (call counts), `errors` (classes), `cost`, `build_runs`, `effort`.
- `followup_user_turns.json` — every user message in a lead session after the first one (399). These are where the owner corrects, redirects or answers the agent.
- `correction_candidates.json` — the subset of follow-ups with correction words (111). A heuristic; it has false positives.
- `tool_errors.jsonl` — every tool call that returned an error, with a class.
- `delegate_final_reports.md` — for each delegate session: the first 1200 characters of its brief and its final report.
- `analyse_out.txt` — aggregate counts (cost by effort and role, permission denials by rule, biggest tool results).
- The raw session files are at `~/.pi/agent/sessions/--workspace-chronicler-engine--/<file>`. Read one only to confirm a pattern. They are large; use `rg` or a short Python filter, not a full read.

The existing retro tracker is `.scratch/retro-issue-bucket/map.md`. Read its **Won't do** and **Open tickets** sections first. A pattern that matches a Won't do line is reported only with new evidence, and you name the line. A pattern that matches an open ticket is reported as "adds evidence to ticket NN".

## Rules

- Read only. Write only under `tmp/retro-2026-11-04/findings/`. Do not run `build.py` or cargo.
- Every pattern carries at least 2 instances: session file prefix (for example `2026-11-01T13-51`) and a short verbatim quote.
- Label epistemic status: known (you read it), inferred, or guessed.
