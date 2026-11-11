# Audit pool manifest

Built by ticket 01. This is the pool every downstream ticket (03 extraction, 04 verification, 05 assessability) reads from. Source of truth: the flattened transcripts in `tmp/flattened_pool/` (built by `tmp/flatten_sessions.py` from the raw jsonl).

## Scope finding (read first)

**The 30-day window is only 10 days of data.** The map's Notes (Q5=B) specify 2026-07-21 → 2026-08-20. The chronicler-engine session directory's earliest top-level file is **2026-08-11**. Sessions before 2026-08-11 do not exist under this project because the repo was moved out of `mrn-general` into its own chronicler-engine project on 2026-08-11 (session `019ff259`, user turn: "I just moved the chronicler_engine from the mrn-general repository, including the git history"). Pre-2026-08-11 work lives under the `mrn-general` project, which is out of scope (Q5=B, chronicler-engine only).

So the audit pool is inherently limited to **2026-08-11 → 2026-08-20** (10 days). This is a data-availability constraint, not a scoping choice. Flagged for the user: if pre-08-11 sessions are wanted, that requires redrawing scope to include `mrn-general` — currently out of scope.

## Selection criteria

Definition applied (from the ticket): a **substantive software-problem-solving turn** is a user turn that states a hypothesis, makes a design or architecture choice, pushes back on an AI suggestion, asks a clarifying question about the problem (not about tooling), reports a bug or observation about behavior, or drives a refactor.

Excluded:
- **Pure skill/subagent auto-runs** — sessions whose only user turn is a `<skill ...>` invocation or a one-word confirmation with no reasoning. This includes all 21 subagent `tasks/` subdirectories (research/code-review subagents) and single-turn `commit-and-push` / `test-police` / `wayfinder`-only sessions.
- **Tooling/environment chatter** — extension install/uninstall, terminal rendering (WezTerm/ghostty), WSL storage, provider/subscription config, output-style plugin setup. Per the definition's "not about tooling" clause.
- **Session-management chatter** — `continue`, `close`, `map`, bare option letters with no reasoning.
- **The two auditor sessions** — prior run `01a02084-3749-7047-9e4b-9df531a609d5` ("Act as a Metacognitive Learning Auditor") and the current session `01a020c2-65de-76d1-9abe-f4a5a9dd6ddc` (wayfinder "run the map"). Both are meta, not software problem-solving.

Included per Q9=B: design, planning, and reasoning sessions count, not only implementation/debugging. So wayfinder/grilling sessions where the user makes design decisions are in.

**Inclusion threshold:** a session qualifies if it has at least one user turn with genuine problem-solving reasoning (a design decision, pushback, investigation, or refactor drive) — not just a brief confirmation after a skill auto-run. Substantive-turn counts below are judged counts (approximate), not mechanical.

## Counts

- Total top-level session files in directory: **83** (spanning 2026-08-11 → 2026-08-20).
- Subagent `tasks/` subdirectories (excluded as auto-runs): 21.
- Auditor sessions excluded: 2.
- **Qualifying sessions in the audit pool: 30.**
- Approximate total substantive user turns across the pool: **~161**.

Pool size (30 sessions, ~161 substantive turns) fits one extraction session reading the flattened transcripts — no batching needed. This clears the map's "Extraction overflow" fog.

## Manifest

Flattened file = `tmp/flattened_pool/<basename>.md`. Session id is the UUID in the filename. Tag: design / planning / impl / debug / meta.

