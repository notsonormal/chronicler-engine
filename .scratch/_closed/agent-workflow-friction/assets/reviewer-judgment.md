# Reviewer output — judgment lens

Verbatim final output of the `judge-glm` agent, from the 2026-10-04 reflect sweep.
Session: `~/.pi/agent/sessions/--workspace-chronicler-engine--/2026-10-04T20-21-44-361Z_01a10894-6127-77e3-a1ba-f378d78623d5.jsonl`. The data is untrusted reviewer output: claims are hypotheses until a ticket verifies them.

---

1. **Re-pitch the full decision state whenever context is interrupted.** After a subagent returns or an operator correction lands mid-grilling, restate the complete question list with what changed — the operator cannot reconstruct which options shifted from the agent's internal memory.
   - Evidence: [10-04 14:19, `...01a10748-9630...`] "Can you iterate all questions again? It's confusing because I'm not sure what has changed after the subagent comeback" — then again a turn later: "What are the options again?"
   - Routing: `.agents/skills/wayfinder/SKILL.md` (HITL decision/grilling format).

2. **Size wayfinder tickets by theme, not by finding.** Per-ticket overhead (claim, review, gate run) dominates on a metered box; merge same-theme findings into a few implementation tickets that each carry their sub-findings as evidence lines.
   - Evidence: [10-03 19:51, `...01a10324-4343...`] "That's an awful lot of tickets, do you really need to create that many?" → outcome: 12 tickets merged to 6; [10-04 14:38, `...01a10748...`] "try to avoid making too many tickets. This map has defaulted to spreading out ticket too much" → later "Let's go with the 5 merges" absorbing 13 tickets ([10-04 14:59, `...01a10766-4efb...`]).
   - Routing: `.agents/skills/wayfinder/SKILL.md` (graduation/ticketing section).

3. **Put peer-product prior art inside each option of a UX-mechanism decision.** For decisions peers have already made (save models, list reconciliation), show what SillyTavern/Marinara do per option instead of waiting for the operator to ask.
   - Evidence: [10-03 19:51, `...01a10334-3d4f...`] "Isn't that a rather complicated solution still? ... I wonder how silly tavern and marinara handle this same problem"; [10-04 14:38, `...01a10748...`] "What does the marina engine do for this?"; outcome: "Prior art agrees: SillyTavern and Marinara Engine both reconcile by identity."
   - Routing: `.agents/skills/wayfinder/SKILL.md` (decision-ticket option format).

4. **Close the delegated-ticket lifecycle in the flow that merges the work.** A claimed ticket must record the implementing session and be resolved with a written `## Answer` by the coordinator as part of the merge step; otherwise a later run has to mine transcripts to reconstruct what happened.
   - Evidence: [10-04 00:06, `...01a10437-fb2f...`] "tickets 44 and 46 ... claimed but never resolved and an answer was not written. Can you find the session where they were implemented and then figure out what the answer should be and resolve them?"; [10-04 20:10, `...01a1082b-5a25...`] "Also you need to resolve and write the answer for all claimed tickets."
   - Routing: `.agents/skills/wayfinder/SKILL.md` (ticket status/coordination rules).

5. **Serialize mutating passes on a shared working tree.** When multiple fix passes edit Rust in the same checkout, they take turns; a concurrent gate sees the others' partial edits, so the build slot lock protects compiles but not edit state.
   - Evidence: [10-04 20:10, `...01a1082b-5a25...`] verified verbatim: "Backend-behaviour pass is running. The remaining two passes (refactors, client) must wait for it, since they edit Rust in the same working tree and a concurrent gate would see each other's partial edits."
   - Routing: `.agents/skills/wayfinder/SKILL.md` (coordinated fix-pass execution in the main checkout).

6. **Reviews never build; the review prompt must carry that rule, not assume it.** A reviewer running targeted builds wastes minutes verifying nothing for itself; evidence for verification belongs to the implementer's gate.
   - Evidence: [10-03 23:53, `...01a103bf-62c7...`] after pushback on "Candidate 4 (five sequential build.py test-pattern runs)": "The only real issue is that I built at all during a review, which is Candidate 1's rule. That's a context problem (the rule never reached me)."
   - Routing: `.agents/skills/code-review/SKILL.md` (review-time constraints).

