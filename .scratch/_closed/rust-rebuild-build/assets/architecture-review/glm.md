# Architecture review — GLM 5.3 (ticket 15)

Tree: `wf/rebuild-core` @ `2885df2c` (HEAD `d7cf3eb0`; the two commits between touch only `.scratch/`, `src/` and `tests/` verified identical). Dirty files under `src/`+`tests/`: 0.

Steps 1 and 2 of `/improve-codebase-architecture`, with the `/codebase-design` vocabulary and the `CONTEXT.md` terms. No `docs/adr/` exists; the decided rules found are `arch-lint.toml` (enforced by the architecture test binary) — flagged in-card where a candidate brushes one.

Five candidates. Strengths: Strong 1, Worth exploring 3, Speculative 1.

## C1 · One domain module for the room of a Swipe state — Strong

**Files:** `src/domain/model/utils/game_rules.rs` (`room_of`, `room_name`), `src/application/pipeline/job.rs` (`room_of`, `room_name`, `resolve_room`), `src/application/games/view_query.rs` (`get_current_room_view`), `src/application/generation/gate.rs` (`opening_room_report`).

**Problem:** "The room a Swipe state stands in" (the map's room, else the dynamic room) and its display name are one domain rule with four copies. `game_rules.rs` holds it privately; `job.rs` re-implements it; `view_query.rs` inlines the `or_else(dynamic_rooms)` chain; `gate.rs` re-implements the name lookup and silently drops the dynamic-room branch. The rule leaks across its seam: each copy decides the dynamic-room question on its own, and they already disagree.

**Solution:** One interface on the domain module — the room of a Movement (`MapDef::room_of(&movement, room_id) -> Option<&Room>` or a public function beside `game_rules.rs`) plus the room's display name. The four call sites call it.

**Benefits:**
- **Locality:** the dynamic-room rule changes in one module; the copies cannot diverge again.
- **Leverage:** one interface, four call sites; a caller learns one fact instead of four.
- **Tests:** a pure `domain/model/` function is exactly where the test strategy puts unit tests; the rule gets one test instead of none.

**Recommendation strength:** Strong.

## C2 · Split the Storage seam along the catalogue / story line — Worth exploring

**Files:** `src/adapters/driven/storage/core.rs` (`Backend`, `BackendKind`), `stories.rs`, `transaction.rs`, `docks.rs`, `in_memory_data.rs`.

**Problem:** `Storage` presents one interface with two adapters, but the InMemory adapter implements only the catalogue half (Worlds, Personas, Characters, Settings, Presets, LLM forensics). Every story read (7 sites) and every save unit answers `no_in_memory("...")` at runtime. One adapter per seam half means the story half's seam is only hypothetical; the interface promises behaviour one adapter refuses, and the runtime error says so instead of the type.

**Solution:** Name the line the adapters already draw: split the interface into a catalogue seam (two adapters — a real seam) and a story seam (StoryReads plus the save units — one adapter, honestly named). The `no_in_memory` match arms disappear.

**Benefits:**
- **Locality:** which reads work per adapter lives in the type, not in nine refusal arms.
- **Leverage:** catalogue-side tests (settings service, preset service, world catalogue, recorder) exercise a full second adapter.
- **Tests:** the `tests/storage/` seam contract splits along the same line; each half's cases stay pinned.

**Recommendation strength:** Worth exploring — the split is real but the churn touches the storage seam contract; needs the grilling step.

## C3 · One hook-input assembly behind the two hooks — Worth exploring

**Files:** `src/application/pipeline/job.rs` (`analyse_step`, `options_step`), `src/domain/model/agent.rs` (`WorldView`, `HookInput`, `STORY_WINDOW`).

**Problem:** The analysis hook and the Options hook in `job.rs` each repeat the same assembly: read the World bundle, read the start/current Swipe state, read the `STORY_WINDOW` story window, then build the same `WorldView` and `HookInput` (two near-identical blocks, ~20 lines apart). The hook has no single assembly point. `CONTEXT.md` names before-narration and rewrite hooks as coming Agents; each new hook copies the assembly again.

**Solution:** One assembly in the Job — "the hook input of the tail" — that both hook steps call. The room rule from C1 sits inside its `WorldView` build.

**Benefits:**
- **Locality:** hook-input rules (which window, which state, which room) concentrate in one place.
- **Leverage:** N hooks, one assembly; a new hook becomes one call, not a copy.
- **Tests:** both hook steps cross one seam; the hook tests share one setup shape.

**Recommendation strength:** Worth exploring.

## C4 · One read-side module for the dashboard — Worth exploring

**Files:** `src/application/games/view_query.rs`, `src/application/world_catalogue.rs`, `src/application/persona_catalogue.rs`, `src/application/games/catalogue.rs`, `src/adapters/driving/http/games/handlers/games.rs`, `src/adapters/driving/http/app_state.rs`.

**Problem:** The dashboard's read side is spread over four application modules. A handler such as `list_games_fragment` fetches GameCatalogue, WorldCatalogue and PersonaCatalogue separately, then `GameViewQuery` for the rest. `WorldCatalogue` is a field-for-field pass-through over `Storage` — interface nearly as wide as the implementation; its only real rules (the create refusal, `WorldHasGames`) are write-side.

**Solution:** Fold the catalogue *reads* into the one read-side module (`GameViewQuery`); keep the write-side orchestration (create, switch, rename, delete) where its rules live. The HTTP adapter learns one read interface.

**Benefits:**
- **Leverage:** one read interface, N handlers; a handler stops stitching four modules.
- **Locality:** dashboard read bugs concentrate in one module.
- **Tests:** HTTP fragment tests cross one seam.

**Decided-rule note:** `arch-lint.toml` forbids the server layer importing storage directly — the catalogues exist partly to carry that seam. This candidate stays inside the rule (reads fold into an application module); the layer rule itself is not on the table.

**Recommendation strength:** Worth exploring.

## C5 · Collapse the story-window projection trio — Speculative

**Files:** `src/domain/model/state/message_types.rs` (`StoryRow`, `MessageEntry`), `src/domain/model/agent.rs` (`StoryEntry`), `src/domain/model/message.rs`, `src/adapters/driven/storage/stories.rs`.

**Problem:** Three projections carry the same story window. `MessageEntry` duplicates `StoryRow` field for field (eight fields, a `From` copy); `StoryEntry` is the Agent's narrow view. Every window read converts twice (`StoryRow → MessageEntry → StoryEntry`), and understanding "what the story window holds" needs three domain files. The deletion test on `MessageEntry` moves complexity, it does not concentrate it.

**Solution:** Collapse to two projections: the storage row and one domain window type. The Agent view (`StoryEntry`) stays narrow if the agent-edge rule needs Agents blind to full rows.

**Benefits:**
- **Locality:** the message vocabulary concentrates in fewer files.
- **Leverage:** one fewer projection per caller to learn.
- **Tests:** the same domain outcomes hold; test seeding loses one mapping layer.

**Decided-rule note:** `arch-lint.toml`'s agent-edge rule and serde on `MessageEntry` may each justify a separate type — this one needs the grilling step before any move.

**Recommendation strength:** Speculative.

## Top recommendation

**C1 — one domain module for the room of a Swipe state.** The four copies of the rule already disagree (`gate.rs` drops the dynamic-room branch — a live divergence, not a hypothetical one). The fix is pure `domain/model/` code with zero architecture risk, the interface is small, and it feeds C3, whose hook-input assembly re-uses the same rule. Deepen here first.
