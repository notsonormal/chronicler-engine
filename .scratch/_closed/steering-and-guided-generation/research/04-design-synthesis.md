# Design Synthesis: AI Steering & Guided Generation

Asset for wayfinder map `.scratch/steering-and-guided-generation/map.md`, ticket 04 (`issues/04-design-steering-feature-synthesis.md`). The resolved design for all three steering surfaces, ready for implementation tickets to graduate. Every decision is grounded in the three research summaries (`research/01-marinara-engine.md`, `research/02-guided-generations-extension.md`, `research/03-sillytavern-core.md`) and re-verified against current chronicler source.

## Domain boundary (framing)

Two of the three features share a surface but split on axis:

- **Guided generation** steers **content** — what the AI says.
- **Impersonate** substitutes the **speaker** — who says it.

They are distinct mechanisms, not unified. The third — **narrator action** — is neither: it is a permanent author directive, not a transient nudge.

## Guided Generation

- **What.** A transient per-turn steering instruction. The input text *is* the guide (no player action); the turn runs the continue path with the guide layered on.
- **Prompt layer.** A new `Guide` layer rendered **last**, after `<PlayerInput>`, using the Marinara/GG wrapper verbatim: `Take the following into special consideration for your next message: {guide}`. Guide wins recency over output-format and player input. *(Q2=A, Marinara model.)*
  - Verified: Marinara pushes the guide after the fully-assembled `finalMessages` (`generate.routes.ts:7140`); ST's default order is `chatHistory → jailbreak` (output-format wins, `PromptManager.js:2085`). Chronicler's existing layer order already prioritizes recency (`<PlayerInput>` after output-format), so Marinara's rule is the consistent extension.
- **Transience.** Never persisted as a `MessageEntry`. Stored only as a per-swipe replay blob on `Swipe`, so retry re-applies it. *(Q4=A, Marinara `GenerationReplay`.)*
- **Surface.** New generation (continue path) + retry; not retrigger. *(Q9=A.)* Reconciled with Q12: "new generation" means continue-with-guide (no player action), NOT `process_action` with a guide attached.
- **Entry.** `/guide <text>` slash command. *(Q10=A, Q13=A.)*
- **Retry.** The replay blob on the swipe re-applies the guide automatically.

## Narrator Action

- **What.** A permanent author directive from the omniscient voice, persisted in history and rendered distinctly.
- **Storage.** New `MessageType::Narrator` variant. A `MessageEntry` in history; the type is the discriminant. Keeps `MessageType::System` meaning "engine notice" (today: `"[System] NPC detection uncertain…"` at `pipeline_run.rs:200`). *(Q1=A, Marinara/ST.)*
- **Rendering.** Bare text, no speaker prefix. The `Narrator` branch in `render_history_layer` skips the `{sender}: ` format — a change to the current renderer, which forces a sender on every line (`sender.unwrap_or("Narrator")`, `assembler.rs:320`). Bare text is the chronicler-analog of Marinara's `role: system` + no-prefix: chronicler folds its whole history into one `<ConversationHistory>` block with no per-line roles, so the absent prefix is itself the narrator signal. *(Q5=A, Marinara-roleplay + ST convergence.)*
  - Verified: Marinara roleplay/game maps `narrator` → `role: "system"`, no prefix (`generate.routes.ts:1448`); ST suppresses the name prefix for `extra.type === 'narrator'` (`openai.js:580,586`). Correction to map fog: narrator **is** in ST (`/sys` alias `/nar`), not Marinara-only.
- **Entry.** `/narrator <text>` slash command. Persists the narrator message AND immediately triggers a narration generation — a continue, since no player action. Shares the "no-player-input generation" shape with guide and continue. *(Q11=B, ST `/sysgen`.)*
  - Verified: Marinara has NO manual narrator slash command (narrator rows created only by automated scene/game flows); ST is the only reference and splits `/sys` (add-only) from `/sysgen` (generate-then-add). B chosen on UX. Continue path confirmed implemented (`actions.rs:27-28`, `action.rs:55`).

## Impersonate

