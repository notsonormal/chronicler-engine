# Chronicler Engine Learnings

## Core Architectural Patterns

### 0. Hypothetical Design (The Blueprint Pattern)
- **Problem**: Large architectural ideas (like Dual-LLM Scene Quantification) can pollute the active "Contract" or "Logic" tiers before implementation begins, causing confusion for the AI about what currently exists.
- **Solution**: Utilize the `/docs/specs/blueprints/` directory for "Design Artifacts" and "RFCs."
- **Benefit**: This allows us to "Capture" complex designs immediately while maintaining a strict boundary between what is **Imagined** (Blueprint) and what is **Implemented** (Contract/Logic).

### 1. Tiered Modularity (SDD 2.0)
- **Problem**: Flat source structures (10+ files in `src/`) create "Context Friction" for AI agents. The agent has to parse the entire root to distinguish between data models and rendering logic.
- **Solution**: Reorganize into Domain directories (`model/`, `engine/`, `narrative/`, `ui/`, `server/`).
- **Breakthrough**: This "Tiered SDD" approach allows the AI to "Domain-Lock" its research. If asked to fix a UI bug, the agent immediately knows only the `ui/` directory is relevant, significantly increasing speed and reducing hallucination.

### 2. Arc-Based Zero-Cost Sharing
- **Problem**: Rust's ownership rules make it difficult to share the entire `GameState` with background threads for LLM processing without expensive multi-megabyte `clone()` calls.
- **Solution**: Wrap core immutable state (World, Map, Player) in `Arc<T>` (Atomic Reference Counting).
- **Benefit**: Background threads can now access the world lore and player history with zero cloning cost, ensuring the UI remains ultra-fast even as the world map grows.

### 3. LLM Backend Trait Pattern
- **Problem**: Need to support both mock LLM (for testing/fast iteration) and real LLM (OpenRouter).
- **Solution**: Define `LlmBackend` trait in `src/narrative/llm.rs` with methods like `generate()`.
- **Implementation**: `MockLlmBackend` for tests, `OpenRouterClient` for production.
- **Configuration**: Configure connections in `data/settings.json` — see `docs/adr/adr-007-settings-system.md`.

## Repeating Mistakes & Gotchas

### State Initialization Drift
- Whenever updating the `GameState` struct (e.g., adding `narration_history` or `ui_state`), **Unit Tests in `state.rs` must be updated immediately**.
- Failing to do so causes "Missing field" compilation errors that stall the build.

### Rust Lifetimes in Async
- Moving data into `tokio::spawn` blocks requires `'static` lifetimes.
- **Learning**: References to objects (e.g., `&llm`) cannot be moved into async tasks. Objects must either be **Cloned**, wrapped in an **Arc**, or instantiated inside the task.

## HTMX & Web UI Learnings

### WebSocket Limitations in Headless Browsers
- **Problem**: WebSocket connections via HTMX SSE extension are unreliable in Playwright's headless Chromium mode. Tests fail intermittently with connection errors.
- **Solution**: Implement polling as a reliable fallback. HTMX's `hx-trigger="load, every 5s"` provides 100% reliable updates.
- **Trade-off**: Polling has a 5-second delay vs instant WebSocket updates, but works reliably in all test environments.

### LLM State Tracking for Tests
- **Problem**: Tests need to wait for LLM to finish generating before checking UI. Blind sleep delays are flaky (too short sometimes, too long other times).
- **Solution**: Add a `/status/generating` endpoint that returns "generating" or "idle" based on `state.tui_state.is_generating`. Tests poll this endpoint to wait for LLM completion.
- **Pattern**:
  ```rust
  // Server sets is_generating = true before LLM call, false after
  state_guard.tui_state.is_generating = true;  // Before async LLM thread
  // ... LLM processing ...
  state_guard.tui_state.is_generating = false; // After completion
  ```

### HTMX innerHTML Swap Gotcha
- **Problem**: `render_story_log()` returned `<div id="story-log">...</div>`, but HTMX's `hx-swap="innerHTML"` replaces the content INSIDE the target. This caused duplicate wrappers and content not appearing.
- **Solution**: Fragment endpoints should return just the content (log entries), not the wrapper div. The wrapper div exists in the page template.
- **Correct**: Returns `log_entry_html`
- **Incorrect**: Returns `<div id="story-log">log_entry_html</div>`

### Test Port Conflicts
- **Problem**: Multiple test files use different ports but may conflict when run in parallel with `--test-threads=1`.
- **Solution**: Run sequential with `cargo nextest run --test-threads 1` or use distinct ports per test file.
- **Current ports**:
  - `flow_mock_tests.rs`: 3006
  - `flow_llm_tests.rs`: 3007
  - `behavior_tests.rs`: 3003
  - `layout_tests.rs`: 3002
  - `spec_tests.rs`: 3001

### Test Helper Pattern
- **Problem**: Tests needed a reusable way to wait for LLM completion.
- **Solution**: Created `wait_for_llm_idle(port, timeout)` in `test_utils.rs`:
  ```rust
  pub async fn wait_for_llm_idle(port: u16, timeout: Duration) -> Result<(), ()> {
      // Poll /status/generating until "idle" or timeout
  }
  ```

## Image Handling

### Character Images
- **Location**: `data/images/` directory stores character sprites
- **Files**: Full images (e.g., `louise.png`) and cropped headshots (e.g., `louise_headshot.png`)
- **Usage**: Loaded via `data/characters/<world>/` JSON configs that reference image paths

## Data Structure

### World Configuration
- **world.json**: High-level rules, universe facts, LLM prompt templates
- **map.json**: `Overworld -> Region -> Room` topology and navigation
- **personas/**: Player state and inventory (shared across worlds)
- **characters/**: NPC definitions with image references (shared across worlds)