# Plan: Restructure Prompt Presets into XML Sections

## Goal
Split the monolithic `prompt_text` into four hardcoded sections (`role`, `instructions`, `writing_style`, `output_format`) and assemble them as XML-wrapped tags. Global rules from `world.json` become a builder-generated `<global_rules>` section. Response length is appended into the `output_format` section content.

## Success Criteria
- [ ] `default.json` uses `role`/`instructions`/`writing_style`/`output_format` instead of `prompt_text`
- [ ] Assembled system prompt is a single message with XML-wrapped sections
- [ ] `OUTPUT_FORMAT_TEMPLATE` is removed from `builder.rs`; its content moves into the preset
- [ ] Global rules appear as `<global_rules>` before `<output_format>`
- [ ] Response length is appended inside `<output_format>` content
- [ ] Quantifier preset uses the same section structure
- [ ] All tests pass (`cd chronicler_engine && python build.py`)

---

## New JSON Schema

```json
{
  "id": "system_default",
  "name": "Default",
  "role": "You are an interactive fiction author...",
  "instructions": "Input validation rules:\n- ...\n\nState tracking rules:\n- ...",
  "writing_style": "Third-person limited perspective, focused on the player character.\nPast tense narrative prose.",
  "output_format": "The player's next action is provided above. Your only job is to narrate what happens now.\n\nDo not re-narrate events...\n\nNo GPTisms..."
}
```

## Final Assembled System Prompt

```xml
<role>
    You are an interactive fiction author with your own free will...
</role>

<instructions>
    Input validation rules:
    - Treat the player's input as an attempted action...

    State tracking rules:
    - Track physical state...
</instructions>

<writing_style>
    Third-person limited perspective, focused on the player character.
    Past tense narrative prose.
</writing_style>

<global_rules>
    - No explicit content.
    - Another world-specific rule.
</global_rules>

<output_format>
    The player's next action is provided above...

    No GPTisms/AI Slop...

    Response Length:
    flexible, based on the current scene...
</output_format>
```

---

## Step 1: Update Domain Model
**Files:** `src/model/prompt_preset.rs`, `src/model/prompt_preset_tests.rs`

```rust
pub struct PromptPreset {
    pub id: String,
    pub name: String,
    pub role: Option<String>,
    pub instructions: Option<String>,
    pub writing_style: Option<String>,
    pub output_format: Option<String>,
    pub is_default: bool,
    pub preset_type: PresetType,
    // Legacy fallback
    pub prompt_text: Option<String>,
}
```

Add `PromptPreset::assemble_prompt_text(&self, global_rules: &[String], response_length: Option<&str>) -> String`:
1. If any section is present, wrap each non-empty section in XML tags and join with `\n\n`
2. If world has `global_rules`, wrap them in `<global_rules>` and insert before `<output_format>`
3. If `response_length` is set, append `\n\nResponse Length:\n{length}` to the `output_format` content before wrapping
4. If no sections, return `prompt_text` as-is (legacy fallback)

## Step 2: Database Migration & Storage Layer
**Files:** `src/storage/db.rs`, `src/storage/models/prompt_preset.rs`, `src/storage/prompt_preset_storage.rs`, `src/storage/prompt_preset_storage_tests.rs`

