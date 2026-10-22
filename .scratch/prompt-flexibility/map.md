# Map: Prompt flexibility

Labels: wayfinder:map

## Destination

Every part of every story and state LLM call is decided: an author can change it through `data/` JSON or the existing UI, or it is a written engine rule with its reason. How a fixed test setup runs through Autoplay and what its reading pack holds is decided, and a prototype has made one reading pack on the current engine, so two setups can be compared. A build plan for the engine changes is ready.

## Notes

- **Origin.** The real-LLM UI pass of the Autoplay review fixes (2026-10-22) showed faults in the prompts. Most of them sit in the parts of a call that the engine fixes in code, not in the preset text. Autoplay is the first tool that makes a long story with no player, so it is the first way to test the prompts. Evidence: [story transcript](assets/autoplay-story-transcript.md), [the stored Impersonate prompt of Input #93](assets/impersonate-prompt-157.txt), [the review of the Impersonate output](assets/impersonate-output-review.md).
- **Two kinds of work.** This map is the engine work: make every part of a prompt changeable, and make trials possible. The authoring of World data, cards and presets is a second effort that uses this tool.
- **Scope of the calls (owner, charting).** All story and state calls: the Narrator role (Narration, Event Narration, the regenerate of a user Input, Impersonate), the Quantifier and Options. The connection test is not in scope.
- **"Changeable" (owner, charting).** An author can change it in `data/` JSON or in the existing UI. A new UI is not part of this map.
- **ST and ME (owner, charting).** SillyTavern and Marinara Engine are a reference: we read their source and their default prompts. A live run of either app is not decided (see Not yet specified). Pinned commits: SillyTavern `06bde939fb1e9c4c8d8641d810f0a916b5bce127`, Marinara Engine `457e827a3d3291be978121f8c50c98c5dd9d644d`. Clone them outside the repo (for example `/tmp/research/`).
- **Judging (owner, charting).** It is too early to define what a good output is. In this map a comparison gives the owner a reading pack: the transcripts and the prompt of each call, side by side. The long-term direction is an LLM judge that sorts and an owner who decides.
- **Known engine defects wait (owner, plan review).** The global `response_length` reaches the Impersonate call, and an empty `<PlayerInput>` block is its last text. Neither is fixed before the build plan: [Decide which parts of a Narrator-role call an author can change](issues/06-the-layout-model.md) and [Decide the parameters of each call](issues/07-call-parameters.md) decide them. The first trial prototype runs on the current engine and shows both.
- **Names.** No `CONTEXT.md` changes for now (owner, charting). "Test setup" and "trial" are working names only: a test setup is a fixed World, Persona, opening, presets, connections and number of cycles; a trial is one execution of it through Autoplay.
- **Existing references. Start from these; do not redo them.** `docs/external_applications/` (`sillytavern_prompt_system.md`, `marinara_engine.md` with its "Game Mode Prompt Architecture", `marinara_engine_system_prompt.md`), `.scratch/_closed/steering-and-guided-generation/research/03-sillytavern-core.md`, and the engine rules that already have a written reason in `docs/diataxis/explanation/prompt_system_design.md` (system and user halves, the style and format sections after the history, XML tags as content labels, two tiers of NPC cards).
- **Skills.** `/grilling` for grilling tickets, `/research` for research tickets, `/prototype` for the prototype ticket. Read `docs/diataxis/reference/narrative/prompt_system.md` and `docs/diataxis/explanation/prompt_system_design.md` first.
- **Standing rules.** Plan, don't do: the map ends with a build plan. The owner wants plans written directly in the repo, not through Plannotator. Any new API takes a `GameId` and adds no read of `ActiveGame` (per-tab rule from the Autoplay work).

## Decisions so far

## Not yet specified

- **The criteria and the judge.** What makes an output good, and how an LLM judge sorts trials for the owner. Revisit after [Find the tools that evaluate characters and prompts](issues/05-evaluation-tools.md) and after real trial transcripts exist.
- **A live run of ST and ME** with the original cards and the same model. Decide after [How ST and ME let an author change a call](issues/02-st-me-prompt-configuration.md) shows whether reading their source is enough.
- **A UI for the new changeable fields.** The build plan may add fields that only `data/` JSON can set.

## Out of scope

- The authoring of the World, the cards and the presets: the "father" error in the Persona card, the wording of the Impersonate preset, the Quantifier model choice for this World. These belong to the second effort.
- An automated, repeated comparison with ST and ME (owner, charting).
