# 02 — Run audit and confirm violation set

Type: task
Status: resolved
Blocked by: 01

## Question

Run the new `guardrails_inherent_impl_locality` rule against current `main` and confirm the violation set matches the expected list.

Expected violations (from earlier manual scan):

| Type | Defining file | Impl files | Reason |
|---|---|---|---|
| `ActionPipeline` | `application/action_pipeline/pipeline.rs` | `pipeline.rs`, `phases.rs` | folder holds other types too |
| `PipelineRun` | `application/action_pipeline/phases.rs` | `phases.rs`, `pipeline.rs` | folder `action_pipeline/` ≠ `pipeline_run` |
| `DefaultApplicationService` | `application/application_service.rs` | `application_service.rs`, `action_pipeline/retry.rs`, `message_editing.rs` | split across 3 different folders |
| `Storage` | `adapters/driven/storage/backend/core.rs` | 14 files in `backend/` + `bootstrap/load.rs` | folder `backend/` ≠ `storage` |
| `InMemoryData` | `adapters/driven/storage/backend/core.rs` | `backend/{messages,swipes}.rs` | folder `backend/` ≠ `in_memory_data` |
| `DbCharacter` | `adapters/driven/storage/models/character.rs` | `models/character.rs`, `backend/characters.rs` | split, file names don't match type |
| `DbGame` | `models/game.rs` | `models/game.rs`, `backend/games.rs` | same |
| `DbPersona` | `models/persona.rs` | `models/persona.rs`, `backend/personas.rs` | same |
| `DbPromptPreset` | `models/prompt_preset.rs` | `models/prompt_preset.rs`, `backend/presets.rs` | same |
| `DbSettings` | `models/settings.rs` | `models/settings.rs`, `backend/settings.rs` | same |
| `DbWorld` | `models/world.rs` | `models/world.rs`, `backend/worlds.rs` | same |
| `PromptPreset` | `domain/model/prompt_preset.rs` | `domain/model/prompt_preset.rs`, `application/narrative_prompt/assembler.rs` | cross-layer split |
| `QuantifierParseResult` | `domain/model/quantifier.rs` | `domain/model/quantifier.rs`, `application/agents/quantifier/parser.rs` | cross-layer split |

Procedure:
1. Run `cargo test --test guardrails guardrails_inherent_impl_locality -- --nocapture` from the repo root.
2. Capture the full violation output.
3. Diff the actual set against the expected set above.
4. If new violations appear that the manual scan missed, list them separately — do not silently absorb them into the expected set.
5. If any expected violation is missing from the rule's output, file that as a bug in ticket 01's rule (do not suppress silently).

Acceptance:
- Either the two sets match exactly, or discrepancies are itemized as findings.
- No fixes to source code in this ticket — audit only.
- Output: a markdown summary appended to this ticket under `## Answer`, posted as the resolution.

## Answer

**Closed without re-running the rule.** The audit was effectively performed during ticket 01's trial run (before the rule was removed at user direction). The full 27-violation set and three discrepancies against this ticket's expected table are recorded in ticket 01's body under "Trial run findings" — preserved there for the refactor tickets.

### Discrepancies against the expected table (procedure step 5)

1. **`ActionPipeline` is NOT flagged** by the rule as specified. `phases.rs` lives in folder `action_pipeline/`, which matches the folder exemption (`snake_case(ActionPipeline) == "action_pipeline"`). The rule formula encodes "parent dir ends with `/snake`", not "folder holds only this type." 02's `Reason: folder holds other types too` was informal and is not captured by the map's rule statement. **Finding**: either the rule formula is too loose (map change) or ticket 04's expectation is wrong. Hand to 04 to resolve when worked.
2. **`QuantifierResult` IS flagged** (sibling of `QuantifierParseResult`). Ticket 08's narrative mentions both siblings but this ticket's expected table listed only `QuantifierParseResult`. **Finding**: 08 must handle both siblings, not just `QuantifierParseResult`.
3. **`AppState`** (def `adapters/driving/http/app_state.rs`, impl in `fragments/renderers/fragment_renderers.rs`) and **`PromptContext`** (def `application/narrative_prompt/types.rs`, impl in `assembler.rs`) are flagged by the rule but absent from 02's expected table. **Finding**: these are newly-surfaced violations the manual scan missed. No refactor ticket exists for them yet — 01's "Notes for downstream tickets" flagged them; a new ticket should be created (or `PromptContext` folded into the `PromptPreset` / 07 refactor since it lives in the same file) when the frontier reaches them.

### Violation set handed to refactor tickets

| Refactor ticket | Types from the 27-violation trial set it owns |
|---|---|
| 03 | `Storage`, `InMemoryData`, `DbWorld`, `DbCharacter`, `DbSettings`, `DbPromptPreset`, `DbPersona`, `DbGame` (plus other `Db*` mappers per 06) |
| 04 | `PipelineRun` (and `ActionPipeline` per discrepancy 1 — formula refinement needed) |
| 05 | `DefaultApplicationService` |
| 06 | `DbCharacter`, `DbGame`, `DbPersona`, `DbPromptPreset`, `DbSettings`, `DbWorld` |
| 07 | `PromptPreset`, and consider folding `PromptContext` (discrepancy 3) |
| 08 | `QuantifierParseResult`, `QuantifierResult` (discrepancy 2) |
| New ticket (TBD) | `AppState` (discrepancy 3) — not yet specified, see map |

### Audit procedure followed (per ticket)

Steps 1–3 (run rule, capture output, diff against expected): done via the 01 trial run, results preserved in 01.
Step 4 (new violations itemized, not silently absorbed): done — `AppState` and `PromptContext` listed above.
Step 5 (expected violations missing from rule output flagged as bugs in 01's rule, not suppressed): done — `ActionPipeline` is the one expected violation not in the rule output; root cause is the folder exemption formula in the map, not a bug in 01's implementation. Surfaced for 04 to resolve.

### Acceptance

- ✅ Either the two sets match exactly, or discrepancies are itemized as findings — three discrepancies itemized above.
- ✅ No fixes to source code in this ticket — audit only. 27 violations stand on `main` until refactor tickets.
- ✅ Output: this markdown summary is the resolution.
