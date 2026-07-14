# Specification: Core Architecture (Modular)

## Tier Map

Eight tiers organize the codebase:

- `crate::domain::model` — Pure data structures; single source of truth for game state.
- `crate::domain::engine` — Pure simulation logic.
- `crate::application` — Orchestration layer; owns port traits under `application/ports/`. Sub-modules from the T2 land package:
  - `crate::application::persistence_gate` — application persistence boundary; owns `Arc<Storage>` + `Arc<PresetStore>`.
  - `crate::application::generation_gate` — per-game registry (write-side truth, `Arc<RwLock<HashMap<u64, GenerationSlot>>>`) + atomic projection (`Arc<AtomicBool>`, read-only cache of "any slot Generating"); `is_shutting_down` lives on `DefaultApplicationService` reading `AppState.shutdown_token`.
  - `crate::application::game_catalogue` — game-lifecycle orchestration; borrows `Arc<PersistenceGate>`.
  - `crate::application::world_catalogue` — worlds/presets persistence; takes raw `Arc<Storage>` (deliberate asymmetry vs GameCatalogue — independent seams).
- `crate::adapters::driven` — Outbound adapters (storage, LLM providers, text check).
  - `crate::adapters::driven::storage::preset_store` — adapter around `Storage` for prompt preset CRUD (storage-backed).
- `crate::adapters::driving` — Inbound adapters (HTTP server, CLI).
- `crate::bootstrap` — Composition root.
- `crate::settings` — Settings data model; DB-backed, loaded once at startup.
- `crate::test_support` — Shared fixtures, `TestDataBuilder`, and the `TestAppBuilder` API.

## Hexagonal Architecture (Ports & Adapters)

```mermaid
flowchart TD
    subgraph Core["Core (domain + application)"]
        DOM["domain/<br/>entities + pure rules"]
        APP["application/<br/>use cases + ports"]
    end
    PORT_L["Port trait<br/>(driving-side)"]
    PORT_R["Port trait<br/>(driven-side)"]
    DRIVING["Driving adapter<br/>HTTP, CLI"]
    DRIVEN["Driven adapter<br/>SQLite, LLM, Harper"]
    BOOT["bootstrap/<br/>composition root"]

    DRIVING -.impls.-> PORT_L
    DOM --> PORT_L
    APP --> PORT_R
    DRIVEN -.impls.-> PORT_R
    BOOT --> PORT_L
    BOOT --> DRIVING
    BOOT --> DRIVEN
    BOOT --> PORT_R
```



## Dependency Invariant

- `domain/` + `application/` depend on port traits only.
- Adapters implement port traits.
- Only `bootstrap/` imports both port traits and adapter impls.
- Driven-side port traits are owned by `application/ports/`.

## Port Inventory

| Port | Prod impls | Test impls |
|------|------------|------------|
| `LlmProvider` | 4 (`openrouter`, `deepseek`, `ollama`, `mock`) | 0 |
| `LlmMessageRepository` | 1 (`storage/backend/llm_messages.rs`) | 2 |
| `TextChecker` | 1 (`harper_text_checker.rs`) | 1 |

Trait paths under `application/ports/`.

## Storage Direct Access

Three `src/application/` files import `Storage` directly under `// arch-lint: storage-direct`:

- `game_service.rs` — intentional persistence boundary.
- `agents/registry.rs`, `agents/quantifier/agent.rs` — deferred to T2 reliability plan.

## Settings Flow

Settings are loaded **once** at startup and propagated through the construction chain:

```mermaid
flowchart TD
    A["bootstrap/run.rs"] --> B["load_settings() — ONCE"]
    B --> C["Arc<RwLock<AppSettings>>"]
    C --> D["AppState.settings"]
    D --> E["GameService::with_storage(storage, preset_storage, settings)"]
    E --> F["bootstrap::llm_factory::get_llm_recorder_for(connection, storage)"]
    E --> G["AgentRegistry::from_configs_with_storage(configs, storage, &settings)"]
    G --> H["QuantifierAgent::from_config_with_storage(config, storage, &settings)"]
```

## Default Settings

`impl Default for AppSettings` in `src/domain/model/settings.rs` is the runtime source of truth. The JSON seed `data/settings.json` is not consulted at startup.

| Provider | Default `max_context_tokens` |
|----------|------------------------------|
| `LlmBackendType::Ollama` | 8192 |
| `LlmBackendType::OpenRouter` / `DeepSeek` | 32768 |
| `LlmBackendType::Mock` | 4096 |

Default connections (3): `openrouter-gpt-4o-mini`, `openrouter-euryale`, `ollama-gemma-4-26B`. Default `narration_connection_id` and `quantifier_connection_id`: `"openrouter-gpt-4o-mini"`.

**Reload rules**: no business logic layer reloads settings from disk after bootstrap. Connection changes require a server restart to take effect.

## Document References

- [ADR-027: Hexagonal Architecture Migration](../adr/adr-027-hexagonal-architecture-migration.md) — phantom port heuristic + rejected ports + ports/traits collapse
- [ADR-030: is_generating Dual-Source Invariant](../adr/adr-030-is-generating-invariant.md) — `AtomicBool` as cached projection of persisted `is_generating` status
