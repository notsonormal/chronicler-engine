# ADR-007: Settings System Architecture

**Date:** 2026-05-01
**Status:** Superseded (partially) — the flat settings design was replaced by the Connection Profiles system. Core principles (JSON persistence, runtime mutability, HTMX tabbed UI) remain unchanged.

> **Reference**: Current settings schema and configuration details are in the implementation at `src/model/settings.rs`.

---

## Context

LLM backend configuration previously relied on environment variables (`LLM_BACKEND`, `LLM_MODEL`, `QUANTIFIER_MODEL`, `OPENROUTER_API_KEY`). This had three hard problems:

1. **Server restart required** — Any configuration change required stopping and restarting the server
2. **No UI** — Non-technical users could not change settings without CLI access to the deployment environment
3. **Test coupling** — Integration tests were tightly coupled to environment variable state

---

## Decision

**Adopt JSON-based settings persisted to `data/settings.json`, with a tabbed Settings UI for runtime configuration.**

### Why JSON over YAML/TOML

Native `serde` support, human-readable, and no additional dependencies.

### Why tabbed UI over modal or slide-out

Persistent visibility. The Settings tab mirrors the SillyTavern convention; users of AI chat interfaces expect a permanent settings panel, not a hidden modal.

### Why Connection Profiles replaced flat settings (v2)

The flat design stored a single `LLM_MODEL` and `QUANTIFIER_MODEL`. Adding a third endpoint (e.g., a dedicated agent model) would have required adding more top-level fields indefinitely. Connection Profiles introduce a named `Vec<Connection>` with `narration_connection_id` and `quantifier_connection_id` pointing into it — extensible without schema changes.

### Environment variable policy

- **API keys**: Fall back to provider-specific env vars (`OPENROUTER_API_KEY`) if not in the connection — preserved for deployment convenience.
- **Backend type**: `LLM_BACKEND` env var is **no longer consulted** — settings file is sole authority. Integration tests should write a mock settings file and set `CHRONICLER_SETTINGS_PATH`.

### Concurrency

Settings exposed via `Arc<RwLock<AppSettings>>` in `AppState`. Write-lock held only during save.

---

## Consequences

### Positive
- Runtime reconfiguration without server restart
- Non-technical users can manage LLM backends through the browser UI
- Test isolation via `CHRONICLER_SETTINGS_PATH` pointing to a temp file

### Negative
- API keys stored in plain JSON (mitigated by masked UI display and `.gitignore` exclusion)
- Flat-to-Connection migration was a breaking schema change (acceptable — settings file is ephemeral)

### Trade-offs
- Chose JSON over YAML/TOML (native serde, no extra deps)
- Chose tabbed UI over modal (persistent visibility, SillyTavern convention)
- Chose env var fallback for API keys only (deployment convenience preserved)

---

## Related ADRs

- [ADR-001: HTMX Web Dashboard](./adr-001-htmx-web-dashboard.md) — UI foundation hosting the Settings tab
- [ADR-006: Quantifier-Driven Game Systems](./adr-006-quantifier-systems.md) — Quantifier model configuration via settings

---

## History

- **2026-05-01**: Flat settings system — env vars replaced by `settings.json` with `LLM_MODEL` / `QUANTIFIER_MODEL`
- **2026-05-02**: Connection Profiles v2 — flat model fields replaced by `Vec<Connection>` with named profiles and `narration_connection_id` / `quantifier_connection_id` selectors