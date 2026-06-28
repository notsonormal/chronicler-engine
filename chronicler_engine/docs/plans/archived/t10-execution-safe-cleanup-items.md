# T10 Execution Plan: Safe Cleanup Items

**Parent Plan:** [t10-low-priority-cleanup-bundle.md](./t10-low-priority-cleanup-bundle.md)
**Status:** Ready for implementation
**Date:** 2026-06-28
**Depends on:** none
**Blocks:** none
**Priority:** P3
**Scope:** 9 mechanical items in a single bundled PR. Investigation reports under `investigations/`.

---

## Summary

Implement the 9 cleanup items that require no further product decisions, driven
by scout investigation reports (`investigations/*.md`). One bundled PR.

### Excluded (with reason)

- **B12** `trigger_eval.rs` cohesion — investigation proved plan description
  stale. `NpcEncounterLog` CRUD lives in `model/trigger.rs`, not
  `engine/trigger_eval.rs`. File already well-separated. No action.
- **D9** `add_status_swap_headers` — plan claimed 1 caller, investigation found
  3 (all in `action_check_handler`). Helper is justified. No change.
- **N14** `Confidence` derive `Ord` — requires variant reorder; needs separate
  decision (serde positional encoding concern).
- **M3** `response_length: Option<&str>` — investigation recommends keep as
  free-form natural-language LLM instruction; enum/u32 would break configs.

---

## Items (suggested implementation order)

### 1. A9 — Add `#[inline]` to `push_section`

**File:** `chronicler_engine/src/narrative/prompt/assembler.rs:49`

Investigation (`investigations/A9-M3-prompt-assembler.md`): 6 homogeneous
callers in same file. Helper justified.

```rust
#[inline]
fn push_section(parts: &mut Vec<String>, content: Option<&str>, tag: &str) {
    if let Some(content) = content {
        parts.push(wrap_xml(content, tag));
    }
}
```

**Verify:** `cargo build` clean.

---

### 2. M4 — Add `is_low()` / `is_medium()` to `QuantifierConfidence`

**File:** `chronicler_engine/src/model/quantifier.rs:18-22`

Investigation (`investigations/A11-N14-M4-types-models.md`): existing call sites
use `== Low` or match arms; helpers improve readability without `Ord`
complexity.

```rust
impl QuantifierConfidence {
    pub fn is_high(&self) -> bool {
        matches!(self, Self::High)
    }

    pub fn is_medium(&self) -> bool {
        matches!(self, Self::Medium)
    }

    pub fn is_low(&self) -> bool {
        matches!(self, Self::Low)
    }
}
```

Optional: refactor `orchestration.rs:81` `== QuantifierConfidence::Low` ->
`.is_low()`.

**Verify:** tests pass; clippy clean.

---

### 3. A11 — Add `impl From<&Message> for MessageEntry`

**Files:**
- `chronicler_engine/src/model/state/message_types.rs` (add impl)
- `chronicler_engine/src/model/message_history.rs:137-151` (collapse caller)

Investigation (`investigations/A11-N14-M4-types-models.md`): `MessageEntry`
mirrors `Message` (domain type at `model/message.rs:21-49`). Single mapping
site.

```rust
impl From<&crate::model::message::Message> for MessageEntry {
    fn from(msg: &crate::model::message::Message) -> Self {
        Self {
            id: msg.id,
            sender: msg.sender.clone(),
            text: msg.text().to_string(),
            message_type: msg.message_type.clone(),
            timestamp: msg.timestamp,
            location_header: msg.location_header().map(|s| s.to_string()),
            event_header: msg.event_header().map(|s| s.to_string()),
            swipe_count: msg.swipe_count(),
            active_swipe_index: msg.active_swipe_index,
        }
    }
}
```

`message_history.rs:137-151` becomes:
```rust
pub fn to_message_entries(&self) -> Vec<MessageEntry> {
    self.messages.iter().map(MessageEntry::from).collect()
}
```

**Verify:** existing `message_history` tests pass.

---

### 4. D2 — Inline `empty_to_none` with `Option::filter`; delete helper

**Files:**
- `chronicler_engine/src/storage/backend/helpers.rs` (delete function)
- `chronicler_engine/src/storage/backend/personas.rs:63-65` (3 sites)
- `chronicler_engine/src/storage/backend/characters.rs:70-72` (3 sites)

Investigation (`investigations/D2-D11-storage.md`): 6 homogeneous callers.
Idiomatic replacement: `Option::filter`.

```rust
// Before
empty_to_none(card.sheet.summary.as_deref().unwrap_or(""))

// After
card.sheet.summary.as_deref().filter(|s| !s.is_empty())
```

Check `helpers.rs` for remaining exports; delete file if now empty or leave
empty module decl (whichever matches existing convention).

**Verify:** `personas_tests.rs`, `characters_tests.rs` pass.

---

### 5. N16 — Remove `ApplicationService::list_personas` passthrough

**Files:**
- `chronicler_engine/src/application/application_service.rs:331-336` (delete method)
- `chronicler_engine/src/server/games_fragment/handlers.rs:58` (update caller)

Investigation (`investigations/D7-D9-N13-N16-server-fragments.md`): pure
passthrough, 1 caller. Matches `list_worlds` direct-storage pattern.

```rust
// Before (games_fragment/handlers.rs:58)
match state.application_service.list_personas(ctx.clone()) { ... }

// After
match ctx.storage.list_personas() { ... }
```

Error type changes from `ApplicationError` to `EngineError` at call site -
adjust match arms.

