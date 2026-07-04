# Zone A: src/model/ — Abstraction Anti-Pattern Report

## Summary (12 findings)

| Severity | Count |
|----------|-------|
| high     | 4     |
| med      | 5     |
| low      | 3     |

## Findings

### A1. Premature generalization — `StatePatch` enum with single variant

- **File:** `agent.rs:96`
- **Evidence:**

  ```rust
  pub enum StatePatch {
      Scene {
          npc_ids: Vec<String>,
          movement_destination: Option<String>,
          confidence: Confidence,
      },
  }
  ```

  `impl StatePatch` (line 33) matches `(self, other)` and destructures both sides into the sole `Scene` variant, producing 50+ lines of boilerplate for a concept that never branched.
- **Why smell:** Enum implies future variants, but pipeline has only one. `merge` is needlessly generic; callers pay match tax for no benefit.
- **Severity:** high
- **Proposed fix:** Replace with `struct ScenePatch { ... }` and a plain `merge(self, other: ScenePatch) -> ScenePatch` method.

### A2. Premature generalization — `TriggerRequirement` enum with single variant

- **File:** `trigger.rs:15`
- **Evidence:**

  ```rust
  pub enum TriggerRequirement {
      TimesMet(ComparisonOperator, u32),
  }
  ```

- **Why smell:** Same pattern as A1. No second requirement type exists; every match site has exactly one arm. Struct conveys the same semantics without indirection.
- **Severity:** med
- **Proposed fix:** Convert to `struct TimesMetRequirement { op: ComparisonOperator, count: u32 }` until a sibling requirement is introduced.

### A3. Wrong abstraction / false duplication — `Confidence` vs `QuantifierConfidence`

- **File:** `agent.rs:88` and `quantifier.rs:7`
- **Evidence:**

  ```rust
  // agent.rs
  pub enum Confidence { High, Medium, Low }

  // quantifier.rs
  pub enum QuantifierConfidence { High, Medium, Low }

  // quantifier.rs
  impl From<Confidence> for QuantifierConfidence { ... }
  impl From<QuantifierConfidence> for Confidence { ... }
  ```

- **Why smell:** Two types for the same domain concept, kept separate by module boundary rather than semantics. Bidirectional `From` impls are a耦合 tax that proves the split is artificial.
- **Severity:** med
- **Proposed fix:** Use `Confidence` (or a shared `model::Confidence`) in both modules; delete `QuantifierConfidence`.

### A4. Refactor-be-damned extraction — `Message` runtime-mirrored fields

- **File:** `message.rs:18` and `message.rs:9`
- **Evidence:**

  ```rust
  pub struct Message {
      text: String,
      location_header: Option<String>,
      event_header: Option<String>,
      snapshot_id: Option<u64>,
      pub swipes: Vec<Swipe>,
      ...
  }

  pub struct Swipe {
      pub text: String,
      pub snapshot_id: Option<u64>,
      pub location_header: Option<String>,
      pub event_header: Option<String>,
  }
  ```

  `set_active_swipe` and `update_active_swipe_text` manually sync the duplicated fields.
- **Why smell:** Root cause is a DB schema mismatch (messages stored flat, swipes stored separately). Instead of fixing the mapping layer, the code duplicates state and adds sync logic. Risk of drift; `from_db` creates an invalid Message (empty text, no swipes) that must be hydrated outside the type.
- **Severity:** high
- **Proposed fix:** Remove mirrored fields from `Message`; derive active values via accessors on `Message` reading `swipes[active_swipe_index]`, or normalize DB to store swipes as first-class rows.

### A5. Leaky wrapper / helper smell — `MessageHistory`

- **File:** `message_history.rs:15`
- **Evidence:**

  ```rust
  pub struct MessageHistory { messages: Vec<Message> }
  ```

  Exposes `replace`, `retain`, `iter_mut`, `as_slice`, `clear`, which let callers mutate or swap the inner `Vec` arbitrarily. Only `append` enforces `MAX_MESSAGES`.
- **Why smell:** Promises encapsulation ("Callers cannot bypass rules with direct `.push()`") but provides multiple bypasses. Wrapper is a thin helper that does not own its invariant.
- **Severity:** med
- **Proposed fix:** Either drop `MessageHistory` and enforce the 1000-message cap at the single append site in `GameState`, or remove `replace`/`retain`/`iter_mut` and expose a truly read-only view.

