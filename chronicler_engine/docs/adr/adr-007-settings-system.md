# ADR-007: Settings System Architecture

**Date:** 2026-05-01

---

## Context

LLM backend configuration previously relied on environment variables (`LLM_BACKEND`, `LLM_MODEL`, `QUANTIFIER_MODEL`, `OPENROUTER_API_KEY`), which presented significant operational challenges:

- **Server restart required**: Any configuration change demands stopping and restarting the server, making iterative testing with different backends cumbersome
- **No UI configuration**: Non-technical users cannot modify settings without direct access to deployment environment variables
- **Poor UX**: Players or team members without CLI knowledge cannot experiment with different LLM providers or models

The team needed a solution that enabled runtime configuration without server restarts, provided a user-friendly interface for settings management, and maintained backward compatibility with existing deployments.

---

## Decision

**Adopt JSON-based settings with tabbed Settings UI.**

The Chronicler Engine now loads configuration from `data/settings.json` into an `AppSettings` struct, exposes it via `AppState` (wrapped in `Arc<RwLock<AppSettings>>`), and provides a dedicated Settings tab for runtime configuration.

### Configuration Scope

| Setting | Type | Options | Default |
|---------|------|---------|---------|
| `llm_backend` | enum | `deepseek`, `openrouter` (`mock` is test-only, not shown in UI dropdown) | `openrouter` |
| `llm_model` | string | Any OpenRouter model ID | `openai/gpt-4o-mini` |
| `quantifier_model` | string | Any OpenRouter model ID | `openai/gpt-4o-mini` |
| `openrouter_api_key` | string | Masked in UI, stored plain | `None` (falls back to env var) |

### Data Flow

```
settings.json → AppSettings (loaded at startup)
                    ↓
              AppState.settings (Arc<RwLock<AppSettings>>)
                    ↓
     ┌───────────────┴───────────────┐
     ↓                               ↓
get_llm_backend()           get_llm_model()
(uses settings.backend)    (uses settings.llm_model)
```

The settings file serves as the single source of truth for backend selection. On first launch, the system auto-creates `settings.json` with default values if missing.

### Tabbed UI Design

Following Silly Tavern conventions, the dashboard provides a tabbed interface:

```
┌─────────────────────────────────────────┐
│ [Game Tab] [Settings Tab]               │
├─────────────────────────────────────────┤
│                                          │
│         (Tab content here)               │
│                                          │
└─────────────────────────────────────────┘
```

- **Game Tab**: Current layout (story log, visual sidebar, action area)
- **Settings Tab**: Settings form with backend dropdown, model inputs, masked API key field, and Save button

### Backward Compatibility

- **Model names**: Environment variables `LLM_MODEL` and `QUANTIFIER_MODEL` are checked first; if set, they override settings file values (allows CI/testing to override without modifying JSON)
- **API key**: If `openrouter_api_key` in settings is `None`, the system falls back to `OPENROUTER_API_KEY` env var
- **Backend selection**: The `LLM_BACKEND` environment variable is **no longer consulted** after this change; the settings file is the sole source of truth for backend type

---

## Consequences

### Positive
- **Runtime configuration**: Settings changes take effect without server restart, enabling rapid iteration
- **User-friendly UI**: Non-technical users can modify LLM settings through the browser
- **Test simplification**: Mock backend tests no longer require `LLM_BACKEND=mock` env var manipulation
- **Centralized configuration**: All LLM settings in one JSON file simplifies deployment management

### Negative
- **Persistence complexity**: Requires file I/O for settings load/save operations, introducing potential failure modes
- **Security consideration**: API key stored in plain JSON (mitigated by masked UI display and `.gitignore` exclusion)
- **Race conditions**: Concurrent settings writes need handling (mitigated by RwLock)

### Trade-offs
- Chose JSON over YAML/TOML for native `serde` support and human readability
- Chose tabbed UI over modal/slide-out for persistent visibility
- Env var fallback preserved for model names (CI convenience) but removed for backend type (settings file authority)

---

## Related ADRs

- [ADR-001: HTMX Web Dashboard](./adr-001-htmx-web-dashboard.md) - UI foundation that hosts the Settings tab
- [ADR-006: Quantifier-Driven Game Systems](./adr-006-quantifier-systems.md) - Quantifier model configuration via settings

---

## History

- **2026-05-01**: Initial decision based on settings-system.md plan

---

## Historical Note

This ADR formalizes the Settings System that was designed to replace environment variable-based LLM configuration. The tabbed UI approach mirrors best practices from Silly Tavern and similar AI chat interfaces, providing a familiar experience for users accustomed to configuring AI backends through web interfaces.