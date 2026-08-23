# Grill: Options-autogeneration design

Type: grilling
Status: pending
Blocked by: 03

## Question

Design the options-autogeneration feature: the engine generates pickable options the player can choose from. This is a first-class feature available in BOTH modes (not mode-gated — Round 2 Q2=B). Blocked by ticket 03 (prior-art research) so the design is informed by what other systems do.

### Sub-questions

1. **Trigger model.** Round 2 noted "push button to request choices" vs. "always give choices" vs. both. Decide: (a) on-demand only (a "give me options" button), (b) always (every narration turn appends options), or (c) both (a setting or per-turn toggle). Grill the UX weight: always-on options can feel like a rail; on-demand keeps player agency.

2. **Generation mechanism.** A new `GenerationPhase` (alongside `Narrating`/`Quantifying`)? A new agent (like the quantifier agent)? A separate LLM call after narration, or a side-channel in the narration call? The prompt must produce N coherent, distinct, non-overlapping options grounded in the current scene. Grill N (3? configurable?), the prompt structure, and whether options are generated from the narrator's view of the scene or as "what the player could do next."

3. **Rendering (HTMX).** How are options rendered in the chat window? A row of buttons below the last narration? A separate panel? Does picking an option highlight/disable the others? Confirm this is a new template + view-model surface, mirroring the swipe-controls pattern in `templates.rs`.

4. **Selection flow.** Does picking an option (a) submit its text as the player's input (triggering normal narration — the option *is* the chosen action), (b) submit a canonicalized version, or (c) do something else (e.g., advance without a new narration)? The natural model is (a): an option is a pre-written player input. Confirm this and work out the message-type (it becomes `MessageType::Input`, consistent with ticket 15's speaker axis).

5. **Retry of an option set.** Can the player regenerate the options (swipe-style: "give me different options")? If so, is it the existing retry machinery (ticket 17's `RetryMode`) extended, or a new path? Grill whether options-retry shares the swipe infrastructure or is separate.

6. **Interaction with steering features.** Can a player use guide or narrator-action in a turn that also has options? Can options be generated for an impersonate? This is fog on the map; this ticket resolves whether it stays fog or graduates into a sub-decision. Likely mode-agnostic, but confirm guide/options and impersonate/options compose or conflict.

## Notes for the session

- Read before grilling: `src/application/pipeline/pipeline_run.rs` (`phase_narrate`, `call_narrator`), `src/domain/model/state/generation_status.rs` (`GenerationPhase`), `src/application/agents/` (the quantifier agent pattern — a options agent may mirror it), `src/adapters/driving/http/templates.rs` (swipe-controls pattern for the rendering model), `docs/diataxis/reference/narrative/ai_steering.md` (steering-feature composition).
- Ticket 03's research asset (`research/03-option-generation-prior-art.md`) is the blocking input — read it first; it may resolve or reshape several sub-questions.
- Skills: `/grilling`, `/domain-modeling`, `/prototype` (a stub options UI may sharpen the rendering/selection design).
