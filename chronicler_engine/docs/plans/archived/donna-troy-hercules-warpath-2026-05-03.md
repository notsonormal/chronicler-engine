# Plan: Improve Quantifier Prompt for Movement Certainty

## Problem

The quantifier LLM is uncertain about player movement when narration describes a scene transition. In the user's example:

- `<CurrentRoom>` = Front Gates
- `<LatestNarration>` describes the mansion interior, foyer, Carla in the doorway
- LLM reasoned through multiple self-correction steps and failed to confidently detect movement

The core issue: the prompt doesn't give the LLM a clear way to recognize that a described scene differs from the current location.

## Root Cause

1. **No decision framework**: Rules are listed but don't tell the LLM *how* to decide. The LLM defaults to looking for explicit movement verbs ("walk", "enter") instead of comparing the described scene to `<CurrentRoom>`.

2. **Examples are too narrow**: All examples show explicit verbs or clear blocking. None show the common case where the narrator simply describes a new location after movement has occurred.

## Changes

### Task 1: Update quantifier system prompt with decision framework

**File:** `src/narrative/quantifier.rs`

In `build_system_prompt()`, replace:
```
Movement is determined ONLY by what happens in <LatestNarration>, not by earlier history.
```

With a 3-step decision framework:
```
How to determine movement:
1. Read <CurrentRoom> — this is where the player is right now.
2. Read <LatestNarration> — this is what just happened.
3. Ask: does the narration describe the player being in a different place than <CurrentRoom>?
   - If YES → movement occurred. Set type to "entering" and destination to the new room.
   - If NO → no movement. Set type to null.
   - If unclear → assume no movement. Set type to null.
```

Keep blocking/interposing rules as exceptions to the framework.

Add 4 diverse examples:
- Explicit verb: "You walk through the door into the kitchen." (CurrentRoom was hallway) → entering kitchen
- Blocked: "The guard blocks your path." → no movement
- Scene-implied: "The foyer felt claustrophobic. Carla stood in the doorway." (CurrentRoom was Front Gates) → entering entrance_hall
- No movement: "You examine the ancient vase carefully." → no movement

**Acceptance criteria:**
- [ ] System prompt contains the 3-step framework
- [ ] System prompt contains 4 diverse examples
- [ ] Existing blocking rules are preserved

---

### Task 2: Update documentation

**File:** `docs/reference/quantifier_prompt.md`

- Document the 3-step decision framework
- Update example set to match new prompt

**Acceptance criteria:**
- [ ] Documentation reflects the new system prompt structure
- [ ] Documentation includes all 4 example types

---

### Task 3: Update tests

**File:** `src/narrative/quantifier.rs` (test module)

Update `test_quantifier_prompt_builder_basic` to assert:
- System prompt contains "How to determine movement" framework

**Acceptance criteria:**
- [ ] Tests verify decision framework text is present

---

## Checkpoint: Complete

- [ ] All tests pass: `cd chronicler_engine && cargo test quantifier`
- [ ] Build succeeds: `cd chronicler_engine && cargo build`
- [ ] Documentation is consistent with code

## What We're NOT Doing

- NOT adding `<PreviousRoom>` — would bias the LLM toward always reporting movement
- NOT adding `<PlayerAction>` — quantifier runs after any narration (triggers, dialogue), not just player actions
- NOT adding overly specific heuristic rules ("different atmosphere", "different NPCs") — overfits to one scenario
- NOT changing the JSON format, NPC detection logic, or confidence scoring

## Risks

| Risk | Impact | Mitigation |
|------|--------|------------|
| New prompt could be too prescriptive for other scenarios | Low | Framework is general ("different place"); examples are diverse |
