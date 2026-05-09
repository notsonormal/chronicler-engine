# Debugging Playbook: Chronicler Engine

Use this file when investigating a bug report or unexpected behaviour in the Chronicler Engine.
Follow the relevant section for your symptom — do not skip steps.

---

## General First Steps

1. **Reproduce with logs enabled.**
   ```bash
   RUST_LOG=info cargo run -- --world redmist_estate --port 3000
   # For deep HTTP-level tracing:
   RUST_LOG=debug cargo run -- --world redmist_estate --port 3000
   ```
2. **Read the actual error message verbatim** from the terminal — do not paraphrase it.
3. **Check generation status** via `GET /status/generating` (returns `"idle"` or `"generating"`).
4. **Dump current state** via `GET /debug/state` — shows room, NPCs in area, character_state, and the last 5 log entries.

---

## Symptom: Trigger Not Firing

**Most common causes (in order):**

1. **Wrong `room_id` on the trigger** — global triggers have no `room_id`. Room-scoped triggers only fire when `state.current_room_id == trigger.room_id`. A typo here silently prevents the trigger.
   - Check: `python build.py --validate-data` automatically catches orphan `room_id` references, but double check that the room_id you actually *want* is what is listed.

2. **`times_met` counter already incremented** — `TimesMet Eq 0` only fires when `times_met == 0`. The counter increments when the quantifier detects the NPC entering the area.
   - Check: `GET /debug/state` → look at `character_state.<npc_id>.times_met`.
   - See: `docs/system/triggers.md §Timing: Evaluate BEFORE Increment`.

3. **Trigger already marked fired** — non-repeatable triggers set `trigger_fired[idx] = true` after firing once.
   - Check: `GET /debug/state` → look at `character_state.<npc_id>.triggers_fired`.

4. **NPC not in `state.npcs` map** — triggers are evaluated against `state.npcs`, not just the current room.
   - Check: `GET /debug/state` → `npcs_in_area`. If the NPC is absent, verify it's listed in `data/characters/` and loaded.

5. **Quantifier returning Low confidence** — if the quantifier fails, `npcs_in_area` falls back to static room NPCs and `times_met` may not increment.
   - Check logs for `[Quantifier] Low confidence` warnings.

---

## Symptom: Narration Is Empty, Wrong, or "[Trigger narration failed]" Appears

1. **Check `generation_state.status`** via `GET /debug/state`. If it shows `Error(...)`, the error message is the starting point.

2. **Check LLM logs** — with `RUST_LOG=info`, look for `[LLM][req:N]` lines:
   - `Response status: 200` — API call succeeded; check parse.
   - `Non-success HTTP status: 4xx/5xx` — API key, rate limit, or model issue.
   - `All attempts failed` from the quantifier — backend unreachable.
   - With `RUST_LOG=debug`, `Request payload:` shows the full prompt sent to the API.

3. **Check `build_split()` failure** — if logs show `Failed to build trigger continuation prompt:`, the prompt budget calculation failed. Check `docs/reference/prompt_budget.md` for context limits.

4. **Check `DeepSeekBackend` misconfiguration** — if the narration connection is set to `DeepSeek`, all calls return `Err` ("not yet implemented"). Switch to OpenRouter or Ollama.

5. **Check `MockBackend` in test environments** — if `LLM_BACKEND=mock` is set, narration returns a predictable `[MockNarration] ...` string. Empty response tests use `MockBackend::with_empty_response()`.

---

## Symptom: Player Is in the Wrong Room (Navigation Bug)

1. **Check quantifier movement detection** — logs show `[Quantifier] Detected movement: Entering destination: <room_id>` or `No movement detected`.

2. **Check room ID vs room name** — the quantifier returns a destination string that `attempt_semantic_walk` matches against room IDs and names. A mismatch creates a dynamic room (`dynamic_<name>`).
   - `GET /debug/state` → `current_room_id` — if it starts with `dynamic_`, the destination didn't match any real room.

