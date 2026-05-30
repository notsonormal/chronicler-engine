# Specification: Game Flow

> **Related Decisions**: [ADR-006](../adr/adr-006-quantifier-systems.md), [ADR-008](../adr/adr-008-sqlite-snapshot-persistence.md), [ADR-010](../adr/adr-010-concurrency-generation-gate.md), [ADR-013](../adr/adr-013-message-domain-model.md), [ADR-017](../adr/adr-017-message-swipes.md)

## Overview

This document defines the core game loop - the play-by-play experience from starting the game to receiving LLM responses. This is the fundamental user experience that must work reliably.

## The Game Flow

```mermaid
flowchart TD
    Start(["**START GAME**<br>*(Server starts, loads world)*"])
    
    Phase1["**PHASE 1: INITIALIZE**<br>1. Load world data<br>2. Set player in starting room<br>3. Render initial UI (Header, Story Log, Sidebar)<br>4. Establish HTMX polling (story-log every 2s, status every 5s, others every 5s)"]
    
    Phase2["**PHASE 2: AWAIT INPUT**<br>*(Status: 'Ready')*<br>User types command → submits form"]
    
    Phase3["**PHASE 3: PROCESS ACTION**<br>1. Generation gate: reject if action already in flight<br>2. Parse command & execute game logic<br>3. Log command as 'Input'<br>4. Set status to 'Generating' + phase 'Narrating'<br>5. Offload to `tokio::task::spawn_blocking` for LLM work"]
    
    Phase4["**PHASE 4: MAIN LLM NARRATION**<br>*(Phase: Narrating)*<br>1. Build prompt via LayeredPromptAssembler<br>2. Send to LLM (see Context Pipeline below)<br>3. Add to history as 'Narration'<br>4. **Cancellation checkpoint** — aborts if token cancelled"]
    
    Phase45["**PHASE 4.5: QUANTIFIER & MOVEMENT**<br>*(Phase: Quantifying)*<br>1. Post-narration Quantifier analyzes<br>2. Process movement intent<br>3. If moved: update room state (no additional LLM call — arrival is part of main narration)<br>4. Determine NPC Enter/Leave events"]
    
    Phase5["**PHASE 5: TRIGGER EVALUATION**<br>*(Phase: GeneratingEvent — only if trigger fires)*<br>1. `evaluate_triggers(state)` — first match only (inside lock)<br>2. Build prompt with continuation context<br>3. **Cancellation checkpoint** — aborts before second LLM call if token cancelled<br>4. Call LLM (frontend can poll main narration)<br>5. **Cancellation checkpoint** — aborts before commit if token cancelled<br>6. Re-acquire lock → add event header + trigger narration<br>7. Mark trigger as fired"]

    Phase55["**PHASE 5.5: POST-EVENT QUANTIFIER**<br>*(Phase: Quantifying)*<br>1. Post-continuation Quantifier analyzes<br>2. Detect NPCs introduced by event text<br>3. Determine NPC Enter/Leave events<br>4. Update scene.npcs_in_area"]
    
    Phase6["**PHASE 6: POLLING UPDATE**<br>1. Client polls /fragment/story-log (2s)<br>2. Client polls /status/generating (5s)<br>3. When status becomes idle, JS immediately triggers story-log refresh<br>4. Server returns updated HTML<br>5. HTMX swaps content"]

    Start --> Phase1
    Phase1 --> Phase2
    Phase2 --> Phase3
    Phase3 --> Phase4
    Phase4 --> Phase45
    Phase45 --> Phase5
    Phase5 --> Phase55
    Phase55 --> Phase6
    Phase6 -.->|BACK TO 2| Phase2
```

## The LLM Context Pipeline

When the engine needs LLM narration (during Phase 4), it builds a comprehensive prompt using the **7-layer system** (see [`prompt_system.md`](prompt_system.md)):

