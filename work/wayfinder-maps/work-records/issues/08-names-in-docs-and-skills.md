# Decide where the names are written and how skills find the folder

Type: grilling
Status: resolved
Blocked by: 04

## Question

Which doc defines the names of the work records (decided in tickets 04 and 06: `work/`, workstream, `general`, issue, `wiki/`, `wayfinder-maps/`, `plans/`)? Do the skills and `AGENTS.md` refer to the folder through the tracker doc, or by its path? What happens when skills are synced from upstream again, and they name `.scratch/`?

## Context

- **Research (tickets 01 to 03).** Read the `## Answer` of [01](01-claude-projects.md), [02](02-chief-of-staff-wiki.md) and [03](03-other-layouts.md) first. Each asset ends with "Facts for tickets 04 to 08". Zoom into the asset only for the detail.
- **Where the names go (working view, owner, 2026-11-04).** In `docs/agents/`, not in `CONTEXT.md`, because `CONTEXT.md` is the glossary of the game domain.
- **Who names `.scratch` today.** 11 live files, six of them skills ([the charting facts](../assets/charting-facts.md), "The name `.scratch`"). If only the tracker doc named the path, a later rename would change one file.
- **A path resolver (from tickets 02 and 03).** Three tools keep the folder path out of their instructions: OpenRig (`rig config get topology.root`), Spec Kit (a script prints `FEATURE_DIR`) and Backlog.md (`backlog instructions`). Upstream PR #1208 (merged 2026-10-07) changed `code-review` to resolve the tracker "through the provided tracker doc instead of a hard-coded path". The path it removed was `docs/agents/issue-tracker.md`, not `.scratch/`. So upstream removes hard-coded paths from skills, but `to-tickets` still names `.scratch/<feature-slug>/issues/` at `4ae2317`.
- **Upstream.** Pocock's `code-review` and `to-tickets` skills name `.scratch/` directly, and `setup-matt-pocock-skills` offers `.scratch/` as the local tracker.
- **The gate.** `scripts/validate_docs.py` (`check_source_paths`) checks only backticked `src/`, `tests/`, `scripts/` and `assets/` paths. So a stale `.scratch/` path, or a stale path to its replacement, fails no check today, in a doc or in a skill. The link check of the build (map Notes, "The outside check") covers only links inside the folder.
- **Two status sets (from ticket 04).** A ticket and a one-off issue are both issues. A ticket uses `Status:` `open`, `claimed` and `resolved`. A one-off issue uses the triage statuses of `docs/agents/triage-labels.md`. The tracker doc must say which set applies where.
- **Layout (ticket 06, 2026-11-11).** [Decide the layout of the folder](06-layout.md) decided the path rule `work/workstreams/<name>/…`, a `workstream.md` in each workstream, and a `Status:` line (`open` or `closed`) in each `map.md`. The tracker doc must define them. Plans land in `work/plans/`.
- **Placement (ticket 07, 2026-11-11).** [Decide which maps go where](07-maps-to-expeditions.md) made six workstreams: `long-play`, `dashboard`, `code-health`, `state-machines`, `agent-workflow` and `general`. `retro-issue-bucket` stays a map, at `work/workstreams/agent-workflow/wayfinder-maps/retro-issue-bucket/`. One skill names that bucket by its path: step 6 of `.agents/skills/simple-after-plan/SKILL.md` ("Add new issues to `work/wayfinder-maps/retro-issue-bucket/`"). `docs/agents/triage-labels.md` says that its statuses are "used in `.scratch/` ticket files".

## Answer

Grilled with the owner on 2026-11-11. The owner chose the recommended option for all five questions.

1. **The tracker doc defines the names.** `docs/agents/issue-tracker.md` gets a short "Names" section and a layout tree. It defines `work/`, workstream, `general`, map, ticket, one-off issue, `wiki/`, `plans/`, `_closed/` and the path rule `work/workstreams/<name>/…`. The doc keeps its file name, because upstream skills name it by its path. No new doc is added.
2. **`AGENTS.md` names `work/` and the tracker doc.** The "Issue tracker" section loads in every session, so it gives the folder name in one line. A later rename changes two files: the tracker doc and `AGENTS.md`.
3. **A skill that needs the folder rule points at the tracker doc and does not name `work/`.** This covers `chronicler-after-plan-workflow`, `chronicler-after-plan-workflow-plus-review` and `document-review`. A file that cites **one specific record** keeps its path, as a citation, and the move script rewrites that path. This covers step 6 of `simple-after-plan` (the retro bucket), `verify-gate/SKILL.md` and `verify-gate/mise.toml` (the `statechart-models` trial log and ticket), and `chronicler-ui-investigator/ENVIRONMENT.md` (a closed review).
4. **Upstream skills are not edited.** The tracker doc says: "Upstream skills and older records say `.scratch/`. In this repo, that means `work/`." On 2026-11-11, step 2 of `code-review` is the only synced skill that names `.scratch/`. Its search misses the folder and falls back to asking the user, so nothing breaks. An upstream PR is not planned.
5. **One status table in the tracker doc**, with one row per record kind:

   | Record | Path | `Status:` values |
   |---|---|---|
   | Workstream | `work/workstreams/<name>/workstream.md` | `open`, `closed` |
   | Map | `…/wayfinder-maps/<map>/map.md` | `open`, `closed` |
   | Ticket | `…/wayfinder-maps/<map>/issues/NN-<slug>.md` | `open`, `claimed`, `resolved` |
   | One-off issue | `…/<workstream>/issues/NN-<slug>.md` | the triage values of `triage-labels.md` |

   `triage-labels.md` says that its values apply to one-off issues, and drops the word `.scratch/`.

Rejected:

- **A new `docs/agents/work-records.md`.** Need 6 asks for one doc.
- **`AGENTS.md` names only the tracker doc.** Every agent would need to open a second file to learn the folder name.
- **No paths in skills at all**, with specific records listed in the tracker doc. The tracker doc would grow a list of citations that goes stale.
- **A local patch to upstream skills after each sync.**

Facts for later tickets:

- **The build.** The files to edit for the folder rule are `AGENTS.md` (line 386), `docs/AGENTS.md` (line 170, "Plans (`docs/plans` and `.scratch`) are exempt"), `docs/agents/issue-tracker.md`, `docs/agents/triage-labels.md` and the three skills of point 3. The config and generated files are `.pi/plan-mode.json`, the plan-mode test value, `arch-lint.toml` line 18 (`**/.scratch/**`, which is the source of the scan scope in the generated `guardrails.md`) and `.gitignore` line 215. `guardrails.md` line 137 also cites `work/_closed/turn-state-model/…`, so its source needs a rewrite too.
- **The link check.** Point 3 is safe only if a stale citation fails a check. `scripts/validate_docs.py` (`check_source_paths`) checks only backticked `src/`, `tests/`, `scripts/` and `assets/` paths. The fog item "The link check" decides whether the gate also checks `work/` paths in docs and skills.
- **The old path.** The mapping line of point 4 also answers part of the fog item "The old path": the old name is explained in the tracker doc, even if `.scratch` itself does not keep working.

Changed later (2026-11-11): [Decide what a workstream is for, or drop it](11-workstream-purpose.md) dropped the workstream tier. The tracker doc defines no workstream and no `general`. Its status table loses the workstream row, and a one-off issue's status is its folder (`pending/`, `ready/`, `in-progress/`, `closed/`), not a `Status:` line. `triage-labels.md` maps each role to a folder. The path of the retro bucket is `work/wayfinder-maps/retro-issue-bucket/`. The other points hold.
