# Map: Finish pipeline implementation after post-review findings

## Summary

You claimed **Ticket 05 — Fix check_wiredapp_scope citation** on the `pipeline-review-hygiene` map. The defect is the `// Guardrail:` block at `chronicler_engine/tests/infrastructure/guardrails/layers.rs:7-9`, which cites two non-existent documents (`"guardrail-inventory §2"`, `"axis R two-consumer discipline"`). The block is also pure meta-commentary: it paraphrases the `WIREDAPP_SCOPE_ALLOWLIST_PREFIXES` constant and the `_tests.rs` exemption that the function body shows literally, which AGENTS.md §3 bans. Per your direction, the *comment itself* is the problem — not the citation inside it. The minimum fix is to delete the block. The two real anchors (`guardrails.md` §2, `adr-027-hexagonal-architecture-migration.md`) remain grep-able via `grep -r 'WiredApp' chronicler_engine/docs/` and via the function name + named allowlist + `_tests.rs` exemption that stay. The in-file precedent for this fix is the one-liner on line 50 of the same file (`// Guardrail: \`messages.rs\` must not reference the \`message_swipes\` table.`) — it states the *rule*, not where the rule is documented, and so does not need to be expanded to a 3-line citation block.

One ticket resolved in this session, same scope as the prior plans: a single 3-line deletion in one source file, plus the ticket-closure edits in `.scratch/`. No edits to `src/application/agents/registry.rs`, `src/application/agents/quantifier/agent.rs`, or `docs/diataxis/reference/coding_standards/guardrails.md`. The ticket's "Optional" steps (Storage-provenance comments in the two agent files, `guardrails.md` §2 clarification) remain skipped for the same reason as the prior plan: AGENTS.md §4 (no self-referential migration pointers) and ADR-027 2026-08-01 (boundary discipline lives in the ADR, not in code comments) make them *anti-optional*; the `guardrails.md` §2 clarification would also be stale once ticket #12 lands its machine-checked HTTP-layer Storage walker.

## Key Changes

- **`chronicler_engine/tests/infrastructure/guardrails/layers.rs`** — delete the 3-line `// Guardrail:` block at lines 7-9. The file's first non-`//!` line stays as `const WIREDAPP_SCOPE_ALLOWLIST_PREFIXES: &[&str] =`; the `pub fn check_wiredapp_scope(...)` declaration shifts up to line 7. No code change, no behavior change, no test change. The function's self-documentation (name, the named allowlist constant, the explicit `_tests.rs` exemption in the body) carries the meaning the deleted comment was paraphrasing.
- **`.scratch/pipeline-review-hygiene/issues/05-fix-guardrail-citations.md`** — append `## Answer` block summarizing: (a) the two non-existent citations removed by the deletion, (b) the rule remains self-documenting (function name + `WIREDAPP_SCOPE_ALLOWLIST_PREFIXES` + the `_tests.rs` exemption in the function body), (c) the real anchors (`guardrails.md` §2, ADR-027) remain grep-able without an in-file pointer, (d) verification log path. Set `Status: resolved`. Append a one-line context pointer to the map's `## Decisions so far`.

No new files, no new tests, no edits outside the one source file + the ticket scratch file. Three files stay untouched that earlier plans would have touched.

## Implementation

### Phase 1: Resolve Ticket 05

- [ ] #### Task 1.1: Delete the 3-line `// Guardrail:` block at `layers.rs:7-9` (1 SP)
  - File: `chronicler_engine/tests/infrastructure/guardrails/layers.rs`, lines 7-9.
  - Old (3 lines):
    ```rust
    // Guardrail: `WiredApp` is consumed only by the composition root, HTTP setup, test support,
    // and integration tests (axis R two-consumer discipline; guardrail-inventory §2).
    // `*_tests.rs` is exempt — unit tests may wire collaborators freely.
    ```
  - New: nothing. The lines are removed. After the deletion the file reads:
    ```rust
    //! Layer-boundary guardrail tests: server vs. application vs. storage separation, handler return-type enforcement, and tests-vs-messages/swipes separation.

    use crate::Violation;

    const WIREDAPP_SCOPE_ALLOWLIST_PREFIXES: &[&str] =
        &["bootstrap/", "adapters/driving/http/", "test_support/"];

    pub fn check_wiredapp_scope(file_path: &str, content: &str) -> Vec<Violation> {
    ```
  - The `//!` line 1 (module doc anchor) stays — it is the project's canonical anchor and is required by AGENTS.md §2. The `use crate::Violation;` import and the `WIREDAPP_SCOPE_ALLOWLIST_PREFIXES` constant shift up unchanged. The `pub fn check_wiredapp_scope` declaration now sits at line 7 directly under the constant; the function body is untouched.

- [ ] #### Task 1.2: Verification (1 SP)
  - `cd chronicler_engine && cargo check --all-targets --all-features` — must be green. Pure deletion; the only load-bearing check is that the file still parses.
  - `cd chronicler_engine && cargo nextest run --test guardrails` — must be green. The 6 existing `test_check_wiredapp_scope_*` tests in the same file assert on the violation message text (`"WiredApp"`, `"Composition-root"`) and on the allowlist; neither changed. They are the regression net.
  - `cd chronicler_engine && python build.py` — must be green. Full CI chain (clippy, arch-lint, syn walkers, invariant contract, `cargo test`, `cargo-llvm-cov`) confirms no other walker is affected.

