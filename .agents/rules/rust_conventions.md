# Rust Conventions for Chronicler Engine

## Error Handling

- **Prefer `Result` over `panic!`** - Never use `.expect()` or `unwrap()` on fallible operations
- Use the `EngineError` enum from `src/error.rs` for custom errors
- Always propagate errors with `?` or `map_err()`
- Never panick in library code; return meaningful errors instead

```rust
// Good
fn get_current_room(state: &GameState) -> Result<&Room, EngineError> {
    get_room_by_id(state, &state.current_room_id)
        .ok_or_else(|| EngineError::RoomNotFound(state.current_room_id.clone()))
}

// Bad - never do this
fn get_current_room(state: &GameState) -> &Room {
    get_room_by_id(state, &state.current_room_id).unwrap()
}
```

## Naming Conventions

- **Functions & variables**: `snake_case`
- **Types & traits**: `PascalCase`
- **Constants**: `SCREAMING_SNAKE_CASE`
- **Private fields**: prefix with `_` if unused

## Struct Design

- Use `pub` fields directly for simple data containers (DTOs)
- Use getter methods for computed/derived values
- Implement `Debug`, `Clone`, `Serialize`, `Deserialize` where appropriate

## Imports

- Group imports in this order:
  1. Standard library (`std`, `core`)
  2. External crates (`serde`, `crossterm`)
  3. Local modules (`crate::`)

```rust
use std::sync::Arc;

use serde::{Deserialize, Serialize};
use crossterm::event::Event;

use crate::model::state::GameState;
use crate::error::Result;
```

## Tests

- Place tests in `#[cfg(test)]` modules within the same file
- Use descriptive test names: `test_<function>_<scenario>`
- Test both success and failure paths
- Use helper functions to reduce setup duplication

## Thread Safety

- When spawning threads, clone all data needed *before* the `move` closure
- Never borrow from `state` inside a spawned thread
- Use `Arc` for shared ownership

```rust
// Good - extract data before spawning
let room_npcs = room.npcs.clone();
let world = Arc::clone(&state.world);
thread::spawn(move || {
    // use room_npcs and world here
});

// Bad - borrows escape the function
thread::spawn(move || {
    let npc = state.npcs.get(id); // ERROR: state borrowed
});
```

## Documentation

- Document public APIs with `#[doc = "..."]`
- Document error variants in `EngineError`
- Keep README.md updated with build/run instructions
