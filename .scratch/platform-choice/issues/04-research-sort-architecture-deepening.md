# Research: sort the open architecture-deepening tickets against the new design

Type: research
Status: resolved
Blocked by: —

## Question

For each of the ten open tickets of the [architecture-deepening map](../../architecture-deepening/map.md), which result applies?

- **Settled by the design.** Target design version 2, sketch 25 or sketch 24 already answers it. Name the section.
- **A ticket in this map.** The question is still open and the build plan needs the answer. Give the sharp question, its type and its blockers.
- **A later effort.** The question is real but outside the build plan. Say why.

The open tickets: 01 Research the Storage seam constraints, 02 Deepen Storage into a StorageBackend trait, 03 Replace CRUD services with a Repository port, 06 Deepen PromptAssembler, 11 One generation-status strip, 12 Share one Agent call core, 14 One definition of the connection card, 15 One fragment error policy, 16 Bootstrap through GameCatalogue, 17 Flatten LLM transport.

## Context

- [Decide: keep Rust and build the new design in place](03-decide-keep-rust-build-in-place.md): this map supersedes `architecture-deepening`, because the new design changes the architecture that map deepens.
- First estimate from the charting grilling (inferred, not checked): 01 to 03, 06 and 12 overlap partly (the design covers the game-state records and the Agent hooks, not World, Persona, Settings, Preset storage or the inside of the Agents); 11 and 16 are probably settled (derived status, §6 view reads; create and reset); 14, 15 and 17 have no overlap.
- The design: [target design](../../turn-state-model/assets/target-design.md) version 2, [sketch 25](../../turn-state-model/assets/sketch-25-new-design.md) (with its §7 fate tables), [sketch 24](../../turn-state-model/assets/sketch-24-agent-hooks.md). Turn-state-model ticket 24 already closed seven `architecture-deepening` tickets: read its lines in that map's Decisions so far for the pattern.
- Each `architecture-deepening` ticket has a `Current state (2026-10-04)` section. Check it against the code at today's tree.

## Done when

- An asset `../assets/research-04-architecture-deepening-sort.md` with one row per ticket: result, evidence (design section or file), and for "a ticket here" the drafted question.
- The resolution creates the new tickets in this map and graduates the matching fog.
- The `architecture-deepening` map gets a resolution: each open ticket closed with a pointer, and the map marked superseded by this map.

## Answer

Resolved 2026-10-16 (AFK research). Asset: [research 04](../assets/research-04-architecture-deepening-sort.md). Code read at commit `d067420b`.

| AD ticket | Result | New home |
|---|---|---|
| 01 Storage seam constraints | ticket here: failure injection cannot fail half a `Tx` today, and arch-lint forbids storage to import ports | [Research: the storage constraints that the rebuild must keep](10-research-storage-constraints-for-rebuild.md) |
| 02 StorageBackend trait | ticket here: the build writes about 330 new storage lines for two backends | [Decide the storage seam of the rebuild](11-decide-storage-seam-of-rebuild.md) |
| 03 Repository port | ticket here: the repositories' shape is open in sketch 25 §3.1; the CRUD services join as the last question | the same |
| 06 PromptAssembler | settled in part (`build_narration_prompt` goes with arrival; the token budget comes in as a value); the rest is a later effort | — |
| 11 Status strip | settled: one rule in `GameViewQuery::status`; the two renderings are two HTTP contracts | — |
| 12 Agent call core | settled in part (the window and the settings come in as values); the shared prompt blocks and retry loop are a later effort | — |
| 14 Connection card | moot: commit `6916556a` deleted the builder; the card exists once | — |
| 15 Fragment error policy | settled for the build's routes (`Refusal` → `200`/`400`/`500`); the rest is a later effort | — |
| 16 Bootstrap through GameCatalogue | creation settled (sketch 25 §3.5); the lookup by World goes to the storage ticket | [Decide the storage seam of the rebuild](11-decide-storage-seam-of-rebuild.md) |
| 17 LLM transport | later effort: no file of the build, no test seam | — |

**Map changes.** Tickets 10 and 11 created; 07 and 09 now also wait on 11. The fog "The design beyond scope B (the edges)" graduates: storage into tickets 10 and 11, and the other edges need no design for the build (asset §3). The later efforts go to the map's Out of scope. The `architecture-deepening` map is closed as superseded, each open ticket with a pointer.

**Correction to the first estimate.** The charting grilling guessed that 14 had no overlap. It is moot. The estimate did not see the arch-lint conflict of a Repository port (`arch-lint.toml:164-169`).