```mermaid
flowchart TD
    Start(["**PHASE 4: LLM GENERATION**<br>*(If narrative action)*"])
    
    Step1["**1. Build 7-layer prompt (SillyTavern-style)**"]
    Sub1["Layer 0: System prompt (XML-wrapped sections: role, instructions, writing_style, global_rules, output_format)<br>Layer 1: Game state (room, NPCs)<br>Layer 2: NPC cards (in-room NPCs only)<br>Layer 3: Player persona<br>Layer 4: World info (keyword-triggered lore)<br>Layer 5: Full narration history (up to 1000 entries)<br>Layer 6: User input (current action)"]
    
    Step2["**2. Token budget check**<br>*(8192 max, truncate if overflow)*"]
    Step3["**3. Send to LLM**<br>*(OpenRouter/DeepSeek)*"]
    Step4["**4. Receive narration response**"]
    Step5["**5. Add to narration history**<br>*(as 'Narration')*"]
    Step6(["**6. Set status back to 'Ready'**"])

    Start --> Step1
    Step1 --> Sub1
    Sub1 --> Step2
    Step2 --> Step3
    Step3 --> Step4
    Step4 --> Step5
    Step5 --> Step6
```

## Test Scenarios

These Gherkin scenarios are covered by **browser/integration tests** (`tests/browser/trigger.rs`, `tests/browser/structure.rs`). The `flow_mock/` test suite covers sequential service-level retry and state consistency flows rather than end-to-end browser scenarios.

### Scenario 1: Initial Load
```gherkin
Given the server is running with "test" world
When the user opens http://127.0.0.1:3000
Then the header shows "Chronicler Engine | <starting_room>"
And the story-log shows a minimal header with the room name
And after LLM generates: the story-log shows LLM-generated arrival narration
And the status shows "Ready"
```
- **Covered by**: `tests/browser/structure.rs` — `test_page_loads`, `test_header_displays_game_title`

### Scenario 2: Free Action (e.g., "look around")
```gherkin
Given the game is loaded
When the user enters "look around" and submits
Then the status shows "Generating narration..."
And after LLM generates response, the story-log shows the LLM description
And the status shows "Ready"
```
- **Covered by**: `tests/browser/trigger.rs` — `test_look_command_adds_narration_entries`, `test_freeaction_without_movement_works`

### Scenario 3: Quantifier-Driven Movement
```gherkin
Given the game is loaded at starting room
When the user enters "I walk to the village square" and submits
Then the status shows "Generating narration..."
And after LLM generates response, the quantifier detects movement intent
Then the status shows "Quantifying scene..."
And the story-log shows the LLM narration for arrival with an inline location header for the new room
And the visual-sidebar shows the new room's image and NPCs
And the status shows "Ready"
```
- **Covered by**: `tests/browser/trigger.rs` — `test_freeaction_with_movement_no_triggers`

### Scenario 4: Free Action (LLM Narration)
```gherkin
Given the game is loaded
When the user enters "examine the mysterious orb" and submits
Then the status shows "Generating narration..."
And after LLM generates response, the story-log shows the LLM's description of the orb
And the status shows "Ready"
```
- **Covered by**: `tests/browser/trigger.rs` — `test_freeaction_without_movement_works`

## Error Handling

### Invalid Command
- Show helpful error in story-log
- Status returns to "Ready"

### Granular Status Phases

During LLM processing, the UI displays granular status phases instead of a single "Thinking..." message:

| Phase | Display Text | Endpoint Value | When Active |
|-------|-------------|----------------|-------------|
| `Narrating` | "Generating narration..." | `narrating` | During main LLM narration (Phase 4) |
| `Quantifying` | "Quantifying scene..." | `quantifying` | During post-narration quantifier analysis (Phase 4.5 or 5.5) |
| `GeneratingEvent` | "Generating event..." | `generating-event` | During trigger continuation narration (Phase 5) |

**Design Principles:**
- `GenerationStatus` (Idle/Generating/Error) remains unchanged for backward compatibility
- `is_generating()` is the single source of truth for disabling UI elements
- Phase is a secondary display concern only — all phases use the same `.thinking` CSS class
- The frontend maps endpoint values (`narrating`, `quantifying`, `generating-event`) to human-readable text
- An optimistic "Thinking..." is shown immediately on form submit before the first poll response

### Retry Flow

Retrying a response (via the right swipe arrow on the last message) branches based on whether the last AI-generated content was a **main narration** or a **trigger event continuation**:

```mermaid
flowchart TD
    Start(["User clicks right arrow (latest swipe)"])
    Check{Last response was event?}
    Main["**Main Narration Retry**"]
    Event["**Event Continuation Retry**"]
    FindMain["Find last Input message<br>load its `snapshot_id`"]
    FindEvent["Find last non-event message<br>load its `snapshot_id`"]
    SoftDelete["Soft-delete messages after anchor"]
    Preserve["Preserve old target as a swipe"]
    Apply["Apply snapshot to structural state"]
    ReGenMain["Re-run Phase 4→5→5.5<br>(new quantifier + triggers)"]
    ReGenEvent["Re-run Phase 5 only"]
    Phase55["**PHASE 5.5: POST-EVENT QUANTIFIER**<br>*(Phase: Quantifying)*<br>1. Post-continuation Quantifier analyzes<br>2. Detect NPCs introduced by retried text<br>3. Update scene.npcs_in_area"]
    Migrate["Migrate old swipes to new message"]
    Purge["Purge soft-deleted messages"]
    Save["Save final state"]

    Start --> Check
    Check -->|No| Main
    Check -->|Yes| Event
    Main --> FindMain
    Event --> FindEvent
    FindMain --> SoftDelete
    FindEvent --> SoftDelete
    SoftDelete --> Preserve
    Preserve --> Apply
    Apply --> ReGenMain
    Apply --> ReGenEvent
    ReGenMain --> Migrate
    ReGenEvent --> Migrate
    Migrate --> Purge
    Purge --> Save
```

**Key behaviors** (enforced by `tests/flow_mock/`):
- **Main retry** finds the last `Input` message, loads the snapshot stored in its `snapshot_id`, soft-deletes all messages after that input, preserves the old narration as a swipe, and re-runs the full pipeline: quantifier, movement, triggers, and post-event quantifier. On success, old swipes migrate to the new message and soft-deleted messages are purged. On failure, soft-deleted messages are restored (`test_retry_main_narration_applies_new_quantifier_result`, `test_main_retry_reevaluates_triggers`).
- **Event retry** finds the last non-event message (the anchor before any event messages), loads its `snapshot_id` snapshot, soft-deletes event messages, preserves the old event as a swipe, and regenerates only the continuation text using stored trigger prompts (`StoredTriggerContext`) (`test_retry_event_continuation_preserves_quantifier_result`).
- **Swipe navigation** (left/right arrows on the last message) switches between swipes. Restoring a swipe loads its `snapshot_id` and rewinds world state to that point. Only the last message is swipeable (`test_switch_swipe_changes_active_swipe`).
- **Retrigger Event** appears on a narration swipe when its restored snapshot contains `last_trigger` and there are no event messages after it. Clicking it runs `run_trigger_continuation()` from that snapshot state without rerunning the main narration.
- Snapshots are standalone — no `base_snapshot_id` chain. Each message carries the `snapshot_id` of the state captured after it was created. Each `Swipe` also stores its own `snapshot_id` so switching swipes restores the exact state that produced that text.
- If the anchor message has no `snapshot_id` or the snapshot is missing, retry fails gracefully (`test_retry_no_pre_main_snapshot`).

### Polling-based Updates
- HTMX polls story-log every 2 seconds for update visibility
- Status-display polls `/status/generating` every 5 seconds for responsive button state
- When `/status/generating` returns `idle`, JavaScript immediately triggers a story-log refresh via `htmx.trigger('#story-log', 'htmx:refresh')` — no waiting for the next story-log poll
- `/status/generating` returns phase endpoint values (`idle`, `narrating`, `quantifying`, `generating-event`)
- No manual reconnection needed

## Reference Implementation

- **Server**: `src/server/fragments/actions.rs` - `action_handler`, `process_action`
- **HTMX Polling**: `assets/index.html` - story-log `hx-trigger="load, every 2s"`; status-display `hx-trigger="load, every 5s"`; visual-sidebar & action-hints `hx-trigger="load, every 5s"`
- **LLM**: `src/narrative/llm/backend.rs` - `LlmBackend` trait (`narrate_action`, `narrate_arrival`)
- **Prompt Assembler**: `src/narrative/prompt/assembler.rs` - 7-layer prompt construction
- **Mock Flow Tests**: `tests/flow_mock/` - Sequential service-level flow tests with mock backends (retry, state consistency, quantifier movement)
- **LLM Tests**: `tests/flow_llm_tests.rs` - Real LLM smoke tests (requires `OPENROUTER_API_KEY`)