### A6. Type exists only for one function — `TemplateVars`

- **File:** `template.rs:5`
- **Evidence:**

  ```rust
  pub struct TemplateVars {
      pub user: String,
  }

  pub fn render_template(text: &str, vars: &TemplateVars) -> String {
      text.replace("{{user}}", &vars.user)
  }
  ```

- **Why smell:** Struct has one field and is consumed by exactly one function, which only uses that field. Abstraction adds indirection with no current payoff.
- **Severity:** med
- **Proposed fix:** Replace with `render_template(text: &str, user: &str) -> String`. Re-introduce `TemplateVars` when ≥2 fields are needed.

### A7. Coincidental cohesion — `state.rs` grab-bag

- **File:** `state.rs`
- **Evidence:** File contains `MessageType`, `MessageEntry`, `GenerationStatus`, `GenerationPhase`, `InputBuffer`, `MovementState`, `StoredTriggerContext`, `NarrativeState`, `SceneState`, `GameState`, `GameStateBuilder`. No shared behavior; concepts span messaging, UI status, map movement, triggers, narrative, scene, and game lifecycle.
- **Why smell:** Grouped because they are "in the state", not because they share a concept. Changes to unrelated subsystems touch the same file, increasing merge conflict surface and obscuring boundaries.
- **Severity:** med
- **Proposed fix:** Split into focused modules: `generation.rs`, `movement.rs`, `scene.rs`, `narrative.rs`, or at least move `MessageEntry`/`MessageType` adjacent to `message.rs`.

### A8. Forced merge of unrelated concerns — `PromptPreset::assemble_prompt_text`

- **File:** `prompt_preset.rs:68`
- **Evidence:**

  ```rust
  pub fn assemble_prompt_text(
      &self,
      global_rules: &[String],
      response_length: Option<&str>,
  ) -> String
  ```

  Method stitches preset sections, world-level `global_rules`, and settings-level `response_length` into one XML blob.
- **Why smell:** `PromptPreset` (a config object) is forced to know about world rules and response length preferences. It becomes a god-assembler for cross-cutting inputs. If a third external concern appears, another parameter is added.
- **Severity:** low
- **Proposed fix:** Move assembly to a `PromptAssembler` service that owns the combination of preset + world + settings.

### A9. Premature extraction — `push_section` helper

- **File:** `prompt_preset.rs:113`
- **Evidence:**

  ```rust
  fn push_section(parts: &mut Vec<String>, content: Option<&str>, tag: &str) {
      if let Some(content) = content {
          parts.push(wrap_xml(content, tag));
      }
  }
  ```

  Used exactly once inside `assemble_prompt_text`.
- **Why smell:** One-line wrapper around `if let` + `wrap_xml`. Extracted for nominal clarity but adds indirection and a private helper that can never be reused without duplicating `wrap_xml` anyway.
- **Severity:** low
- **Proposed fix:** Inline the `if let` blocks directly in `assemble_prompt_text`.

### A10. DB-born invalid object — `Message::from_db`

- **File:** `message.rs:130`
- **Evidence:**

  ```rust
  pub(crate) fn from_db(
      id: u64,
      sender: Option<String>,
      message_type: MessageType,
      timestamp: DateTime<Utc>,
      active_swipe_index: usize,
      is_deleted: bool,
  ) -> Self {
      Self {
          id,
          sender,
          text: String::new(),       // invalid empty
          message_type,
          timestamp,
          location_header: None,
          event_header: None,
          snapshot_id: None,
          active_swipe_index,
          swipes: Vec::new(),        // invalid empty
          is_deleted,
      }
  }
  ```

- **Why smell:** Factory produces a `Message` that violates runtime invariants (mirrored fields out of sync, no swipes). Relieves DB hydration mismatch rather than fixing the repository mapping layer.
- **Severity:** high
- **Proposed fix:** Move hydration logic into the repository/DB layer so `Message` is always constructed with its swipes and active text, or use a separate `DbMessageRow` type that does not claim to be a valid domain `Message`.

### A11. Thin DTO layering — `MessageEntry` duplicates `Message` + `Swipe`

- **File:** `state.rs:24` and `message_history.rs:131`
- **Evidence:**

  ```rust
  pub struct MessageEntry {
      pub id: u64,
      pub sender: Option<String>,
      pub text: String,
      pub message_type: MessageType,
      pub timestamp: DateTime<Utc>,
      pub location_header: Option<String>,
      pub event_header: Option<String>,
      pub swipe_count: usize,
      pub active_swipe_index: usize,
  }
  ```

  `MessageHistory::to_message_entries` flattens `Message` + active `Swipe` into this struct. No other behavior on `MessageEntry`.