- [ ] #### Task 1.3: Close the ticket (1 SP)
  - Append an `## Answer` block to `.scratch/pipeline-review-hygiene/issues/05-fix-guardrail-citations.md` covering: (a) the two non-existent citations removed by the deletion, (b) the rule is self-documenting via the function name + `WIREDAPP_SCOPE_ALLOWLIST_PREFIXES` constant + the `_tests.rs` exemption in the function body, (c) the real anchors (`guardrails.md` §2, ADR-027) remain findable by `grep -r 'WiredApp' chronicler_engine/docs/`, (d) verification log path.
  - Set `Status: resolved` in the same ticket file.
  - Append a one-line context pointer to the map's `## Decisions so far`:
    `- [Fix check_wiredapp_scope citation](issues/05-fix-guardrail-citations.md) — deleted the 3-line "guardrail-inventory §2" / "axis R two-consumer discipline" comment at layers.rs:7-9. False citations gone; no replacement (rule is self-documenting; real anchors remain grep-able). Dropped the optional Storage-provenance comments per AGENTS.md §4 and ADR-027 2026-08-01. Verification: cargo check + build.py green.`

## Test Plan

- `cargo check --all-targets --all-features` — pure deletion, must compile.
- `cargo nextest run --test guardrails` — 6 existing `test_check_wiredapp_scope_*` tests cover behavior. No new test needed: this is a comment deletion; behavior is asserted by the existing tests; the citations were documentation, not executable.
- `python chronicler_engine/build.py` — full CI chain. The build log path (`chronicler_engine/logs/build_<timestamp>.log`) is the verification artifact cited in the ticket's `## Answer`.

## Per Task/Sub Task Validation Steps

- **1.1**: After edit, `grep -n 'guardrail-inventory\|axis R' chronicler_engine/tests/infrastructure/guardrails/layers.rs` returns zero matches. `sed -n '1,9p' chronicler_engine/tests/infrastructure/guardrails/layers.rs` shows: line 1 `//! Layer-boundary...`, line 2 blank, line 3 `use crate::Violation;`, line 4 blank, line 5 `const WIREDAPP_SCOPE_ALLOWLIST_PREFIXES: &[&str] =`, line 6 `    &["bootstrap/", "adapters/driving/http/", "test_support/"];`, line 7 blank, line 8 `pub fn check_wiredapp_scope(file_path: &str, content: &str) -> Vec<Violation> {`. The deleted block is gone; no replacement.
- **1.2**: All three commands exit 0. `build.py` writes a new log to `chronicler_engine/logs/build_<timestamp>.log`; the log path is cited in the ticket's `## Answer`.
- **1.3**: `head -10 .scratch/pipeline-review-hygiene/issues/05-fix-guardrail-citations.md` shows `Status: resolved`. `grep '05-fix-guardrail-citations' .scratch/pipeline-review-hygiene/map.md` shows the new pointer line in `## Decisions so far`.

## Assumptions

- The defect is *false citations*, not *missing citations*. The ticket's `## Question` names two non-existent documents; the minimum fix is to remove them.
- Per your direction, the *comment itself* is the problem — not the citation inside it. A "where this lives" pointer is meta-commentary that tells a reader to read X without adding any rule. AGENTS.md §3 bans comments that paraphrase the code; the in-file precedent in the same `layers.rs` (line 50) states the *rule* (one line) rather than where the rule is documented. A replacement line would have been the same kind of meta-pointer the project does not use elsewhere in this file.
- The function name `check_wiredapp_scope`, the named `WIREDAPP_SCOPE_ALLOWLIST_PREFIXES` constant, and the explicit `if file_path.ends_with("_tests.rs") { return violations; }` line in the function body are sufficient self-documentation. The rule is also encoded in `arch-lint.toml` and named in `guardrails.md` §2 (arch-lint scope rows) and ADR-027 (the `WiredApp` two-consumer discipline).
- ADR-027 remains findable by `grep -r 'WiredApp' chronicler_engine/docs/` — no in-file pointer is needed for discoverability.
- The "Optional" provenance comments for the two `Storage` import sites in `registry.rs` and `quantifier/agent.rs` are *anti-optional* in the current state. AGENTS.md §4 (No Self-Referential Comments) bans in-file policy pointers that rot. ADR-027 2026-08-01 records that the in-file `// arch-lint: storage-direct` markers were intentionally deleted with the same intent ("Storage boundary discipline lives in the ADR, not in code comments"). Re-adding analogous comments contradicts that decision. The architectural exception is recorded in ADR-027 and the map's `## Out of scope` block.
- The `guardrails.md` §2 clarification sentence is unnecessary. Ticket #12 (currently `ready-for-agent` on the same map, blocked by #10 and #11) adds a new machine-checked walker in `tests/infrastructure/guardrails/layers.rs` that catches `Storage` imports in `src/adapters/driving/http/**`. Once #12 lands, the "Storage-direct boundary is enforced by architectural review" sentence would be stale.
- No follow-up ticket needed. The remaining tickets on the map (#06 assembler budget, #07 phase ownership, #08 double-persist, #10/#11 catalogues/MessageService, #12 HTTP storage block) are independent of the citation fix.
- One ticket resolved in this session. Map's `## Not yet specified` and `## Out of scope` blocks remain unchanged.
