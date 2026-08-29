# Grill: IF system preset design (Agency Rule surgery for elaboration)

Type: grilling
Status: resolved
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

## Answer

The IF system preset is a **sibling** of the novel preset (`data/prompt_presets/system/default.json`), not a rewrite. Four blocks change; six carry over verbatim. Draft asset: [`research/02-if-preset-draft.json`](../research/02-if-preset-draft.json).

**Posture (the core inversion).** Novel mode is a chatbot back-and-forth: the player writes the PC, the narrator world-builds, and the Agency Rule forbids the narrator from voicing the PC. IF mode inverts this. The player supplies a terse command; the narrator elaborates it into the protagonist's action, dialogue, and the world's adjudication. The PC is not a cipher in IF mode. Hard line, both modes: elaborate only what the command directed; do not invent uncommanded actions.

**The four changed blocks.**
1. `role` — the "except the protagonist, who is played by the user" clause is replaced with a direction/elaboration clause: "The player directs the protagonist through terse commands; you elaborate those directions into the protagonist's actions, speech, and the world's response." Opener changed to "You are a parser-style interactive fiction narrator..." to distinguish from the novel preset's "interactive fiction author" (working phrase — final genre label deferred to the author).
2. `instructions` → input validation — one framing bullet added: "The player's input is a terse command, not rich prose." (Elaboration/adjudication live in the Agency Rule and role block, not duplicated here.)
3. `instructions` → narrative, Agency Rule — inverted: "The player directs the protagonist through terse commands. You elaborate those directions into the protagonist's actions, speech, and the world's response. Voice the protagonist's commanded actions and dialogue directly. Do not invent actions the player did not direct. The protagonist's interior — thoughts, feelings — is expressed through the player's commands and choices." The novel preset's "Never write, assume, or infer the player's actions, thoughts, or feelings" is removed. The closing "Never end with questions or prompts for action. Never suggest possible actions or choices" is kept.
4. `output_format` — opener changed: "The player's next command is provided above. Your job is to elaborate it into the protagonist's action and narrate what happens now." The GPTisms/no-repeat tail carries over verbatim.

**Carried over verbatim.** State tracking, world dynamics, dialogue, general rules; `writing_style` (keeps the `{{narrative_perspective}}`/`{{narrative_tense}}` macros — posture is per-game per ticket 01, so the preset must not hardcode second person); the GPTisms/no-repeat tail of `output_format`; `is_default: true`; `preset_type: System`.

**Mechanics (settled by code, not grilling).** Preset id `system_if_default`; seeds under `data/prompt_presets/system/` (filename `if_default.json`); `is_default: true` (semantics unchanged — "built-in and protected," not "the active default"; the per-mode registry from ticket 05 selects which preset each mode uses). The `is_default` field is hardcoded `true` by the seeding loader (`src/bootstrap/run.rs:284`); the JSON field is ignored on seed.

**Decisions on the ticket's sub-questions.**
- Q1 (what carries over vs. changes): four blocks change (above); the Agency Rule is removed, not trimmed.
- Q2 (elaboration contract): the narrator voices the PC's commanded actions + dialogue + the world's adjudication, driven by the command. This reframes the ticket's original (a)/(b)/(c) options.
- Q3 (input validation under elaboration): prompt-instructed, LLM-interpreted. The novel preset's "narrate the failure, confusion, or the physical reality asserting itself" is carried over; no parser-IF rejection style is hardcoded.
- Q4 (perspective and tense): `writing_style` keeps the macros; posture flows from the game per ticket 01.
- Q5 (preset id and seeding): `system_if_default`, `is_default: true`, seeds under `data/prompt_presets/system/`.

**Impersonate in IF mode (Q1, round 2).** Impersonate stays **available in both modes** — no mode-gating, no adaptation. Rationale: keeping it is simpler than restricting it; the IF default already voices the PC, so impersonate is redundant but harmless, and a player wanting a full-voice PC monologue can type it as a command. This confirms the map's existing Out-of-scope entry (impersonate available in both modes); no map amendment. The impersonate preset is unchanged.

**Interiority and failure style (Q2/Q3, round 2).** Both are prompt-instructed, LLM-interpreted, not engine-enforced. The preset carries guidance (interiority expressed through commands/choices; narrate failure per the carried-over input-validation rule); the LLM interprets. The engine does not draw a hard boundary.

**Inline suggestion.** The "never suggest possible actions or choices" rule is kept. The options UI (ticket 04) is the suggestion surface; the narrator's prose does not double-suggest.

**Graduation.** The IF system preset seed (authoring `data/prompt_presets/system/if_default.json` from the approved draft) graduates to a new task ticket **08**. The quantifier-perspective question (does the quantifier preset need an IF/second-person variant?) stays in the map's Not-yet-specified — it is not blocking (ticket 01 set the initial state to "same as Novel") and is not answerable until IF mode runs end-to-end.