| # | Date | Session id | Subst. turns | Tag | Reason for inclusion |
|---|------|------------|--------------|-----|----------------------|
| 1 | 08-11 | 019ff259-d346-7a2b-9b5b-dd79be792c22 | ~7 | impl/planning | Repo move from mrn-general; path-investigation, README scoping, plan-then-implement |
| 2 | 08-11 | 019ff32f-54d7-7431-911d-c82731dd3e32 | ~6 | design | preset_store.rs purpose questioned; removal planned; field-design pushback |
| 3 | 08-12 | 019ff7a6-9c84-7142-b656-c0d3f8f63598 | ~5 | impl/debug | Implement remove-preset-store plan; code-review fixes; git "Index already up-to-date" hook bug diagnosed |
| 4 | 08-12 | 019ff7a8-12d1-700f-b0eb-1530ced86051 | ~4 | planning | Plans triaged valid/useless/needs-update; `-revised` rewrite strategy chosen over in-place edit |
| 5 | 08-12 | 019ff7e2-1b9c-7bc7-9a93-4c4c40186831 | ~6 | impl/design | Orphan-orchestrator-tests plan; review-comment investigation; test-pairing rule grilling |
| 6 | 08-12 | 019ff7fc-2386-71ca-83e7-338af5d3f614 | ~3 | planning | Pushback: "separate ticket for each issue is excessive"; task-vs-grilling ticket distinction; ticket deletion |
| 7 | 08-13 | 019ffd31-44b7-78c7-ba9d-c76b4bd8de5f | ~7 | impl/planning | After-plan workflow; WSL-crash recovery; fix-issue plan; state re-explanation after absence |
| 8 | 08-13 | 019ffd32-d753-7bc0-9ef9-636692e8223b | ~5 | planning/impl | Plan reiteration; "Oh, not implement" / "Oh, now implement" corrections; separate import-locality plan |
| 9 | 08-14 | 01a0014d-5a40-7f75-9d1d-25eb05485ddf | ~5 | meta/design | AGENTS.md overlap cleanup; ASD-STE100 vs telegraphese output-style decision |
| 10 | 08-14 | 01a0015a-b8a5-7d97-8ce2-02462db05006 | ~7 | meta/design | Second-opinion review of AGENTS/APPEND_SYSTEM; status roll-up source; "maybe it doesn't belong in system prompt" |
| 11 | 08-14 | 01a0017d-53d3-7a69-b98f-468e0953ed12 | ~15 | design | guardrails.md auto-gen; invariants rethink (tautology, vs unit tests, where they live); Cargo.toml test layout; `//` vs `//!` vs `///` |
| 12 | 08-14 | 01a0019e-796e-7fb0-a866-c7c3bad7d20f | ~4 | impl/design | LlmMessage DTO application→domain move; consistency check; "leave it be unless required" |
| 13 | 08-15 | 01a0056d-4b9a-7b28-812d-1d3a6f75617c | ~6 | design | Inherent-impl-locality folder structure; subfolder-vs-single; mod.rs-no-methods; pipeline_run.rs naming |
| 14 | 08-15 | 01a005cb-9bb1-7221-8f2f-5c408157efa1 | ~5 | design | Consolidate meaning; split impls into files; smell-2 fix scope; duplicated-code judgment |
| 15 | 08-15 | 01a0063e-63a8-7a58-bd45-ac39565e0c5f | ~6 | planning/design | Ticket-11 blocking-direction reasoning; "don't change resolved tickets"; storage/backend naming; consolidation rejected |
| 16 | 08-15 | 01a00654-0408-7d00-a254-bf2556dd61b6 | ~5 | design | Quantifier-vs-system-prompt distinction; test-naming finding queried; repeated-switches finding queried |
| 17 | 08-15 | 01a00688-36f1-7af7-a852-cf50dcf2dc15 | ~10 | design | Guided-generations: impersonate-vs-guide; Marinara/sillytavern comparison; snapshot-vs-swipe; FreeAction leftover |
| 18 | 08-15 | 01a00697-c737-7e8b-9dbb-340e411873e5 | ~2 | design | "Are there actual problems you haven't fixed?"; error-mapping boilerplate how-handled-elsewhere |
| 19 | 08-15 | 01a006e8-6dab-7862-8ea4-6c301ae293a5 | ~2 | planning | Pushback: "I don't understand why a research ticket is changing code"; revert + new grill ticket |
| 20 | 08-15 | 01a0073e-19f8-797c-b613-0fbb44913561 | ~10 | impl/design | After-plan review: middle-man/free-function/private-method; inline-vs-separate; snake_case tests; build.py --llm-only broken |
| 21 | 08-15 | 01a00741-35d5-74e6-a861-5d5ed25ea354 | ~4 | design | Persona-replacement vs user-string-replacement distinction; impersonate output = input; narrator+guide composition |
| 22 | 08-16 | 01a009c2-2290-7761-b76f-6293747f68f9 | ~4 | planning | Map correction: "the map is wrong"; spec questions unanswerable pre-implementation; don't-block-ticket-14 |
| 23 | 08-16 | 01a00c8f-45de-7366-9d09-5fec325e29b6 | ~7 | design | Unit-test-pairing pattern questioned; in-memory-test speed rationale; plan to investigate removing in-memory storage |
| 24 | 08-19 | 01a01bda-7748-7313-b0e5-efe5950c09e9 | ~6 | design | Guided-generations: mutual-exclusivity one-way; Marinara persona-info removal; cache-breaking; narrator placement |
| 25 | 08-19 | 01a01be6-902d-71ff-ab9b-3de2035e21c0 | ~1 | planning | Scoping: "I don't know what I want"; filter to 2026-updated repos first |
| 26 | 08-19 | 01a01bf0-c66f-7a09-a120-e2b6f17a9c57 | ~2 | planning | Research framing: "wrong perspective — need CODE DEVELOPMENT not application-LLM"; subagent dispatch |
| 27 | 08-19 | 01a01c06-becb-754d-9683-e650affd6c57 | ~2 | planning | Terminology pushback: "pattern suggests non-deterministic"; practice/technique/tooling adopted |
| 28 | 08-19 | 01a01c11-357a-76b8-9d2b-f64f759c250a | ~5 | planning | Report mixing "using for LLM" vs "code development"; "already covered" dismissed too aggressively; new-map benefit questioned |
| 29 | 08-19 | 01a01c34-482c-7b29-9722-990586f56c9e | ~6 | design/planning | preflight.py vs build.py; centralize cargo in build.py; gerkin-vs-markdown specs; cucumber explanation |
| 30 | 08-20 | 01a020a2-6f47-7e2f-b25e-e55b79a4338a | ~4 | meta | Output-style comparison; pros/cons vs APPEND_SYSTEM; session-history read for evidence; nopus-rewrite confound noted |

