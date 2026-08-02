# Ticket 06: Unify `PromptAssembler` token-budget resolution to per-call settings read

## Summary

`ActionPipeline::with_storage` resolves `max_context_tokens` and `max_tokens` once at construction and bakes them into `PromptAssembler`, while `with_backends` / `with_mock_quantifier` pass `settings` so the assembler reads on every `assemble()` call. The test `test_budget_read_from_settings_per_call` locks in the per-call behavior for `max_context_tokens`. Unify all three constructors to the per-call pattern so runtime settings changes are picked up. Two source files plus one test file; no behavior change for unchanged settings; runtime-change becomes the supported behavior in production.

## Key Changes

- `chronicler_engine/src/application/pipeline/pipeline.rs::ActionPipeline::with_storage` — drop the 6-line construction-time resolve. Build the assembler as `PromptAssembler::new(MAX_CONTEXT_TOKENS).with_settings(settings.clone())` (same shape as `with_backends` and `with_mock_quantifier`). The `MAX_CONTEXT_TOKENS` const import is already in scope.
- `chronicler_engine/src/application/prompting/assembler.rs::PromptAssembler` — replace `resolve_max_context_tokens` with a combined `resolve_budget(&self) -> (u32, Option<u32>)` that returns `(max_context_tokens, max_tokens)` from a single read lock. `assemble()` calls it once and passes both to `render_and_fit`.
- `chronicler_engine/src/application/prompting/assembler_tests.rs::test_budget_read_from_settings_per_call` — extend the existing test to also flip `max_tokens` between two `assemble()` calls and assert that the returned `AssembledPrompt::max_tokens` changes. This locks in the per-call behavior for `max_tokens`, which the current test does not cover (its settings leave `max_tokens: None`).

`max_tokens` is moved to per-call alongside `max_context_tokens`. Rationale: `with_storage` is the only site that reads `conn.max_tokens`; moving only `max_context_tokens` would silently drop the user's response cap. Per the decision document: "if the active connection's `max_tokens` can change at runtime, read it inside `assemble()`" — `LlmProviderConfig::max_tokens: Option<u32>` is configurable, settings is `Arc<RwLock>`, pipeline lives for the app process lifetime.

Skip `M2` rename (`resolve_max_context_tokens` → `effective_max_context_tokens`): cosmetic, out of scope per the decision document.

## Implementation

### Phase 1: Resolve Ticket 06

- [ ] #### Task 1.1: Unify `with_storage` constructor to per-call settings read (1 SP)
  - File: `chronicler_engine/src/application/pipeline/pipeline.rs`, `impl ActionPipeline`.
  - Replace the `with_storage` body. Drop the `let (max_context_tokens, max_tokens) = { … };` block and the `let mut assembler = PromptAssembler::new(max_context_tokens); if let Some(max) = max_tokens { assembler = assembler.with_max_tokens(max); }` block. Build the assembler inline as `PromptAssembler::new(MAX_CONTEXT_TOKENS).with_settings(settings.clone())` (matches the line in `with_backends`).
  - The `MAX_CONTEXT_TOKENS` import already exists at the top of the file (used by `with_backends` and `with_mock_quantifier`).
  - The `tracing::info!("ActionPipeline: backend={}, model={}", …)` line stays.
  - **Validation:** `grep -n 'max_context_tokens\|max_tokens' chronicler_engine/src/application/pipeline/pipeline.rs | grep -v 'use '` returns only unrelated sites.

- [ ] #### Task 1.2: Extend `PromptAssembler` to resolve `max_tokens` per call (1 SP)
  - File: `chronicler_engine/src/application/prompting/assembler.rs`.
  - Replace `fn resolve_max_context_tokens(&self) -> u32` with a combined helper that returns `(u32, Option<u32>)`:
    ```rust
    fn resolve_budget(&self) -> (u32, Option<u32>) {
        let Some(settings) = &self.settings else {
            return (self.max_context_tokens, self.max_tokens);
        };
        let guard = settings.read().unwrap_or_else(|e| e.into_inner());
        let conn = guard.narration_connection();
        (conn.resolve_max_context_tokens(), conn.max_tokens)
    }
    ```
  - In `assemble()`, replace the existing `self.resolve_max_context_tokens()` and `self.max_tokens` references with a single call: `let (max_context_tokens, max_tokens) = self.resolve_budget(); renderer.render_and_fit(max_context_tokens, max_tokens)?;`.
  - **Validation:** `grep -n 'resolve_budget\|render_and_fit' chronicler_engine/src/application/prompting/assembler.rs` shows the new helper + the call site in `assemble()` only.

