# Spec: Align chronicler_engine Prompts with Marinara Engine Battle-Tested Patterns

**Status:** Specified — open questions answered 2026-05-04  
**Created:** 2026-05-04  
**Source:** Deep investigation of Marinara-Engine default preset (`default-preset.json`) and prompt architecture. Full extraction saved to `docs/reference/marinara_engine_system_prompt.md`. Architectural comparison saved to `docs/reference/marinara_engine.md`.  
**Priority:** Medium — improves output quality and model reliability  

---

## Objective

chronicler_engine's `SYSTEM_PROMPT_TEMPLATE` is under-specified compared to Marinara Engine's Default preset. Marinara has been battle-tested with multiple LLM families (GPT-4o, Llama 3.3, Gemini, Claude, DeepSeek) and multiple interaction modes (roleplay, conversation, game). chronicler_engine can adopt Marinara's proven prompt rules directly — they are not architecturally incompatible.

### What we're building

1. **Overhaul `SYSTEM_PROMPT_TEMPLATE`** in `src/narrative/prompt.rs` to incorporate Marinara's proven anti-repetition, anti-GPTism, knowledge boundary, character complexity, and proactive narrative rules.
2. **Remove token waste** by eliminating duplicate `global_rules` injection.
3. **Strengthen internal thought separation** to prevent NPC telepathy.
4. **Add proactive momentum rules** so the AI drives scenes forward instead of passively waiting for player input.
5. **Preserve chronicler's architectural strengths**: explicit physical state tracking, causality validation, and turn-based ending rules.

### What we're NOT building

