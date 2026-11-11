# Chief of staff, wayfinder and a cheap software factory

Date: 2026-11-04 (revision 2, after your feedback). Scope: the Chronicler Engine workflow of 2026-10-28 to 2026-11-04.

> Copied from `tmp/chief-of-staff/report.md` when the [Work records](../map.md) map was charted (2026-11-04, at `e6715c60`). The numbers are a snapshot of that date. The scripts of the appendix are in `tmp/chief-of-staff/analysis/`, which can be cleared. The report was checked by a fresh reviewer, and its findings were fixed. The later points about the layout (expedition, errand, `general`, the folder name) are not in this report. They are in the map's Notes and in [the charting facts](charting-facts.md).

## Answer

Your bottleneck is the HITL tickets, not the AFK tickets. An AFK ticket costs you one "send the research agents" message. A HITL ticket needs you present for a grilling round, and 13 live maps compete for that time.

The part of `chief-of-staff` that helps with this is its **goal memory**: one long session, and a wiki scoped to one goal. Such a session learns your goal, your preferences and your standing rules. It can then ask you fewer and sharper questions, and answer "where are we" by itself. Its other parts you already have (fresh subagents, context pointers, retro) or do not want (a runner that starts AFK work because the work is unblocked).

Recommendation: run `chief-of-staff` for **one goal**, adapted:

- **One goal.** For example, long play, which spans Prompt flexibility, Layout trials and Long-term memory.
- **Wayfinder stays** the store for each effort.
- **A goal wiki holds what spans the maps**: the goal, your preferences, the standing rules, and a log of the decisions the chief made for you.
- **AFK work starts only on your "go".**
- **HITL rounds come with the prework done.** The chief asks only what its record cannot answer.