- [ ] #### Task 1.3: Extend `test_budget_read_from_settings_per_call` to cover `max_tokens` per call (1 SP)
  - File: `chronicler_engine/src/application/prompting/assembler_tests.rs`, function `test_budget_read_from_settings_per_call` (line 357).
  - Append a new section after the existing `large_count > 50` assertion. The new section reuses the same `assembler`, `context`, and `preset` (do not rebuild them). Shape:
    ```rust
    // Per-call max_tokens: flip the response cap and observe the assembled
    // prompt's max_tokens change. Use a large context window so max_tokens is
    // the binding constraint (not the available-context floor).
    {
        let mut guard = settings.write().unwrap();
        guard.connections[0].max_context_tokens = Some(32768);
        guard.connections[0].max_tokens = Some(50);
    }
    let small_max = assembler
        .assemble(&context, &preset, &world.global_rules, Some("Short"))
        .expect("small max_tokens assemble should succeed");
    let small_budget = small_max.max_tokens;

    {
        let mut guard = settings.write().unwrap();
        guard.connections[0].max_tokens = Some(500);
    }
    let large_max = assembler
        .assemble(&context, &preset, &world.global_rules, Some("Short"))
        .expect("large max_tokens assemble should succeed");
    let large_budget = large_max.max_tokens;

    assert!(
        large_budget > small_budget,
        "larger max_tokens should yield larger budget: {large_budget} > {small_budget}"
    );
    assert!(
        small_budget <= 50,
        "small budget should be bounded by requested max_tokens=50: {small_budget}"
    );
    ```
  - `world.global_rules`, `preset`, and `context` are already in scope from the existing test. The `assembler` is the same instance (settings is `Arc<RwLock<…>>` shared via `with_settings`).
  - **Validation:** `grep -n 'test_budget_read_from_settings_per_call' chronicler_engine/src/application/prompting/assembler_tests.rs` shows the extended test; `cargo nextest run --test assembler test_budget_read_from_settings_per_call` is green.

- [ ] #### Task 1.4: Verify (1 SP)
  - `cd chronicler_engine && cargo check --all-targets --all-features` — green.
  - `cd chronicler_engine && cargo nextest run --test assembler` — green; the extended `test_budget_read_from_settings_per_call` passes.
  - `cd chronicler_engine && cargo nextest run --test pipeline` and `cargo nextest run --test retry` — green (production-path callers of `with_storage` exercise the unified path).
  - `cd chronicler_engine && python build.py` — full CI chain green. Record log path under the ticket's `## Answer`.

