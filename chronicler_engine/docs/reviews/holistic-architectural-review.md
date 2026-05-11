# Holistic Architectural Review: Chronicler Engine

**Date:** 2026-05-09  
**Scope:** Full engine architecture (model → engine → narrative → server)  
**Method:** Four-phase review — Domain Alignment, Structural Forces, Evolution Stress Test, Health Metrics  
**Reviewer:** Kimi Code CLI  
**Status:** Complete

---

## Executive Summary

The Chronicler Engine is a **healthy, well-tested codebase** with a clear layered architecture and strong guardrails. It is **elastic for local features** (new actions, triggers, UI fragments) but **rigid for systemic changes** (multiplayer, non-LLM backends). The recent defensive-architecture refactor improved readability without significantly improving extensibility.

**Verdict:** The architecture is **fit for purpose** as a single-player LLM-driven text adventure engine. It would require major restructuring to support multiplayer or deterministic rules-based narration.

### At a Glance

| Dimension | Score | Notes |
|-----------|-------|-------|
| Correctness | ✅ Strong | 631+ tests pass, zero unwrap in production |
| Readability | ✅ Good | GameState decomposition helped; naming debt remains |
| Architecture | ⚠️ Mixed | Clean layers, but porous boundaries and tight engine↔narrative coupling |
| Performance | ✅ Good | Excellent lock-to-unblock ratio, zero-cost diagnostics |
| Extensibility | ⚠️ Limited | Local features easy; systemic changes hard |
| Testability | ✅ Excellent | 2.04 test-to-code ratio, mock backends, property tests |
| Observability | ⚠️ Mixed | Debug endpoint exists, but `map_llm_error` loses debug info |

---

## 1. Domain Alignment (Phase 1)

### 1.1 The Domain Model

The engine sits at the intersection of three domains:
1. **Interactive Fiction** — rooms, exits, NPCs, inventory, actions
2. **LLM Narrative Generation** — prompts, context windows, token budgets, backends
3. **Web UI** — HTMX fragments, SSE polling, generation status

The code generally respects these boundaries. The **friction is in vocabulary**, not structure.

### 1.2 Naming Debt (The Biggest Finding)

Three types are **misnamed relative to their actual meaning**:

| Type | Current Name | Actual Meaning | Recommended Fix |
|------|-------------|----------------|-----------------|
| `model::trigger::CharacterState` | "CharacterState" | Map of ALL NPC encounter histories | `NpcRelationsState` |
| `model::trigger::TriggerAction` | "TriggerAction" | Narration effect when trigger fires | `TriggerEffect` |
| `model::state::GenerationState` | "GenerationState" | UI input buffer during generation | `GenerationInputState` |

**Impact:** Every engineer who touches trigger eval or action processing pays a mental tax to translate names to meanings.

### 1.3 Overloaded Terms

| Term | Meanings | Risk |
|------|----------|------|
| **Character** | CharacterSheet, PlayerCard, NpcCard, CharacterState, NpcEncounterState | High |
| **Trigger** | Struct, evaluation act, firing act, continuation LLM call | Medium |
| **Event** | LogType::Event, NpcEvent, generic occurrence | Medium |
| **Action** | Action enum, TriggerAction, free-text input | Medium |
| **Layer** | Prompt layer (0–7), architecture tier | Medium |
| **Scene** | SceneState (NPC presence), narrative scene | Medium |

### 1.4 Concepts Without Code Counterparts

Several domain concepts are documented but not modeled:
- **Game Master** — conceptual LLM role, no struct
- **Lorebook** — mentioned in docs, implemented as `WorldCard` fields
- **Memory / Long-Term Memory** — on roadmap, not implemented
- **Function Calling / Engine Intents** — on roadmap, not implemented
- **NPC Schedule** — on roadmap, not implemented

---

## 2. Structural Forces (Phase 2)

### 2.1 The Five Load-Bearing Decisions

