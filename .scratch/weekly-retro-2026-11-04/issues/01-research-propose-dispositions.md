# Propose a disposition for each candidate

Type: research (AFK)
Status: resolved
Blocked by: —

## Question

For each candidate of [the weekly retro report](../assets/README.md), what disposition does the evidence support at the current `HEAD`: fix in this map, route to a named map or ticket, or drop? The answer gives the three grilling tickets their agenda, so the owner decides proposals and does not start from 24 raw rows.

## Context

- **Inputs.**
  - [The report](../assets/README.md): candidates R1 to R24, each with a severity, a category, a fix class and evidence.
  - The findings files in `../assets/findings/`. They hold the full evidence of each candidate.
  - [The outside check](../assets/findings/outside-check.md): 12 of 13 claims hold. C12 partly holds.
  - The active maps: each `.scratch/*/map.md` outside `.scratch/_closed/`, and their open tickets.
  - The [retro issue bucket](../../retro-issue-bucket/map.md): its **Won't do** section and its open tickets. The report says that some candidates add evidence to bucket tickets 23, 87, 88 and 99, and that R17 and R18 meet Won't do lines 31 and 09.
  - [Unclebob quality tools](../../unclebob-quality-tools/map.md): its open grilling tickets on the value and the shared-tree safety of mutation testing own the decision on a mutation run (map, Notes, "Failure modes"). Propose R3 as evidence for them.
  - The [bucket audit](../../retro-issue-bucket/audit.md): its five classes of an environment fix. Give each environment row its class. Propose a vigilance-rule fix only with evidence that such a rule held before.
- **Two more items.**
  - The report's "Bitter lesson" section, environment point 5, has no R label: the lesson of the hex-comparison conclusion is not in `.agents/skills/wayfinder/SKILL.md`. Give it a row as **B5**.
  - The findings files hold patterns that the report did not promote, for example patterns 4 to 9 and 12 of [the owner-friction findings](../assets/findings/sessions-owner.md). List them in one section, each with a one-line proposal: promote to a candidate, or leave.
- **The owner already decided.** The trial with a stronger model is out of scope (map, **Out of scope**). Do not propose it again.
- **Read only.** Do not change code, maps or tickets. Do not run `build.py` or cargo, because the build slot is shared. You may read the JUnit report that an earlier gate left under `target/nextest/`.

## Steps

1. Run `git diff --name-only 20d6d23c` once. For each row, re-read a cited `file:line` only when that list has its file. Record **unchanged since `20d6d23c`**, **holds**, **changed** (say how) or **gone**, and the commit you read (map, Notes, "Evidence is re-checked, not trusted").
2. Find where the candidate belongs. Search the open tickets of the active maps and of the bucket. If a candidate matches a Won't do line, name the line and say whether the evidence is new.
3. Estimate the size of the fix: one session or less, or larger. Name the files that it changes.
4. Propose one disposition with one reason:
   - **fix here**
   - **route**: name the map, and whether it is a new ticket there or evidence for an open ticket
   - **drop**
5. Make the cheap measures that a proposal needs:
   - R1: for each `tmp/` folder that an open map cites, the date of its oldest file, whether the citation is a repo path or a `/tmp/` path, and the open tickets that need it.
   - R17: the 10 slowest browser checks, from the newest JUnit report.
   - R3: the state of the mutation-testing tickets of Unclebob quality tools.
6. Mark each row **needs the owner** only when the disposition is a real choice. Mark a row with only one sensible route **confirm in a batch**.
7. Sort the rows by the grilling ticket that confirms them:
   - [Decide how an effort closes and what review owns](03-grilling-effort-close-and-review.md): R2, R3, R14, R15, R22, B5
   - [Decide the changes to the build tooling, the trial harness and the target dirs](04-grilling-tooling-trial-harness-target-dirs.md): R1, R6, R11, R17 to R21, R23
   - [Confirm where each product fault goes](05-grilling-product-fault-routes.md): R4, R5, R7 to R10, R12, R13, R16, R24

   Move a row to another grilling if it fits better, and say why.

## Output

One asset: `../assets/dispositions.md`.

1. One table for each grilling ticket. One row for each candidate, with these columns: label and name, evidence status and commit, owner (map and ticket), fix size and files, fix class, proposal, reason, needs the owner or confirm in a batch, and **Decision** (left empty: the grilling fills it).
2. For each grilling ticket: the questions it asks, at most 3 for each round, each with its context and one recommendation.
3. The section of patterns that the report did not promote.

## Done when

- Each of the 25 rows (R1 to R24 and B5) has every column filled.
- Each cited `file:line` in a file that changed since `20d6d23c` was re-read in this session.
- No owner cell says "unknown" without the search that found nothing.

## Answer

All 25 rows have a proposal: 14 fix here, 8 route, 3 drop. 9 rows need the owner (R1, R2, R5, R6, R9, R10, R14, R17, R22) and 16 are for a batch confirm. The three grilling tickets have 3, 3 and 2 questions, and each question has one recommendation. No row moved to another grilling. R17 became a real choice: new journal data may reopen Won't do 31. The step 5 measures are in the asset: the open tickets for each `tmp/` folder, the 10 slowest browser checks, and the R3 ticket state. The long-term-memory map is the route for R7, R12, R13 and R16, if the owner agrees. R6 depends on ticket 02.

See [the dispositions](../assets/dispositions.md).