- **Why smell:** Type exists solely to bridge `MessageHistory` to serialization/consumer layers. It mirrors fields already present in `Message`/`Swipe`, adding a parallel struct that must stay in sync when fields change.
- **Severity:** low
- **Proposed fix:** Implement `From<&Message> for MessageEntry` (if the DTO is still needed) so the mapping is centralized and colocated with the DTO definition, or let consumers work with `Message` directly.

### A12. Partial snapshot application — `GameStateSnapshot::apply_to`

- **File:** `state_snapshot.rs:45`
- **Evidence:**

  ```rust
  pub fn apply_to(&self, state: &mut crate::model::state::GameState) {
      state.movement = self.movement.clone();
      state.narrative.input_buffer = self.narrative.input_buffer.clone();
      state.narrative.last_trigger = self.narrative.last_trigger.clone();
      ...
  }
  ```

  Every field is cloned manually; `messages` are skipped because they live in a separate table.
- **Why smell:** Snapshot and `GameState` share partially overlapping shapes. Adding a new field to `GameState` requires remembering to update both `from_game_state` and `apply_to`. This is a refactor-be-damned fix for the fact that `GameState` is not fully snapshot-compatible.
- **Severity:** high
- **Proposed fix:** Treat snapshot load as full reconstruction via `GameState::from_snapshot` instead of partial mutation, or use a derive/lens to guarantee `apply_to` covers every field.

## Cross-cutting notes

- **Single-caller private helpers:** `push_section` (`prompt_preset.rs`), `wrap_xml` (`prompt_preset.rs`), and the cluster of `default_*` free functions in `settings.rs` are each used once. They are low-severity but reinforce a pattern of extracting names rather than concepts.
- **Cross-module DTO coupling:** `MessageEntry` lives in `state.rs` while its only producer `to_message_entries` lives in `message_history.rs`. The DTO should live with the module that defines the conversion or be eliminated.
- **Enum singletons:** `StatePatch` (agent.rs) and `TriggerRequirement` (trigger.rs) both commit the same premature-generalization sin. A project-wide lint or convention against single-variant enums would catch these early.

```acceptance-report
{
  "criteriaSatisfied": [
    {
      "id": "criterion-1",
      "status": "satisfied",
      "evidence": "12 concrete findings with file paths, line numbers, and severity ratings documented in zone-a-model.md"
    }
  ],
  "changedFiles": [],
  "testsAddedOrUpdated": [],
  "commandsRun": [],
  "validationOutput": [
    "READ-ONLY review completed — no project files modified"
  ],
  "residualRisks": [
    "A4 (Message mirrored fields) and A12 (partial snapshot apply) are the highest-risk abstraction debt; they require schema or architectural changes and cannot be fixed by local refactoring alone",
    "A7 (state.rs cohesion) is a large file split that may affect downstream imports and should be planned as a dedicated cleanup task"
  ],
  "noStagedFiles": true,
  "diffSummary": "No source changes made — review only",
  "reviewFindings": [
    "no blockers",
    "high: agent.rs:96 - StatePatch enum with single variant",
    "high: message.rs:18 - Message runtime-mirrored fields (refactor-be-damned extraction)",
    "high: message.rs:130 - Message::from_db produces invalid domain object",
    "high: state_snapshot.rs:45 - GameStateSnapshot::apply_to partial snapshot application",
    "med: trigger.rs:15 - TriggerRequirement enum with single variant",
    "med: agent.rs:88 / quantifier.rs:7 - Confidence vs QuantifierConfidence false duplication",
    "med: message_history.rs:15 - MessageHistory leaky wrapper",
    "med: template.rs:5 - TemplateVars type exists for one function",
    "med: state.rs - Coincidental cohesion grab-bag",
    "low: prompt_preset.rs:68 - PromptPreset assembles external concerns",
    "low: prompt_preset.rs:113 - push_section premature extraction",
    "low: state.rs:24 - MessageEntry thin DTO layering"
  ],
  "manualNotes": "Report written to E:\\John\\Github\\mrn-general\\reports\\zone-a-model.md. No edits performed on chronicler_engine source files."
}
```