- **What.** Forces the AI to write the next turn as the player's persona. Distinct mechanism from guide — orthogonal axes (content vs speaker). *(Q3-bis=B.)*
- **Target.** Player character only. `/impersonate [text]` — the optional text is the direction; no persona picker. The ticket sketch's `<persona>` argument is dropped. NPC impersonation deferrable. *(Q3=A, all three repos.)*
- **Prompt structure.** A separate `PresetType::Impersonate` preset. When `impersonate=true`, the assembler selects the impersonate preset instead of the system preset. *(Q7=A, Marinara `impersonatePromptPresetId`.)*
  - Verified: the default system preset's voice apparatus is entirely narrator-framed and contradicts impersonate — Role: *"You are an interactive fiction author… acting as the narrator, the world, and every character… except the protagonist, who is played by the user"*; WritingStyle: *"Third-person limited… focused on the player character"*; OutputFormat: *"Your only job is to narrate what happens now."* (`data/prompt_presets/system/default.json`). These are the voice itself — they must be **replaced**, not toggled.
  - Verified: Marinara suppresses all non-marker sections during impersonate (`assembler.ts:367,505`); ST/GG's additive approach is incoherent (keeps "you are the narrator" + appends "write as the user" = direct contradiction).
- **Context layers.** Voice apparatus dropped (Role/Instructions/WritingStyle/OutputFormat); context layers kept (`<GameState>`, `<KnownNpcs>`/`<NpcsInRoom>`, `<WorldLore>`, `<ConversationHistory>`).
- **Persona layer.** `<PlayerCharacter>` is dropped; persona data is injected into the impersonate instruction at assembly time from `persona.sheet.*` — the same source `render_persona_layer` reads today (`assembler.rs:266-290`), relocated into the instruction rather than a standalone layer. `{{user}}` continues to substitute the player name only, as in the system preset (no new macro is needed). Single coherent voice instruction. *(Q7b=A, Marinara `DEFAULT_IMPERSONATE_PROMPT`.)*
- **Output.** Generated text saved as a player-voiced `MessageEntry`. The replay blob (impersonate=true + direction + preset) enables retry via swipe without deleting the player message. *(Q6=A, Marinara.)*
  - Verified: Marinara saves impersonate output as a `user` message with `extra.generationReplay` (`generate.routes.ts:6930`); on regenerate of that user message it reads the blob, re-sets impersonate, excludes the old text from context (`:1342`), re-runs, saves new result as a swipe (`:6950`).
- **Entry.** `/impersonate [text]` slash command. *(Q10=A.)*

## Prompt-layer coordination

Guide and impersonate are **mutually exclusive per turn**. A turn is either guided, impersonated, or plain — never both. Exclusivity dissolves the placement conflict: only one transient nudge ever exists, so it is always last. *(Q8=A, Marinara.)*

Verified: Marinara `applyGenerationReplayToRegenerateInput` strips the guide when impersonate is true (`generation-replay.ts:95-110`). Composition (guide + impersonate together) is net-new with no ported reference and deferrable to a later ticket.

## Replay blob (shared mechanism)

A unified turn-conditions blob on `Swipe`, mirroring Marinara's `GenerationReplay`. Fields:

- `guide: Option<String>` — the guide text (when the turn was guided).
- `impersonate: bool` — whether the turn was impersonated.
- `impersonate_direction: Option<String>` — the impersonate direction text.
- `impersonate_preset_id: Option<String>` — the impersonate preset used.

Lives on `Swipe` (not `GameStateSnapshot`):

- Verified: `Message` has no `snapshot_id` field — `set_snapshot_id` delegates to `active_swipe_mut().snapshot_id` (`message.rs:108`); the snapshot is associated with the swipe, not the message.
- `GameStateSnapshot` is documented as "Frozen game state" via `from_game_state` (clones `movement`/`narrative`/`scene`/`npc_encounter_log`). Steering is generation metadata, not world state; putting it on the snapshot would break the snapshot's "faithful freeze of game state" invariant.
- A swipe already owns its generation's state anchor (`snapshot_id`); the replay blob is the other half of "reproduce this generation" (the steering inputs). Same shape, not a new responsibility.

## UI

All three features enter via slash commands in the single input box. Typing `/` opens an auto-suggestion menu of available commands. No dedicated buttons. *(Q10=A, Q13=A, Q14=B.)* Matches ST's command-palette convention; diverges from GG's button-heavy model.

## Implementation ticket graduation

The design splits into these implementation tickets (created as map children):

1. `MessageType::Narrator` variant + `render_history_layer` no-prefix branch.
2. Replay blob field on `Swipe` + storage migration + retry replay.
3. Slash-command parser (replacing the `Action::FreeAction` stub) for `/narrator`, `/impersonate`, `/guide`.
4. `Guide` layer in `LayerRenderer` (final position) + `PromptContext` guide field + continue-path wiring.
5. `PresetType::Impersonate` + impersonate preset selection + persona-injection instruction + context-layer filtering.
6. Narrator generate-then-add path (continue trigger).
7. Slash-command auto-suggestion UI.
8. Research: existing specs + `tests/http/requires_migration` audit, then spec + integration-test authoring.
9. Documentation.