3. **Check dynamic rooms** — `state.dynamic_rooms` accumulates rooms created when the quantifier returns an unrecognised destination. These are ephemeral and not in `map.json`.

4. **Check `navigation_description`** — rooms can have a `navigation_description` hint that the quantifier uses to improve movement detection accuracy. If it's missing or wrong, movement detection degrades.

---

## Symptom: Test Failure

**Mandatory protocol — do not skip:**

1. **Quote the verbatim failure message.** Do not summarise it.
2. **Read the test code** — understand what the test is actually asserting before forming a hypothesis.
3. **Read the relevant system doc** — check `docs/system/` for the subsystem under test.
4. **Do not rationalize failures away** — a test failure is a real signal.

**Common causes:**

- `make_test_state()` / `TestGameState::in_room()` uses `"room1"` as the default room ID. If the test expects a different room, use a named builder variant from `src/test_support/`.
- Port conflicts in integration tests — each test file uses a distinct port (see `CHRONICLER_LEARNINGS.md` in `chronicler_engine/`).
- `MockBackend::default()` always succeeds. Tests for failure paths must use `MockBackend::failing()`, `MockBackend::with_empty_response()`, or `MockBackend::with_failing_trigger_narration()`.

---

## Error Taxonomy

**Primary reference:** `docs/diagnostics/error_catalog.md` — structured catalog with "First Check", "Common Causes", and "Related Invariants" for every variant.

Use `match` on structured variants instead of string grepping. The old `msg.contains(...)` pattern has been removed.

| `EngineError` variant | Most likely cause | First file to check |
|---|---|---|
| `Llm(LlmFailure::Http { status, .. })` | API key, rate limit, or model routing issue | `src/narrative/llm_client.rs` → `[LLM][req:N]` logs |
| `Llm(LlmFailure::EmptyResponse)` | Model returned empty content field | `src/narrative/llm_client.rs` → `extract_content_from_response` |
| `Llm(LlmFailure::Network { .. })` | Backend unreachable or timeout | `src/narrative/llm_client.rs` → check URL connectivity |
| `Llm(LlmFailure::ParseError { .. })` | Non-JSON or unexpected response shape | `src/narrative/llm_client.rs` → raw response in debug logs |
| `Llm(LlmFailure::Timeout)` | Request exceeded 180s | `src/narrative/llm_client.rs` → elapsed time in logs |
| `Narrative(NarrativeFailure::PromptBuild { .. })` | Prompt budget exceeded | `src/narrative/prompt.rs` → `build_split()` |
| `Narrative(NarrativeFailure::Generation { .. })` | Backend failed after prompt built | Backend impl (e.g. `mock.rs`, `deepseek.rs`) |
| `RoomNotFound(String)` | `current_room_id` not in map or `dynamic_rooms` | `src/engine/logic.rs` → `get_current_room()` |
| `Config(String)` | Settings file missing, malformed, or backend not implemented | `src/settings.rs`, `src/narrative/llm.rs` |
| `DataLoad { path, .. }` | JSON file doesn't match schema or has wrong field names | `data/schemas/` → run `python build.py --validate-data` |
| `Internal(InternalError { invariant })` | Logic invariant violated — log entry not found, retry with no input | `src/model/state.rs` → method named in the `invariant` field |
| `Parse(String)` | Serde deserialization failure on a data file | Check the file path in the error, compare to schema |

---

## State Mutation Order Invariant

Inside `execute_freeaction_impl` in `src/engine/action_processing.rs`, state is mutated in this exact order:

1. `handle_movement()` — may change `current_room_id`
2. Resolve current NPCs from quantifier result
3. `state.add_log(narration)` — narration logged to history
4. `evaluate_and_narrate_triggers()` — reads history (sees step 3's narration as context), may add more logs
5. `compute_npc_events()` + `apply_npc_events()` — mutates `character_state`

**This order is load-bearing.** Swapping steps 3 and 4 means triggers won't see the current narration as context. Swapping 4 and 5 means `character_state` changes happen before triggers evaluate them.

See: `docs/system/triggers.md §Mutation Order Invariant`.