| # | Decision | Enforced By | Escape Hatches |
|---|----------|-------------|----------------|
| 1 | Centralized mutable state (`Arc<Mutex<GameState>>`) | Type system | `with_state_lock` helper |
| 2 | Layer boundaries (`model→engine→narrative→server`) | `arch-lint.toml` | One upward violation |
| 3 | Sync processing in async HTTP (`spawn_blocking`) | Convention + guardrails | None needed |
| 4 | LLM as black-box trait (`LlmBackend`, `QuantifierBackendTrait`) | Trait system | Deep orchestration coupling |
| 5 | No `.unwrap()` in production | Clippy + arch-lint | `.expect()` for infallible ops |

### 2.2 Force Interactions

**Reinforcements (+):**
- `spawn_blocking` + centralized state → excellent lock-to-unblock ratio
- No unwrap + centralized state → poison recovery is explicit and safe
- Layer boundaries + no unwrap → errors propagate cleanly across tiers

**Tensions (−):**
- **Centralized state ↔ LLM traits:** `DefaultGameService` must clone 8+ fields before every LLM call. The LLM backend feels like an implementation detail of state mutation, not an independent service.
- **Layer boundaries ↔ LLM traits:** `engine/` imports 9 narrative types. `narrative/quantifier/core.rs` imports `engine::logic::get_current_room` (upward violation).
- **Centralized state ↔ layer boundaries:** `GeneratingGuard` lives in `model/state.rs` but is conceptually engine orchestration.

### 2.3 Lock Duration Analysis

**FreeAction timeline:**
1. Lock → clone data → drop (~μs)
2. LLM narrate_action (~1–10s, **no lock**)
3. Lock → apply quantifier → drop (~μs)
4. Trigger continuation LLM (~1–10s, **no lock**)
5. Lock → commit results → drop (~μs)

**Ratio:** ~20 seconds of LLM work with ~3× microseconds of lock time. **Excellent.**

### 2.4 Error Handling Gaps

**Critical:** `map_llm_error` discards `ParseError.raw_response`. When the LLM returns broken JSON, the user sees "unexpected response format" but the actual model output (the only debug evidence) is **gone**.

**Moderate:** 10 `.ok()` swallow sites in production. Diagnostic failures and lock poison are silently ignored. In tests, this means diagnostic failures don't fail the test suite.

**Minor:** `EngineError::NpcNotFound` exists but is never constructed (dead code).

### 2.5 Concurrency Health

- **Deadlock risk:** None (single mutex, no nesting)
- **Lock contention:** Very low (locks are brief)
- **Poison handling:** Inconsistent. `GeneratingGuard` recovers. `with_state_lock` silently skips.
- **Missing guard usage:** `GeneratingGuard` is only used in `bootstrap.rs`, not the main action paths. A panic in `execute_action` won't auto-reset `generation.status`.

---

## 3. Evolution Stress Test (Phase 3)

### 3.1 Scenario A: Combat System

**Verdict:** Feasible with medium refactor.

**First blocker:** No slot for ephemeral combat state in `GameState`.

**Path:**
1. Add `CombatState` to `GameState` (touches `GameState::new`, all fixtures)
2. Add `Attack`, `Defend`, `UseItem` to `Action` enum
3. Add `HpBelow`, `InCombat` to `TriggerCondition`
4. Decide: LLM-driven combat (easy, inconsistent) or rules-driven (harder, consistent)

**Blast radius:** 3–5 files. Core data model + engine action processing + trigger eval.

### 3.2 Scenario B: Multiplayer

**Verdict:** Major restructure required.

**First blocker:** `GameState` assumes single player at every layer.

**The problem is deeper than "add a players field":**
- `player` → `players` (touches 48 references across 15 files)
- `MovementState.current_room_id` → per-player location
- `SceneState.npcs_in_area` → per-player scene or per-room scene
- `NarrativeState.history` → interleaved history with player attribution
- `GenerationState` → per-player generation status
- `GameService::execute_action(player_name)` → per-player dispatch
- `Mutex<GameState>` → contention point with many concurrent players

