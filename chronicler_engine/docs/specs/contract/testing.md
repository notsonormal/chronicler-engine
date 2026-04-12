# Specification: Testing Strategy and Architecture

## Objective
Establish a formal policy and architectural design pattern for ensuring the Chronicler Engine remains heavily tested locally without incurring financial costs or massive latency from interacting with external LLM APIs (like OpenRouter).

## Policy Rules
1. **Isolated Unit Tests**: All modules (`parser`, `map`, `state`) must continue to maintain fully isolated, embedded unit tests `#[test]` that evaluate standard library behaviors with zero networking overhead.
2. **Integration Capabilities**: As the engine develops, an overarching `tests/` directory will be required. These integration tests will evaluate the state graph moving from end-to-end.
3. **LLM Abstraction (The Trait Pattern)**: No component outside of the executable `main.rs` loop should ever be hardcoded to contact OpenRouter. 

## The `LlmBackend` Interface
To satisfy the LLM Abstraction policy, `llm.rs` must implement an interface:
```rust
pub trait LlmBackend {
    fn generate_dialogue(&self, world: &WorldCard, room: &Room, npc: &NpcCard, user_message: &Option<String>) -> String;
    fn narrate_action(&self, world: &WorldCard, room: &Room, nearby_npcs: &[&NpcCard], player: &PlayerCard, player_input: &str) -> String;
}
```

The engine will provide two implementations of this trait:
- `OpenRouterBackend`: Used by the live executable. Contacts the HTTP API using `reqwest` and parses the JSON response.
- `MockBackend`: Used automatically inside `#[cfg(test)]` scenarios. Will immediately return a static string such as `"[Mock Generated Response]"`.

The main REPL loop must accept an instantiated implementation of `LlmBackend`, allowing us to pivot the brain of the engine at compile-time or runtime.
