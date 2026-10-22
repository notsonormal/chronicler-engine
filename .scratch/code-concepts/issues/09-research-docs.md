# Research: the concepts the docs name

Type: research
Status: resolved

## Question

Which concepts does the code in `docs/diataxis/` (about 1,800 lines) follow, and where do opposing or overlapping concepts meet in it?

Write the asset in the shape that the map's Notes give (Concepts, Meeting points, Shapes with no name, Questions). Do not rule on a concept or label a meeting point; write it as a question.

Dig the concepts out of the code. You get no list of concepts: do not open `assets/research-01-lens.md` or the other research assets. Read the code, and find the shapes that it repeats and the choices that it makes. Give each one the name that software-engineering books and well-known essays give it (a principle, an architecture style, a design style, a pattern or an idiom), and its opposite. When no such name fits, put the shape under Shapes with no name.

Here the evidence is the doc text, not code. Record the concepts each doc names or assumes, with `path:line`. Also record the shape of the docs themselves (Diataxis modes, DOC anchors, what a reference doc holds). Do not check the claims against the code; [the synthesis](10-research-synthesis.md) does that.

## Done when

Every module in the scope is read, or is named in the asset as sampled with the reason. Every row cites `path:line` evidence and has a known, inferred or guessed label.

## Answer

Asset: [research-09-docs.md](../assets/research-09-docs.md). The researcher read all 23 files under `docs/diataxis/` (1,775 lines; 8 explanation docs and 15 reference docs) and sampled `docs/AGENTS.md` (251 lines) as the source of the writing conventions of the docs tree. The researcher did not open the other research assets, `scripts/`, `src/` or `tests/`. The researcher did not check any claim against the code. The researcher checked the 430 citations with a script and corrected 3 of them: 2 wrong lines, and 1 quote that its line did not support.

- **74 concepts.** The five with the strongest evidence (known): ports and adapters with the dependency rule and a composition root (`reference/architecture_system.md:8,19-26`); single writer with an ownership table, and the Unit of Work as the save unit (`reference/architecture_system.md:44-59`, `reference/storage.md:44-54`); concrete type over trait, with one `Backend` enum and a decorator for failure injection (`explanation/rust_idioms.md:8`, `explanation/storage_design.md:12-16`); a fencing check on the job id with cooperative cancellation (`reference/game_flow.md:47-48,98`); two state stores for two lifetimes, one in memory and one on the Game record (`explanation/two-state-channels.md:8-20`). The docs themselves follow Diátaxis: 15 reference docs, 8 explanation docs, no tutorial and no how-to.
- **22 meeting points, each written as a question.** Examples: which rule governs a new persistence call, the port rule or the direct-call rule, with two stated reasons for a concrete `Storage` (MP1); `AppSettings` as a database row and as a global static (MP4); three atomicity mechanisms and two locks that nest (MP5, MP6); the Game record with two writers (MP7); a Reference doc that gives a reason and an Explanation doc that gives a mechanism (MP13); `{{…}}` as both a filtered pattern in player input and a template macro (MP15); "code-indexer" docs against seam identifiers (MP16).
- **13 shapes with no name, 12 questions.** One question (Q1): the ticket names DOC anchors, but the docs under `docs/diataxis/` contain none.
- **Hazards to check (inferred, not tested):** `explanation/rust_idioms.md:38` says what happens to requests at shutdown but not to a Job that runs after its request ended; `reference/coding_standards/integration_test_standards.md:152-153` says `AppSettings` lives in a global static, while `explanation/rust_idioms.md:22` says it is a database row read at the point of use; `reference/narrative/prompt_system.md:43` filters `{{…}}` in player input, and no doc says what happens to a closing `</PlayerInput>` tag in that text; `explanation/architecture.md:12` says "four LLM impls" while the diagram at `explanation/architecture.md:40-42` shows three; `reference/storage.md:84` lists `settings` as a standalone table, but the list at `reference/storage.md:102-103` does not describe it; `reference/coding_standards/guardrails.md:114` takes its rule numbers from a `.scratch` plan, and `docs/AGENTS.md:241` exempts plans from the doc rules.
- **Check by the coordinator:** spot check of four citations (`explanation/architecture.md:12` against `:38-43`, `explanation/rust_idioms.md:22`, `reference/coding_standards/integration_test_standards.md:152-153`) against the docs. All match. Correction to Q1 (known): the DOC anchors are in the code, not in the docs. Module headers in `src/` carry 210 lines of the form `//! [DOC: docs/diataxis/...]` (for example `src/application/ports/llm_provider.rs:1`), and `scripts/validate_docs.py` checks them. The researcher did not open `src/`, so Q1 is a scope effect, not a gap. Drift risk: six docs under `docs/diataxis/` had uncommitted edits during the run (`git status`), so the synthesis must re-check citations in those files.
