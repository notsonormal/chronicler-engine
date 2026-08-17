# Guide layer in LayerRenderer and continue-path wiring

Type: task
Status: pending

## Question

Add the transient `Guide` layer to the prompt assembler and wire it through the continue path.

Per the design synthesis (`../research/04-design-synthesis.md`, Q2 + Q4 + Q9 + Q12):

1. Add a `Guide` layer rendered **last** in `LayerRenderer::render_and_fit` (`src/application/prompting/assembler.rs`), after `<PlayerInput>`. Wrapper text verbatim from Marinara/GG: `Take the following into special consideration for your next message: {guide}`. Guide wins recency over output-format and player input (Marinara model, Q2=A).
2. Add a transient `guide: Option<String>` field to `PromptContext` (not persisted to history — transience, Q4). The guide is never a `MessageEntry`.
3. Wire the continue path: a `/guide <text>` command (from ticket 07) sets the guide on `PromptContext` and runs `continue_narration` (not `process_action` — Q12=A: the guide replaces the player input, so the turn is a continue-with-guide). Verified continue is implemented (`actions.rs:27-28` → `action.rs:55`).
4. Surface: new generation (continue path) + retry; not retrigger (Q9=A). On retry, the replay blob (ticket 06) re-applies the guide.
5. Guide and impersonate are mutually exclusive per turn (Q8=A) — the dispatch must reject/redirect a guide when impersonate is active.

Grounding: Marinara pushes the guide after fully-assembled `finalMessages` (`generate.routes.ts:7140`); GG uses depth-0 (after last message). Chronicler's existing layer order already prioritizes recency (`<PlayerInput>` after output-format), so the guide-last placement is the consistent extension.

Blocked by: 06 (replay blob for retry), 07 (slash parser for entry).