**Verify:** `games_fragment` handler tests pass.

---

### 6. N13 — Replace `Ok(_) => unreachable!()` idiom with single match (8 sites)

**Files:**
- `chronicler_engine/src/server/fragments/history.rs:22-26,35-39`
- `chronicler_engine/src/server/fragments/misc/retrigger.rs:12-16`
- `chronicler_engine/src/server/fragments/misc/retry.rs:12-16`
- `chronicler_engine/src/server/games_fragment/handlers.rs:28-32,108-112,125-129`
- `chronicler_engine/src/server/worlds_fragment/handlers.rs:201-205`

Investigation (`investigations/D7-D9-N13-N16-server-fragments.md`): current
pattern double-calls `ctx_or_error(&state)` and uses `unreachable!()`. Replace
with single match - no double-call, no `unreachable!()`.

```rust
// Before
let Ok(ctx) = ctx_or_error(&state) else {
    return match ctx_or_error(&state) {
        Ok(_) => unreachable!(),
        Err(e) => *e,
    };
};

// After
let ctx = match ctx_or_error(&state) {
    Ok(ctx) => ctx,
    Err(e) => return *e,
};
```

**Verify:** clippy clean; handler tests pass.

---

### 7. D11 — Add `from_row` to 6 missing Db* structs

**Files (struct defs):**
- `storage/models/game.rs` — `DbGame`
- `storage/models/game_state_snapshot.rs` — `DbGameStateSnapshot`
- `storage/models/message.rs` — `DbMessage`, `DbSwipe`
- `storage/models/llm_message.rs` — `DbLlmMessage`
- `storage/models/prompt_preset.rs` — `DbPromptPreset`

**Files (call sites):**
- `storage/backend/games.rs:26` (inline closure)
- `storage/backend/snapshots.rs:64` (inline closure)
- `storage/backend/messages.rs:83` (inline closure)
- `storage/backend/swipes.rs:120` (inline closure)
- `storage/backend/llm_messages.rs:72` (inline closure)
- `storage/backend/presets.rs` (`db_row_to_preset` standalone fn)

Investigation (`investigations/D2-D11-storage.md`): 5 of 11 Db* structs have
`from_row`; all 11 used in `query_map`/`query_row`. Adding the missing 6
matches the established pattern.

```rust
impl DbGame {
    pub fn from_row(row: &rusqlite::Row<'_>) -> rusqlite::Result<Self> {
        Ok(DbGame {
            id: row.get(0)?,
            // ...fields by column index, copied from current inline closure
        })
    }
}
```

Call-site update:
```rust
// Before
.query_map([], |row| {
    Ok(DbGame { id: row.get(0)?, /* ... */ })
})

// After
.query_map([], DbGame::from_row)
```

**Critical:** Confirm SELECT column order matches index order in each new
`from_row`.

**Verify:** per-struct storage tests pass (`games_tests.rs`,
`snapshots_tests.rs`, `messages_tests.rs`, `swipes_tests.rs`,
`llm_messages_tests.rs`, `presets_tests.rs`).

---

### 8. D7 — Split `CheckTextForm` from `ActionForm`

**Files:**
- `chronicler_engine/src/server/fragments/misc/text_check.rs` (new struct + handler change)
- `chronicler_engine/src/server/fragments/actions.rs:22-24` (`ActionForm` stays for action handlers)

**Template audit (DONE):** `/check-text` endpoint receives `name="command"` from
`assets/index.html:404` (JS `formData.append("command", text)`). 4 test bodies
in `tests/http/fragment.rs` (lines 947, 982, 1139, 1170) also use `command=...`.

**Decision:** Keep wire field name `command`. Only the Rust type changes -
semantic clarity gained without touching frontend or tests.

Implementation (add struct in `text_check.rs` co-located with handler):
```rust
#[derive(serde::Deserialize)]
pub struct CheckTextForm {
    pub command: String,
}
```

Handler:
```rust
// Before
Form(form): Form<ActionForm>,
// After
Form(form): Form<CheckTextForm>,
```

Body unchanged: `form.command` continues to work.

**Verify:** `tests/http/endpoints/text_check.rs` and `tests/http/fragment.rs`
check-text cases pass.

---

## Verification

Per-item: `cargo build` clean, no new clippy warnings, affected tests pass.

Final gate:
```bash
cd chronicler_engine && python build.py
```
Must pass clean: fmt + clippy + tests + coverage.

---

## Implementation Notes

- All 9 items in one bundled PR.
- Items 1-7 independent and purely mechanical; any order works.
- Item 8 (D7) audit complete - keep `command` field, add new `CheckTextForm` struct.
- Per AGENTS.md: prefer subagents. `worker` for implementation, `delegate` for
  running `build.py`. Top-tier model reserved for final diff review.
- Suggested worker split (each is a focused mechanical task):
  - Worker A: items 1, 2, 3 (type-only changes in `model/`)
  - Worker B: items 4, 5 (storage + application service)
  - Worker C: item 6 (N13 - 8 mechanical sites, same pattern)
  - Worker D: item 7 (D11 - largest; 6 structs + 6 call sites)
  - Worker E: item 8 (D7 - 1 struct + 1 handler signature)
  - Workers B-E can run in parallel worktrees; Worker A is independent.

---

## Pre-Implementation Checklist

- [x] Scope confirmed (9 items; B12/D9/N14/M3 excluded)
- [x] D7 template audit complete (keep `command` wire name)
- [x] Bundle strategy confirmed (single PR)
