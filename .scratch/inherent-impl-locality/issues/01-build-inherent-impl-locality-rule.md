# 01 — Build inherent impl locality rule

Type: prototype
Status: ready-for-agent
Blocked by: 03, 04, 05, 06, 07, 08

> **Deferred per user direction (this session).** The rule was built, trial-run
> (27 violations, captured by ticket 02's resolution), then **removed from the
> repo entirely** — the file `tests/infrastructure/guardrails/inherent_impl.rs`
> no longer exists, no `pub mod inherent_impl` registration, no `#[test]
> guardrails_inherent_impl_locality` entry. `build.py` and
> `cargo test --test guardrails` are clean on `main`.
>
> **Re-add as a later step**: blocked by 03–08. Once those refactor tickets
> land (working against the 27-violation set + 3 discrepancies captured in
> ticket 02's resolution), this ticket is re-worked (re-create the file,
> re-register, re-run, confirm zero violations) immediately before ticket 09
> promotes it to a gating step. The implementation blueprint and trial-run
> findings are preserved below for that future pass.

## Question

Build `guardrails_inherent_impl_locality` in `tests/infrastructure/guardrails/inherent_impl.rs`, registered from `guardrails/mod.rs`, riding the existing `cargo test --test guardrails` harness.

Rule (strict mode, audit-only at this stage — do NOT yet wire into `build.py` as a gate):

```text
For every inherent impl `impl Foo` in production src/ (trait impls excluded,
test files excluded):

  Let snake = snake_case(Foo simple name).
  Let impl_path = relative path of impl file.
  Let def_path = relative path of file where Foo is defined.

  Violation if:
    impl_path != def_path, AND
    NOT (impl_path's parent dir ends with /snake)
```

Exclusions:
- Paths ending in `_tests.rs` skipped.
- `#[cfg(test)] mod` inner blocks skipped.
- Trait impls (`item.trait_.is_some()`) skipped.
- `main.rs` is vacuously clean (no inherent impls).

Implementation guidance:
- Use existing `check_src_files` runner from `guardrails/mod.rs` — feeds `(relative_path, content)` to each rule. May need a two-pass variant (collect all type defs first, then check impls against the index).
- Use `syn` dev-dependency (already present).
- Normalize `impl<'a> PipelineRun<'a>` to simple name `PipelineRun` by walking `item.self_ty` and stripping generics, lifetimes, qualified paths.
- Build the type-definition index (`struct`/`enum`/`union` simple name → defining file path) in pass 1, then scan impl blocks in pass 2.
- Emit one `Violation::error` per offending impl block, pointing at the impl's line, with message naming the type, the defining file, and the impl file.

Acceptance:
- Rule exists and compiles.
- Running `cargo test --test guardrails guardrails_inherent_impl_locality` fails (the rule fires) — this is expected because current code has violations. Ticket 02's resolution recorded the violation set and discrepancies against the expected table.
- No per-type exception list. No config file. Pure structural check.
- Unit tests in the rule file covering: (a) impl + def in same file (OK), (b) impl split across files in a folder named after the type (OK), (c) impl split across files in a folder NOT named after the type (violation), (d) impl in different folder from def entirely (violation), (e) trait impl ignored, (f) generic self type normalized.

Links:
- Map: `.scratch/inherent-impl-locality/map.md`
- Existing harness: `tests/infrastructure/guardrails/mod.rs`

## Trial run findings (preserved for the refactor tickets and 02's resolution)

> Captured during the trial implementation before removal. **Do not re-run** until
> this ticket is re-worked — the rule file is gone from `main`.

### Implementation blueprint (for re-creation in the later step)

- Two-pass: pass 1 builds a `HashMap<type_name, defining_path>` from `ItemStruct` / `ItemEnum` / `ItemUnion` over all non-excluded production files; pass 2 walks `ItemImpl` blocks and classifies each inherent impl against the index.
- `check_files(&[(path, content)]) -> Vec<Violation>` is the pure fs-free core; `check_inherent_impl_locality()` reads from `src/` and delegates to it.
- `check_src_files` runner not reused (it hands one file at a time; the rule needs the cross-file def index). The entry walks directly via `discover_rs_files("src")`.
- Inherent-impl self type normalised by walking `Type::Path | Type::Reference | Type::Paren | Type::Group | Type::Slice | Type::Array | Type::Ptr`, returning `None` for anything else (skipped, not flagged). Generic args / lifetimes stripped by reading the last path segment's `ident`.
- `to_snake_case` handles acronym boundaries: `DbCharacter → db_character`, `LLMMessage → llm_message`, `DefaultApplicationService → default_application_service`.
- Folder exemption: parent dir of impl path must end with `/{snake}` (or equal `{snake}` at repo root). `impl_path == def_path` always clean. Trait impls (`item.trait_.is_some()`) skipped. `#[cfg(test)] mod` subtrees skipped by overriding `visit_item_mod` in both collectors.
- `_tests.rs` / `_test.rs` / `main.rs` excluded from both passes (rule scope is production src/).
- One `Violation::error` per offending impl block, pointing at `self_ty.span().start().line`, naming the type, the defining file, and the impl file.
- Imports needed: `use quote::ToTokens;`, `use syn::spanned::Spanned;`.
- `mod.rs` wiring: add `pub mod inherent_impl;` + `pub use inherent_impl::*;`; make `discover_rs_files`, `relative_path`, `assert_violations` `pub(crate)`; add `#[test] fn guardrails_inherent_impl_locality { run_check(); }` at the end.

### `ponytail:` shortcuts to re-apply

- `is_cfg_test` uses substring match (`attr.to_token_stream().to_string().contains("test")`) on the `#[cfg(...)]` attribute — false-positive risk on `#[cfg(feature = "test")]`. Upgrade to ident-level scan if a real false positive surfaces.
- Type-def index uses `or_insert_with` (first def wins) on type-name collisions. Same-name types in different modules are out of scope.

### Trial-run violation set (27 violations, no longer reproducible until re-created)

```
adapters/driving/http/fragments/renderers/fragment_renderers.rs:17  impl AppState
adapters/driven/storage/backend/worlds.rs:16                       impl DbWorld
adapters/driven/storage/backend/worlds.rs:48                       impl Storage
adapters/driven/storage/backend/characters.rs:11                    impl Storage
adapters/driven/storage/backend/characters.rs:95                    impl DbCharacter
adapters/driven/storage/backend/snapshots.rs:9                      impl Storage
adapters/driven/storage/backend/swipes.rs:10                       impl Storage
adapters/driven/storage/backend/swipes.rs:178                      impl InMemoryData
adapters/driven/storage/backend/settings.rs:11                     impl Storage
adapters/driven/storage/backend/settings.rs:73                     impl DbSettings
adapters/driven/storage/backend/messages.rs:9                     impl Storage
adapters/driven/storage/backend/messages.rs:243                    impl InMemoryData
adapters/driven/storage/backend/llm_messages.rs:9                   impl Storage
adapters/driven/storage/backend/presets.rs:9                      impl Storage
adapters/driven/storage/backend/presets.rs:139                     impl DbPromptPreset
adapters/driven/storage/backend/personas.rs:11                     impl Storage
adapters/driven/storage/backend/personas.rs:94                     impl DbPersona
adapters/driven/storage/backend/games.rs:10                       impl Storage
adapters/driven/storage/backend/games.rs:135                       impl DbGame
application/action_pipeline/pipeline.rs:268                        impl PipelineRun
application/action_pipeline/retry.rs:12                            impl DefaultApplicationService
application/utils/retry.rs:79                                      impl DefaultApplicationService
application/agents/quantifier/parser.rs:8                         impl QuantifierParseResult
application/agents/quantifier/parser.rs:14                        impl QuantifierResult
application/narrative_prompt/assembler.rs:82                      impl PromptContext
application/narrative_prompt/assembler.rs:310                      impl PromptPreset
bootstrap/load.rs:28                                              impl Storage
```

### Three discrepancies against ticket 02's expected table

Ticket 02's resolution itemizes these as findings; refactor tickets absorb them here:

1. **`ActionPipeline` is NOT flagged** by the rule as specified. `phases.rs` lives in folder `action_pipeline/`, which matches the folder exemption (`snake_case(ActionPipeline) == "action_pipeline"`). 02's `Reason: folder holds other types too` is informal — the rule formula encodes "parent dir ends with `/snake`", not "folder holds only this type." 02 should flag this as a discrepancy (procedure step 5) — either the rule formula needs tightening (map changes) or 02's expectation is wrong.
2. **`QuantifierResult` IS flagged** (sibling of `QuantifierParseResult`). 02's narrative mentions both but the expected table lists only `QuantifierParseResult`. 02 should reconcile.
3. **`AppState`** (def `adapters/driving/http/app_state.rs`, impl in `fragments/renderers/fragment_renderers.rs`) and **`PromptContext`** (def `application/narrative_prompt/types.rs`, impl in `assembler.rs`) are flagged by the rule but absent from 02's expected table. 02 should list these as newly-surfaced violations (procedure step 4).

### Build.py gating status

`build.py` was never wired as a gate during the trial (per ticket 01 scope). `build.py` runs `cargo nextest run ... --features testing` with `check=False`, so even if the test entry had stayed, the build would have continued past a failing rule. Promotion to an actual gate is ticket 09, which now presupposes this ticket is re-worked first.
