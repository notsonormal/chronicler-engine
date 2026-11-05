# Decide the changes to the build tooling, the trial harness and the target dirs

Type: grilling (HITL)
Status: open
Blocked by: 01, 02

## Question

Which changes to the build tooling, the trial harness and the location of the target dirs does the owner adopt? Which hand-kept copies become a check or a generated value?

## Context

- **Candidates** of [the report](../assets/README.md):
  - trial evidence is in a folder that the gate deletes (R1)
  - the target dirs are inside the checkout, so the permission rules deny safe commands (R6)
  - `trial.py` is the lasting evaluation harness, but it has the shape of a script (R11)
  - the browser tier grew to about 210 s, and the cost tables show old values (R17)
  - the generated structure index fills 321 of the 422 lines of `AGENTS.md` (R18)
  - the build journal keeps about one week (R19)
  - the dead DB dump and the stale-data trap in the `build.py` tail (R20)
  - tests that copy a production table row for row (R21)
  - maps and plans are read again and again (R23)
- **The `tmp/` cleanup.** Cited evidence starts to go with the first green gate on or after 2026-11-05 (map, Notes). A protective copy of each cited folder is in `../assets/local-only/tmp-evidence/`, and [the manifest](../assets/tmp-evidence.md) lists them. R1 here decides how evidence is kept for the long term:
  - the final home of each copied folder, and the change of the citations in the maps that name it
  - where new evidence goes. A candidate (owner, 2026-11-04): each map keeps the evidence it must not lose in its own `assets/local-only/` (`docs/agents/issue-tracker.md`), and `scripts/trial.py` writes its packs there
  - when the protective copy can go
- **The agenda** is the section for this ticket in `../assets/dispositions.md`, from [Propose a disposition for each candidate](01-research-propose-dispositions.md). The facts for R6 are in `../assets/target-dir-dependencies.md`, from [Find what depends on the target dirs inside the checkout](02-research-target-dir-dependencies.md). Ask at most 3 questions in each round (map, Notes).
- **Corrections to R6 from ticket 02 (checked by the coordinator, 2026-11-04).**
  - The 5 rules about `target/` denied 80 calls, not 108. The 108 include `timeout *` (21) and `*.env` (7).
  - The move makes only `grep * . *` and `grep * ./ *` unnecessary. `grep * /workspace/chronicler-engine *`, its `/` twin and `find / *` stay, because a root walk still reads `tmp/` (2.7 GB) and `find /` would read the new location.
  - `/workspace` and `/home/node` are two bind mounts of host folders on one device. A move between them is a copy, not a rename.
  - Ticket 02 asks whether `/home/node` survives a container rebuild. It is a bind mount of the host folder `/home/minipc/app-data/ai-dev/home` (known, `/proc/self/mountinfo`). So it probably survives (inferred). `ENVIRONMENT.md`, under Coverage, says that a rebuilt container drops `~/.cargo/bin`. One of the two is wrong.
  - If the move is adopted, this ticket decides its size: a task ticket here, or a route. The move changes 12 files and adds one module. The owner edits the permission config after the move, when `du -sh target` shows less than 1 MB.
- **Owner decisions that apply.** Retro bucket, Won't do: 09 (keep the structure index in `AGENTS.md`) and 31 (no skip of the browser tier). A proposal that reopens one of them must give its new evidence. Only the owner edits the permission config.

## Answer

Record each decision. For each adopted fix of one session or less, create a task ticket in this map. Route each larger fix, and record where it went.
