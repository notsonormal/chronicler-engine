# AI Steering & Guided Generation

**Status:** Planned  
**Created:** 2026-05-08  
**Priority:** Medium-High — improves user control over narrative flow  

---

## Goal

Implement AI steering mechanisms in `chronicler_engine` to allow users to "nudge" or "force" specific narrative outcomes. This includes porting the "Guided Generation" pattern from SillyTavern and Marinara Engine, as well as adding support for permanent "Narrator" instructions and "Impersonate" functionality.

---

## Background

Users often need to correct AI behavior or introduce specific plot points without those instructions being treated as character dialogue or permanent story text. 

1.  **Guided Generation**: Invisible, transient instructions used during a single generation (often a retry) to steer the output.
2.  **Narrator Command**: Permanent instructions added to the history that steer the story from an "omniscient" perspective.
3.  **Impersonation**: Forcing the AI to write from a specific persona's perspective (usually the player's).

---

## Proposed Changes

### Phase 1: Guided Generation (Transient Steering)

Add support for one-off instructions that are not saved to history but are included in the prompt for the next generation.

#### [MODIFY] [types.rs](file:///e:/John/Github/mrn-general/chronicler_engine/src/narrative/prompt/types.rs)
- Add `generation_guide: Option<&'a str>` to `PromptContext` and `PromptAssembler`.

#### [MODIFY] [assembler.rs](file:///e:/John/Github/mrn-general/chronicler_engine/src/narrative/prompt/assembler.rs)
- Implement `render_guided_generation_layer()` which wraps the guide in a `<Consideration>` block.
- Update `assemble()` to include this layer at the absolute end of the user prompt (after the Output Format layer or just before it) to leverage recency bias.
- Add `with_generation_guide(mut self, guide: &'a str)` to `LayeredPromptAssembler`.

**Example Block:**
```xml
<Consideration>
Take the following into special consideration for your next message: [USER_GUIDE_TEXT]
</Consideration>
```

---

### Phase 2: Narrator Command (Permanent Steering)

Support permanent narrative instructions in the conversation history.

#### [MODIFY] [state.rs](file:///e:/John/Github/mrn-general/chronicler_engine/src/model/state.rs)
- Ensure `LogType` has a suitable variant (using `System` or adding `Narrator`).
- `Narrator` instructions should be rendered distinctly in the history.

#### [MODIFY] [assembler.rs](file:///e:/John/Github/mrn-general/chronicler_engine/src/narrative/prompt/assembler.rs)
- Update `render_history_layer()` to handle `Narrator` (or `System`) logs by wrapping them in an instruction block like `[Narrator: ...]`.

---

### Phase 3: Impersonate (Persona Forcing)

Allow the user to force the AI to write as a specific character.

#### [MODIFY] [types.rs](file:///e:/John/Github/mrn-general/chronicler_engine/src/narrative/prompt/types.rs)
- Add `impersonate_persona: Option<&'a str>` to `PromptContext` and `PromptAssembler`.

#### [MODIFY] [assembler.rs](file:///e:/John/Github/mrn-general/chronicler_engine/src/narrative/prompt/assembler.rs)
- Implement `render_impersonate_layer()`.
- If `impersonate_persona` is set, append a strict instruction: `[Write the next response as {{persona}}.]`.
- This should ideally replace or modify the standard `OUTPUT_FORMAT_TEMPLATE` to ensure the AI doesn't revert to standard narrator mode.

### Phase 4: UI & Command Integration (Frontend)

Expose the new steering features via the HTMX-based web interface.

#### [MODIFY] [ActionAreaTemplate](file:///e:/John/Github/mrn-general/chronicler_engine/src/server/templates.rs)
- Add a "Guide Generation" checkbox/toggle near the command input.
- When enabled, the current input is sent as `generation_guide` during a retry or new generation.

#### [MODIFY] [fragments.rs](file:///e:/John/Github/mrn-general/chronicler_engine/src/server/fragments.rs)
- Update `process_action` to handle slash commands:
    - `/narrator <text>`: Adds a `LogType::Narrator` entry to history and triggers a generation.
    - `/impersonate <persona> [text]`: Triggers a generation with `impersonate_persona` set.
- Update `ActionForm` to include `generation_guide` and `impersonate` flags.

---

## Verification Plan

### Automated Tests
- **Prompt Construction Tests**: Update `assembler_tests.rs` to verify that `generation_guide` appears at the absolute end of the prompt for recency bias.
- **History Rendering Tests**: Verify `LogType::Narrator` entries are rendered as `[Narrator: ...]` in the history layer.
- **Command Parsing Tests**: Add unit tests for the new slash command handlers in `process_action`.

### Manual Verification
1.  **Guided Generation**: Toggle the "Guide" mode, type "Narrate in the style of a noir novel", and click Retry. Verify the AI changes style.
2.  **Narrator Command**: Type `/narrator It begins to rain heavily`. Verify a permanent instruction block appears in the log and the AI acknowledges the weather.
3.  **Impersonation**: Type `/impersonate [Name]`. Verify the AI responds as that character.

---

## Design Decisions (Resolved)

- **Recency Bias**: The `Consideration` block will be appended AFTER the Output Format layer (the absolute end of the prompt) to ensure it is the freshest instruction in the model's context.
- **Narrator Role**: We will add `LogType::Narrator` to `state.rs` for semantic clarity and easier UI styling (italics/system-style rendering).
- **Format**: Narrator instructions in history will use the `[Narrator: <text>]` format, matching the user's preference for Marinara-style steering.
