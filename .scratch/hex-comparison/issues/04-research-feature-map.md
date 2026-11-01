# Research the Hex feature map and its proof records

Type: research (AFK)
Status: resolved
Blocked by: —

## Question

How does `docs/features/` describe behavior and its proof, and how does it stay in step with the code? Where does it differ from Chronicler's feature specs and their link to tests?

## Context

- **Hex sources.**
  - `docs/features/` has 10 files and 61k words, which is too much to read in full. Read `README.md` and `recovery.md` in full, and a sample of the other files.
  - The rules are in `AGENTS.md`, `## Feature Map` (lines 49–71):
    - organize by user capability
    - stable behavior ids
    - `ts` fences with tree diagrams
    - keep three states apart: built, a check exists, and someone saw it work
    - update the map in the same change
- **What to find.**
  - whether code or tests refer to the behavior ids (search for the ids in `src/` and `tests/`)
  - how a record of "someone saw it work" is written and dated
  - how often the maps change together with code (`git log`)
  - known defects and missing coverage, and how they are recorded
- **Chronicler side.**
  - the feature specs that `scripts/validate_feature_spec.py` reads
  - `docs/diataxis/`
  - the closed map `narrator-modes-and-options` (its spec-coverage tickets 12 and 18–20)
  - the closed map `http-test-spec-mapping`
  - the open map `statechart-models`, which proposes paired rule ids
- **Candidates this touches** ([findings asset](../assets/findings.md)): 2 (invariants in prose) and 3 (a feature map that states its proof).

## Answer

Hex keeps one map of 10 files and 61k words in `docs/features/`, organized by user capability. Each of its 142 behaviors carries a stable dotted id inside a `ts` tree diagram, and the map keeps three states apart: the behavior is built, a check exists, and someone saw it work. The third state is a dated paragraph that names the machine and the build, and the negative state is as explicit as the positive one ("Proof limit: …", "not yet observed"). No code, test or gate refers to an id, and no check reads the map, so drift survives: the index still describes the old Labs model, and 11 check names no longer resolve. Chronicler enforces the spec-to-test link by the `spec-coverage` gate step (554 scenarios, 31 files) and places invariants in Diátaxis Reference, but it has no record of what someone saw work, no proof limit beside a test, and it keeps known gaps in closed tickets. 15 practices recorded, 3 of them new.

Asset: [research-04-feature-map.md](../assets/research-04-feature-map.md)