**Blast radius:** 8+ files, core data model redesign.

### 3.3 Scenario C: Rules Engine Replacement

**Verdict:** Feasible with medium refactor.

**First blocker:** Quantifier is load-bearing in the action pipeline.

**Path:**
1. Make quantifier optional (bypass for deterministic movement/NPC presence)
2. Generalize `LlmBackend` trait to `NarratorBackend` (less LLM-shaped)
3. Redesign trigger continuation from LLM prompt to template string
4. Support synchronous FreeAction in server (no `spawn_blocking`)

**Blast radius:** 4–6 files. Narrative layer largely replaced; model layer unchanged.

### 3.4 Cross-Scenario Insight

**`GameState` extensibility is manual, not structural.** Adding any new concern requires touching:
- `GameState` struct definition
- `GameState::new`
- All `TestGameState` constructors (5 of them)
- Any direct `GameState` literal construction

The decomposition into sub-structs helped **readability** but not **extensibility**.

---

## 4. Health Metrics (Phase 4)

### 4.1 Mechanical Health

| Metric | Value | Grade |
|--------|-------|-------|
| Production code lines | 6,857 | Moderate |
| Unit test lines | 7,248 | Strong |
| Integration test lines | 6,739 | Strong |
| **Test-to-code ratio** | **2.04** | **A+** |
| Production `.unwrap()` | 0 | A+ |
| `TODO` / `FIXME` / `HACK` | 0 | A+ |
| `unsafe` | 0 | A+ |
| `panic!` (production) | 0 | A+ |
| Dead error variants | 1 (`NpcNotFound`) | B |

### 4.2 Module Coupling

| Module | model refs | engine refs | narrative refs | server refs |
|--------|-----------|-------------|----------------|-------------|
| model | 13 | 0 | 0 | 0 |
| engine | 57 | 23 | 33 | 0 |
| narrative | 66 | 1 | 83 | 0 |
| server | 46 | 11 | 8 | 35 |

**Finding:** `engine/` imports 33 narrative references. `narrative/` imports 1 engine reference (layer violation). The engine-narrative boundary is the most coupled in the system.

### 4.3 File Lengths

**Longest production files:**
1. `server/fragments.rs` — 582 lines (rendering + handlers + forms)
2. `server/settings_fragment.rs` — 570 lines (settings UI)
3. `engine/game_service.rs` — 349 lines (orchestration)
4. `model/state.rs` — 312 lines (state + sub-structs + guard)

All within guardrail limits.

### 4.4 Test Coverage Gaps

| Area | Coverage | Gap |
|------|----------|-----|
| State mutations | Strong | — |
| Trigger evaluation | Strong | — |
| Navigation | Strong | — |
| Prompt building | Very Strong | — |
| LLM backends | Very Strong | — |
| Quantifier | Strong | — |
| HTTP/server | Strong | — |
| **Error handling** | **Weak** | No systematic error-path tests |
| **Poison recovery** | **Weak** | No poison scenario tests |

---

## 5. Recommendations (Ranked)

### Critical — Do Before Next Feature

1. **Fix `map_llm_error` to preserve `ParseError.raw_response`**
   - Include first 500 chars in the error string shown to users.
   - One-line fix in `engine/game_service.rs:83-101`.

### Important — Do Within Two Weeks

2. **Delete `EngineError::NpcNotFound`**
   - Dead variant. Zero references. Safe removal.

3. **Rename the three misnamed types**
   - `CharacterState` → `NpcRelationsState`
   - `TriggerAction` → `TriggerEffect`
   - `GenerationState` → `GenerationInputState`
   - These are breaking changes for internal code only (no serde impact).

