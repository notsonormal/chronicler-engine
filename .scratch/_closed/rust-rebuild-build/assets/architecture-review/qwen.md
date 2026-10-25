# Architecture review — qwen run (ticket 16)

- **Tree:** `wf/rebuild-core`, HEAD `d7cf3eb0` (`src/` and `tests/` identical to `2885df2c`; the two later commits touch only `.scratch/`). Dirty files under `src/`+`tests/`: 0.
- **Method:** steps 1–2 of `/improve-codebase-architecture`, vocabulary of `/codebase-design` and `CONTEXT.md`. No `docs/adr/` exists. Blind run: the whole-rebuild review assets and tickets 13/17 were not read. Read-only on code; no build run.
- **HTML report:** `/tmp/architecture-review-arch-qwen-20261019-233248.html`

## Candidate 1 — One type family for the analysis hook — **Strong**

**Files:** `src/domain/model/agent.rs`, `src/domain/model/quantifier.rs`, `src/application/agents/runner.rs`, `src/application/agents/quantifier/agent.rs`, `src/application/agents/quantifier/types.rs`, `src/domain/model/state/scene_state.rs`, `src/application/pipeline/job.rs` (consumer).

**Problem:** The Quantifier's answer crosses its seam twice. `determine_npcs_in_room` produces a `QuantifierResult`; `QuantifierAgent::run` converts it to `StatePatch` + `Confidence`; `AgentRunner::analysis` merges patches and rebuilds a fresh `QuantifierResult` field-by-field for the Game rules. The rebuild is lossy — `movement_type` and the movement confidence die in the merge — and nothing notices because no caller reads them (they are only parsed and logged). `Confidence` and `QuantifierConfidence` are duplicate three-variant enums with two-way `From` impls; `quantifier/types.rs` is mostly re-exports of `domain/model/quantifier.rs`; `SceneState.quantifier_confidence: Option<String>` is a third confidence representation that no production code writes. Understanding one analysis means bouncing between six modules.

**Solution:** Pick one family for the analysis hook: Agents return it, the `AgentRunner` merges it (multiple analysis Agents are the reason the merge exists), and `apply_game_rules` consumes it. Delete the duplicate enum, both `From` directions, the re-export file, and the dead string field on the Swipe state.

**Benefits:**
- *locality:* one analysis shape lives in one module; a Quantifier bug has one home.
- *leverage:* the Agents and the Game rules share one interface; no conversion per crossing.
- *deletion test:* deleting one family concentrates nothing — the conversions exist only to bridge the two, so they vanish.
- *tests:* the Quantifier Agent's seam cases (`CASE` tags) assert one type end to end; the lossy round-trip becomes impossible rather than untested.

## Candidate 2 — Make the storage record seam honest — **Worth exploring**

**Files:** `src/adapters/driven/storage/core.rs`, `stories.rs`, `transaction.rs`, `docks.rs`, `utils/records.rs`, `test_support.rs`.

**Problem:** The `Backend` enum seam is real for the seed/config tables — two adapters, SQLite and InMemory, and unit tests substitute InMemory. For the record tables (story reads, dock record, the save unit) it is hypothetical: one adapter, SQLite, and every method carries a dead `Backend::InMemory(_) => Err(no_in_memory(...))` arm (9 arms across stories/transaction/docks; 41 enum-dispatch sites in total). Read failure injection is keyed on `&'static str` method names (`with_backend_mut("stories_tail", …)`), while the write injection already moved to the compiler-checked `TxWrite` enum — "keyed by a type, so a failure injection stays compiler-checked". A renamed method silently detunes a planted read failure.

**Solution:** Split the seam in two. The seed side keeps the two-adapter enum. The record side admits one adapter (SQLite), drops the dead arms, and keys read injection on a `TxRead`-style type like `TxWrite`. (Alternative: a `Backend` trait — but one adapter means a hypothetical seam, so the honest split is the smaller interface.)

**Benefits:**
- *locality:* one record read has one implementation, no dead arm.
- *tests:* injection keys compiler-checked; a rename fails the build, not the diagnosis.
- deletion test: deleting the dead arms concentrates nothing — they refuse and return.

