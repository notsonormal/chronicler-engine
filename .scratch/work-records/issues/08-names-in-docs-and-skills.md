# Decide where the names are written and how skills find the folder

Type: grilling
Status: open
Blocked by: 04

## Question

Which doc defines the names of the work records (decided in ticket 04: `work/`, initiative, `general`, issue, `wiki/`, `wayfinder-maps/`)? Do the skills and `AGENTS.md` refer to the folder through the tracker doc, or by its path? What happens when skills are synced from upstream again, and they name `.scratch/`?

## Context

- **Research (tickets 01 to 03).** Read the `## Answer` of [01](01-claude-projects.md), [02](02-chief-of-staff-wiki.md) and [03](03-other-layouts.md) first. Each asset ends with "Facts for tickets 04 to 08". Zoom into the asset only for the detail.
- **Where the names go (working view, owner, 2026-11-04).** In `docs/agents/`, not in `CONTEXT.md`, because `CONTEXT.md` is the glossary of the game domain.
- **Who names `.scratch` today.** 11 live files, six of them skills ([the charting facts](../assets/charting-facts.md), "The name `.scratch`"). If only the tracker doc named the path, a later rename would change one file.
- **A path resolver (from tickets 02 and 03).** Three tools keep the folder path out of their instructions: OpenRig (`rig config get topology.root`), Spec Kit (a script prints `FEATURE_DIR`) and Backlog.md (`backlog instructions`). Upstream PR #1208 (merged 2026-10-07) changed `code-review` to resolve the tracker "through the provided tracker doc instead of a hard-coded path". The path it removed was `docs/agents/issue-tracker.md`, not `.scratch/`. So upstream removes hard-coded paths from skills, but `to-tickets` still names `.scratch/<feature-slug>/issues/` at `4ae2317`.
- **Upstream.** Pocock's `code-review` and `to-tickets` skills name `.scratch/` directly, and `setup-matt-pocock-skills` offers `.scratch/` as the local tracker.
- **The gate.** `scripts/validate_docs.py` (`check_source_paths`) checks only backticked `src/`, `tests/`, `scripts/` and `assets/` paths. So a stale `.scratch/` path, or a stale path to its replacement, fails no check today, in a doc or in a skill. The link check of the build (map Notes, "The outside check") covers only links inside the folder.
- **Two status sets (from ticket 04).** A ticket and a one-off issue are both issues. A ticket uses `Status:` `open`, `claimed` and `resolved`. A one-off issue uses the triage statuses of `docs/agents/triage-labels.md`. The tracker doc must say which set applies where.