Check it for one week against the baseline in [The outside check](#the-outside-check).

The bitter lesson supports this split. A record of what you want is **data**. A better model makes better use of data, so the record gains value as models improve. Orchestration prose and a hand-built factory lose value as models improve.

## What changed in revision 2

On 2026-11-04 you corrected two points of revision 1. I agree with both.

| Revision 1 said | You said | Revision 2 |
|---|---|---|
| The value is a runner that runs the AFK frontier while you are away | AFK tickets are easy to run. HITL tickets are what blocks. You do not want AFK work to start just because it is unblocked | Agreed. The sessions confirm it: the AFK dispatch is one message, and the frontier is not your priority (see [HITL is the bottleneck](#hitl-is-the-bottleneck)). AFK work now starts only on your order |
| A chief-of-staff wiki would bring back the failure of the 33 ADRs | ADRs described the whole application. A chief-of-staff wiki is local to the goal | Agreed. I conflated the two. The ADRs were read for the current architecture. A goal wiki holds decisions, preferences and problems of one goal, and `docs/` keeps the architecture. One small caveat stays (see [The goal wiki](#the-goal-wiki)) |

The second correction changes the conclusion. Revision 1 said "do not adopt chief-of-staff". Revision 2 says adopt its goal memory for one goal.

## Sources and method

| Source | What I read | Status |
|---|---|---|
| `mattpocock/skills@4ae2317` (2026-10-31) | `skills/in-progress/chief-of-staff/SKILL.md`, `software-factory/` (SKILL.md and EXAMPLE-TASKS.md), `loop-me`, `claude-handoff`; `engineering/wayfinder`, `retro`, `implement-spec` for the diff against your copies | known, read directly |
| `notsonormal/mrn-wiki@37db57b` | `concepts/software-factory`, `chief-of-staff-pattern`, `wayfinder-decision-map`, `orchestration-axes`, `orchestration-principles`, `agent-team-allocation`, `repo-as-memory`, `bitter-lesson-agent-organization`, `model-task-split`, `entities/chronicler-engine` | known, read directly. Its claims are second-hand to me |
| This repo | `.agents/skills/{wayfinder,grilling,retro,implement-spec,simple-after-plan,chronicler-after-plan-workflow,consider-the-bitter-lesson}`, `docs/agents/issue-tracker.md`, `.scratch/*/map.md`, `.pi/extensions/delegate`, `git log --since=2026-10-28` | known |
| 254 pi sessions, 2026-10-28 to 2026-11-04 | All top sessions and all delegate sessions of this project. The scripts are in `tmp/chief-of-staff/analysis/` | numbers known; the message classes are a regex sort, so they are approximate |

Two claims of the wiki page `chief-of-staff-pattern` hold at `4ae2317`. The skill is still under `skills/in-progress/`. Its four procedure paths (`../retro`, `../to-spec`, `../to-tickets`, `../implement-spec`) point at folders that do not exist next to it inside the repo, because those skills are in `skills/engineering/`. **Correction (2026-11-05, ticket 02 of the Work records map):** PR #1363 says the paths are written for the installed layout, where `scripts/link-skills.sh` puts every skill side by side. They resolve after install, so they are not broken.

## What chief-of-staff is

It is a 75-line prompt for one long session that owns a goal. Its rules:

1. **All work in fresh subagents.** The chief protects its context. Messages to and from subagents are sparse **context pointers**.
2. **A wiki is the memory.** Interlinked pages, an `index.md`, dated claims that cite commits, "supersede, never rewrite". Ingest, query and lint operations. The skill gives the page kinds as "a decision, a recurring problem, a subsystem, a user preference".
3. **Procedures.** `/retro` on about 1 in 5 subagent sessions, failures first. Large work goes `to-spec` → `to-tickets` → `implement-spec`.
4. **Two tracks.** Tactical (do the task) and strategic (improve the environment for the next task). "Be relentless… Use every user message as an excuse." A "no workarounds" rule.
5. **Schedules**, "harness-permitting".

Its companion, `software-factory` (same commit), is a glossary for the next step up: triggers, runs, runners, an orchestrator. In that design "a run never waits for a human".

## Your workflow in numbers

All numbers are for 2026-10-28 to 2026-11-04.

| Measure | Value |
|---|---|
| Sessions | 254: 66 top sessions, 188 delegate sessions |
| Top sessions that start with `/wayfinder` | 47 of 66. 31 work one ticket, 8 dispatch subagents, 8 chart a map or other |
| Delegate runs by role | reviewer 67, implementer 47, researcher 32, strong-implementer 29, scout 13 |
| Delegate runs that ended normally | 187 of 188 |
| Delegate runs that were a later round (fix pass, its review, retry, art round 2 or 3) | 18 |
| Ticket resolutions in git (`+Status: resolved` lines) | about 74 (inferred: moved tickets can count twice, deleted retro tickets do not count) |
| Commits | 79, of which 29 touch `src/` |
| Efforts named in the top sessions of one day | up to 10 (2026-10-31 and 2026-11-01) |
| API-equivalent cost | top sessions $262, delegates $239. You pay by subscription, so this is a relative measure only |
| Top session length (first to last entry) | median 57 min |
| Sessions that compacted | 3 of 66 |

Your 463 messages, sorted by a regex (approximate):

| Class | Count | Share |
|---|---|---|
| Steering, context, a new idea | 227 | 49 % |
| Answers to agent questions (`Q1 -> A`) | 99 | 21 % |
| A `[/wayfinder]` invocation: open a ticket (31 sessions), dispatch, chart, or a second invocation in a session | 51 | 11 % |
| Invoke another skill (`/improve-ai-plan`, `/sharpen`, `/code-review`) | 33 | 7 % |
| A question back to the agent: what a term means, a design question, where an effort stands | 28 | 6 % |

By hand, 6 of your messages ask where an effort stands, what its goal is, or what its next step is. All 6 are in Prompt flexibility and Layout trials (2026-10-31 to 2026-11-02).
| Git routing (`commit and push`, `merge`) | 25 | 5 % |

What the sessions show (known, quoted from the sessions):

- **You are the scheduler.** Each ticket starts with your message. On 2026-11-03 at 00:20 you started three sessions in 34 seconds, for tickets 5, 6 and 8 of the same map.
- **You lose the thread across maps.** "What exactly is the next step? There's only one open ticket, but I don't think that's necessarily important" (prompt-flexibility, 2026-11-01). "Lets zoom out What is the long goal the current map" (layout-trials, 2026-11-02).
- **Grilling rounds tire you.** "You just asked me a ton of questions, so I'm not really confident in my answers" (hex-comparison, 2026-11-01).
- **A HITL task can be unclear.** "Even if I read it, I'm not sure what you are expecting from me" (layout-trials, 2026-11-02). The agent had asked you to read two trial packs with no clear question.
- **Some efforts produce process, not change.** Hex comparison: "lots of research and grilling… all 'already covered' or 'not worth doing'… around and round a point frustating and pointless" (2026-11-02). You closed that map the same day (`6ca0e58f`).
- **You already prune by the bitter lesson.** You deleted the `tdd` skill ("Just because it adds unique rules, doesn't mean that it adds value"). Ticket 10 of long-term memory chose "a minimal form" after `/consider-the-bitter-lesson`. On layout-trials: "deepseek flash is a popular enough model for RP… we have to accept the inconsistencies rather than trying to engineer around them".
- **Delegation is one level deep.** "I don't think subagents can call other subagents" (2026-11-03). This is correct: `.pi/extensions/delegate/index.ts` registers no tools in a child. So a chief must be the top session, and it receives every result.

## HITL is the bottleneck

**AFK work is cheap for you.** A dispatch is one message: "send research subagents for 1 and 2", "Send all 7 research tickets". The rest of those 8 dispatch sessions (median 4 messages) is your discussion of the results.

**The frontier is not your priority.** Three signs (known):

- On 2026-10-31 you committed "Blocking 11 by ticket 24 so it doesn't keep getting picked up". You added a false blocker to stop an agent from taking a ticket.
- "I don't want to work on this one, skip this and work on the next" (prompt-flexibility, 2026-10-31).
- "There's only one open ticket, but I don't think that's necessarily important in the context of everything else" (2026-11-01).

So "unblocked" does not mean "do it now". A runner that takes the frontier in order would do the wrong work.

**Most of your answers accept the recommendation.** The `/grilling` skill ends each question with a `➡️` recommended answer. I compared your answers with those lines (`tmp/chief-of-staff/analysis/answers.py`). The script matched 59 answers in 23 rounds. It sees only the rounds in the `❓ **Qn**` format, so the sample is small (approximate).

| Your answer | Count | Share |
|---|---|---|
| The recommended option, or "confirm" / "accept" | 46 | 78 % |
| A different option | 6 | 10 % |
| Free text: a question back, a doubt, your own answer | 7 | 12 % |

What this means (inferred):

- About 1 answer in 5 changes the outcome. Those answers carry your taste and goals ("Let's just match ME", "How does ST/ME handle it?"). Neither you nor the agent knows in advance which question will get such an answer.
- Some of the 78 % is fatigue, not agreement. You said so on 2026-11-01.
- So the HITL cost is mostly the time to read and confirm. The value is in the few answers that are not predictable. Fewer, sharper questions keep the value and cut the cost.

## Where each design sits

The five axes are from the wiki page `orchestration-axes`. They show what each design changes.

| Axis | Your workflow today | chief-of-staff | software factory |
|---|---|---|---|
| 1. Context allocation | Opus top session, fresh delegates on Sonnet, Opus, DeepSeek | Same: one DRI session, fresh subagents | Each run is fresh |
| 2. Scheduling (who decides that work exists) | **You**, one ticket at a time | You, by message, and the chief fans out. Schedules if the harness has them | Triggers: tracker change, clock, another run |
| 3. Continuity | One map per effort, `CONTEXT.md`, `docs/`, commits; observational memory in each session | A goal wiki of dated claims, and the long session | Tracker and git host only |
| 4. Execution authority | Agents edit; you order every commit and merge (permission system) | The chief and its subagents edit | Runs open PRs; a human merges |
| 5. Human position | Every grilling question, every commit | At the ends; retro samples 1 in 5 | Only at review and merge |

You already match chief-of-staff on axis 1. On axis 2 the skill also leaves the scheduling to you: it only suggests schedules, "harness-permitting". So the skill differs from today on axes 3, 4 and 5. The adapted design keeps your commit gate on axis 4. Its gain is on axis 3 (a store above the maps) and axis 5 (fewer questions for you).

## The goal wiki

**What is above the maps today.** Your efforts chain. Prompt flexibility "kept room" for Long-term memory, and Layout trials "found the faults that this effort must fix" (`long-term-memory-and-state/map.md`, Notes). No single file holds the goal that the chain serves. So each map copies the shared parts forward:

| Text in map Notes | Maps that hold a copy |
|---|---|
| "Standing rules (from the earlier maps)" | prompt-flexibility, layout-trials, long-term-memory-and-state |
| Write plans "not through Plannotator" | the same three |
| "Any new API takes a `GameId`" | the same three |
| The migration rule (`ALTER TABLE … ADD COLUMN`) | the same three |
| "How a trial runs" | layout-trials, long-term-memory-and-state |

The Notes section of `long-term-memory-and-state/map.md` is 11,353 bytes, and each session of that map loads it. A goal wiki holds these once. Each map links to it, and its Notes keep only what is local to that map.

**What a goal wiki holds.** It is scoped to one goal:

- the goal, and how each map serves it. This answers "zoom out" with no new research;
- your preferences, each with its date and source. Example: the OpenCode Go Connection is a subscription, so token cost does not matter (2026-10-31);
- the standing rules that span the maps;
- recurring problems, for example the trial noise and the judge method;
- the log of the decisions that the chief made for you (see below).

**The caveat that stays.** The skill lists "a subsystem" as a page kind, and it tells the chief to cite commits for claims about code. A page about a subsystem is the part that can drift into a second `docs/`. A one-line rule prevents it: "no subsystem pages; link `docs/diataxis/` instead". This is small, and it does not change the recommendation.

**Where it lives** is your decision. The skill says to use a shared wiki first. The mrn-wiki is shared, but its `SCHEMA.md` scope is homelab and AI subscriptions. Delegates in this repo also read it less easily. A folder beside the maps, for example `.scratch/_goals/long-play/`, is versioned with the maps.

## The bitter lesson

Sutton (2019): "general methods that leverage computation are ultimately the most effective". The two methods that scale are search and learning. Built-in human knowledge helps only until compute catches up. The wiki page `bitter-lesson-agent-organization` separates a soft claim from a hard one. The soft claim has held: compute-using general methods beat hand-built rules. The hard claim, "scale is all you need", does not follow.

For each part of the workflow, ask one question: **will a better model need this part less, or get more value from it?**

| Part of your workflow | Better model makes it | Why |
|---|---|---|
| The gate: `build.py`, guardrails, arch-lint, the spec↔test check | More valuable | A better model iterates faster against a check. A check never expires |
| Trials, the memory test, LLM judges, `prefix_measure.py` | More valuable | They turn compute into evidence. More runs give less noise |
| Data access: probe server, trial packs, build logs, session files | More valuable | A stronger agent uses more of what it can see |
| A record of your goal, preferences and past answers | More valuable | It is data about the one thing the model cannot find in the repo: what you want. A better model predicts more of your answers from it |
| Fresh-context reviewers | More valuable, for now | A vendor measure: Cognition on its own product, ~2 bugs per PR (wiki `model-task-split`). No independent figure |
| Map Notes that restate rules | Less needed | Prose that every session loads. Much of it restates rules that a model would infer from the code |
| Step lists in skills (`chronicler-after-plan-workflow` has 11 steps) | Less needed | Scaffolding. The wiki records "scaffolding that expires in 3–6 months" |
| Role and model routing, prompt rules for delegation | Less needed | Cohere (wiki `agent-team-allocation`): whether a lead delegates well depends on the model, not on the chart |
| Retro changes | No evidence either way | Treude & Baltes (wiki `harness-self-evolution`) measured transfer, not model change. Of 7 informative transfers, 2 removed the behaviour. Both covered the cases that later sessions met (a test-runner instruction, a known-failures file). So measure each change, and retire a change that does not help |

What follows for this decision:

1. **Your scarce input is what only you know: the goal, your taste, the priority.** Every question that a model could answer from a record of your earlier answers is a cost with little value. As models improve, that share grows. So the record is the right thing to build, and the question list is the thing to shrink.
2. **Move decisions from grilling to measurement where a check exists.** You already do this: "trial → build → trial", the memory test, the judge. A ticket that a trial can decide does not need a grilling round.
3. **Do not build a factory now.** An orchestrator, triggers and runners are hand-built organisation. Vendors are making them a commodity (wiki `software-factory`, vendor landscape, 2026-11-04). Your core work is HITL decisions and taste, and that work does not fit "a run never waits for a human".
4. **Keep the chief's instructions thin.** The DeepSeek design paper (wiki `orchestration-principles`, rule 11) says: "A supervisor that can do the work will do the work". Remove a capability instead of asking politely. "Be relentless. Use every user message as an excuse" is the kind of prose to leave out.

The counterweight (known, wiki `agent-team-allocation`): today the lead model still sets the ceiling. A weak lead with good workers stays below a strong solo agent. So keep Opus in the chief seat. Also, no model yet replaces your judgement of taste in art or in story quality. You said on 2026-11-02: "I don't think there is any benefit is some pre-defined judge right now".

## Options

| Option | What changes | Cost to set up | What you get | Main risk |
|---|---|---|---|---|
| A. Keep the workflow | Nothing | None | Known behaviour | You stay the scheduler and the only memory across maps. Each grilling round asks everything |
| B. chief-of-staff as written | One long session, a wiki, retro on 1 in 5, a strategic track, schedules | Low | A goal view | The strategic track adds process work (the hex-comparison lesson). Pages about subsystems duplicate `docs/` |
| C. chief-of-staff for one goal, adapted (recommended) | One long session per goal; a goal wiki; HITL prework; questions that a record answers become a "decided for you" log; AFK on your order | One skill of about 40 lines; one wiki folder; the shared rules moved out of three map Notes | Fewer and sharper questions; "where are we" answered from the wiki; less text copied between maps | The chief decides a question you wanted to answer. A log and a veto limit the harm; the check measures it |
| D. A software factory (cron or GitHub Actions, runs without you) | Triggers and an orchestrator | High: runners, secrets, a security review | Work that starts without you | Your bottleneck is HITL, which a factory does not remove. It is the most hand-built option |

## The adapted chief of staff

### What it does

1. **Start from the wiki.** The chief reads the goal page and the index, then the maps of the goal at low resolution. It reports where the goal stands in a few lines, and what waits on you.
2. **You choose the work.** The chief proposes the next tickets, with a reason for each. You say which ones. AFK tickets start only on your order. Unblocked does not mean go.
3. **Prework for each HITL ticket.** Before it asks, the chief does the fact work in subagents, as `/grilling` already requires ("Finding facts is your job, never the user's"). You asked for this on the hex-comparison map: "a separate ticket to do some 'prework' before the grilling ticket starts".
4. **Ask less.** For each question the chief checks the wiki. If a dated preference or an earlier decision covers the question, the chief decides it and writes it in the "decided for you" log, with the page that covers it. It asks you only the rest. You read the log in one pass and veto a line if you disagree. This changes the wayfinder rule that the agent "never stands in for the human's side", so it is your decision (see [Decisions that are yours](#decisions-that-are-yours)).
5. **Ingest each answer.** An answer that shows a preference or a standing rule becomes a dated wiki line. The next round then asks less. An answer that shows a recurring fault becomes a check or a deletion, not a new paragraph.
6. **Async when you are away.** The chief sends one `:::question` round to the Plannotator Inbox. You answer from Windows when you can.
7. **Stop at the gate.** It commits or merges only on your order, as today.

### What to take from the skill

- The long session that owns one goal, with the wiki as the memory that survives compaction.
- Fresh subagents and context pointers. You already have them.
- "Write history, not the present": dated preferences and decisions, superseded instead of rewritten.
- `/retro` on about 1 in 5 delegate runs, failures first. Today step 6 of `/simple-after-plan` runs it on the top session and on every subagent session of the run (once per effort in slice mode). Each retro ticket names the check that would fail, or a rule to delete. It also names how you will see later that it worked, so that a change that does not help can be retired.

### What to leave

- AFK work that starts because it is unblocked.
- "Be relentless. Use every user message as an excuse." The hex-comparison map shows what that costs you.
- Subsystem pages. Link `docs/diataxis/` instead.
- Schedules. pi has no scheduler (inferred: the pi docs describe none). A cron job that starts `pi` is option D.

## The outside check

Run the adapted chief for one week on one goal, or until 10 HITL tickets resolve. Measure with the scripts in `tmp/chief-of-staff/analysis/`.

| Measure | Baseline (7 days) | Pass line |
|---|---|---|
| Questions that you answer, per resolved HITL ticket | Measure first: `answers.py` sees only the `❓` rounds | Lower by about a third |
| Your vetoes of the "decided for you" log | — | At most 1 line in 10. More than that stops the trial |
| Messages that ask where an effort stands, its goal or its next step (hand count) | 6 in 7 days, all on the long-play maps | At most 2 |
| Answers that are free text or a different option | 22 % | Not lower. A lower share means the chief asks only the easy questions |
| Standing rules copied into a map's Notes | 3 rules, each in 3 maps | 0 new copies |

If it fails, go back to option A. Keep the goal page and the moved standing rules, because they are useful on their own.

## Decisions that are yours

1. Which goal should the trial use? Long play (Prompt flexibility → Layout trials → Long-term memory) has the most shared text, so it is the best test.
2. May the chief decide a question that a dated preference or an earlier decision covers, with a log and a veto? Or must it ask every question, as wayfinder says today?
3. Where does the goal wiki live: in the repo beside the maps, or in the mrn-wiki?
4. At the end of `/simple-after-plan`, should the retro read about 1 in 5 subagent sessions, failures first, instead of all of them?

## Appendix: reproduce the numbers

```bash
python3 tmp/chief-of-staff/analysis/sessions.py   # writes sessions.json from ~/.pi/agent/sessions
python3 tmp/chief-of-staff/analysis/metrics.py    # every session number, the message classes and the re-orient candidates
python3 tmp/chief-of-staff/analysis/answers.py    # your answers against the ➡️ recommendations
git log --since=2026-10-28 -p -- .scratch | grep -c '^+Status: resolved'   # resolutions
git log --since=2026-10-28 --oneline | wc -l; git log --since=2026-10-28 --oneline -- src | wc -l   # commits
```

The scripts find their files from their own folder, so they run from any directory. The hand count of the 6 re-orient messages starts from the candidates that `metrics.py` prints.

`tmp/chief-of-staff/analysis/top_sessions.txt` lists each top session with your messages, cut to 260 characters each.
