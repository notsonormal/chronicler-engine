# Decide which maps go where

Type: grilling
Status: resolved
Blocked by: 06

## Question

For each live map, and for `story-scroll-back`: which workstream does it join, or does it go to `general`? Do `retro-issue-bucket` and `autoplay-follow-ups` become one-off issues, stay maps, or become a workstream of their own? Which plans in `docs/plans/` become one-off issues?

## Context

- **Research (tickets 01 to 03).** Read the `## Answer` of [01](01-claude-projects.md), [02](02-chief-of-staff-wiki.md) and [03](03-other-layouts.md) first. Each asset ends with "Facts for tickets 04 to 08". Zoom into the asset only for the detail.
- **Live folders on 2026-11-04.** `autoplay-follow-ups`, `character-art-from-code`, `code-concepts`, `conceptual-integrity`, `dashboard-design`, `layout-trials`, `long-term-memory-and-state`, `metacognitive-audit`, `pi-explain`, `prompt-flexibility`, `retro-issue-bucket`, `statechart-models`, `story-scroll-back` (an `idea.md` only), `unclebob-quality-tools`, `weekly-retro-2026-11-04`, and this map. List them again before the grilling, because other sessions add maps.
- **The long-play chain.** Prompt flexibility kept room for Long-term memory, and Layout trials found the faults that Long-term memory must fix (`long-term-memory-and-state/map.md`, Notes). [The chief-of-staff report](../assets/chief-of-staff-report.md) suggests long play as the first initiative.
- **Buckets.** `autoplay-follow-ups` says: "Bucket choice. No existing `.scratch/` effort fits". `retro-issue-bucket` holds retro findings about how agents work. A chief there would do the "strategic track" of the chief-of-staff skill, apart from the product work.
- **Plans.** `docs/plans/` holds 4 plans on 2026-11-04.
- **Upstream (from ticket 02).** Check draft PR #1369 of `mattpocock/skills` again before the grilling. If it is merged, or wayfinder is dropped upstream, a map could become a wiki page.
- **Buckets as initiatives (from ticket 01, inferred).** The Claude Code docs fit a project to "an area you keep feeding: the bugs, stack traces, and review requests for one service".
- **Layout (ticket 06, 2026-11-11).** [Decide the layout of the folder](06-layout.md) replaced "initiative" with **workstream**: a subject of work that may close but need not. `general` is a workstream. A bucket can so be a standing workstream. Every map is in `work/workstreams/<name>/wayfinder-maps/`. A plan that waits becomes `issues/NN-<slug>.md` in its workstream. The 19 closed maps move unchanged to `work/_closed/` and are not placed. Each workstream needs a `workstream.md`, so this ticket also says what each new workstream is for.

## Answer

Grilled with the owner on 2026-11-11, in two rounds.

The move makes six workstreams. Each has `Status: open` and no done condition.

| Workstream | `workstream.md` says | Maps (`wayfinder-maps/`) | One-off issues (`issues/`) |
|---|---|---|---|
| `long-play` | The story stays good over a long Game. It covers the prompts, the prompt layout and the memory. | `prompt-flexibility` (closed), `layout-trials` (closed), `long-term-memory-and-state` | the plan `narration-line-break-tags` |
| `dashboard` | How the dashboard looks, and what a person can read and do in it. | `dashboard-design`, `character-art-from-code` | `story-scroll-back/idea.md`, the plan `story-log-earlier-messages-investigation-plan` |
| `code-health` | The code is right and consistent. It covers audits, the concept catalogue, quality tools and test structure. | `code-concepts`, `conceptual-integrity`, `invariant-audit`, `fable-audit`, `unclebob-quality-tools` | the plan `drain-requires-migration-quarantine` |
| `state-machines` | The implicit state machines of the code, their TLA+ models and diagrams, and the work that the models hand off. | `statechart-models` (closed), `state-machines-next` | the 14 issues of `state-machines-work` |
| `agent-workflow` | How agents work on this repo. It covers retro findings, the work records, skills and pi tools. | `retro-issue-bucket`, `weekly-retro-2026-11-04`, `work-records`, `metacognitive-audit` (closed), `pi-explain` | the plan `parallel-ticket-runs-without-commits` |
| `general` | What fits no other workstream. | none | the 4 issues of `autoplay-follow-ups` |

