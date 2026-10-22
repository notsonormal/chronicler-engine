# Map: the concepts the code follows

Labels: wayfinder:map

## Destination

A catalogue in the repo, with a pointer from `AGENTS.md`, that names the concepts the Chronicler Engine code actually follows. Each concept has a pretrained name, its opposite, and evidence from the code. The catalogue also shows where opposing or overlapping concepts meet. The catalogue is descriptive: it tells an agent "this repo does X; the opposite is Y", so the agent follows X by default and chooses Y only on purpose. It is not a rule list.

## Notes

- **Waits for** the map [conceptual integrity across writers with no memory](../conceptual-integrity/map.md) (owner, 2026-10-22). That map writes a framework note and tests it on this catalogue in [Prototype: worked case, the concept catalogue](../conceptual-integrity/issues/07-prototype-worked-case-catalogue.md). This map resumes when that ticket is resolved. Its research assets are evidence for that map, and that map does not change them.
- **Questions first.** The owner is still looking for the questions, not for answers. A research ticket records what it sees and the questions it raises. It does not rule on a concept or label a conflict "drift" or "exception". The limits of the catalogue come later, from what the research drags out.
- **Execution allowed at the end.** The map ends when the catalogue is in the repo and `AGENTS.md` points to it (owner, charting session). Before that, tickets produce research assets and decisions.
- **Scope:** `src/`, `tests/` (with the `*_tests.rs` unit tests in `src/`) and `docs/diataxis/`.
- **What counts as a concept:** anything an LLM could do in more than one way, at three levels: **stance** (boundary discipline, single writer), **pattern** (ports and adapters, transaction script, functional core) and **idiom** (error enums, newtypes, enum dispatch, `spawn_blocking`). The test: a scout can cite a file that shows it. Process principles (how an agent works, for example "prove it works") stay out, because the code does not show them. The owner treats this as a start, not as a limit.
- **Origin of the idea:** [the pstack principles](assets/pstack-principles.md). pstack shows the *form*: a stance and its opposite, named so the name works as a leading word. It is not a source list of concepts. Its 24 principles are one author's selection, and the field of software design is much larger. pstack is also prescriptive and universal; this map is descriptive and specific to this repo.
- **Research asset shape** (every area ticket, so the synthesis can merge them): write `assets/research-NN-<slug>.md` with four sections.
  1. **Concepts.** One row per concept: pretrained name; its opposite or opposites; evidence (`path:line`, two or three sites); how far it holds in the area (everywhere, mostly, in one place); label known, inferred or guessed.
  2. **Meeting points.** Places where two opposing or overlapping concepts serve the same kind of problem in the area. Write each one as a question with its evidence. Do not judge it.
  3. **Shapes with no name.** Repeated shapes that have no pretrained name. Describe each one in one line, with evidence.
  4. **Questions.** Anything else the reading raises.
- **Concepts come out of the code.** A research ticket gets no list of concepts. The researcher reads the code, finds the shapes it repeats and the choices it makes, and names each one with the name that software-engineering books and well-known essays give it, and its opposite (owner, after Draft the lens).
- **Terms:** use the `CONTEXT.md` glossary for domain words (Game, Swipe, Job, Narration).
- **Skills:** `/writing-for-agents` (leading words, negation), `/codebase-design` (deep module, seam), `/domain-modeling`, `/grilling`.

## Decisions so far

