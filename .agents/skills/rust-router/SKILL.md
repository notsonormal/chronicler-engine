---
name: rust-router
description: "Router for all Rust questions — error codes drive Layer 1 (m01-m07) dispatch, design questions Layer 2 (m09-m15), domain questions Layer 3."
globs: ["**/Cargo.toml", "**/*.rs"]
---

---

# Rust Question Router

> **Version:** 2.0.0 | **Last Updated:** 2025-01-22
>
> **v2.0:** Context optimized - detailed examples moved to sub-files

## Meta-Cognition Framework

### Core Principle

**Don't answer directly. Trace through the cognitive layers first.**

```
Layer 3: Domain Constraints (WHY)
├── Business rules, regulatory requirements
├── domain-fintech, domain-web, domain-cli, etc.
└── "Why is it designed this way?"

Layer 2: Design Choices (WHAT)
├── Architecture patterns, DDD concepts
├── m09-m15 skills
└── "What pattern should I use?"

Layer 1: Language Mechanics (HOW)
├── Ownership, borrowing, lifetimes, traits
├── m01-m07 skills
└── "How do I implement this in Rust?"
```

### Routing by Entry Point

| User Signal | Entry Layer | Direction | First Skill |
|-------------|-------------|-----------|-------------|
| E0xxx error | Layer 1 | Trace UP ↑ | m01-m07 |
| Compile error | Layer 1 | Trace UP ↑ | Error table below |
| "How to design..." | Layer 2 | Check L3, then DOWN ↓ | m09-domain |
| "Building [domain] app" | Layer 3 | Trace DOWN ↓ | domain-* |
| "Best practice..." | Layer 2 | Both directions | m09-m15 |
| Performance issue | Layer 1 → 2 | UP then DOWN | m10-performance |

### CRITICAL: Dual-Skill Loading

**When domain keywords are present, you MUST load BOTH skills:**

| Domain Keywords | L1 Skill | L3 Skill |
|-----------------|----------|----------|
| Web API, HTTP, axum, handler | m07-concurrency | **domain-web** |
| 交易, 支付, trading, payment | m01-ownership | **domain-fintech** |
| CLI, terminal, clap | m07-concurrency | **domain-cli** |
| kubernetes, grpc, microservice | m07-concurrency | **domain-cloud-native** |
| embedded, no_std, MCU | m02-resource | **domain-embedded** |

---

## Layer 1 Skills (Language Mechanics)

| Pattern | Route To |
|---------|----------|
| move, borrow, lifetime, E0382, E0597 | m01-ownership |
| Box, Rc, Arc, RefCell, Cell | m02-resource |
| mut, interior mutability, E0499, E0502, E0596 | m03-mutability |
| generic, trait, inline, monomorphization | m04-zero-cost |
| type state, phantom, newtype | m05-type-driven |
| Result, Error, panic, ?, anyhow, thiserror | m06-error-handling |
| Send, Sync, thread, async, channel | m07-concurrency |
| unsafe, FFI, extern, raw pointer, transmute | **unsafe-checker** |

## Layer 2 Skills (Design Choices)

| Pattern | Route To |
|---------|----------|
| domain model, business logic | m09-domain |
| performance, optimization, benchmark | m10-performance |
| integration, interop, bindings | m11-ecosystem |
| resource lifecycle, RAII, Drop | m12-lifecycle |
| domain error, recovery strategy | m13-domain-error |
| mental model, how to think | m14-mental-model |
| anti-pattern, common mistake, pitfall | m15-anti-pattern |

## Layer 3 Skills (Domain Constraints)

| Domain Keywords | Route To |
|-----------------|----------|
| fintech, trading, decimal, currency | domain-fintech |
| ml, tensor, model, inference | domain-ml |
| kubernetes, docker, grpc, microservice | domain-cloud-native |
| embedded, sensor, mqtt, iot | domain-iot |
| web server, HTTP, REST, axum, actix | domain-web |
| CLI, command line, clap, terminal | domain-cli |
| no_std, microcontroller, firmware | domain-embedded |

---

## Error Code Routing

| Error Code | Route To | Common Cause |
|------------|----------|--------------|
| E0382 | m01-ownership | Use of moved value |
| E0597 | m01-ownership | Lifetime too short |
| E0506 | m01-ownership | Cannot assign to borrowed |
| E0507 | m01-ownership | Cannot move out of borrowed |
| E0515 | m01-ownership | Return local reference |
| E0716 | m01-ownership | Temporary value dropped |
| E0106 | m01-ownership | Missing lifetime specifier |
| E0596 | m03-mutability | Cannot borrow as mutable |
| E0499 | m03-mutability | Multiple mutable borrows |
| E0502 | m03-mutability | Borrow conflict |
| E0277 | m04/m07 | Trait bound not satisfied |
| E0308 | m04-zero-cost | Type mismatch |
| E0599 | m04-zero-cost | No method found |
| E0038 | m04-zero-cost | Trait not object-safe |
| E0433 | m11-ecosystem | Cannot find crate/module |

---

## Functional Routing Table

| Pattern | Route To | Action |
|---------|----------|--------|
| latest version, what's new | **rust-learner** | Use agents |
| API, docs, documentation | **docs-researcher** | Use agent |
| code style, naming, clippy | **coding-guidelines** | Read skill |
| unsafe code, FFI | **unsafe-checker** | Read skill |
| code review | **os-checker** | See `integrations/os-checker.md` |

---

## Priority Order

1. **Identify cognitive layer** (L1/L2/L3)
2. **Load entry skill** (m0x/m1x/domain)
3. **Trace through layers** (UP or DOWN)
4. **Cross-reference skills** as indicated in "Trace" sections
5. **Answer with reasoning chain**

### Keyword Conflict Resolution

| Keyword | Resolution |
|---------|------------|
| `unsafe` | **unsafe-checker** (more specific than m11) |
| `error` | **m06** for general, **m13** for domain-specific |
| `RAII` | **m12** for design, **m01** for implementation |
| `crate` | **rust-learner** for version, **m11** for integration |
| `tokio` | **tokio-*** for API, **m07** for concepts |

**Priority Hierarchy:**

```
1. Error codes (E0xxx) → Direct lookup, highest priority
2. Negotiation triggers (compare, vs, best practice) → Enable negotiation
3. Domain keywords + error → Load BOTH domain + error skills
4. Specific crate keywords → Route to crate-specific skill if exists
5. General concept keywords → Route to meta-question skill
```

---

## Sub-Files Reference

| File | Content |
|------|---------|
| `patterns/negotiation.md` | Negotiation protocol details |
| `examples/workflow.md` | Workflow examples |
| `integrations/os-checker.md` | OS-Checker integration |
