# ADR-011: Text Check Integration

**Date:** 2026-05-09

---

## Context

Player input was sent directly to the LLM without any pre-flight validation. Typos, grammar errors, and unclear commands reached the narrator unchecked, degrading narrative quality. The engine needed a lightweight, local, privacy-respecting way to catch common errors before LLM submission.

---

## Decision

**Integrate `harper-core` for pre-flight spell and grammar checking of player input.**

### Modes

| Mode | Behavior |
|------|----------|
| `Disabled` | No checking |
| `Spell` | Spell check only |
| `Grammar` | Grammar check only |
| `SpellGrammar` | Both |

### Architecture

- **`HarperBackend`** wraps `harper-core` with a merged dictionary: `FstDictionary::curated()` + `MutableDictionary` for user-ignored words
- **`TextCheckSettings`** in `AppSettings`: mode, `enable_auto_check`, `ignored_words`
- **Automatic pre-flight**: `POST /action/check` intercepts player input before LLM submission
- **Manual on-demand**: `POST /check-text` for checking any text
- **Fail-open**: If linting fails, original text is forwarded silently
- **Player choice**: Preview UI shows original vs corrected; player can always choose "Send Original"

### UI Flow

1. Player hits **Send**
2. Form POSTs to `/action/check`
3. If no issues → silently forwarded to `/action`
4. If issues → `TextCheckPreviewTemplate` replaces action area via HTMX
5. Player chooses "Send Corrected" or "Send Original"

---

## Consequences

### Positive
- **Local-only**: No network calls; pure Rust, no API keys
- **Privacy**: Player text never leaves the machine for checking
- **Fail-open**: Broken linter does not block gameplay
- **Configurable**: Per-mode settings + personal ignore list

### Negative
- **Dictionary size**: `FstDictionary::curated()` is ~8 MB stripped; ~130 MB full
- **Fantasy names**: Game-specific terms require manual addition to ignore list
- **False positives**: Aggressive grammar rules may flag stylistic choices

### Trade-offs
- Chose `harper-core` over `nlprule` or LLM-based checking for speed, privacy, and no external dependencies
- Chose pre-flight interception over post-hoc correction to give player agency
- Chose fail-open over fail-closed to avoid blocking gameplay on linter bugs

---

## Related ADRs

- [ADR-001: HTMX Web Dashboard](./adr-001-htmx-web-dashboard.md) — Preview UI uses HTMX fragment swap

---

## History

- **2026-05-09**: Initial implementation — automatic pre-flight + manual check + preview UI
