# 03: Should allowlisted modules pair with signature-shape constraints?

Status: open
Type: grilling
Assignee: (unassigned)
Blocked by: (none)

## Question

Should each allowlisted module/category pair with a **signature-shape constraint** that the free fn must satisfy (in addition to living in the allowed module), to reduce the risk that a new method-shaped free fn inside an allowed module passes silently?

## Context

A pure module-allowlist (file-level or folder-level, per 02) has a known weakness: a new free fn added *inside* an allowed module passes silently, even if it is shape-identical to the smell pattern (one borrowed domain receiver, minor other args). Pairing each category with a signature constraint narrows this:

| Category | Module | Shape constraint |
|----------|--------|------------------|
| Mappers | `storage/mappers/*.rs` | return type ≠ input type (converter) |
| Engine algorithms | `domain/engine/{action_processing,logic,state_diagnostics,trigger_eval}.rs` | two+ domain-type inputs (multi-input algorithm) |
| Generic utility | `adapters/driving/http/locks.rs` | generic type param `T` |
| Spawn-blocking helpers | `application/action_pipeline/{retry,actions}.rs` | (after 01 — likely empty, verify) |
| Prompt builders | `application/narrative_prompt/*.rs` | 3+ inputs OR return is assembled object |
| Persistence boundary | `settings.rs`, `bootstrap/load.rs`, `test_support/context.rs` | second param is `&Storage` / `&Path` (boundary) |
| Composition root | `bootstrap/validate.rs`, `bootstrap/load.rs` | multi-entity input (no single owner) |
| Slot helpers | `application/generation_gate/slot.rs` | takes `&Arc<RwLock<...>>` (shared-state handle) |

### Tradeoff

- *Without constraints*: simplest scanner. Pure module-allowlist. Risk of in-allow-zone misses.
- *With constraints*: tighter, but reintroduces signature classification — the same family of approach that failed in the original script and forced 21 per-function suppressions. The constraints above are paired with module context, so they are narrower than the original "first param is `&DomainType`" rule. But they still cannot prove ownership semantics.

This is the central tension of the effort: narrow static rules risk recreating the signature-classifier failure; wide rules hide future smells. Advisory mode (see 04) relaxes the tension by accepting that static analysis cannot carry the weight alone.

## Recommendation

**Pair with constraints, but keep them as coarse filters, not proof.** Each category gets one signature constraint that the honest free fns in that category already satisfy. A free fn in an allowed module that fails its category constraint becomes a REVIEW flag (not a build failure — see 04). This recovers some of the precision the pure allowlist loses without re-creating the suppression-list problem: flags, not suppressions.