- [Draft the lens](issues/01-draft-the-lens.md) — a list of 178 axes in 25 families. The owner then ruled that the lens is not an input: the researchers dig the concepts out of the code. The asset stays as a record.
- [The concepts in the domain](issues/02-research-domain.md) — 26 concepts, led by functional core, imperative shell and closed-world enum dispatch; 15 meeting points such as newtype or raw id and pure rule or ambient clock.
- [The concepts in the application layer](issues/03-research-application.md) — 35 concepts, led by the single writer principle and the dependency rule; 19 meeting points such as storage with no port and three error envelopes; four concurrency hazards to check.
- [The concepts in the driven adapters](issues/04-research-driven-adapters.md) — 51 concepts, led by ports and adapters (with no port for storage), Data Mapper with JSON columns and a Unit of Work scoped to one Game; 18 meeting points such as three atomicity mechanisms and the domain serde shape as the stored format; seven hazards to check.
- [The concepts in the driving adapters](issues/05-research-driving-adapters.md) — 49 concepts, led by hypermedia-driven HTML fragments, a front controller with RPC-style routes and asynchronous request-reply; 26 meeting points such as one `Validation` refusal answered at 400 and at 200, "an unchecked checkbox" meaning three things, and three ways to mark markup as trusted; seven hazards to check.
- [The concepts the docs name](issues/09-research-docs.md) — 74 concepts, led by ports and adapters with a concrete `Storage`, the single writer with a save unit, and Diátaxis with a Reference–Explanation split (no tutorial, no how-to); 22 meeting points such as `AppSettings` as a row and as a global static, two nested locks, and a Game record with two writers.
- [The concepts in the test infrastructure](issues/07-research-test-infrastructure.md) — 56 concepts (21 of them rules the guardrail tests enforce), led by test tiers defined by what is faked, a strict `ScriptedModel`, shrink-only seam-contract lists and test hooks compiled into production behind a default-on `testing` feature; 20 meeting points such as five ways to fake the model, four ways to seed a Game row, and allowlists that go stale without failing.
- [The concepts in the composition root and the cross-cutting code](issues/06-research-composition-and-cross-cutting.md) — 60 concepts, led by a Composition Root with pure manual dependency injection, one crate-wide typed error enum translated at each layer boundary, and lint-enforced "no panics in production"; 24 meeting points such as per-call against boot-time settings reads, seven error types on one failed request, and skip against stop on a bad seed file; eight hazards to check.
- [The concepts in the test bodies](issues/08-research-test-bodies.md) — 47 concepts (sampled), led by tiers split by what is faked, ports and adapters as the test seam (a strict `ScriptedModel` and real in-memory SQLite), behavioural assertions on read seams, and `SCENARIO`/`CASE` tags; 23 meeting points such as three fakes behind the LLM port, handler behaviour tested at two tiers, and `test_sqlite_` twin tests that differ only in the storage constructor; six hazards to check.
- [Merge the areas and find where concepts meet across them](issues/10-research-synthesis.md) — 97 merged concepts (19 stance, 58 pattern, 20 idiom; 13 hold in five or more code areas, led by test hooks behind the default-on `testing` feature, fail fast and ports and adapters with no storage port); 22 meeting points across areas such as four atomicity mechanisms and two writers of one Game record; the docs against the code: 23 places where the code follows a doc concept in part or not at all, and 25 concepts that no doc names.

## Not yet specified

- **The limits of the catalogue.** Which concepts earn a place, and at which level. The owner is not sure yet and wants the research to show it.
- **What to do at a meeting point.** Whether each place where opposing concepts meet is a scoped exception, drift, a leftover of the rebuild, or open. Some may need an owner ruling. Fixes in code may be a separate effort.
- **The agent-facing form.** One file or a section of `CODING_STANDARDS.md`; the `AGENTS.md` pointer wording; leading words or short lines; how the "opposite" is written without the negation failure that `/writing-for-agents` names.
- **How the catalogue stays current.** Concepts change slower than structure, but they change.
- **The outside check.** How we know the catalogue is right and useful. Two ideas from charting: a different model family names the concepts of one area without the catalogue, and we compare; or a fresh agent re-does past tasks with and without the catalogue. A third idea: compare the research with the lens asset from [Draft the lens](issues/01-draft-the-lens.md), to find canon concepts that no researcher named.

## Out of scope

- Process principles: how an agent works, plans or verifies. The code does not show them.
- Enforcing the principles on top of the work, as pstack's `/poteto-mode` does.
- `scripts/`, `build.py`, the front-end files in `assets/` and the other docs outside `docs/diataxis/` (owner, charting session).
