# ADR-011: Text Check Integration

**Date:** 2026-05-09
**Status:** Accepted

> **Reference**: Full architecture, types, endpoints, settings schema, and UI integration details are in [`docs/system/text_check.md`](../system/text_check.md).

---

## Context

Player input was forwarded directly to the LLM without any pre-flight validation. Typos and grammar errors in player commands degraded narrative quality and occasionally confused the narrator into generating off-topic responses.

---

## Decision

**Integrate `harper-core` for local spell and grammar checking before LLM submission.**

### Why `harper-core` over alternatives

| Option | Reason rejected |
|--------|----------------|
| `nlprule` | Archived / unmaintained |
| LLM-based checking | Adds a full round-trip API call; privacy leak; latency |
| `languagetool` (JVM) | Requires JVM runtime; heavyweight for this use case |
| `harper-core` | Pure Rust, no network, ~8MB FST dictionary, fast (<10ms per check) |

### Why fail-open

Blocking gameplay on a linter bug is worse than forwarding an unchecked typo. If harper fails, the original text is forwarded silently.

### Why pre-flight interception, not post-hoc correction

Players should choose between their original text and the corrected version — never have the engine silently change what they typed. The preview UI (`Send Corrected` / `Send Original` / `Cancel`) preserves agency.

---

## Consequences

### Positive
- Entirely local — no network call, no API key, no privacy exposure
- Fail-open — linter failure never blocks gameplay
- Configurable per-mode (Disabled / Spell / Grammar / Both) + personal ignore list for fantasy names

### Negative
- `FstDictionary::curated()` is ~8MB stripped binary size
- Fantasy proper nouns and game-specific terms require manual addition to the ignore list
- Aggressive grammar rules can flag stylistic choices

### Trade-offs
- Chose `harper-core` over all alternatives for speed, privacy, and zero runtime dependencies
- Chose pre-flight interception over silent auto-correction to preserve player agency

---

## Related ADRs

- [ADR-001: HTMX Web Dashboard](./adr-001-htmx-web-dashboard.md) — Preview UI uses HTMX fragment swap

---

## History

- **2026-05-09**: Initial implementation — automatic pre-flight check + manual check button + preview UI
