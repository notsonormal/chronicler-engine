# How Pocock's chief of staff and the LLM wiki keep memory

Research for ticket [Find how Pocock's chief of staff and the LLM wiki keep memory](../issues/02-chief-of-staff-wiki.md) of the [Work records](../map.md) map. Access date of every web source: 2026-11-05. Labels: **known** (a source says it), **inferred**, **guessed**, **I don't know**. This file builds on [the chief-of-staff report](chief-of-staff-report.md), which already analysed the skill.

Sources:

- Karpathy, "LLM Wiki" gist, 2026-04-04: `https://gist.github.com/karpathy/442a6bf555914893e9891c11519de94f` (the primary source, read as raw text).
- The skill: `mattpocock/skills`, `skills/in-progress/chief-of-staff/SKILL.md` at `4ae2317`. A full clone is at `/tmp/research/mp-skills-full`. `origin/HEAD` was still `4ae2317` on 2026-11-05.
- Pocock's posts: `https://x.com/mattpocockuk/status/<id>`, read through `api.fxtwitter.com`.
- Pull requests #1369 and #1387 of `mattpocock/skills`.
- Wayfinder: `docs/engineering/wayfinder.md` and `skills/engineering/wayfinder/SKILL.md` at `4ae2317`.
- The owner's mrn-wiki at `37db57b`: `SCHEMA.md`, `concepts/chief-of-staff-pattern.md`, `concepts/repo-as-memory.md`.

## 1. The LLM wiki

**Origin (known).** Karpathy published an "idea file" on 2026-04-04. It is a document that you paste into your own agent. It is abstract on purpose.

**The idea (known).** Retrieval over raw documents "rediscovers knowledge from scratch on every question". Karpathy wants the LLM to "incrementally build and maintain a persistent wiki". The knowledge is "compiled once and then *kept current*". The human chooses sources and asks questions. The LLM does the bookkeeping.

**Three layers (known).**

- **Raw sources.** Immutable. The LLM reads them and never edits them.
- **The wiki.** Markdown pages that the LLM writes and owns.
- **The schema.** A file such as `CLAUDE.md` or `AGENTS.md`. It holds the conventions and the workflows. The human and the LLM change it together.

**Operations (known).**

- **Ingest.** The LLM reads a source and writes a summary page. It updates the index and the related pages. It appends a log entry. One source can touch 10 to 15 pages.
- **Query.** The LLM reads the index first, then the pages. A good answer can be filed back as a new page.
- **Lint.** Look for contradictions, "stale claims that newer sources have superseded", orphan pages, missing pages and missing links.

**Index and log (known).** `index.md` lists each page with a link and a one-line summary, by category. The LLM updates it on each ingest and reads it first on each query. Karpathy says this works at "~100 sources, ~hundreds of pages". `log.md` is append-only. Each entry has a fixed prefix, for example `## [2026-04-02] ingest | Article Title`, so that `grep` can read it.

**Where the skill differs from the gist (all known).**

| Point | Gist | `chief-of-staff` at `4ae2317` |
|---|---|---|
| Voice | "Kept current" | "A historical record. It never claims to describe how things are now" (`5beb35b`, 2026-10-19) |
| Old claims | Lint finds stale claims | Add a new dated claim. Mark the old one superseded. "Don't rewrite the old one" |
| `log.md`, schema file | Yes | Not named |
| Raw layer | `raw/`, immutable | "Sources … are never copied in or edited. Pages cite them" |
| Karpathy | Author | Not named in the skill. Pocock's posts say "Karpathy-style" |

"Write history, not the present" is Pocock's rule, not Karpathy's (known). The skill gives one reason: the wiki "cannot keep up with the code". I found no other reason from Pocock.

## 2. Scope

**What the skill says (known).**

- The wiki is "a folder of interlinked markdown pages that outlives this session". The chief writes it.
- "Look for a shared wiki first, one the user or other agents already keep." If there is none, "agree a location with the user before the first write". This text came with `5ae4816` (2026-10-31). Before it, the text said "If the goal has no wiki yet, agree its location".
- One session pursues one "long-running goal".
- The skill has no goal folders, no scope rule and no folder name. It does not name the readers. The text implies the next session, other agents and the user.

**What Pocock does (known, from his posts).**

| Date | Post | Scope |
|---|---|---|
| 2026-06-29, and again 2026-10-19 (`2071607607714890181`, `2112131671097020895`) | "A personal, entirely agent-managed Karpathy-style wiki". X, Discord and Gmail go in "every few hours". It is "the environment for all of my future loops". On 2026-10-19 it is "critical infrastructure for my business" | One user |
| 2026-07-09 (`2075315723631497705`) | A 15-minute podcast each weekday on what entered the wiki. It is also "a vibe check on how well the stuff in the wiki reflects reality". A CLI lets agents query it | One user |
| 2026-09-09 (`2097638166232457451`) | "Give each 'deliverable' a Karpathy-style LLM-wiki". A deliverable is a course or a talk of more than one day. Several threads share the wiki. He dictates braindumps and reacts to HTML reports. A "linting" skill runs at night with many subagents | One deliverable |
| 2026-11-03 (`2117570847548268820`, `2117571010660581804`) | "Long-running threads covering a single domain". Example: "my Cohort 005 agent". "Each agent builds up a wiki over time so I can drop the conversation history and switch harnesses" | One domain agent |
| 2026-11-03 (`2117570433134313951`) | "Thinking about trialling" a domain orchestrator, "maybe with my /chief-of-staff skill" | A plan, not a practice |
| 2026-11-05 (PR #1387 body) | "A chief of staff running against personal-wiki". Ticket numbers #698 and #686 of `mattpocock/personal-wiki` | The personal wiki |

- **known.** Pocock keeps at least three scopes: one user, one deliverable and one domain agent. An agent writes each wiki. He reads reports, hears a podcast or talks to the agent.
- **inferred.** The scope follows the unit of work. No post says that one chief uses several goal wikis, or that one wiki holds goal folders.
- **inferred.** The public skill is not the whole practice. The closing comment of PR #1387 moves one rule to a private skill.
- **I don't know.** Whether his personal wiki has a folder for each goal. Whether the domain agents of 2026-11-03 run the `chief-of-staff` skill. I found neither wiki public.

## 3. The tracker

**Chief of staff (known).** The skill uses the tracker for one thing. For "large chunks of work" the chief writes a spec and tickets, publishes both "to the issue tracker", and runs `implement-spec`. The memory is not in the tracker. The skill does not name wayfinder.

**Wayfinder (known).** A map is one issue labelled `wayfinder:map`. Its tickets are child issues. The map is "an index, not a store". A decision lives in one place, its ticket. A session claims a ticket by assignment. The skill finds the tracker through the tracker doc, "Wayfinding operations" section. The docs say the agent finds that doc "through the pointer in your `CLAUDE.md` / `AGENTS.md`, not at a fixed path". GitHub Issues is the best-supported tracker. Local markdown is allowed and "not recommended, because material stored in the repo tends to persist by accident".

**The two compared (inferred).** A wayfinder map closes when nothing is left to decide. A chief-of-staff goal has no close step. Wayfinder has a human who picks tickets. The chief picks its own work.

**The direction of travel (known).** PR #1369 opened as a draft on 2026-11-01 and is still open. Its body says: "Source: Matt's voice note of 2026-11-01. Chief-of-staff may replace wayfinder, because the wiki outlives the session where a closed wayfinder map doesn't." The PR adds a `destination.md` wiki page with Destination ("the current dated claim, above the ones it superseded"), Fog of war and Out of scope. It adds "frontier" as the word for the next task. It leaves out Decisions so far ("the wiki already gives each decision its own page"), tickets, claiming, blocking and ticket types. Deprecating wayfinder is "a follow-up for Matt to decide". The PR says "The change has not been run yet." Claude Code wrote the PR text for Pocock (inferred from its footer).

## 4. Changes after `4ae2317`

- **known.** No commit to `main` changes `skills/in-progress/` or `skills/engineering/wayfinder/` after `4ae2317`. `git log 4ae2317..origin/HEAD` shows 0 commits (fetched 2026-11-05).
- **known.** Two pull requests touch the skill. Neither is merged. #1369 is the destination page (point 3). #1387 added a `### Factory` rule on when to hand work to a software factory. Pocock opened it and closed it 22 minutes later on 2026-11-05: "this rule moves to a private factory-foreman skill instead of the public repo".
- **known.** The last wayfinder change is `8295b8e` (2026-10-07): no triage labels, no placeholder references, no research PRs.
- **known.** The skill was added 2026-10-05, and it stays in `in-progress/`, which "can change or disappear without warning" (`skills/in-progress/README.md`).
- **known, a correction.** The chief-of-staff report says the four procedure paths (`../retro` and the others) point at missing folders. That holds inside the upstream repo. PR #1363 says the paths are written for the installed layout, where every skill sits directly under `~/.claude/skills/`. They resolve after install.

## 5. What stops a wiki of dated claims from being read as current state

This section lists mechanisms. It does not recommend one.

| Mechanism | Source | Limit |
|---|---|---|
| A date on every claim | Skill | The reader must read the date |
| A commit and repo on a claim about code | Skill | Nothing checks the pin. The skill is prose with no script (known) |
| No "currently", "now" or "is" about code without a date or commit | Skill | Covers claims about code only |
| Supersede, do not rewrite. The old claim stays, marked superseded | Skill | The old line stays in the page. A reader who skips the mark reads it as current (inferred) |
| The current claim on top, above the superseded ones | PR #1369, `destination.md` | A draft. Covers one page |
| Pages cite sources and never copy them | Skill | The cited source can move or change |
| Lint on a schedule: contradictions, orphans, claims "newer sources have overtaken" | Skill, gist | The skill says only "suggest this as a schedule". It has no lint script |
| A human check against reality: the daily podcast | Pocock, 2026-07-09 | Manual. A personal wiki |
| A night lint skill with many subagents | Pocock, 2026-09-09 | I did not find the skill |
| `updated` date, `confidence` and `contested` in the front matter. Superseded pages go to `_archive/` | mrn-wiki `SCHEMA.md` | The owner's wiki. It moves the old page out of the read path. The skill keeps the old claim in the page |

Three facts:

- **known.** The gist wants a wiki "kept current". The skill wants history. I found no source where Pocock says why he changed the voice.
- **known.** No source reports a test of the history voice.
- **inferred.** The skill lists "a subsystem" as a page kind. A subsystem page is the kind with claims about code, so it is the page kind most exposed to drift.

The owner's account of the ADRs (`entities/chronicler-engine.md`) says a rationale artifact must be "edited in place and carry no superseded history". The skill says the opposite for a wiki. Ticket 05 decides which voice a wiki here uses.

## 6. The needs

Each cell tests the skill as written, and Pocock's practice where a source shows it.

| Need | Chief of staff | Fit |
|---|---|---|
| 1. See each goal and where it stands, without asking an agent | `index.md` lists the pages. PR #1369 adds `destination.md`, a map at low resolution. Both cover one goal. Nothing lists several goals | partly |
| 2. Put a one-off item in one obvious place | The skill is silent. A one-off item fits no page kind. Large work goes to the tracker | no |
| 3. Keep rules that span maps in one place | A preference page, or a shared wiki, holds each rule once. A rule is a dated claim with a history | yes, in one wiki |
| 4. Let a chief own one folder and find everything from there | The wiki is the folder. Pocock runs one agent for each domain with its own wiki. Spec and tickets go to the tracker, outside the folder | partly |
| 5. Close a goal as one unit | No close step. The goal is long-running, and PR #1369 calls it "unbounded". A wiki outlives a closed map | no |
| 6. Let a fresh agent find any record from one doc | "Read `index.md`" opens the wiki. The skill does not say how the chief finds the wiki. Wayfinder finds the tracker through the tracker doc | partly |

## Facts for tickets 04 to 08

**04, names** ([Decide the names of the work records](../issues/04-names.md))

- **known.** "Wiki" and `index.md` are words of both the gist and the skill. A well-known source uses "wiki" for this folder.
- **known.** Pocock's unit words: "goal" (skill), "deliverable" and "domain" (posts), "destination", "fog of war", "frontier", "map" and "ticket" (wayfinder, PR #1369).
- **known.** No source uses "expedition" or "errand". The skill names no goal folder.

**05, wiki contents** ([Decide what a wiki holds and what it never holds](../issues/05-wiki-contents.md))

- **known.** Page kinds: a decision, a recurring problem, a subsystem, a user preference. Sources are cited and never copied.
- **known.** PR #1369 drops Decisions so far because each decision has a wiki page. Wayfinder says a decision lives "in exactly one place", its ticket. Both rules give a decision one place, and they name different places.
- **known.** The mechanisms against a stale read are in point 5. The skill has a rule and no check.
- **known.** Pocock's deliverable wiki holds the integrated work itself, not only process notes.

**06, layout** ([Decide the layout of the folder](../issues/06-layout.md))

- **known.** The skill's wiki is flat: one folder, one `index.md` "grouped by category". It has no `maps/` and no `errands/`.
- **known.** Karpathy separates raw sources from the wiki. The skill separates them by citation, not by folder.
- **inferred.** "Shared wiki first" allows a wiki above the goal wikis, as `general` is. The skill gives no layout for it.
- **I don't know.** Any source on closed records of a wiki.

**07, maps to expeditions** ([Decide which maps go where](../issues/07-maps-to-expeditions.md))

- **known.** In PR #1369 the destination, fog and out-of-scope list of a goal move into a wiki page.
- **inferred.** Under that direction a map is a wiki page, not a folder with tickets. A layout that keeps a map whole with its tickets still fits wayfinder as it stands today.
- **known.** The skill has nothing on buckets or one-off items.

**08, names in docs and skills** ([Decide where the names are written and how skills find the folder](../issues/08-names-in-docs-and-skills.md))

- **known.** Upstream wayfinder finds the tracker through `AGENTS.md` and the tracker doc, "not at a fixed path".
- **known.** Upstream moves the same way. Changeset #1208 says `code-review` now "resolves the issue tracker through the provided tracker doc instead of a hard-coded path". At `4ae2317`, `to-tickets` still names `.scratch/<feature-slug>/issues/`. The local tracker template of `setup-matt-pocock-skills` names `.scratch/<effort>/map.md`.
- **known.** The `chief-of-staff` skill names no fixed wiki path.
- **inferred.** A later sync from upstream can bring direct `.scratch/` names back into `to-tickets`.