- A full preset system (out of scope — chronicler uses hardcoded `PromptBuilder` layers)
- Variable substitution (`{{tense}}`, `{{pov}}`, etc. — chronicler's fixed literary style is intentional)
- Second-person perspective support (chronicler's third-person limited is a design choice)
- Streaming or conversation mode (chronicler is turn-based blocking request/response)

### Success criteria

- [ ] `SYSTEM_PROMPT_TEMPLATE` contains explicit anti-repetition rule with concrete example
- [ ] `SYSTEM_PROMPT_TEMPLATE` bans generic LLM structures ("if X, then Y", "not X, but Y", clichés like "jaws working")
- [ ] Knowledge boundary rules specify: latecomers ignorant, private conversations stay private, rumors travel slowly, default-to-no on uncertain knowledge
- [ ] Character complexity rule requires: opinions, contradictions, boundaries, hypocrisies, judgments — not just "distinct voices"
- [ ] Proactive narrative momentum rule added: introduce challenges, resist comfort, don't resolve tension early
- [ ] Internal thought barrier added: thoughts via narration are never audible unless mind-reading is explicitly established
- [ ] "No plot armor" / player-not-a-Mary-Sue rule added
- [ ] Positive framing rule added: describe what DOES happen, not what doesn't
- [ ] Duplicate `global_rules` injection removed (currently appears in both system layer and world info layer)
- [ ] All existing tests pass after prompt changes
- [ ] At least one new integration test verifies that the assembled prompt contains the new rules
- [ ] Manual validation: same model (`gemma-4-26b-a4b-it-abliterated:iq2xs` or GPT-4o-mini) produces measurably better prose on a test scenario before/after the change

---

## Tech Stack

- **Language:** Rust (Edition 2024, Rust 1.85+)
- **Framework:** chronicler_engine narrative system
- **Key files:** `src/narrative/prompt.rs`, `src/narrative/llm_client.rs`
- **Test runner:** `cargo test`
- **Validation:** `python build.py` (fmt + clippy + tests + coverage)
- **Manual test:** Direct API call via `curl` or test script against Ollama/OpenRouter

---

## Commands

```bash
# Full validation (required before any commit)
cd chronicler_engine && python build.py

# Run tests only
cd chronicler_engine && cargo test

# Run narrative-specific tests
cd chronicler_engine && cargo test narrative

# Build for manual testing
cd chronicler_engine && cargo build

# Format check
cd chronicler_engine && cargo fmt -- --check

# Clippy
cd chronicler_engine && cargo clippy --all-targets --all-features
```

---

## Project Structure

```
chronicler_engine/
├── src/
│   └── narrative/
│       ├── prompt.rs          # SYSTEM_PROMPT_TEMPLATE lives here
│       ├── llm_client.rs      # Reasoning extraction, API calls
│       └── ...
├── tests/
│   └── narrative_tests.rs     # Integration tests for prompt assembly
├── docs/
│   ├── reference/
│   │   ├── marinara_engine.md              # Architectural comparison
│   │   └── marinara_engine_system_prompt.md # Full Marinara reference
│   └── plans/
│       └── prompt-alignment-with-marinara.md # This spec
└── data/
    └── worlds/
        └── *.json             # World cards with global_rules
```

---

## Code Style

Follow chronicler_engine conventions:

- Rust Edition 2024, no `.unwrap()` or `.expect()` in production code
- Use `Result` for error handling
- Import order: `std` → external crates → local modules
- No "What" comments — if code isn't clear, rename symbols
- String constants use raw string literals (`r#"..."#`) for multi-line templates

Example of prompt constant style (existing):

```rust
const SYSTEM_PROMPT_TEMPLATE: &str = r#"You are an interactive fiction author...

State tracking rules:
- Track physical state: clothing, positions, locations, injuries, objects held.
"#;
```

---

## Testing Strategy

### Unit tests
- Verify `PromptBuilder::build()` produces a prompt containing the new rules
- Verify `render_system_layer()` no longer duplicates `global_rules`
- Verify token estimates remain reasonable after prompt expansion

### Integration tests
- `cargo test` — full test suite must pass
- Add a new test in `tests/` that assembles a prompt and asserts the presence of key new rules (anti-repetition, knowledge boundaries, etc.)

### Manual validation
1. Pick a test world and a test scenario (e.g., "Player enters a tavern and insults the bartender")
2. Run the scenario against the current prompt — save the output
3. Run the same scenario against the updated prompt — save the output
4. Compare: Does the new output show better character complexity? Does it avoid GPTisms? Does it proactively introduce conflict? Does it respect knowledge boundaries?

### What NOT to test
- Do not test that the LLM "follows" the prompt — that's the LLM's job, not ours. We test that the prompt *contains* the rules.
- Do not add tests that call live LLM APIs in CI.

---

## Boundaries

### Always do
- Run `python build.py` before committing
- Follow Rust 2024 edition conventions
- Update this spec if scope changes during implementation
- Add integration tests for prompt content assertions

### Ask first
- Removing any existing rule from `SYSTEM_PROMPT_TEMPLATE` (some old rules may be redundant — check with reviewer)
- Changing the XML tag structure in the user prompt (e.g., `<GameState>`, `<PlayerInput>`)
- Adding new `render_*` layers to `PromptBuilder`
- Modifying `fit_messages_to_context()` behavior

### Never do
- Commit secrets or API keys
- Use `.unwrap()` in production Rust code
- Skip `cargo fmt` or `cargo clippy`
- Remove the existing physical state tracking list (this is chronicler's strength — keep it)
- Remove the PHI_NARRATION_TEMPLATE (it works — keep it)
- Change the fixed third-person limited / past tense perspective (design choice, not a bug)

---

## Open Questions

1. **Should we adopt Marinara's "free will" framing?**  
   Marinara opens with: "you have your own free will, intellect, and emotional intelligence that you're unrestricted in wielding."  
   chronicler currently frames the AI as a service: "Your primary job is maintaining world-state consistency. Your secondary job is narrating that world with quality prose."  
   → **Question:** Does the "free will" framing produce better prose in practice? Should we A/B test this?

2. **Should we keep the existing "Never do" list?**  
   Marinara does not have an explicit "Never do" bulleted list — it scatters prohibitions throughout the Instructions and Output Format sections.  
   chronicler's current "Never do the following" list is very scannable.  
   → **Question:** Reorganize Marinara's rules into chronicler's existing "Never do" format, or scatter them as Marinara does?

3. **Length control**  
   Marinara has a `{{length}}` variable with detailed per-scene guidance. chronicler has no length control.  
   → **Question:** Should we add a fixed length guidance section to the system prompt, or is `PHI_NARRATION_TEMPLATE` sufficient?

4. **Global rules duplication**  
   `global_rules` currently appears twice. Removing the duplication from `render_world_info_layer()` is a clear win.  
   → **Question:** Should we also move `global_rules` out of `render_system_layer()` and put them exclusively in the user prompt's `<WorldLore>` section? This would reduce system prompt size but might reduce the LLM's adherence to them.

---

## Implementation Plan (High-Level)

### Phase 1: Content Audit
- Read `src/narrative/prompt.rs` completely
- Map every chronicler rule to its Marinara equivalent (or absence)
- Identify which Marinara rules transfer directly, which need adaptation, and which conflict with chronicler's architecture

### Phase 2: Prompt Rewrite
- Rewrite `SYSTEM_PROMPT_TEMPLATE` incorporating new rules
- Remove duplicate `global_rules` from `render_world_info_layer()`
- Update `PHI_NARRATION_TEMPLATE` if needed (probably keep as-is)

### Phase 3: Test Updates
- Update any tests that assert prompt content
- Add new integration test verifying key rules are present
- Run `python build.py` — fix any fmt/clippy/test failures

### Phase 4: Manual Validation
- Run before/against test scenario with a local model
- Document results in this spec's "Validation Results" section

---

## Notes

- Marinara's Default preset is `isDefault: true` and has been validated across multiple LLM families. We should trust its rule ordering and phrasing unless we have a specific architectural reason to deviate.
- The anti-repetition example ("Gooner?" → "What type of question is that?") is especially valuable because it gives the LLM a concrete pattern to avoid.
- Marinara's "Describe what DOES happen" rule is positively framed, which LLMs follow better than negative framing ("don't do X").
- We are NOT implementing Marinara's preset system, variable substitution, or choice blocks. This is a content upgrade to a hardcoded template, not an architectural rewrite.
