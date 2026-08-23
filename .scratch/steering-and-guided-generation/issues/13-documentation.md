# Documentation for AI steering features

Type: task
Status: resolved
Blocked by: 05, 06, 07, 08, 09, 10, 11, 12

## Question

Document the three steering surfaces in the repo's docs, following the diataxis reference convention and the Documentation Strategy: Semantic Mapping from AGENTS.md.

Per the design synthesis (`../research/04-design-synthesis.md`):

1. Author reference docs under `docs/diataxis/reference/` for the three features. Likely targets: a steering/guide reference, a narrator-action reference, an impersonate reference — or a consolidated `ai_steering.md` if the features share enough surface. Follow the existing doc structure (see `docs/AGENTS.md` index).
2. Add `[DOC: docs/diataxis/reference/<area>/<name>.md]` line-1 anchors + human-readable summaries to every new/changed `src/` file (per AGENTS.md "Module-Level Two-Line Headers"). The implementation tickets will create files in `src/domain/model/`, `src/application/prompting/`, `src/adapters/driving/http/` — each needs the anchor.
3. Update `CONTEXT.md` if the effort introduces or sharpens domain terms (per the domain-modeling skill). Candidate terms: "guided generation," "narrator action," "impersonate," "replay blob," "steering." The domain boundary (guide steers content / impersonate substitutes speaker / narrator is a permanent directive) is the kind of distinction CONTEXT.md exists to record.
4. Regenerate the docs index: `python scripts/generate_docs_index.py` (the pre-commit hook does this, but run it explicitly to verify).

Blocked by: 05, 06, 07, 08, 09, 10, 11, 12 (documentation follows implementation; it documents what was built).

## Answer

Documentation for the three steering surfaces is committed. `python build.py` green (all 12 steps OK, 2 LLM tests skipped per policy; 109s).

### Reference docs
- New `docs/diataxis/reference/narrative/ai_steering.md` (Reference mode) — the three surfaces by axis (content vs speaker vs permanent directive): entry slash commands, the `<Guide>` final layer, `MessageType::Narrator` bare-render, the impersonate preset replacing the system preset + `<PlayerCharacter>` drop, the shared `GenerationReplay` blob on `Swipe`, mutual exclusivity, and retry/re-trigger behavior. Cross-references prompt_system, narration_system, game_flow, storage, dashboard.
- Updated `docs/diataxis/reference/narrative/prompt_system.md` — new "Conditional Layers" subsection (`<Guide>` after `<PlayerInput>`; `<PlayerCharacter>` drop on impersonate), `{{persona_description}}`/`{{persona_personality}}`/`{{persona_background}}` macros added to Context Templates, Prompt Presets section now covers `PresetType` (System/Quantifier/Impersonate) + `active_impersonate_prompt_preset_id`.
- Updated `docs/diataxis/reference/storage.md` — `message_swipes` row now notes the replay blob and cross-references ai_steering from Document References.
- `docs/diataxis/reference/frontend/dashboard.md` and `ui_design.md` already document the slash-command palette (ticket 12); no change needed.

### DOC anchors (Semantic Mapping)
- `src/domain/model/action.rs` → `docs/diataxis/reference/narrative/ai_steering.md` (was `game_flow.md`); summary names the slash-command entry.
- `src/domain/model/message.rs` → `docs/diataxis/reference/storage.md` (was `narrative/agent_system.md`, wrong); summary names Message/Swipe/replay blob.
- All other changed `src/` files already carry correct anchors from their implementation tickets; the synthesis doc is the natural home for the cross-cutting steering behavior.

### Domain terms (CONTEXT.md)
Added six terms after `Snapshot`: **Steering**, **Guided Generation**, **Narrator Action**, **Impersonate**, **Replay Blob** — each with the domain boundary and _Avoid_ list (e.g. Narrator Action vs System message; Replay Blob = steering half of "reproduce this generation", snapshot = state half).

### Index
- `python scripts/generate_docs_index.py` ran; `ai_steering.md` appears under `docs/diataxis/reference/narrative/`.
- `python scripts/validate_docs.py` — 0 errors, 0 warnings across 403 files.

### Notes
- No `src/` Rust logic changed; the two anchor repoints are comment-only. The build's test suite is the same code shipped green at 2c9e78f6.
- Deferred grilling tickets 14/15/16 remain pending (not chosen this session); they are not documentation work.
