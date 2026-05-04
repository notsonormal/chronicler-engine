# Plan: Unify PHI Layer — Remove PhiMode::Continuation

## Problem

Chronicler deviates from the SillyTavern/Marinara pattern by having a mode-dependent PHI layer (`PhiMode::Narration` vs `PhiMode::Continuation`). In both SillyTavern and Marinara, post-history instructions are **universal behavioral constraints** — they don't change based on context. The task instruction belongs in the user message (Layer 6), not the PHI (Layer 7).

The continuation PHI is also missing universal constraints (end descriptively, don't ask questions, match pacing) that the narration PHI has, making trigger narrations more ambiguous.

## Approach

Remove `PhiMode` entirely. Make PHI a single universal template. Move the continuation-specific task instruction into the trigger user message where it sits next to the context it applies to.

## Architecture Decision

- **Keep `PHI_NARRATION_TEMPLATE`** as the universal post-history instruction (rename not needed)
- **Delete `PHI_CONTINUATION_TEMPLATE`** — its content moves to Layer 6 (user message)
- **Delete `PhiMode` enum** — single-variant enums are unnecessary
- **Remove `phi_mode` field** from `PromptBuilder`

---

## Task List

### Task 1: Remove PhiMode enum and unify PHI template

**File:** `src/narrative/prompt.rs`

**Description:**
Remove the `PhiMode` enum and all code that branches on it. The PHI layer should always use the narration template.

**Changes:**
1. Delete `PhiMode` enum definition
2. Delete `PHI_CONTINUATION_TEMPLATE` constant
3. Remove `phi_mode` field from `PromptBuilder`
4. Remove `with_phi_mode()` method
5. Update `render_phi_layer()` to always return `PHI_NARRATION_TEMPLATE`
6. Update `PromptBuilder::from_context()` to not set `phi_mode`
7. Update ~20 test struct literals that set `phi_mode: PhiMode::Narration`

**Acceptance criteria:**
- [ ] `PhiMode` enum no longer exists in codebase
- [ ] `PHI_CONTINUATION_TEMPLATE` no longer exists
- [ ] `cargo build` succeeds

**Estimated scope:** Small (1 file, many mechanical edits)

---

### Task 2: Move continuation instruction into trigger user message

**File:** `src/engine/action_processing.rs`

**Description:**
The trigger narration currently relies on `PhiMode::Continuation` to tell the LLM to continue the scene. Move that instruction into the user message (Layer 6) where the context lives.

**Changes:**
```rust
// BEFORE:
let continuation_user_msg = format!(
    "Previous narration:\n{narration_text}\n\nTrigger event: {}",
    trigger.action.narration_prompt
);

// AFTER:
let continuation_user_msg = format!(
    "Previous narration:\n{narration_text}\n\nTrigger event: {}\n\n\
     Continue the scene naturally, incorporating the trigger event into the narrative. \
     Do NOT repeat or contradict what was already described. Build naturally on the existing scene.",
    trigger.action.narration_prompt
);
```

Also remove `pb.phi_mode = PhiMode::Continuation;` line.

**Acceptance criteria:**
- [ ] Trigger user message includes continuation instruction
- [ ] No `phi_mode` assignment in trigger evaluation

**Estimated scope:** XS (1 file, 2 edits)

---

### Task 3: Update tests

**Files:**
- `src/narrative/prompt.rs` (test module)
- `src/narrative/llm.rs` (test module)

**Description:**
Remove all references to `PhiMode` and `Continuation` from tests.

**Changes:**
- `prompt.rs`: Remove `phi_mode` from all `PromptBuilder` test struct literals (~20 locations)
- `prompt.rs`: Update or remove `test_build_split_phi_continuation_mode`
- `prompt.rs`: Update assertions that check for continuation-specific content
- `llm.rs`: Update `test_narrate_action_from_prompt_mock` assertion

**Acceptance criteria:**
- [ ] All prompt tests pass
- [ ] All llm tests pass
- [ ] `cargo test` passes

**Estimated scope:** Small (2 files, mechanical edits)

**Dependencies:** Task 1

---

### Task 4: Update documentation

**Files:**
- `docs/reference/system_prompt.md`
- `docs/system/prompt_system.md`
- `docs/system/game_flow.md`
- `docs/system/narration_engine.md`
- `docs/architecture/system.md`
- `docs/adr/adr-005-layered-prompts.md`

**Description:**
Remove all references to `PhiMode::Continuation` and explain that PHI is now universal.

**Changes:**
- `system_prompt.md`: Replace "PHI Layer Modes" section with "Universal PHI Layer"
- `prompt_system.md`: Remove mode references
- `game_flow.md`: Update Phase 5 (remove PhiMode::Continuation mention)
- `narration_engine.md`: Update continuation narration description
- `architecture/system.md`: Remove PhiMode mention
- `adr-005-layered-prompts.md`: Note the simplification

**Acceptance criteria:**
- [ ] No docs reference `PhiMode::Continuation`
- [ ] Docs explain unified PHI

**Estimated scope:** Small (6 files, text edits)

**Dependencies:** Task 1

---

## Checkpoint: Complete

- [ ] `cargo test` passes (all test suites)
- [ ] `cargo build` succeeds
- [ ] Documentation is consistent with code

## Risks

| Risk | Impact | Mitigation |
|------|--------|------------|
| Removing PhiMode breaks external callers | Low | PhiMode is internal; only used in `action_processing.rs` |
| Trigger narration tone changes | Low | Continuation instruction moves to user message, preserving exact prompt content |
| Tests need widespread mechanical updates | Low | Mostly removing `phi_mode` field from struct literals |
