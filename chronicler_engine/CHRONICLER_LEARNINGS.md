# Chronicler Engine Learnings

## Core Architectural Patterns

### 0. Hypothetical Design (The Blueprint Pattern)
- **Problem**: Large architectural ideas (like Dual-LLM Scene Quantification) can pollute the active "Contract" or "Logic" tiers before implementation begins, causing confusion for the AI about what currently exists.
- **Solution**: Utilize the `/docs/specs/blueprints/` directory for "Design Artifacts" and "RFCs."
- **Benefit**: This allows us to "Capture" complex designs immediately while maintaining a strict boundary between what is **Imagined** (Blueprint) and what is **Implemented** (Contract/Logic).

### 1. Non-Blocking TUI Logic (The Worker Pattern)
- **Problem**: Calling LLM APIs (OpenRouter) typically takes 1-5 seconds. In a Terminal User Interface (TUI), this freezes the event loop, stopping cursor blinking and re-rendering.
- **Solution**: Use a combination of `std::thread::spawn` and `std::sync::mpsc::channel`. 
    - The main TUI thread remains responsive (polling events at ~10ms).
    - The "Action" triggers a background thread.
    - The background thread sends a `Message` through the channel when the LLM finishes.
- **Mistake Avoided**: Don't use `reqwest::blocking` on the main UI thread.

### 2. High-Fidelity Terminal Visuals (The "Bust Crop")
- **Problem**: Full-body character images (1080x1800) rendered in a terminal grid are blurry because the terminal's character height is too compressed (only ~40-60 rows).
- **Solution**: Implement an automatic "Bust Crop" utility using the `image` crate. 
    - Crop character sprites to the top 40% (Head/Shoulders). 
    - This doubles the pixel-to-terminal-character density, making facial features significantly sharper.
- **Learning**: Always prioritize "Half-block" rendering fallback for container-based terminals that lack Sixel/Kitty support.

### 4. Tiered Modularity (SDD 2.0)
- **Problem**: Flat source structures (10+ files in `src/`) create "Context Friction" for AI agents. The agent has to parse the entire root to distinguish between data models and rendering logic.
- **Solution**: Reorganize into Domain directories (`model/`, `engine/`, `narrative/`, `ui/`).
- **Breakthrough**: This "Tiered SDD" approach allows the AI to "Domain-Lock" its research. If asked to fix a UI bug, the agent immediately knows only the `ui/` directory is relevant, significantly increasing speed and reducing hallucination.

### 5. Arc-Based Zero-Cost Sharing
- **Problem**: Rust's ownership rules make it difficult to share the entire `GameState` with background threads for LLM processing without expensive multi-megabyte `clone()` calls.
- **Solution**: Wrap core immutable state (World, Map, Player) in `Arc<T>` (Atomic Reference Counting).
- **Benefit**: Background threads can now access the world lore and player history with zero cloning cost, ensuring the UI remains ultra-fast even as the world map grows.

### 3. Dependency Constraints
- **Chafa C-Lib**: `ratatui-image` defaults to using the `chafa` C-library for high-res rendering. This library is often missing in cloud/container dev environments.
- **Mitigation**: When using `ratatui-image` in sandbox environments, disable default features and enable only `crossterm` and `image-defaults`. Use `Picker::halfblocks()` as a reliable fallback.

## Repeating Mistakes & Gotchas

### State Initialization Drift
- Whenever updating the `GameState` struct (e.g., adding `narration_history` or `ui_state`), **Unit Tests in `state.rs` must be updated immediately**. 
- Failing to do so causes "Missing field" compilation errors that stall the build.

### Rust Lifetimes in Threads
- Moving data into `thread::spawn` blocks requires `'static` lifetimes. 
- **Learning**: References to objects (e.g., `&llm`) cannot be moved into threads. Objects must either be **Cloned**, wrapped in an **Arc**, or instantiated inside the thread.

## UI Best Practices
- **Story Continuity**: Users prefer a "Continuous Scroll" log for narrative games rather than clearing the console on every room change.
- **Input Immersion**: While the LLM is generating, disable user input and show a "The Game Master is thinking..." status indicator to manage user expectations.

## HTMX & Testing Debugging Learnings

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
- **Solution**: Run sequential with `cargo test -- --test-threads=1` or use distinct ports per test file.
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
