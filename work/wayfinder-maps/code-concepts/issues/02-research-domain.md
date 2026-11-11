# Research: the concepts in the domain

Type: research
Status: resolved

## Question

Which concepts does the code in `src/domain/` (about 3,200 lines without the unit tests) follow, and where do opposing or overlapping concepts meet in it?

Write the asset in the shape that the map's Notes give (Concepts, Meeting points, Shapes with no name, Questions). Do not rule on a concept or label a meeting point; write it as a question.

Dig the concepts out of the code. You get no list of concepts: do not open `assets/research-01-lens.md` or the other research assets. Read the code, and find the shapes that it repeats and the choices that it makes. Give each one the name that software-engineering books and well-known essays give it (a principle, an architecture style, a design style, a pattern or an idiom), and its opposite. When no such name fits, put the shape under Shapes with no name.

## Done when

Every module in the scope is read, or is named in the asset as sampled with the reason. Every row cites `path:line` evidence and has a known, inferred or guessed label.

## Answer

Asset: [research-02-domain.md](../assets/research-02-domain.md). The researcher read all 36 non-test files in `src/domain/` and sampled nothing. The researcher did not open the lens asset.

- **26 concepts.** The five with the strongest evidence (known): functional core, imperative shell (`game_rules::apply`, `JobPlan::for_request`); closed-world enum dispatch (the area has no `trait`, `dyn` or `async`); reference other aggregates by identity; single source of truth ("one home" comments); a sum type that carries only the data of its variant.
- **15 meeting points, each written as a question.** Examples: strict or lenient enum parse; newtype or raw id (`GameId` against `Game.id`); a pure rule or an ambient clock, log and environment; rules on the entity or as free functions; a sentinel value or `Option`.
- **14 shapes with no name.** Examples: a reason line plus raw text behind a disclosure; the kind / payload / row enum trio; doc-comment tags that guardrails read.
- **Two hazards to check (inferred, not tested):** dynamic room ids come from the clock in whole seconds (`map.rs`), and the Trigger order comes from `HashMap` iteration (`game_rules.rs`).