- [ ] #### Task 1.5: Close the ticket (1 SP)
  - Append `## Answer` to `.scratch/pipeline-review-hygiene/issues/06-unify-assembler-budget-resolution.md` summarizing: (a) `with_storage` now mirrors `with_backends` / `with_mock_quantifier`, (b) `max_tokens` is also per-call (rationale: user-configurable cap; symmetry with `max_context_tokens`; per the decision document's "if it can change at runtime, read inside `assemble()`" criterion), (c) `test_budget_read_from_settings_per_call` extended to lock in the per-call `max_tokens` behavior, (d) `M2` rename skipped per map's `## Out of scope`, (e) verification log path.
  - Set `Status: resolved`.
  - Append a one-line context pointer to the map's `## Decisions so far`:
    `- [Unify PromptAssembler token-budget resolution](issues/06-unify-assembler-budget-resolution.md) — with_storage now mirrors with_backends/with_mock_quantifier (per-call settings read); max_tokens also resolved per call for symmetry (was: silently baked at construction). Extended test_budget_read_from_settings_per_call to lock in max_tokens per-call. Skip M2 rename. Verification: cargo check + build.py green.`

## Test Plan

- `cargo check --all-targets --all-features` must be green.
- `cargo nextest run --test assembler` must be green; the extended `test_budget_read_from_settings_per_call` is the regression net for both per-call fields.
- `cargo nextest run --test pipeline` and `cargo nextest run --test retry` must be green (production-path callers of `with_storage`).
- `python chronicler_engine/build.py` must be green (full chain: fmt, clippy, arch-lint, invariant contract, syn walkers, `cargo test`, `cargo-llvm-cov`).
- One new test section in the existing test function. The fix is a refactor of the production path; the per-call test now covers both `max_context_tokens` (existing assertion) and `max_tokens` (new section). Pipeline tests still exercise the production constructor.

## Per Task/Sub Task Validation Steps

- **1.1**: `sed -n '40,75p' chronicler_engine/src/application/pipeline/pipeline.rs` shows `with_storage` body: `let assembler = PromptAssembler::new(MAX_CONTEXT_TOKENS).with_settings(settings.clone());` (or equivalent single line) immediately followed by `tracing::info!(...)` and the `Self { … }` literal. No `resolve_max_context_tokens` or `max_tokens` references in this constructor.
- **1.2**: `grep -n 'resolve_budget\|render_and_fit' chronicler_engine/src/application/prompting/assembler.rs` shows one `fn resolve_budget` definition and one call site in `assemble()`. `render_and_fit` is called with `max_context_tokens, max_tokens` (both bound from `resolve_budget`).
- **1.3**: `grep -n 'max_tokens' chronicler_engine/src/application/prompting/assembler_tests.rs` shows the new test section (the two `connections[0].max_tokens = Some(...)` writes + the two `assert!` lines). `cargo nextest run --test assembler test_budget_read_from_settings_per_call` exits 0.
- **1.4**: All four commands exit 0. `build.py` log path captured for the ticket.
- **1.5**: `head -5 .scratch/pipeline-review-hygiene/issues/06-unify-assembler-budget-resolution.md` shows `Status: resolved`. `grep '06-unify-assembler-budget-resolution' .scratch/pipeline-review-hygiene/map.md` shows the new pointer line in `## Decisions so far`.

## Assumptions

- The unification must cover `max_tokens` too, not only `max_context_tokens`. `with_storage` is the only site that reads `conn.max_tokens`; moving only `max_context_tokens` to per-call would silently drop the user-configurable response cap in production. Per the decision document: "if the active connection's `max_tokens` can change at runtime, read it inside `assemble()`" — `LlmProviderConfig::max_tokens: Option<u32>` is user-configurable and `settings` is `Arc<RwLock<AppSettings>>`, so yes.
- The per-call behavior is correct production behavior, not a test-only artifact. The locked-in test demonstrates the desired runtime semantic: change settings between calls, observe different prompt sizes. `with_storage`'s construction-time read was the bug.
- `MAX_CONTEXT_TOKENS` (the `32768` const) is the right initial value for the unified constructor. It is the per-provider default for `OpenRouter`/`DeepSeek`; the per-call resolve overrides it whenever settings are present. Test constructors use the same value.
- The fallback path in `resolve_budget` (`self.max_context_tokens, self.max_tokens` when `settings` is `None`) is dead code for the three `ActionPipeline` constructors (settings is always `Arc<RwLock<…>>`), but it stays for callers that construct `PromptAssembler` directly via `PromptContext::build_narration_prompt`. No behavior change for the fallback path.
- The new test section's `max_context_tokens = Some(32768)` reset is required: the earlier `max_context_tokens = Some(16384)` write would otherwise leave the available context around 16k, which still bounds `prompt.max_tokens` for `Some(500)` but is wasteful. The reset keeps the test self-contained.
- The assertion `small_budget <= 50` is sound: with `max_context_tokens = 32768` and the same `context` as the existing test (100 history entries × 80 chars ≈ 2k tokens, system prompt non-trivial but small), `fit_messages_to_context` returns `min(requested=50, available) = 50`. The `large_budget` assertion (`> small_budget`) follows from `requested=500` being the binding constraint.
- Skip `M2` rename. Ponytail-lite prefers the smaller diff; the helper is being renamed in this same step anyway (`resolve_max_context_tokens` → `resolve_budget`).
- No doc updates. Per map's `## Out of scope` and the precedent of closed tickets #02 / #03 / #04: pure refactors skip doc churn. `PromptAssembler`'s public surface is unchanged; `with_settings` already exists; `resolve_budget` is private.
- No follow-up ticket. Ticket #07 (phase ownership) and #08 (double-persist) are independent of this fix and remain on the frontier.
- Story points: 3 SP for the whole ticket (single session, two source files plus one test file, mechanical, ~25 line diff).
