# Research canonical output schemas and validation for Chronicler Engine LLM agents

Type: research
Status: closed
Assignee: assistant
Blocked by: 09

## Question

What output-schema and validation approach should Chronicler Engine use to make LLM agent results (starting with the quantifier) canonical, contract-checked, and diagnostically machine-readable?

## Context

The Uncle Bob 2026 applicability report identified `AIR-J`'s canonical output, design-by-contract, and structured diagnostics as applicable to Chronicler's runtime agent behavior. Today:

- `PromptPreset` and agent prompts describe output format in prose.
- `LlmCallRecorder` sanitizes and records raw request/response JSON.
- Agents return `AgentResult` parsed elsewhere, with no formal schema validation step.

The goal is not to adopt `AIR-J` itself, but to apply its principles to agent results.

## Expected output

1. A survey of schema options for agent outputs (JSON Schema, typed DTOs with `serde`, custom validators, etc.) mapped to the quantifier's current JSON format.
2. A proposal for where validation should live (agent executor, `LlmCallRecorder`, or a new validation seam) and what the failure mode should be.
3. A sketch of structured diagnostics (file/line/span/codes) for validation failures so downstream agents or skills can repair them.
4. A recommendation on whether to pilot this on the quantifier, extend it to future agents, or defer.

## Resolution

### 1. Schema options for the quantifier's JSON output

The quantifier currently returns a JSON object that maps to two internal DTOs:

- `QuantifierJsonResponse` (`src/application/agents/quantifier/utils/parser.rs:8`): `npcs_in_room: Vec<String>`, `movement: Option<MovementJson>`.
- `MovementJson` (`src/application/agents/quantifier/utils/parser.rs:15`): `type: Option<String>`, `destination: Option<String>`.

| Option | Fit for quantifier | Pros | Cons |
|---|---|---|---|
| **A. Typed DTOs + `serde` + custom validation** | Best fit | Already partially implemented; no new runtime dependencies; validation rules can be domain-specific (e.g., NPC IDs must exist in `known_npc_ids`). | Requires hand-written validators. |
| **B. JSON Schema + `jsonschema` crate** | Overkill | Formal schema, reusable across languages. | Adds a dependency; the schema duplicates the DTO; Chronicler's agents are internal, not a public API. |
| **C. Protobuf / `prost`** | Poor fit | Strong contracts, codegen. | Far too heavy for a text-parsing agent; conflicts with the current free-text prompt design. |
| **D. Custom validation on raw text before parse** | Partial fit | Catches markdown fence drift, truncation. | Harder to maintain than structured DTO validation. |

**Selected approach:** Option A — typed DTOs with `serde` plus a small, explicit validation step. This is the Chronicler-native path: domain types already live in `src/domain/model/quantifier.rs`, and `serde` is already a dependency.

### 2. Where validation should live and how it should fail

Current flow:

```
QuantifierAgent::execute
  -> determine_npcs_in_room
       -> quantify_room_with_llm_call
            -> recorder.complete(...) returns raw LlmCallResult
            -> parse_with_movement(...) produces QuantifierResult
       -> process_quantifier_result
  -> AgentResult::StatePatch
```

**Proposed seam:** Add an `AgentOutputValidator` trait or inherent validation step inside the agent executor, after `LlmCallRecorder.complete()` returns but before the result is turned into `AgentResult`. Keep `LlmCallRecorder` transport-only; it should sanitize and record, not judge semantics.

For the quantifier specifically:

- **Validation rules:**
  - `npcs_in_room` contains only known NPC IDs.
  - `movement.type` is one of `entering`, `leaving`, `in`, or null.
  - Required top-level fields are present when JSON parses.
- **Failure mode:** Degrade, do not panic. The quantifier already has a three-level confidence model (`High` / `Medium` / `Low`). A validation failure should drop confidence to `Low` and fall back to static NPCs, preserving the pipeline. This matches the engine's existing resilience strategy (`process_quantifier_result` already does this for `Low` confidence).
- For future agents with hard contracts (e.g., a tool-use agent), the same seam can return `Err` and fail the turn.

### 3. Structured diagnostics

Reuse the existing `Violation` shape from `tests/infrastructure/guardrails/mod.rs` as the internal diagnostic type:

```rust
pub struct AgentOutputDiagnostic {
    pub code: String,       // e.g., "quantifier:unknown_npc"
    pub message: String,
    pub severity: DiagnosticSeverity, // Error / Warning
}
```

Emit diagnostics via:

- `tracing::warn!` events for observability.
- A new `diagnostics: Vec<AgentOutputDiagnostic>` field on the persisted `LlmMessage` record, so `LlmCallRecorder` stores them alongside the raw response.

A downstream repair skill or agent can read the `LlmMessage` record, inspect the diagnostics, and produce a corrected prompt or a retry.

### 4. Recommendation

**Pilot on the quantifier.** The quantifier is the right starting point because:

- It already has a well-defined JSON shape.
- It has a built-in fallback path (`Low` confidence → static NPCs).
- Its output is consumed by the action pipeline, so improving robustness has immediate value.

**Extend to future agents** only after the quantifier pilot proves the seam. The next candidates would be any agent whose output is parsed into a strongly typed `AgentResult` variant.

**Do not defer.** The cost is low (a typed DTO and a small validator already fit the existing code), and the payoff in diagnostic clarity is immediate.

Next step: create an implementation ticket to add `QuantifierOutputValidator`, wire it into `quantify_room_with_llm_call`, and persist diagnostics in `LlmMessage`.
