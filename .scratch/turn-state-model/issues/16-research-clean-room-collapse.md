# Research: which known problems a clean-room design would collapse, at each scope

Type: research
Status: open
Blocked by: —

## Question

Which known problems in the generation model and the modules around it would one clean-room design settle in one go, and at which scope? The known problems are the 17 open candidates of the `architecture-deepening` map, this map's open tickets and seed defects, and the defects in the pipeline's history. The answer shows how much a clean-room design could collapse, and which scope buys it. The method and its inputs are the question of [Research: method and inputs for a clean-room design of the generation model](15-research-clean-room-design.md).

## Context

- **Shared roots.** [Research: survey the pipeline's shape for the fix-or-rewrite decision](12-research-pipeline-shape.md) found that six fixed and five open defects share the pointer-timing or writer root (an inferred grouping). It also found five `architecture-deepening` tickets (04, 05, 08, 09, 10) on the same modules. One design for the whole unit could settle these problems together, not one ticket at a time.
- **The `architecture-deepening` map** (`../../architecture-deepening/map.md`). It has 17 tickets, none resolved, ranked in its Priority table. Rank 1 is [Give the input buffer named generation transitions](../../architecture-deepening/issues/09-input-buffer-generation-transitions.md), unblocked. Its fog patch "Cross-cutting migration shape" asks whether the accepted deepenings land as one migration or as independent refactors. Its fog patch "Generation-state cluster ordering" groups tickets 04, 08, 09 and 10. Its Out of scope rules out "Rewrites not surfaced by the reviews". Its reports `assets/architecture-review.html` and `assets/architecture-review-2026-10-04.html` describe each candidate.
- **Terms.** `CONTEXT.md` deprecates "turn". Use Message, Swipe, Snapshot, generation, Action, Action Pipeline. Use the `/codebase-design` vocabulary for module shape (module, interface, depth, seam).

## Scope

1. **The problem list.** Make one row per problem, with its source:
   - Each `architecture-deepening` ticket 01–17. Read its Question and its `Current state (2026-10-04)` section. Open a report only when the ticket is unclear.
   - This map's open tickets [01](01-model-a-generations-states.md), [02](02-replace-system-messages.md) and [03](03-decide-where-the-room-lives.md), and the three defects in the [seed](../assets/seed-evidence.md).
   - The fixed and open defects in the history section of the [research 12 asset](../assets/research-12-pipeline-shape.md).

   For each problem, record its kind. A **behaviour problem** is one a client can observe. It can enter a clean specification as a requirement. A **code-shape problem** is a shallow module, a duplicated path or a wide interface. It stays out of the specification, because it describes the current code, and it is checked only when the clean design is compared with the code.
2. **The scope options.** Describe at least these units. For each, give the black-box boundary the specification must fix (what the clean team may assume), the size of the specification, the behaviour that only the code defines today (take the stale and loose spec claims from [research 11](../assets/research-11-docs-and-decisions.md)), and whether the unit fits this map's Destination or needs its own map.
   - **A. This map's model.** A generation's states, its Snapshot pointers, the game-state writers and the Retry forms. The narrator, Quantifier and Options calls are black boxes.
   - **B. The game-state lifecycle around the Action Pipeline.** A, plus the Message history, Swipes, Snapshots, the generation status, `GenerationGate`, arrival, and the HTTP handlers that write game state (edit, delete, swipe switch, reset).
   - **C. The engine core.** B, plus the Agents, Triggers, prompt assembly and storage. The HTTP views and the LLM transport are black boxes.

   A unit is too narrow when its clean design must assume an answer that belongs to the next unit (for example, the phase order or who holds the slot) and the specification does not give that answer. Apply this test to A and B.
3. **The collapse matrix.** The rows are the problems, and the columns are the scopes A, B and C. Give each cell one value:
   - **settled**: the clean design cannot have the problem, because the specification requires the opposite;
   - **moot**: the module the problem is about has no counterpart in the clean design;
   - **untouched**: the problem is outside the scope;
   - **open**: the clean design must still decide it.

   No clean design exists yet, so most cells are inferred or guessed. Label each cell known, inferred, or guessed.
4. **Coordination with `architecture-deepening`.** If a clean-room run goes ahead at scope B or C, compare three options for the overlapping tickets: they proceed as planned, they pause until the run ends, or they fold into a new effort. For each option, give the cost and the conflict with that map's Out of scope rule and its Priority table. List the overlapping tickets that a session can resolve before this research lands.

Report facts and options. Do not choose a scope, and do not decide whether to run a clean-room design. Do not edit the `architecture-deepening` map. Those are decisions for the owner.

## Output

An asset `../assets/research-16-clean-room-collapse.md` with:

- The problem list table: problem, source, behaviour / code shape.
- The scope options table: unit, boundary, specification size, behaviour only the code defines, fits the Destination (yes / no), too-narrow test result.
- The collapse matrix, with a count per scope of settled, moot, untouched and open cells.
- The coordination options and their costs.

Label each fact known, inferred, or tested. Cite symbols, not line numbers.

## Feeds

The map's fog patch "Fix or rewrite".
