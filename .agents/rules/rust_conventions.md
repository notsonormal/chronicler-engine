# Rust Conventions for Chronicler Engine

## Naming Conventions

- **Functions & variables**: `snake_case`
- **Types & traits**: `PascalCase`
- **Constants**: `SCREAMING_SNAKE_CASE`
- **Private fields**: prefix with `_` if unused

## Struct Design

- Use `pub` fields directly for simple data containers (DTOs)
- Use getter methods for computed/derived values
- Implement `Debug`, `Clone`, `Serialize`, `Deserialize` where appropriate

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
