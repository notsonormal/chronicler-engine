# Other layouts: how other tools and people lay out agent work records

Research for ticket [03](../issues/03-other-layouts.md). Date of reading: 2026-11-05.

Each claim has a label. **Known** means a source says it. **Inferred** means I drew it from known facts. **Guessed** means I have no source. "I don't know" means the sources are silent.

This asset answers [the map](../map.md) and its [six needs](../map.md#notes). It is not a survey. Tickets 01 and 02 cover Claude Projects, ChatGPT Projects, Claude Code memory files, Pocock's chief of staff and the LLM wiki.

## Sources

- Repos, shallow-cloned to `/tmp/research/`: Cline `2db87a6`, Spec Kit `3ca735b`, Taskmaster `c0c98d3`, Backlog.md `3a296c0`, firstmate `b77b66c`, OpenRig `73cf813`, Elephant-Goldfish `0dfb5bf`, Pocock's skills `4ae2317`.
- The `kirodotdev/Kiro` repo holds no layout docs. I read `https://kiro.dev/docs/specs/` and a search result for `https://kiro.dev/docs/steering/` (2026-11-05).
- The mrn-wiki at `37db57b`: `entities/firstmate.md`, `entities/openrig.md`, `entities/elephant-goldfish.md`, `comparisons/ai-skills-landscape.md`.
- PARA: `https://fortelabs.com/blog/para/`. Johnny.Decimal: `https://johnnydecimal.com/10-19-concepts/11-core/11.01-the-system/` and `.../12-advanced/12.03-the-standard-zeros/` (both 2026-11-05).

## 1. Agent tools that keep records in a repo

### Cline Memory Bank

- **Layout. Known.** One folder `memory-bank/` with six files: `projectbrief.md`, `productContext.md`, `activeContext.md`, `systemPatterns.md`, `techContext.md`, `progress.md`. The agent may add files or folders. The rule that loads it is in `.clinerules/memory-bank.md` (`docs/best-practices/memory-bank.mdx`).
- **Units.** One bank for each project. The units are files. I found no unit for one goal or one one-off item.
- **Memory. Known.** The bank is the memory. The files differ by rate of change. `activeContext.md` "updates most frequently".
- **Closing. I don't know.** The doc names only "update memory bank" at a milestone.
- **Fresh agent. Known.** The rule says the agent "MUST read ALL memory bank files at the start of EVERY task".

### Kiro specs and steering

- **Layout. Known.** A spec is a folder `.kiro/specs/<name>/` with `requirements.md` (or `bugfix.md`), `design.md` and `tasks.md`. Steering is in `.kiro/steering/`, with a global copy in `~/.kiro/steering/`.
- **Units. Known.** Spec (feature, bugfix, quick), steering, task, hook.
- **Memory. Known.** Steering gives "persistent knowledge about your project". `tasks.md` shows live status for each spec.
- **Closing. I don't know.** The docs name no archive step.
- **Fresh agent. Known.** Steering loads without a request. The CLI opens a spec with `/spec <name>`.

### GitHub Spec Kit

- **Layout. Known.** The hidden `.specify/` holds tool-owned files: `memory/constitution.md`, and one slug folder each for the bug and assessment extensions (`.specify/bugs/<slug>/`, `.specify/assessments/<slug>/`). Each feature is a visible folder `specs/<NNN-name>/` with `spec.md`, `plan.md`, `research.md`, `data-model.md`, `contracts/`, `quickstart.md`, `tasks.md`.
- **Units. Known.** Constitution, spec, plan, tasks, feature, slug.
- **Memory. Known.** One constitution of "immutable principles" for all features.
- **Closing. I don't know.** I found no close step for a feature folder.
- **Fresh agent. Known.** Each command runs a script that prints `FEATURE_DIR` and the available docs as JSON. The command text does not name the folder. The feature folder name comes from the branch (`spec-driven.md`).

### Taskmaster

- **Layout. Known.** Hidden `.taskmaster/`: `config.json`, `state.json`, `tasks/tasks.json`, `docs/prd.txt`, `reports/`, `templates/`.
- **Units. Known.** Task, subtask, PRD, **tag**. A tag is a separate task list inside one `tasks.json`, with its own numbers. The docs suggest tags for "feature, branch, or milestone".
- **Memory.** No wiki. The state is the task list and the current tag.
- **Closing. I don't know.**
- **Fresh agent. Known.** The agent uses the CLI or the MCP server. It does not read the folder.

### Backlog.md

- **Layout. Known.** Visible `backlog/`: `tasks/`, `drafts/`, `milestones/`, `decisions/`, `docs/`, `archive/`, `config.yml`. `archive/` has one subfolder for each kind (`tasks/`, `drafts/`, `milestones/`, `docs/`, `decisions/`). One task is one file, `back-105 - Remove-dot-from-.backlog-folder-name.md`.
- **Units. Known.** Task, subtask (`back-100.1`), draft, milestone, decision, doc. A **milestone** is one file. A task names it in front matter (`milestone: m-8`).
- **Memory. Known.** Done tasks stay in Git "as a permanent record of what was attempted and why". `decisions/` and `docs/` are flat and cover the whole project.
- **Closing. Known.** Mark the task Done. Then `backlog task archive` moves it to `archive/tasks/`. The task keeps its ID and stays readable.
- **Fresh agent. Known.** `backlog instructions` prints the agent text.

### firstmate

- **Layout. Known.** The repo is a shared template. Private records are in gitignored folders: `data/` (durable), `state/` (runtime), `config/`, `projects/`. `FM_HOME` picks one instance. Plans are `docs/plans/<date>-<time>-<slug>-plan.md`.
- **Units. Known.** Captain, first mate, crewmate, secondmate, task (ship or scout), home, backlog item.
- **Memory. Known.** `data/captain.md`, `data/captain-shared.md` (shared with secondmates), `data/learnings.md` (curated), `data/<id>/report.md` (a scout's report). `/stow` curates "tiered startup memory with decay and cold archival" and enforces "each home's budget".
- **Closing. Known.** `bin/fm-teardown.sh` removes the worker and moves the backlog item to Done. The backlog keeps "only the configured recent Done entries".
- **Fresh agent. Known.** One `AGENTS.md` (50 KB) names each file. Each secondmate has its own home and backlog.

### OpenRig

- **Layout. Known.** The project tree is in the repo: `missions/<name>/slices/<NN-name>/`. A mission and a slice each hold `SPEC.md` and `PROGRESS.md`. A slice also holds `PROOF.md`. The topology tree (`rigs/<rig>/pods/<pod>/seats/<seat>/`) is outside the repo, under `$OPENRIG_HOME`.
- **Units. Known.** Project, **mission**, slice. Rig, pod, seat. A mission has `status: planned` or `active`.
- **Memory. Known.** Chain files. `LEARNED.md` is "what a POSITION has learned". `CRAFT.md` is how work is done. `CULTURE.md` is team norms.
- **Closing. I don't know.**
- **Fresh agent. Known.** The rule is "One filename, identical at every altitude of a tree". `rig context trace` reads that file at each level, from general to specific. The doc says: "Never hardcode the path — `rig config get topology.root` is the answer."

### Elephant-Goldfish and Pocock (short)

- Elephant-Goldfish keeps approved designs in `docs/designs/<feature>.md`. A skill, `egm-drift-report`, compares them to the source. It has no unit for goals or one-offs. **Known.**
- Pocock's local tracker is `.scratch/<feature-slug>/issues/<NN>-<slug>.md`. The unit word is **feature**. **Known.** The skills give no reason for the dot. **I don't know.**

## 2. Personal systems

**PARA** (fortelabs.com):
- **Known.** Four top folders: Projects, Areas, Resources, Archives.
- **Known.** A project is short-term "with a certain goal in mind". An area is a responsibility that needs "ongoing attention" and has no end. The split is by "has an end", not by "has a goal".
- **Work with no project. Known.** It goes in Areas. Reference material goes in Resources.
- **One-off items. Inferred.** PARA names no place for a small one-off. It would be a project.
- **Archive. Known.** One top folder. It takes anything inactive from the other three, including "projects you've completed or put on hold".

**Johnny.Decimal**:
- **Known.** Area, category, ID (`15.22`). Each level has no more than ten choices.
- **Work with no project. Known.** Standard zeros give every level the same entries: `.00` JDex (index), `.01` Inbox, `.02` Task & project management, `.03` Templates, `.04` Links, `.05` AI, `.09` Archive. The same set exists for a category, for an area, and for the whole system.
- **The rule. Known.** "Prefer the most specific zero." Use the item's own ID. If none fits, use the category inbox, then the area inbox, then the system inbox.
- **AI. Known.** `.05 AI` is "a dedicated space that the agent controls" for memory across sessions and per-level instructions.

## 3. Hidden or visible

| Tool | Folder | Reason given |
|---|---|---|
| Kiro | `.kiro/` hidden | I don't know |
| Spec Kit | `.specify/` hidden, `specs/` visible | I don't know. Inferred: tool-owned files hidden, reviewed features visible |
| Taskmaster | `.taskmaster/` hidden | Known: "keep your project directories clean", "hidden from most file browsers" (`docs/migration-guide.md`) |
| Backlog.md | `.backlog/` became `backlog/` | Known: hidden "causes issues with file referencing (e.g., Claude's @ tool) and user interaction outside the CLI" (task `back-105`). Old installs keep the dot through config |
| Cline | `memory-bank/` visible, `.clinerules/` hidden | I don't know |
| firstmate | `data/`, `state/` visible but gitignored | Known: "captain-private and gitignored" |
| OpenRig | `missions/` visible | I don't know |
| Pocock | `.scratch/` hidden | I don't know |

- **Inferred.** Tools hide what a tool owns (config, state, rules). They show what a person or agent must read and reference.
- **Inferred.** Backlog.md is the one tool that moved from hidden to visible. Its reason matches the map's finding that `rg` skips hidden folders ([charting facts](charting-facts.md)). I found no tool that moved the other way.

## 4. The needs against each approach

All marks are **inferred** from the facts above. `✓` meets the need. `~` partly. `✗` no. `?` the sources do not say.

| Approach | 1 See goals and status | 2 One home for a one-off | 3 Cross-goal rules in one place | 4 Chief owns one folder | 5 Close a goal as one unit | 6 Fresh agent finds a record from one doc |
|---|---|---|---|---|---|---|
| Cline Memory Bank | ~ one `progress.md` | ✗ | ✓ `systemPatterns.md`, `.clinerules/` | ✗ one bank | ✗ | ✓ "read ALL files" |
| Kiro | ~ `tasks.md` of each spec | ~ bugfix spec | ✓ `.kiro/steering/` | ✓ spec folder | ? | ~ steering loads, specs by name |
| Spec Kit | ~ no index | ~ `bugs/<slug>` | ✓ constitution | ✓ `specs/<NNN>/` | ? | ✓ script prints the folder |
| Taskmaster | ✓ `list` for each tag | ✗ | ~ `config.json` | ~ tag in one file | ? | ✓ CLI and MCP |
| Backlog.md | ✓ board | ✓ task or draft | ~ flat `decisions/`, `docs/` | ~ milestone is one file | ~ archive task by task | ✓ `backlog instructions` |
| firstmate | ✓ one backlog file | ✓ backlog item | ✓ `captain-shared.md` | ✓ one home each | ✓ teardown, retire a home | ✓ one `AGENTS.md` |
| OpenRig | ✓ `status`, `PROGRESS.md` | ? | ✓ chain files by level | ✓ mission folder | ? | ✓ same filename, `rig context trace` |
| Elephant-Goldfish | ✗ | ✗ | ~ | ✗ | ✗ | ✗ |
| PARA | ✓ Projects folder | ~ a project | ✓ Areas | ✓ project folder | ✓ move to Archives | ✗ no index doc |
| Johnny.Decimal | ~ the JDex | ✓ inbox at each level | ✓ `x0` management | ✓ category or area | ~ `.09 Archive` | ✓ JDex is the one doc |

**Inferred.** No approach meets all six needs. firstmate and Johnny.Decimal come closest. Need 5 is the weakest: six approaches have no close step I could find.

## 5. New ideas

Each idea is **inferred** from its source. None is tested in this repo. The working views already have: a visible folder, one folder for each goal, `general`, `wiki/`, `maps/`, `errands/`, and `_closed/`.

1. **"Most specific home first" (Johnny.Decimal 12.03). Need 2.** An item goes into the goal it belongs to. If no goal fits, it goes to `general`. The working views have both homes but no rule. The rule also settles an item that fits two goals.
2. **The same fixed entries at every level (Johnny.Decimal standard zeros). Needs 2, 5, 6.** Each expedition and `general` get the same names: index, inbox, wiki, archive. A fresh agent learns one shape.
3. **A status view computed from the files (Backlog.md board, Taskmaster `list`, the JDex). Need 1.** A script prints one line for each map from its `Status` and its open tickets. The repo already generates indexes this way (`scripts/generate_*_index.py`). The store stays plain files.
4. **One filename at every level (OpenRig chain files). Needs 3, 4, 6.** Rules and memory use the same name in `general` and in each expedition. A chief reads the general level first, then its own, by one fixed rule. This confirms the working view and makes it mechanical.
5. **Rules that children inherit, not copy (firstmate `captain-shared.md`). Need 3.** The `general` wiki is the one source. An expedition wiki lists only what differs. This targets the "Standing rules" copied into three maps ([charting facts](charting-facts.md)).
6. **Split memory by rate of change (Cline Memory Bank). Need 4.** One stable file (the goal brief). One file that changes each session. One progress file. A chief rereads only the fast file. Caution: Cline reads all files each task, which costs context. A size budget for each wiki limits it. firstmate's `/stow` enforces "each home's budget" (known).
7. **An archive with the same shape as the live folder (Backlog.md `archive/`, PARA Archives). Need 5.** Closing an expedition is one move. `_closed/` already exists, so the new part is only "same layout inside". Backlog.md keeps the ID after archive.
8. **One path resolver, no path in a skill (OpenRig `rig config get topology.root`, Spec Kit command scripts, Backlog.md `backlog_directory`). Need 6 and the build.** Skills call one script or read one key. A rename then changes one place. This is the strongest source for "skills point at the tracker doc".
9. **A file for a small errand, a folder when it grows (Backlog.md one file for each task, Spec Kit one folder for each item). Need 2.** An errand starts as one file. It becomes a folder when it gets assets. The names ticket can use this to avoid empty folders.
10. **The "has an end" test (PARA). Needs 5 and ticket 07.** Work with an end is a project and closes. Work with no end is standing and lives in `general`. Ticket 07 can try it on the 15 live folders.

## Facts for tickets 04 to 08

### 04 Names

| Tool | Unit words (known) |
|---|---|
| Cline | memory bank, project brief, context, progress |
| Kiro | spec, steering, task, hook |
| Spec Kit | constitution, spec, plan, tasks, feature, slug |
| Taskmaster | task, subtask, tag, PRD |
| Backlog.md | task, draft, milestone, decision, doc, archive |
| firstmate | captain, first mate, crewmate, secondmate, task, home, backlog, learnings, scout |
| OpenRig | project, mission, slice, rig, pod, seat, culture, chain file |
| Elephant-Goldfish | design, concepts brief |
| PARA | project, area, resource, archive |
| Johnny.Decimal | area, category, ID, JDex, inbox, archive |
| Pocock | feature, issue, ticket |

- No tool uses "expedition", "errand" or "general". **Known** for these tools.
- OpenRig uses **mission** for the middle unit. Backlog.md uses **milestone** for a goal-like unit. The charting facts say "mission" is taken by Factory and OpenRig.
- PARA and Johnny.Decimal both use "area" for standing work. **Inferred:** it fits `general`. I did not count its use in this repo.
- For the wiki, each tool uses a different word: memory bank, steering, constitution, learnings, JDex, chain files. "Wiki" is the owner's word.
- Backlog.md says "task" and Pocock says "issue" for a one-off. Both clash with words in this repo. **Inferred.**

### 05 Wiki contents

- Memory Bank: six files by role and rate of change.
- OpenRig: `LEARNED`, `CRAFT`, `CULTURE` at each level. Learned memory is "never shared or shipped as-is".
- firstmate: own preferences, shared preferences, curated learnings, a budget and decay.
- Spec Kit: one constitution for all features.
- Johnny.Decimal: index, inbox, templates, links, AI, archive at each level.
- No source gives a size limit as a number. I don't know a good one.

### 06 Layout

- Visible folders suit files that people and agents must reference. Backlog.md moved to visible for this reason. **Known.**
- Tools split tool-owned files (hidden) from records (visible). Spec Kit is the clear case. **Known.**
- Two order patterns exist: a numbered feature folder (`specs/<NNN-name>/`) and a numbered ticket file (`<NN>-<slug>.md`). **Known.**
- Johnny.Decimal caps each level at ten. This repo has 15 live folders today. **Known.**
- An archive can mirror the live layout. Backlog.md does. **Known.**

### 07 Which maps go where

- PARA's "has an end" test and Johnny.Decimal's "most specific home first" rule are two ready tests. **Known** as rules. **Inferred** as a fit.
- The two bucket maps (`autoplay-follow-ups`, `retro-issue-bucket`) match Johnny.Decimal's inbox and Backlog.md's `drafts/`. **Inferred.**

### 08 Names in docs and skills

- Three tools keep a path out of their instructions: OpenRig (a config key), Spec Kit (a script prints the folder), Backlog.md (`backlog instructions` and a configurable directory). **Known.**
- Pocock's upstream skills name `.scratch/` directly ([charting facts](charting-facts.md)). A rename leaves them out of date. **Inferred.**
- Backlog.md kept the old name working: old installs get `backlogDirectory: ".backlog"` written into the config (`back-105`). **Known.** The map's move script can use the same idea.