4. **Fix the `narrative → engine` upward dependency**
   - `narrative/quantifier/core.rs` calls `engine::logic::get_current_room`.
   - Pass `&Room` as parameter instead.

5. **Make diagnostic failures fail tests**
   - `assert_state_consistency(state).ok()` → panic in `#[cfg(test)]` builds.

### Medium — Do Within a Month

6. **Add `CombatState` placeholder to `GameState`**
   - Even if empty, establishes extension pattern.

7. **Add error-path tests**
   - Test `map_llm_error` for each `LlmFailure` variant.
   - Test poison recovery path.
   - Test `NpcNotFound` deletion doesn't break anything (negative test).

8. **Unify poison handling strategy**
   - Either all paths recover (like `GeneratingGuard`) or all paths skip (like `with_state_lock`).

### Low Priority — Nice to Have

9. **Move `NpcEvent` types out of `narrative::quantifier`**
   - They are engine state-transition events, not quantifier-specific.

10. **Move `create_dynamic_room` to `model::map::Room`**
    - Pure factory, no engine logic.

11. **Consider `GeneratingGuard` for action paths**
    - Replace manual `set_phase` / `reset_generating` with RAII guard.

---

## 6. Architecture Decision Record

### ADR: Accessor Traits and Session Types

**Context:** The defensive-architecture follow-up plan proposed accessor traits (`MovementAccess`, `NarrativeAccess`, etc.) and session types (phantom-type builders) to enforce compile-time access control.

**Decision:** **Skip both.**

**Rationale:**
- Accessor traits add ~50 lines of boilerplate + signature changes for marginal safety.
- They don't prevent misuse — any function can still take `&mut GameState` directly.
- Session types add ~100 lines of generic machinery for mutation-order enforcement.
- Runtime diagnostics (feature-gated, zero-cost in release) catch the same violations with better locality.
- The project convention is "keep it stupidly simple."

**Consequences:**
- Mutation order remains conventional (documented in `invariants.md`).
- Direct field access remains possible.
- Decomposition + diagnostics + property tests provide 90% of the safety at 20% of the complexity.

**Status:** Accepted. Documented here to prevent future rediscovery.

---

## 7. How to Use This Review

### For New Features (Combat, Items, Dialogue Trees)
1. Check Section 3 (Evolution Stress) for the closest scenario.
2. Check Section 2.3 for lock duration implications.
3. Add tests following existing patterns (mock backend + state assertions).

### For Bug Fixes
1. Check Section 2.4 for error handling gaps.
2. Run `cargo nextest run --features diagnostics`.
3. Consider if the bug is a missing invariant — add to `state_diagnostics.rs`.

### For Refactors
1. Check Section 1.2 for naming conventions.
2. Check Section 2.2 for force interactions — will your change create new tension?
3. Check `arch-lint.toml` — will your change violate layer boundaries?

### For Major Changes (Multiplayer, Rules Engine, New UI)
1. Read the full scenario in Section 3.
2. Evaluate whether the current architecture is the right foundation.
3. If not, write a new ADR before touching code.

---

## 8. Appendix: Source Documents

| Phase | File |
|-------|------|
| Phase 1: Domain Alignment | `docs/reviews/holistic-review-phase1-domain-alignment.md` |
| Phase 2: Structural Forces | `docs/reviews/holistic-review-phase2-structural-forces.md` |
| Phase 3: Evolution Stress Test | `docs/reviews/holistic-review-phase3-evolution-stress.md` |
| Phase 4: Health Metrics | `docs/reviews/holistic-review-phase4-health-metrics.md` |
| This document (synthesis) | `docs/reviews/holistic-architectural-review.md` |

---

## 9. Verification Log

All findings in this review were verified against:
- `cargo nextest run --features diagnostics` — all 631+ tests pass
- `python build.py` — fmt + clippy + guardrails + build + 644 tests pass
- `cargo nextest run --test architecture` — arch-lint passes
- `cargo clippy --all-targets --all-features -D warnings` — clean
