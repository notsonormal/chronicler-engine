# Research: method and inputs for a clean-room design of the generation model

Type: research
Status: resolved
Blocked by: —

## Question

How would a [clean-room design](https://en.wikipedia.org/wiki/Clean-room_design) work in this repo? Which documents form the clean specification, which are contaminated, which role do the premise sources SillyTavern and Marinara Engine take, who works on each side of the wall, and what checks the wall? The scope of the design (which unit, and which known problems it settles) is the question of [Research: which known problems a clean-room design would collapse, at each scope](16-research-clean-room-collapse.md).

## Context

- **The method.** One team examines the original and writes a functional specification. A reviewer removes copied material from the specification. A second team that never saw the original designs from the specification only. The purpose there is legal. The purpose here is to remove anchoring: a design written without sight of `src/` shows which parts of the current shape the requirements force and which parts are history.
- **Why now.** [Research: survey the pipeline's shape for the fix-or-rewrite decision](12-research-pipeline-shape.md) found 108 commits in five months, 63 of them refactors, and a structural reshape about once a month. Each fix keeps the existing shape. No reference design exists to measure the code against.
- **Premise sources.** SillyTavern and Marinara Engine are the source of much of the application's premise. [Research: how SillyTavern binds state to swipes, retries and notices](13-research-sillytavern-swipe-state.md) and [Research: how Marinara Engine binds state to swipes, retries and notices](14-research-marinara-swipe-state.md) give their swipe-state models, and the two disagree. Marinara keys every store that must rewind by the pair `(messageId, swipeIndex)`. SillyTavern's `swipe()` never touches `chat_metadata`, so that state survives a swipe switch. `docs/external_applications/` holds older captures of both applications. Each capture marks itself "historical/reference, not authoritative".
- **Terms.** `CONTEXT.md` deprecates "turn". Use Message, Swipe, Snapshot, generation, Action, Action Pipeline. The map title keeps "turn" for history.

## Scope

1. **The method.** Read the Wikipedia article and its primary sources (the Phoenix BIOS account, *NEC v. Intel*). Record the roles (examiner, specification, reviewer, clean team), the wall, and what counts as contamination. *NEC v. Intel* found that independent designs converge where functional constraints force them. Record whether that test transfers: a part where the clean design and the code agree is forced; a part where they differ is a free choice, a defect, or a gap in the specification.
2. **Prior art for non-legal use.** Find uses of a clean-room or specification-only redesign to counter design anchoring. Find uses of LLM agents as the clean team, where a fresh context is the wall. Record what went wrong: leaks into the specification, specifications too thin to design from, a clean team that re-derived the old design. Note related methods only to separate them: N-version programming, Cleanroom software engineering (a different method with a similar name), the second-system effect.
3. **The input inventory.** For each candidate input, record whether it states required behaviour only (clean), describes the current mechanism (contaminated), or mixes both, and whether it is current. Candidates:
   - `CONTEXT.md`.
   - `docs/specs/*.md`, mainly `actions.md`, `swipe_new.md`, `swipe_switch.md`, `retrigger.md`, `failure_display.md`, `reset.md`.
   - `docs/diataxis/reference/game_flow.md`, `storage.md`, and the explanation docs `storage_design.md` and `two-state-channels.md`.
   - The owner rulings and the 18 prior decisions in [research 11](../assets/research-11-docs-and-decisions.md).
   - The [seed](../assets/seed-evidence.md) and this map's research assets 04–12 (code facts, so contaminated by design).
   - The premise sources: the assets of [research 13](../assets/research-13-sillytavern-swipe-state.md) and [research 14](../assets/research-14-marinara-swipe-state.md); each file in `docs/external_applications/` (`marinara_engine.md` §File Mapping maps Marinara files to old Chronicler module names, so check it for mixed and stale content); the closed prior-art research `../../_closed/steering-and-guided-generation/research/01-marinara-engine.md`, `03-sillytavern-core.md` and `04-design-synthesis.md`, and `../../_closed/guided-generations-architecture/research/marinara-terminology-cross-reference.md`.

   Take the stale and loose claims from research 11. Do not re-check them.
4. **The role of the premise sources.** Compare three roles for SillyTavern and Marinara Engine:
   - **Requirements.** The clean specification says "behave as the premise source does, unless an owner ruling says otherwise".
   - **Prior art.** The clean team may read their designs.
   - **Excluded.** The clean team never sees them, so it does not anchor on another system.

   Give the cost of each role. Then make one table of the questions this map asks: which state is bound to a Swipe, what a Retry cuts and restores, which writers may run during a generation, where notices live outside the Message history, and where the location lives. Give the SillyTavern answer, the Marinara answer, and the Chronicler owner ruling where one exists. Take the answers from the research 13 and 14 assets and their pattern lists. Do not open the two repos again.
5. **The procedure.** Sketch one clean-room run with this repo's agent tooling:
   - Who writes the specification. It is a session that has read the code.
   - Who reviews the specification for leaks, and with which check. Example: a list of `src/` identifiers and file names, searched for in the specification.
   - What the clean agent may read: an explicit allow-list.
   - How the run isolates the clean agent. Find every leak path. Known candidates: `AGENTS.md` loads on start and its structure index names `src/` modules and phases; `docs/AGENTS.md`; the skills under `.agents/`; `.scratch/`; git history; the session-history tools (`session_search`, `recall`); the Chronicler module names inside `docs/external_applications/`.
   - What the clean agent writes. It writes a design, not code (plan, don't do).
   - How the clean design is compared with the code. Each difference is a defect in the code, a gap in the specification, or a free choice.
   - What cannot be made clean. The owner has seen the code, so every owner ruling and grilling answer carries some of its shape.
6. **Order in the map.** Compare three orders: a clean-room run before [Model a generation's states, its Snapshot pointers and the game-state writers](01-model-a-generations-states.md), so that ticket 01 can compare its model against the clean design; after tickets 01–03, so that their owner rulings enter the specification; or in place of part of ticket 01. Give the cost of each.

Report facts and options. Do not choose a role for the premise sources, and do not decide whether to run a clean-room design. Those choices are grilling decisions for the owner.

## Output

An asset `../assets/research-15-clean-room-method.md` with:

- The method summary, with sources.
- The prior art.
- The input inventory table: input, clean / mixed / contaminated, current / stale, source.
- The premise-source roles and their costs, and the question table (question, SillyTavern, Marinara, Chronicler owner ruling).
- The procedure sketch, with the leak paths.
- The order options and their costs.

Label each fact known, inferred, or tested. Cite symbols, not line numbers.

## Feeds

The map's fog patch "Fix or rewrite", and the order of [Model a generation's states, its Snapshot pointers and the game-state writers](01-model-a-generations-states.md).

## Answer

Asset: [research-15-clean-room-method.md](../assets/research-15-clean-room-method.md). Supporting assets: [research-15-web-sources.md](../assets/research-15-web-sources.md) (the web research; this session re-fetched its key quotes) and [research-15-leakcheck.py](../assets/research-15-leakcheck.py) (the leak check). Tree `9cbf3112` plus the work tree. Tested facts come from the leak check over every candidate input, and from an isolation probe run on four model families.

- **Method.** Four roles: examiner, specification, reviewer, clean team. Contamination is access, not similarity. *NEC v. Intel* inferred "forced by constraints" where a walled-off clean implementation agreed with both originals. The test transfers only as an asymmetry. Here the spec writer and the owner sit on both sides, and Knight and Leveson showed that independent designs correlate. A run therefore yields a list of requirement-forced and history-forced decisions, not proof of independence. The disagreements are the result.
- **Prior art.** No named software practice hides a design to counter anchoring. The nearest are the IETF and W3C "two independent implementations" rules, the Jas paper ("the most informative divergences are … specification gaps") and the chardet 7.0.0 rewrite. chardet shows the four failure modes: a leak (the agent read an original file), training-data recall, a design document in place of a behaviour spec, and convergence for other reasons.
- **Inputs.** Only the specs come close to clean. `swipe_switch.md` and `failure_display.md` are clean. `actions.md` and `swipe_new.md` carry mechanism in domain words. `retrigger.md` and `reset.md` read storage functions. `CONTEXT.md` has no code names, but it defines **Snapshot**, **Action Pipeline** and **Agent** by the current mechanism. The Diátaxis docs keep seam identifiers by rule. Of the 18 prior decisions, 8 are behaviour rulings that a spec can carry, 8 are module shape that must stay out, and 2 are process.
- **Premise sources.** SillyTavern and Marinara give different answers to four of the five questions of this map, so "behave as the premise source does" needs a choice per question. Excluding them is only partial: three of four models describe SillyTavern's swipe storage from training data. None knows Marinara's binding or this repo.
- **Wall.** `pi --no-tools --no-context-files --no-skills --no-extensions --no-approve` gave no tools and no project context on all four models (tested). Still open: the cwd path in the system prompt, `ask_owner`, and training data. The asset lists 17 leak paths. It also notes a Herdsman trap: an inherited model whose provider comes from an extension switches extension discovery back on.
- **Order.** Ticket 01 is on the frontier now. A run before it costs about four sessions and gives ticket 01 a reference point. A run after tickets 01–03 checks rulings that were made with no reference point. A run in place of part of ticket 01 changes that ticket's shape.

**Decisions for the owner** (asset §7): whether to run; the scope (ticket 16); the role of the premise sources; the order; the wall; the reviewer and the owner's part during the run. They graduate into [Decide whether and how to run a clean-room design of the generation model](17-decide-clean-room-run.md).