## Excluded categories (for traceability)

- **Single-turn skill auto-runs (no reasoning):** ~17 sessions — `commit-and-push` (×9), `test-police`, `thermo-nuclear-code-quality-review`, `wayfinder`-only (×7), `improve-codebase-architecture` (×2), `planning-and-task-breakdown`.
- **Tooling/environment:** VSCode pnpm lockfile warning; WSL file-storage/space; synthetic subscription config; thinking-levels model config; pi-emote avatar/WezTerm rendering (×1, 14 turns); extension install/uninstall (pi-cost-counter, ponytail, pi-prompt-template-model, pi-caveman, pi-prompt-template-mode); nopus extension evaluation; pi-on-demand-context evaluation; usage-report/insights extensions; grill-with-docs on coding-provider adoption.
- **Mechanical/trivial:** remove local file paths in docs; check-and-delete two plan files; wayfinder+"apply fixes"; wayfinder+"rewrite ticket"; wayfinder+"close/run build.py"; wayfinder+"map"; wayfinder+"Fix S4"; wayfinder+brief grilling answers; commit-and-push+"delete the pr"; commit-and-push+"Add them in".
- **Auditor sessions:** `01a02084` (prior shallow run), `01a020c2` (this session).

## Notes for downstream tickets

- **Extraction (03):** read each session's flattened transcript in `tmp/flattened_pool/`. The stimulus (assistant text the user reacted to) is preserved verbatim, so post-suggestion user responses are interpretable. Sessions 11, 17, 20, 24 are the richest (design reasoning, multiple pushbacks). Sessions 18, 19, 25, 26, 27 are thin (1–2 substantive turns) — extract what's there, don't force findings.
- **Verification (04):** every quote must substring-match the cited flattened file. The flattened files are the single source.
- **Assessability (05):** the pool is design/planning-heavy (Q9=B), so traps that show in design reasoning (Assumption, Forming, Location) are well-represented; Achievement (band-aid vs refactor) may still need diffs for the impl sessions.
