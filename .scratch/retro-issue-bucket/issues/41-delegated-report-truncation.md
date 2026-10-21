# Deliver the tail of a long delegated report

Type: grilling (HITL)
Status: resolved
Blocked by: —

## Question

A delegated report is delivered to its owner with the tail cut off at 50 KB (the two in this run read `[Output truncated: 728/1337 lines, 49.5KB/102.1KB.]` and `[Output truncated: 528/696 lines, 50.0KB/58.3KB.]`, both with `"truncated": true` in the delivery details). What survives is the first N lines, so a report that puts its decision-relevant section last loses exactly the part the owner needs. Here the first reconnaissance lost its ticket-Answer section and the Lead had to dispatch a second agent for 92 s to re-print it — and that report lost its own tail too.

Where should a report that exceeds the cap go?

- **Write it to `tmp/` and return the path.** The delegate keeps the full text on disk, returns a short summary plus the path, and the owner reads the rest (a Lead holding `read` — see [Give the Lead the file tools it needs to check a quote](40-lead-file-tools.md)).
- **Raise the cap, or make it per-report.** The owner sets the byte budget on the dispatch, so a reconnaissance expected to return 1300 lines is not delivered at 50 KB.
- **Nothing — put the decided-on sections first.** A brief states the order the answer must arrive in and the line budget per section; truncation then costs only colour. Cheapest, and it does not fix the silent cut.

## Context

- Source: `/retro` 2026-10-11 on the phase2-after-plan pass (`/simple-after-plan`).
- Recon `8c9d65d4-bb9c-4ed7-8ef1-293b631c08a9` (session `01a12c57-3c9a-745e-b51b-45da647c02b4`): its final report is 104,075 characters / 1336 lines, delivered to Lead `11b62cb1` truncated at 728 lines — the ticket-Answer sections 3–5 were cut.
- The Lead's next turn: "Recon report arrived (truncated after §2, so I still need the ticket Answers)", which dispatched `recon-answers` (`01a12c59-a7b9-74e7-a424-d5c383570a31`, 92 s), itself delivered truncated at 528/696 lines.
- The truncation notice names the loss, so the failure is legible; what is missing is any recovery path that does not cost a whole agent.
- Related: issue 08 (`read` with `limit=0` returns nothing) is the neighbouring read-side gap.
- More evidence (`/retro` 2026-10-19, whole-rebuild review, lead `01a15614`): the thermo-nuclear reviewer's 20,090-character report arrived truncated before its closing section ("What would make this approvable … H"). The notice named `/home/node/.pi/agent/delegate/0ec8f36d/result.json`, and the lead read the tail from it with one `python` call. The recovery path exists now, but it is not a documented step.

## Answer

2026-10-21. **Dropped (stale).** Triage pass 2 (`../triage.md`), HEAD `511e28dd`; the owner approved the close.

- The cut now lives in repo code: `.pi/extensions/delegate/lib.ts:206-210` cuts at 20 000 chars and appends `[truncated; full result in <result.json>]`.
- The cut is not silent, and the notice names the recovery file. Briefs that ask for a report in `tmp/` and a short reply avoid it.
- Optional later: raise the cap in the extension.