- Migration v7: `ALTER TABLE prompt_presets ADD COLUMN role TEXT`, `instructions TEXT`, `writing_style TEXT`, `output_format TEXT`
- Drop `prompt_text` column (or leave it empty — new code won't read it)
- Update `DbPromptPreset` and storage impls to read/write new columns
- `from_db()`: map new columns directly
- Update all storage tests

## Step 3: Seeding Logic
**Files:** `src/bootstrap/run.rs`

- Update `ensure_defaults()` to read `role`, `instructions`, `writing_style`, `output_format` from JSON seed files
- Support both old (`prompt_text` only) and new (sectioned) seed formats
- Assemble `prompt_text` from sections at seed time so cached string is populated

## Step 4: Restructure Seed JSON Files
**Files:** `data/prompt_presets/system/default.json`, `data/prompt_presets/quantifier/default.json`

### System Default — Section Mapping

| Section | Source |
|---------|--------|
| **role** | Opening identity + agency paragraphs from current `prompt_text` |
| **instructions** | Input validation, State tracking, World dynamics, Narrative rules, Dialogue rules, General rules |
| **writing_style** | "Third-person limited perspective... Past tense..." from old `OUTPUT_FORMAT_TEMPLATE` |
| **output_format** | Anti-recap, "narrate what happens now", GPTisms ban, describe-DOES-happen, anti-repeat (merged from old `OUTPUT_FORMAT_TEMPLATE` + narrative rules) |

### Quantifier Default

| Section | Source |
|---------|--------|
| **role** | "You are a scene quantifier..." |
| **instructions** | Task, movement rules, examples |
| **output_format** | JSON format specification |

Remove `<AvailableNpcIds>` from quantifier seed. Builder generates it at runtime.

## Step 5: Update Prompt Builder (System)
**Files:** `src/narrative/prompt/builder.rs`, `src/narrative/prompt/builder_tests.rs`

### Interaction with user prompt and `build_split()`

Current `build_split()` returns `(system, user)` where:
- **system** = Layer 0: plain text system prompt + global rules + response length
- **user** = Layers 1-7: GameState, KnownNpcs, NpcsInRoom, PlayerCharacter, WorldLore, ConversationHistory, PlayerInput, **OUTPUT_FORMAT_TEMPLATE**

With this change:
- **system** = XML-wrapped sections (role, instructions, writing_style, global_rules, output_format)
- **user** = Layers 1-6 only: GameState, KnownNpcs, NpcsInRoom, PlayerCharacter, WorldLore, ConversationHistory, PlayerInput

**Impact:** The output format content moves from the **user message** to the **system message**. This is correct — output format instructions are system-level constraints, not user input data. The LLM backends (`ollama.rs`, `openrouter.rs`) send system and user separately; this change improves instruction placement.

**Context fitting:** `fit_messages_to_context()` treats system and user as separate budgets. Since we're moving content from user to system, total tokens stay the same. The system prompt grows, the user message shrinks, but the combined budget check remains valid. History trimming (`trim_history_to_budget`) operates on the user message and is unaffected.

### `render_system_layer()` refactor

Current behavior:
1. Start with `system_prompt_override` (plain text)
2. Append `world.global_rules` as `- rule` bullets
3. Append `Response Length: ...`

New behavior:
1. Start with `system_prompt_override` (now the assembled XML)
2. No more appending rules/length — they are handled inside `assemble_prompt_text()`

`assemble_prompt_text()` builds:
```
<role>...</role>

<instructions>...</instructions>

<writing_style>...</writing_style>

<global_rules>
- rule1
- rule2
</global_rules>

<output_format>
...preset content...

Response Length:
...from settings...
</output_format>
```

### Remove hardcoded `OUTPUT_FORMAT_TEMPLATE`

- Delete the constant from `builder.rs`
- Remove `render_output_format_layer()` or make it return empty string
- Update `build_split()`, `build()`, `build_user_only()` to stop appending Layer 7

### Tests that will break and need updating

| Test | File | Why it breaks | Fix |
|------|------|---------------|-----|
| `test_build_split_includes_phi_in_user_half` | `builder_tests.rs` | Asserts "Your only job..." is in user, not system | Update assertions: text should now be in system, not user |
| `test_build_user_only` | `builder_tests.rs` | Asserts user contains "Your only job..." | Remove assertion — text is now in system |
| `test_build_split_phi_narration_mode` | `builder_tests.rs` | Asserts user contains "Your only job..." | Update to check system instead |
| `test_build_layer_7_phi` | `builder_tests.rs` | Asserts full prompt contains text | Still passes (text moves to system) |
| `test_build_returns_all_layers` | `builder_tests.rs` | Asserts full prompt contains text | Still passes |

**No changes needed for:** `fit_messages_to_context`, `trim_history_to_budget`, LLM backend callers, quantifier builder.

## Step 6: Update Quantifier Builder
**Files:** `src/narrative/agents/quantifier/prompt.rs`, `src/narrative/agents/quantifier/prompt_tests.rs`

- `build_system_prompt()` now calls `preset.assemble_prompt_text(None, None)` (quantifier has no global rules or response length)
- After the assembled preset, append `<available_npc_ids>...</available_npc_ids>` and `<available_rooms>...</available_rooms>`
- Remove old logic that expected `<AvailableNpcIds>` in preset text

## Step 7: UI Updates
**Files:** `src/server/prompt_presets_fragment/template.rs`, `src/server/prompt_presets_fragment/fragments.rs`, `src/server/prompt_presets_fragment/fragments_tests.rs`, `src/server/prompt_presets_fragment/handlers.rs`, `src/server/prompt_presets_fragment/handlers_tests.rs`, `tests/components/prompt_presets.rs`

### Template (`template.rs`)
- Replace single textarea with four textareas: Role, Instructions, Writing Style, Output Format

### Fragments (`fragments.rs`)
- Update edit form to render four textareas
- Update card preview to show assembled text or first section

### Handlers (`handlers.rs`)
- Update `PresetForm` and `PresetUpdateForm` with four section fields
- On save: construct `PromptPreset`, call `assemble_prompt_text()`, cache in settings

### Tests
- Update all unit, fragment, handler, and integration tests

## Step 8: Settings Caching
**Files:** `src/bootstrap/run.rs`, `src/server/prompt_presets_fragment/handlers.rs`

- On startup/activation: load preset, call `assemble_prompt_text()` with global rules and response length, cache in `AppSettings.active_system_prompt`

## Step 9: Update Documentation
**Files:** `docs/adr/adr-004-xml-prompt-format.md`, `docs/adr/adr-015-prompt-presets.md`, `docs/reference/system_prompt.md`

- ADR-004: Update v3 section — instruction sections are now XML-wrapped, but individual rules remain plain text inside. This reverses the plain-text-only decision for section containers.
- ADR-015: Update to describe sectioned preset structure
- Reference docs: Show new JSON format and assembled XML

## Step 10: Validation
**Command:** `cd chronicler_engine && python build.py`
- Fix fmt/clippy/test failures
- Verify seed files load on fresh DB
- Verify existing DB presets load cleanly (old `prompt_text` presets will show empty sections and need re-seeding)

---

## Why This Is Clean

- **Fixed XML wrapping**: No configuration noise. Every preset uses the same structure.
- **Hardcoded section order**: Role → Instructions → Writing Style → Global Rules → Output Format. Predictable every time.
- **No marker system**: Global rules and response length are handled by simple builder logic, not a complex marker/variable framework.
- **Single system message**: All sections merge into one system message.
- **Removed Layer 7**: Output format guidance now lives in the preset where users can edit it.
