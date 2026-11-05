# Map: Weekly retro 2026-11-04

Labels: wayfinder:map

## Destination

Every candidate of [the weekly retro report](assets/README.md) (R1 to R24) has a disposition. It is fixed in this map, filed in the map that owns its code (or in the [retro issue bucket](../retro-issue-bucket/map.md)), or dropped with a written reason.

## Notes

- **Origin.** A `/retro` of the sessions and the code of 2026-10-28 to 2026-11-04 (owner, 2026-11-04). The owner asked for a holistic audit of the code, not only of the sessions, because per-ticket review sees one diff at a time. The report is [`assets/README.md`](assets/README.md). Its evidence is in `assets/findings/`. [The outside check](assets/findings/outside-check.md) found that 12 of 13 load-bearing claims hold and 1 partly holds.
- **Domain.** Two kinds of candidate:
  - The environment: skills, `AGENTS.md`, `CODING_STANDARDS.md`, checks, `build.py`, `scripts/`, the permission config, and the `.scratch` maps.
  - The product code: faults that the audit found in `src/` and `tests/`. A product fault goes to the map that owns its code. For example, R7 and R16 are in code that [Long-term memory and state](../long-term-memory-and-state/map.md) changes next.
- **Execution is in this map (owner, charting).** Wayfinder's plan-only default is overridden. A grilling that adopts a fix of one session or less creates a task ticket in this map, and that ticket makes the fix. A larger fix goes to the map that owns it, as a new ticket or as evidence for an open ticket.
- **AFK first (owner, charting).** AFK tickets prepare a proposal with one recommendation for each candidate. The owner answers only the decisions. A grilling round has at most 3 questions. Each question restates its context and gives one recommendation. This overrides the limit of 5 in `/grilling`. The reason (owner, 2026-11-01): "you just asked me a ton of questions, so I'm not really confident in my answeres".
- **Evidence is re-checked, not trusted.** The tree moves. The report read `HEAD` `20d6d23c` (2026-11-03). A ticket re-reads a cited `file:line` only when `git diff --name-only 20d6d23c` lists its file (the command includes the working tree). For an unchanged file, the evidence of the report stands, because the audit read it at that commit and [the outside check](assets/findings/outside-check.md) verified the 13 load-bearing claims there.
- **The `tmp/` cleanup (known, plan review 2026-11-04).** `build.py` `run_gate` runs `_gate_tail` after each green full gate and each green `--no-browser` gate. `_gate_tail` deletes each file under `tmp/` that is older than 30 days, one file at a time. The open maps cite `tmp/` folders whose oldest files date from 2026-10-06 (the retro issue bucket: `tmp/review/`, `tmp/ui-review/`), 2026-10-09 (Unclebob quality tools: `tmp/unclebob/`, `tmp/dup_now/`), and later, to 2026-11-03 (Long-term memory and state: `tmp/ltm03/`, `tmp/ltm09/`). So the first green gate on or after 2026-11-05 starts to delete cited evidence. The cited folders hold about 614 MB. Some matches may be `/tmp/` paths, not repo paths (inferred: the search matched `tmp/<name>` in both). R1 decides how evidence is kept for the long term ([Decide the changes to the build tooling, the trial harness and the target dirs](issues/04-grilling-tooling-trial-harness-target-dirs.md)).
- **The protective copy (owner, plan review 2026-11-04).** Until R1 is decided, a copy of each cited folder is in `assets/local-only/tmp-evidence/` (33 folders, 617 MB, file dates kept). [The manifest](assets/tmp-evidence.md) lists each folder, the maps that cite it, and its oldest file. The maps keep their `tmp/` citations. After a delete, look for the file in the copy. `tmp/marinara/` is a clean public clone, so it was not copied. The manifest gives the clone command.
- **Done when.** `assets/dispositions.md` has a **Decision** column. Each grilling ticket fills the cells of its rows. The map is done when each of the 25 rows has a Decision, and each routed row names the ticket it became.
- **Failure modes.**
  - A ticket works from stale evidence. The diff rule above finds the changed files.
  - Two maps decide one thing. [Unclebob quality tools](../unclebob-quality-tools/map.md) has open grilling tickets on the value and the shared-tree safety of mutation testing. They own that decision, so this map routes R3 to them and does not decide it again.
  - A fault is routed into a map that another session edits at the same time. On 2026-11-04, the long-term-memory map had uncommitted edits of another session. Add a new ticket file, and edit the other map's `map.md` only in one small, separate edit.
- **The retro bucket stays.** This map does not replace the [retro issue bucket](../retro-issue-bucket/map.md). An environment candidate that is not fixed here becomes a bucket ticket in the bucket's format. Check its **Won't do** section first.
- **A fix ticket names its own checks in its body.** [Decide how an effort closes and what review owns](issues/03-grilling-effort-close-and-review.md) decides where the standing rules of a build ticket live (R14), so this map does not copy them.
- **Concurrency.** Other efforts edit this checkout at the same time. The owner decides the scope of each commit.
- **`assets/local-only/`** stays on this machine (git-ignored, owner 2026-11-04, `docs/agents/issue-tracker.md`). It holds the raw session extracts (`data/`, 3 MB, with the owner's messages) and the protective copy.
- **Reuse.** [Prepare the candidate list for the grilling](../_closed/hex-comparison/issues/09-task-prepare-the-candidate-list.md) of the hex-comparison map had the same job as [Propose a disposition for each candidate](issues/01-research-propose-dispositions.md). The [bucket audit](../retro-issue-bucket/audit.md) sorts an environment fix into five classes (mechanical, wrong prose, output request, vigilance rule, model slip), and says which classes work.
- **Skills.** `/grilling` and `/domain-modeling` for the grilling tickets. `/writing-for-agents` for an edit of a skill or of `AGENTS.md`.
- **Names.** R1 to R24 are labels of the report. In text for the owner, give the candidate's name with its label, for example "the byte-500 panic (R4)".

## Decisions so far

<!-- one line per closed ticket: [title](link) — gist -->

- [Propose a disposition for each candidate](issues/01-research-propose-dispositions.md) — all 25 rows have a proposal: 14 fix here, 8 route, 3 drop. 9 need the owner, 16 are a batch confirm. [The dispositions](assets/dispositions.md) hold the agenda and the first round of questions for each grilling.
- [Find what depends on the target dirs inside the checkout](issues/02-research-target-dir-dependencies.md) — a move to `~/.cache/chronicler-engine/target/<checkout key>/<name>` changes 12 files plus one new module, and `CARGO_TARGET_DIR` must always be set. It makes 2 of the 4 `grep` rules unnecessary. No earlier decision keeps the dirs in the checkout.

## Not yet specified

- **Make the weekly retro repeatable.** `assets/scripts/` extracts the session data, and `assets/briefs/` holds the briefs of the audit agents. Decide after the grillings whether this retro paid off, and so whether it becomes a skill step or a script.
- **The fix tickets.** Each one graduates from the answer of a grilling ticket.

## Out of scope

- **A trial with a stronger narrator model, or with reasoning on, before the long-term-memory writers are built** (report, "Bitter lesson", product point 1). Owner, 2026-11-04: the decision "No second model" of 2026-11-02 ([Layout trials](../layout-trials/map.md)) stands.
