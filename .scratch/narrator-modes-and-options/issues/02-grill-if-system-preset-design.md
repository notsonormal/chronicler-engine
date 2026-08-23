# Grill: IF system preset design (Agency Rule surgery for elaboration)

Type: grilling
Status: pending
Blocked by: (none)

## Question

Design the IF/CYOA system preset — the sibling of the novel system preset (`data/prompt_presets/system/default.json`) that allows the narrator to elaborate the player's terse input, where the novel preset's Agency Rule forbids exactly that.

### The core tension

The novel preset's `instructions` carry: *"Agency Rule: Never write, assume, or infer the player's actions, thoughts, or feelings. The player's speech lines must be in indirect speech."* This is the novel posture: the player supplies rich prose, the narrator world-builds around it, never narrating the PC.

IF/CYOA is the opposite posture: the player supplies a terse verb ("take vase", "go north"), and the narrator *elaborates* it ("You take the ancient vase from the dust-covered table."). The Agency Rule blocks exactly this elaboration. IF mode needs a preset that permits it — but without losing the novel preset's other safeguards (input validation, state tracking, world dynamics, accuracy-over-creativity, no Mary Sue, no plot armor).

### Sub-questions

1. **What carries over vs. what changes.** The novel preset has six rule blocks: input validation, state tracking, world dynamics, narrative, dialogue, general. Which survive into the IF preset unchanged, which need IF-mode rewriting, and which (the Agency Rule specifically) get removed or inverted? Grill each block.

2. **The elaboration contract.** How far does elaboration go? Does the narrator (a) restate the player's action in prose and adjudicate its outcome, (b) expand a terse verb into a fuller action the player implied, or (c) also narrate the PC's perceptions and involuntary reactions? Parser-IF convention (Inform 7, Zork) is roughly (a)+(b); novel-mode's allowance for "involuntary physical reactions" and "transitional beats" is the boundary. Grill where the IF line sits.

3. **Input validation under elaboration.** The novel preset treats player input as "an attempted action, not absolute reality" and narrates failure if it contradicts state. In IF mode, does a terse "take vase" when no vase is present become "There is no vase here" (parser-IF rejection), or a narrated failure ("You reach for a vase, but the table is bare.")? Grill the rejection-vs-narrated-failure style.

4. **Perspective and tense.** The preset's `writing_style` uses `{{narrative_perspective}}`/`{{narrative_tense}}` macros (shared with novel). The IF default convention is second person; the perspective-default *setting* interaction is ticket 01. This ticket decides only what the preset's `writing_style` text says — confirm it keeps the macros (so ticket 01's default decision flows through) rather than hardcoding second.

5. **Preset id and seeding.** Confirm the preset id (e.g. `system_if_default`), `is_default` semantics for the IF bundle, and that it seeds under `data/prompt_presets/system/` alongside the novel default.

## Notes for the session

- Read before grilling: `data/prompt_presets/system/default.json` (the novel preset — every rule block), `docs/diataxis/reference/narrative/ai_steering.md` (the steering-feature context), and parser-IF convention references (Inform 7, Twine, Zork) for genre-standard elaboration behavior. No dedicated research ticket — the genre conventions are well-known; cite them inline.
- This ticket is about the preset *content*. The mode *mechanics* (how the preset is selected) is ticket 01 — unblocked, can run in parallel.
- Skills: `/grilling`, `/domain-modeling`, `/prototype` (a prototype IF preset draft may sharpen the elaboration contract).
