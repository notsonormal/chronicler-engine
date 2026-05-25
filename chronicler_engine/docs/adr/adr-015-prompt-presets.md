# ADR-015: Prompt Presets System

**Date:** 2026-05-19
**Status:** Accepted

> **Reference**: Implementation in `src/storage/prompt_preset_storage.rs`, `src/server/prompt_presets_fragment/`, and `src/model/prompt_preset.rs`.

---

## Context

The Chronicler Engine uses two distinct LLM prompts that were previously hardcoded in the Rust source:

1. **System Prompt** — sent to the narration LLM to establish tone, rules, and constraints
2. **Quantifier Prompt** — sent to the quantifier LLM to determine NPC presence and player movement

These prompts were embedded as `const` strings in `src/narrative/prompt/templates.rs` and `src/narrative/agents/quantifier/prompt.rs`. This created three problems:

1. **No runtime customization** — Users could not tweak prompts without recompiling
2. **No versioning** — Experimenting with prompt variations required editing source and rebuilding
3. **No UI discoverability** — Prompt content was invisible to non-technical users

---

## Decision

**Adopt a file-backed, DB-stored prompt preset system with a dedicated HTMX tabbed UI.**

### Architecture

Two independent preset collections — `System` and `Quantifier` — each with:
- **Seed files** (`data/prompt_presets/{system,quantifier}/default.json`) providing factory defaults
- **SQLite table** (`prompt_presets`) for runtime storage with full CRUD
- **Protected defaults** — seed-derived presets are marked `is_default = true` and cannot be edited or deleted through the UI
- **Active selection** — `AppSettings` tracks `active_system_prompt_preset_id` and `active_quantifier_prompt_preset_id` independently

### Preset structure

Each preset is split into four sections that are assembled into XML-wrapped tags at runtime:

| Field | XML Tag | Content |
|-------|---------|---------|
| `role` | `<role>` | Identity and agency description |
| `instructions` | `<instructions>` | Behavioral rules (validation, tracking, narrative, dialogue, general) |
| `writing_style` | `<writing_style>` | Prose constraints (perspective, tense, tone) |
| `output_format` | `<output_format>` | Output constraints (anti-recap, GPTisms ban, response length) |

Assembly order: `role` → `instructions` → `writing_style` → `<global_rules>` (from `world.json`) → `output_format`. The assembled text is cached in `AppSettings.active_system_prompt` / `active_quantifier_prompt` at startup and on activation.

### Why two separate collections

System and quantifier prompts serve fundamentally different purposes and are sent to different LLM backends. Coupling them into a single list would create confusion and accidental misuse.

### Why file-backed seeding → DB storage

- **Seeding**: Guarantees factory defaults exist on first run without requiring manual DB setup
- **DB runtime**: Enables CRUD operations, persistence across restarts, and future multi-world isolation
- **Migration path**: Seed files are read once at bootstrap; subsequent edits flow through the DB
- **Startup caching**: The active preset's sections are loaded from the DB at startup, assembled via `PromptPreset::assemble_prompt_text()` with `world.global_rules` and `response_length`, and cached in `AppSettings.active_system_prompt` / `active_quantifier_prompt` (transient `#[serde(skip)]` fields). This closes the gap between DB-stored presets and the prompt builder, which previously relied on a hardcoded fallback template.

### Why protected defaults

Default prompts are carefully tuned. Allowing destructive edits would make it easy to lose the baseline. Users who want to experiment create copies.

### UI Design

- Dedicated **"Prompt Presets"** tab in the HTMX dashboard
- Two sections: System Prompts and Quantifier Prompts
- Each preset rendered as a card with:
  - Name, preview (truncated), default/active badges
  - "Set Active", "Edit" (non-default only), "Delete" (non-default only)
- Inline add forms at the bottom of each section
- Edit uses HTMX `outerHTML` swap on the card; activate/delete refresh the full panel

---

## Consequences

### Positive
- Runtime prompt customization without server restart
- Non-technical users can experiment with prompt engineering
- Presets are persisted and survive restarts
- Defaults are protected from accidental destruction

### Negative
- Additional DB table and migration (v3)
- Additional UI tab adds cognitive load
- Prompt overrides must be threaded through `PromptContext` and `QuantifierPromptContext`

### Trade-offs
- Chose separate collections over unified list (clarity over generality)
- Chose DB storage over pure file storage (CRUD operations, persistence)
- Chose protected defaults over editable defaults (safety over convenience)

---

## Related ADRs

- [ADR-001: HTMX Web Dashboard](./adr-001-htmx-web-dashboard.md) — UI foundation hosting the Prompt Presets tab
- [ADR-005: Layered Prompt Architecture](./adr-005-layered-prompts.md) — System prompt integration with builder layers
- [ADR-006: Quantifier-Driven Game Systems](./adr-006-quantifier-systems.md) — Quantifier prompt integration
- [ADR-007: Settings System Architecture](./adr-007-settings-system.md) — Active preset IDs stored in `AppSettings`

---

## History

- **2026-05-19**: Initial implementation — DB table, storage trait, seeding, HTMX UI
- **2026-05-19**: Architectural refinements — `preset_type` added to `PromptPreset` domain model; `PromptPresetStorage::save()` simplified to take only `&PromptPreset`; `PresetType::from(&str)` replaced with `TryFrom<&str>` for explicit error handling; cache invalidation added to `update_preset_handler` so editing an active preset updates the cached prompt text immediately
- **2026-05-25**: Sectioned preset refactor — monolithic `prompt_text` replaced with `role`, `instructions`, `writing_style`, `output_format` fields; `assemble_prompt_text()` added for XML assembly; DB migration v7 added section columns; migration v8 dropped `prompt_text` column; UI updated with four textarea fields per preset