## Candidate 3 — One story entry, not three projections — **Worth exploring**

**Files:** `src/domain/model/state/message_types.rs` (`StoryRow`, `MessageEntry`), `src/domain/model/agent.rs` (`StoryEntry`), `src/application/pipeline/job.rs`, `src/application/games/view_query.rs`, `src/adapters/driven/storage/stories.rs`.

**Problem:** One story entry has three near-identical shapes: `StoryRow` (the window read), `MessageEntry` (views, prompt history, debug DTO — same 8 fields minus `kind`, plus `Serialize`/`Default`), `StoryEntry` (the Agent hook — 2 fields). Two field-by-field `From` impls and five `.map(_::from)` chains sit at the use sites. Understanding "what a window read returns" means bouncing between three modules.

**Solution:** The story read seam returns one entry type; the Agent hook and the views take slices of it; a projection survives only where a serde shape genuinely demands one (`MessageEntry`'s `Serialize` for `DebugStateView`). The `kind` field only `Tail` needs stays on `Message`.

**Benefits:**
- *locality:* the entry shape lives in one module.
- *leverage:* one interface crosses the seam from the story reads to every consumer.
- *tests:* tier-1 story assertions and hook inputs speak the same type.

## Candidate 4 — Prompt wording leaks into the Job — **Worth exploring**

**Files:** `src/application/pipeline/job.rs` (`continuation_user_message`, `user_message`, `player_text`, `narrate_step`), `src/application/narration/narrator.rs`, `src/application/prompting/builders/sections.rs`.

**Problem:** The Job — the plan of one request — holds English LLM instructions ("Continue the scene naturally … Do NOT repeat or contradict …") and decides what the narrator is sent: it picks the preset, derives the user message per `Kind`, and assembles the `PromptContext`. Every other wording lives behind the Narrator seam in `prompting/` and the Agent prompt builders. Prompt behaviour has no locality: a wording bug sends the maintainer to the pipeline module, and `narrator_tests.rs` cannot see the event-continuation wording at all.

**Solution:** Move the event-continuation wording and the user-message picking behind the Narrator's interface: the Job hands over the `GenerationOrder`, the `EventRecord` and the window; the Narrator owns the phrasing and the assembly.

**Benefits:**
- *locality:* all prompt wording in one module family.
- *leverage:* the Narrator's interface absorbs the assembly; the Job shrinks toward steps and save units only.
- *tests:* the Narrator's seam contract pins wording; the Job's tier-1 scenarios stop asserting prose.

## Candidate 5 — The one-method catalogue — **Speculative**

**Files:** `src/application/persona_catalogue.rs`, `src/application/world_catalogue.rs`, `tests/infrastructure/guardrails/layers.rs` (`check_http_storage_leak`), `tests/STRATEGY.md` (read seams).

**Problem:** `PersonaCatalogue` is one pass-through method; `WorldCatalogue` is five, of which only `create_world` adds behaviour (an error classification). Interface nearly as wide as the implementation: shallow. They exist because the guardrail bars HTTP from naming `Storage`, and STRATEGY.md names them read seams — a deliberate tax, which is why this is Speculative, not Strong.

**Solution:** Only if the friction stays real: fold the creation-side reads into one deeper module that answers "what can a new Game bind to" (Worlds with their Personas and newest Games), instead of mirroring `Storage` method-for-method. Do not delete the seam — the guardrail mandates it.

**Benefits:**
- *leverage:* the game-creation handler learns one interface.
- *deletion test (weak today):* deleting the catalogues moves the error mapping into N handlers rather than concentrating it — which is exactly what keeps them alive, but a deeper aggregation would earn more of its interface.

## Top recommendation

**Candidate 1 — one type family for the analysis hook.** It is the only candidate that deletes an interface instead of moving one: the duplicate `Confidence` enums, the two-way conversions, the re-export file, the lossy rebuild in the `AgentRunner` and the dead `Option<String>` confidence field all disappear, and the Quantifier, its hook and the Game rules then speak one language. The blast radius is small, everything is in-process, and the Quantifier Agent's seam-contract cases pin the behaviour while it moves.
