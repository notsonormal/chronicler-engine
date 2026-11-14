# Grilling questions use review IDs and rest on facts nobody checked

Type: grilling (HITL)
Status: open
Blocked by: —

## Question

In one grilling session the user had to ask twice what a question meant. Both questions named review-finding IDs and code terms as their subject. One of them also asked the user to decide about a premise that the lead had not checked; when checked, the case could not occur, so there was no decision to make.

Won't do line **10 — Grilling questions cite opaque ids** says that `grilling/SKILL.md` already tells the agent to restate context and to assume the user has not read the files. This ticket is filed with new evidence against that reason: the rule was in the skill, and it did not hold.

What change makes a grilling question readable and grounded in checked facts?

## Context

- Source: `/retro` 2026-10-29, lead session `01a1743c-7303-77ca-953f-4e72116e9d9f` (the action-answers Final gate and its fix pass, branch `wf/change-driven-dashboard-refresh`).
- **Q6** asked about "the answer-model restructure (AA-1 to AA-4)" with the review's labels: an untyped response extension, an import cycle, three enums, `AnswerPolicy`. The user answered: "Reexplain this. Because it's sounding like a really complicated and cumbersome thing at this point." The re-explanation said what the player sees (one wrong status code) and the size (about 20 call sites), and the user decided at once. A fact the question depended on (does `EngineError::Config` hold real setup faults?) was checked only after the user decided.
- **Q8** asked whether to accept "the unbuilt `409`" (plan rule A5). The user answered: "What are you talking about". Checked afterwards: only `refused()` (`src/adapters/driving/http/utils/response.rs:26`) builds a `409`, so the case could not occur; the question became "delete one dead line".
- `.agents/skills/grilling/SKILL.md:27` already says "Finding facts is your job, never the user's". The step has no completion criterion that the agent checks before it asks.
- **Possible shape (a proposal).** Two lines in the skill: check every fact a question rests on before asking it; open each question with the effect the user sees and the size of the change, and name IDs and identifiers only after that.

## Further evidence (2026-11-13, long-term-memory-and-state session `01a1cc56`)

- **known** The owner corrected the questions 7 times on 2026-11-11, for example "What are you trying to get me to decide" and "question one was something we talked about before". One question asked the owner how the memory test should be scored, which is the coordinator's choice; another asked again what the owner had answered in `work/wayfinder-maps/long-term-memory-and-state/map.md`.
- **known** The lesson lives only in `work/wiki/owner-preference-questions-in-application-terms.md`; `grilling/SKILL.md` does not point to it.
- Proposed: two pre-ask checks — search the map's Notes and Answers for the subject, and open each question with what the player or the author gets.