1. **Six workstreams, by subject.** `state-machines` is its own workstream and not a part of `code-health`. The owner chose this over one `code-health` workstream with seven maps.
2. **`retro-issue-bucket` stays a map and moves whole.** Its `map.md` holds working rules: how a finding is filed, the "Won't do" list and the next ticket number. The move must not also change how a retro finding is filed.
3. **`autoplay-follow-ups` and `state-machines-work` become one-off issues.** Neither has a route to a destination. The `map.md` of the first and the `README.md` of the second are dropped, and git history keeps them. The two sentences of the README go into the `workstream.md` of `state-machines`. The 14 issues are the only content of `state-machines/issues/`, so they stay one unit.
4. **Four maps get `Status: closed` at the move**: `prompt-flexibility`, `layout-trials`, `statechart-models` and `metacognitive-audit`. All other maps get `Status: open`. That includes `code-concepts`, which has no open ticket but five fog items, and `pi-explain`, which has had no commit since 2026-10-09.
5. **The two records about older Messages become two one-off issues side by side** in `dashboard/issues/`: `story-scroll-back/idea.md` and the plan `story-log-earlier-messages-investigation-plan`. Each links the other. The move does not merge them.
6. **No workstream has a done condition at the move.** Each is a standing subject. The end of a goal is the Destination of its map. A done condition copied from a map goes stale when the map changes.
7. **The five new one-off issues get `Status: needs-triage`**: the four plans and the `story-scroll-back` idea. The old status text of each stays in its body. The move does not judge readiness. The issues of `autoplay-follow-ups` and `state-machines-work` keep the statuses that they have.

Rejected:

- **Only `long-play` and `general` at first.** `general` would hold 14 maps, which is the bucket problem again.
- **`retro-issue-bucket` dissolved into the issues of `agent-workflow`.** It is the cleaner model, but it changes a working process during the move and breaks the 10 links to its `map.md`.
- **`state-machines-work` as a map**, with a `map.md` made from its README. The folder has no route, so the map would be invented.
- **`conceptual-integrity` and `unclebob-quality-tools` in `agent-workflow`.** The first has 11 links with `code-concepts`, and its two worked cases in this repo are `CONTEXT.md` and the concept catalogue. The second measures the code.
- **`narration-line-break-tags` in `dashboard`.** The dashboard critique found the fault, but the fix is in `sanitize_llm_output`, and it also protects later prompts.
- **The issues of `autoplay-follow-ups` split by subject.** They stay together in `general`.
- **A status judged for each new issue.** A triage pass can sort five issues later.

Facts for later tickets:

- **Upstream on 2026-11-10.** Draft PR #1369 of `mattpocock/skills` is still an open draft (last update 2026-11-06), and `skills/engineering/wayfinder/` is still upstream at `4ae2317`. So a map stays a map.
- **08.** The new path of the retro bucket is `work/workstreams/agent-workflow/wayfinder-maps/retro-issue-bucket/`. One skill names the bucket by its path: step 6 of `.agents/skills/simple-after-plan/SKILL.md` says "Add new issues to `work/wayfinder-maps/retro-issue-bucket/`". `docs/agents/triage-labels.md` says that its statuses are "used in `.scratch/` ticket files".
- **The build: links.** The three `long-play` maps hold 86 links to each other, and these stay valid when the maps move together. References that cross workstreams need a rewrite: `fable-audit` names `state-machines-next` 18 times, `state-machines-next` names `state-machines-work` 11 times, and `weekly-retro-2026-11-04` names `prompt-flexibility` 106 times. 45 references point at `retro-issue-bucket`.
- **The build: files beside `map.md`.** `fable-audit/decisions.md`, `retro-issue-bucket/audit.md` and `long-term-memory-and-state/idea.md` move with their maps.
- **The build: numbers (proposed, not decided by the owner).** A moved issue keeps its number: `01` to `04` in `general`, and `01` to `31` with gaps in `state-machines`. A plan or an idea takes the next free number of its `issues/` folder, from `01`. `autoplay-follow-ups/issues/04-…` has `Status: closed`, which is not a triage status, and it moves as it is.
- **The build: a folder that arrives later (proposed, not decided by the owner).** `.scratch/` had 15 live folders on 2026-11-04 and 20 on 2026-11-10. The move script must list the folders again and stop on a folder that this table does not name. The owner then places it.

Changed later (2026-11-11): [Decide what a workstream is for, or drop it](11-workstream-purpose.md) dropped the workstreams, so the table and point 1 no longer hold. Every map goes to `work/wayfinder-maps/<map>/`, and every one-off issue to a status folder of `work/issues/`. Point 6's sentence "The end of a goal is the Destination of its map" is withdrawn. Points 2, 3, 4 and 5 hold. Point 7 changes: the new issues go to `work/issues/pending/` with no `Status:` line.