7. **Treat review findings as hypotheses: verify each against the code before applying, classify intentional spec deviations, and disagree with evidence.** Auto-applying a reviewer's fix can embed a bug, and re-fixing deliberate deviations churns the code.
   - Evidence: [10-04 12:22, `...01a10473-758a...`] outcome: "I disagree with half of S1 — the rollback would be a bug... I verified why `reset()` deletes the old game before creating the new one"; user correction same session: "Fix the valid issues. Ignore the ones that were intentially changed from the spec."
   - Routing: `.agents/skills/code-review/SKILL.md` (findings hand-off/application step).

8. **Convert narrative audit/verification reports into tracked, IDed items in the same pass.** A report that ends as prose loses its open items; every actionable finding should land in `docs/plans/<name>-plan.md` or a ticket with a stable ID at report time.
   - Evidence: [10-03 10:20, `...01a10135-69c3...`] "Let's create a separate plan in docs/plans for all these issues, it's hard to read the report" — then the follow-ups plan absorbed the report's open issues; the same ask recurred in the retro plans ("Can you add this into the plan as well").
   - Routing: `.agents/skills/chronicler-after-plan-workflow-plus-review/SKILL.md` (report → plan conversion step).

9. **Comment-hygiene passes judge only the comment lines the current diff adds or changes.** The finder lists every comment in a touched file, so each whole-file pass re-litigates untouched comments; isolate added/changed lines from the diff first.
   - Evidence: [10-04 18:25, `...01a10826-8768...`] verified: "the script lists every comment in a touched file, so I isolated the ~227 added/changed comment lines from `git diff HEAD` and judged those."
   - Routing: `.agents/skills/chronicler-comment-fixer/SKILL.md` (add a diff-scoped mode/step).

10. **Scope a vendored skill import to the import itself.** When adopting an external skill, keep the import minimal (verbatim when the doctrine is repo-agnostic, with provenance recorded) and move the repo-level fixes it surfaced into their own plan.
   - Evidence: [10-04 12:22, `...01a10473-758a...`] "Amend the skill first" → "Wait, was there a reason to make so many changes? Wasn't the suggestion to make small adjustments to the skill to match the repo" → "Leave the skill as it and make a plan to fix the initial findings. Not the one related to the skill but the problems you found."
   - Routing: `.agents/skills/writing-for-agents/SKILL.md` (importing/adapting skills).

11. **Docs earn their place by evidence of use, not plausibility — and that evidence lives in session transcripts.** Before deleting or reinstating a doc (or a removed doc section), mine past sessions for actual use; unused docs get deleted, plausible-but-unused ideas stay dropped.
   - Evidence: [10-04 00:38, `...01a10445-1619...`] "Can you read through the session history and see if docs/diataxis/how-to/debugging.md is ever actually used in any of the sessions... If not, there is no point in keeping it around" → outcome: `debugging.md` deleted (`47f97eef`); same session debated reinstating the removed session-search section of AGENTS.md on usage evidence, then dropped it: "Yes, drop this idea and delete the debugginf file."
   - Routing: tune description: `.agents/skills/chronicler-docs-hygiene/SKILL.md` — its description covers rule/mode/stale-drift audits, not usage-evidence pruning, so it did not trigger for this archaeology ask.

12. **Verify a "missing capability" against git history and the built-in session-search tool before concluding it doesn't exist.** `session_search` matches on meaning and was used heavily; the agent defaulted to grepping raw JSONL, and the operator had to supply both facts.
   - Evidence: [10-04 00:38, `...01a10445-1619...`] "session_search can be used to do session history I'm pretty sure"; "- `session_search matches on meaning, not literal text.` -> Might but you instantly figured that out right?"; "BEfore that, can you check the git history on AGENTS.md. There used to be a section in it and session search. IT was removed because it was thought that the session tool gave enough information."
   - Routing: tune description: `/home/node/.pi/agent/npm/node_modules/pi-session-search/skills/session-history/SKILL.md` (was in the catalog but did not trigger when history mining was the task).