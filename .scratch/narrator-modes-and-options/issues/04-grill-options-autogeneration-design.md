# Grill: Options-autogeneration design

Type: grilling
Status: resolved
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

## Answer

Grilled in three rounds (Q1–Q3 trigger/selection/steering; Q4–Q6 mechanism/setting/prompts; Q7–Q9 rendering/retry/impersonate), plus a throwaway UI prototype for the rendering question: `tmp/prototype-options-ui/index.html` (V4 = the chosen shape; delete after the implementation tickets land).

**The settled design:**

1. **Trigger: BOTH (Q1=C).** Always-on via a setting, plus an on-demand `/options` slash-menu item alongside `/guide`, `/narrator`, `/impersonate` — always available regardless of the setting.
2. **Always-on setting: per-game, inherited from a world default, default OFF (Q5=A).** Rides the ticket-01 posture-relocation pattern (author default on the world, player override per-game).
3. **An option IS a pre-written player input (Q2=A).** Picking one submits its text as `MessageType::Input` and runs the NORMAL narration pipeline. Orthogonal to impersonate — impersonate stays a separate, deliberately-invoked steering feature. Coherent in both modes: IF mode elaborates the terse picked option (the IF posture); Novel mode narrates the result without re-writing the action (the Agency Rule). An option's semantic is "a possible player action" (Roadway's action-domain shape), not a story branch.
4. **Options × steering: ORTHOGONAL (Q3=A, Q9=A).** Options never suppress guide/narrator-action and vice versa. `/options` works whenever scene history exists, regardless of the last message's type. Always-on auto-fires only after narration-producing turns — not after impersonate turns (an impersonate already IS the player acting). Edit-before-send is what lets a picked option compose with `/guide` in practice.
5. **Generation mechanism: an `OptionsAgent` mirroring the quantifier (Q4=A).** `ExecutionPhase::PostGeneration`, `BackendSelector::UseNamed("options")` (Roadway's cheap-model two-model pattern; fall back to the narrator backend when unconfigured), registered in `AgentRegistry`. Always-on runs it in `phase_post_generation`; on-demand enters via a new `Action::Options` variant on its own pipeline path, run against the current scene. Known seam cost: `AgentResult::StatePatch` doesn't fit a list-of-strings result — needs a new variant or a side-channel write (decide in ticket 10, record the choice).
6. **Prompts: new `PresetType::Options`, TWO seeds (Q6=A+B).** Default seed = CYOA-style `<suggestion>`-tagged single-sentence beats (`options_default`); second seed = Roadway-style numbered list with explicit domain-diversity instruction (`options_roadway_domains`). Count via a `{{option_count}}`-style placeholder, default 3. The output parser must accept BOTH shapes (tags and numbered list — CYOA's parser already does this as fallback).
7. **Rendering: a vertical stack of full-width option buttons docked above the input box (Q7=V4).** Options are part of the INPUT surface, not the last message — "a different type of input". Per option: **Use** (submit as input) and **✎ Edit** (copy into the input box for modification or `/guide` composition); plus **♻ regenerate** on the row.
8. **Persistence: the offered set is CURRENT GAME STATE (Q7c confirmed).** Survives a page reload mid-turn; NOT anchored to a message — no per-message option history in the story log, no greyed-out consumed set. Picking an option or advancing the turn replaces the set wholesale.
9. **Retry: re-run in place (Q8=A).** ♻ re-runs the `OptionsAgent` against the same scene and REPLACES the stored set. No swipe-style option-set history; `RetryMode { ReNarrate, ReImpersonate, UserRegen }` stays narration-only. Matches all prior art.

**Prior-art grounding (ticket 03):** separate-call generation, selection-as-input, and re-run retry are all attested (ST-CYOA, ST-Roadway); the domain-diversity prompt is Roadway's documented technique for distinct options. Swipe-style option-set retry and options×steering composition remain unattested anywhere — our choices there are fresh design, flagged as such.

**Graduated to implementation tickets:** 09 (data layer: `PresetType::Options`, two seeds, always-on toggle fields, migration) → 10 (`OptionsAgent` + `Action::Options` + parsing + always-on hook) → 11 (UI dock + Use/Edit/regenerate + `/options` menu item). Spec + integration tests = ticket 12. The "Options × steering interaction" fog item is RESOLVED by this ticket (orthogonal, always available on demand).
