# Give the Standards review the clone report

Type: grilling (HITL)
Status: open
Blocked by: —

## Question

The repo ships `jscpd`-based clone detection as the `duplicates` gate step (`python scripts/healthcheck.py duplicates`), report at `report/jscpd-report.json`, subcommand `python build.py duplicates`. It is report-only and currently reports 2446 clones. No review skill names it, so Standards reviews hand-find *Duplicated Code*: "The heal+persist pair is copy-pasted into three new closures"; "Duplicated form→config mapping — *Duplicated Code / Data Clumps*". The parent re-pasted the whole Fowler smell baseline into each §4 brief.

Should the report be a review input?

- **Name it in §3/§4.** The Standards brief reads `report/jscpd-report.json` before applying the smell list.
- **Put it in the bundle.** `prepare_review_bundle.py` copies the jscpd top-pairs summary into the bundle, or its README points at the report.
- **Both.** Name the source and carry a summary so the reviewer never has to find it.

## Context

- Source: `/reflect` 2026-10-07, sessions `01a1127f`, `01a117d3`.
- Evidence: `assets/reflect-2026-10-07/synthesis.md` (Accepted row 11; Backlog "Duplicate report into the review bundle").
- `build.py` gate step `duplicates`; report `report/jscpd-report.json`.
- `.agents/skills/code-review/SKILL.md:48` — the *Duplicated Code* smell line, with no tool pointer.
- Related: [Never tell a review axis to skip `tooling.patch`](21-review-bundle-tooling-patch.md).
