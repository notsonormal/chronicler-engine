# Chronicler Engine Documentation (`docs-diataxis/`)

This folder is the **new documentation tree** for the Chronicler Engine, written in parallel to `chronicler_engine/docs/` and intended to cut over in a single PR once complete. It is being *written*, not *migrated*: the existing `docs/` tree is source material, not the migration target — some content is kept, some dropped, some re-investigated from code, some re-expressed from a different perspective.

This file is the **writing-convention layer** of a three-layer enforcement model:

| Layer | Where | What it checks |
|---|---|---|
| Machine | `chronicler_engine/scripts/validate_docs.py` (extended by ticket 04) | Front-matter presence, required keys, mode vocabulary, link integrity, ADR references |
| Convention | **this file** | How to write a doc in the new frame |
| Semantic | `chronicler-docs-hygiene` skill (extended by ticket 05) | Declared-mode vs. actual-content consistency |

When in doubt about *how* to write something, read this file. When in doubt about *framing*, invoke `/grilling` and `/domain-modeling`. When in doubt about *mode classification*, ask the Diátaxis compass: "What problem does this solve for the reader?"

## The three frameworks

These are adopted as writing perspectives. The old `docs/` tree had no declared perspective, which is why maintaining it required painstaking manual work — the LLM had no frame to write from.

### Diátaxis — doc modes

Every doc in this tree is in exactly one of four Diátaxis modes. The mode is declared in front-matter and in a blockquote at the top of the body.

| Mode | Reader problem | Orientation | Chronicler examples |
|---|---|---|---|
| **Tutorial** | Learn from zero | Learning-oriented (study) | None exist yet — see "Tutorials" below |
| **How-to** | Achieve a goal | Goal-oriented (work) | `diagnostics/DEBUGGING.md` (in old tree); none here yet |
| **Reference** | Look up a fact | Information-oriented (work) | `reference/data_layer.md`, `reference/game_flow.md` |
| **Explanation** | Understand why | Understanding-oriented (study) | `explanation/two-state-channels.md`, `explanation/architecture/overview.md` |

**Compass test** (apply when classifying): "What problem does this solve for the reader?"

- Learn from zero → Tutorial
- Achieve a goal → How-to
- Look up a fact → Reference
- Understand why → Explanation

If the content doesn't solve a reader problem under any of these, drop it. If it mixes modes, split it by mode (see `game_flow.md` ↔ `two-state-channels.md` for the canonical example).

### arc52 — architecture doc sections

The architecture overview is structured as a single document (`explanation/architecture/overview.md`) with arc52 sections as H2 headings. The full 12-section arc52 template is **not** used — only the selective subset §3, §5, §7, §10. Each section answers a different question:

| Section | Question | C4 level |
|---|---|---|
| §3 Context & Scope | What is this and what touches it? | L1 (System Context) |
| §5 Building Block View | How is it built inside? | L2 (Container) + L3 (Component) |
| §7 Deployment View | How do I run it? | Infrastructure topology (`C4Deployment`) |
| §10 Quality Requirements | What guarantees does it make? | n/a (textual) |

ADRs (in `chronicler_engine/docs/adr/`) satisfy arc52 §9 in their own frame — they are not retrofitted to Diátaxis. Plans (in `chronicler_engine/docs/plans/`) are time-capsule content in their own frame. Both stay in their current locations after cutover.

### C4 — diagram levels

C4 diagrams are rendered via Mermaid's C4 directives (`C4Context`, `C4Container`, `C4Component`, `C4Deployment`). The four levels:

- **L1 (Context)** — the engine as one system among external systems. Answer to §3.
- **L2 (Container)** — the engine's major deployable units (HTTP server, application core, SQLite, outbound LLM clients). Answer to §5.
- **L3 (Component)** — the internals of one container. Used **only** when zooming into the Application Core, where the domain/application/adapters/bootstrap decomposition is the load-bearing dependency-invariant story. L3 of SQLite or the HTTP server would be redundant — don't write it. One L3 diagram zooms into one container (the standard C4 pattern).
- **Deployment** — runtime topology (process, files, network boundaries). Answer to §7.

**C4 is for software components with tech stacks and responsibilities, not for database tables.** Don't use C4 primitives for data-layer relationships — use a Mermaid `flowchart` (see "Relationships diagrams" below).

## Front-matter

Every doc carries YAML front-matter at the top of the file. Required keys:

- `diataxis: <mode>` — one of `tutorial`, `how-to`, `reference`, `explanation`. The declared Diátaxis mode.
- `title: <H1 mirror>` — the doc's title, mirroring the H1. Lets downstream indexes consume front-matter without parsing markdown.
- `arc52: [§3, §5, §7, §10]` — only on arc52-shaped docs (the architecture overview). Omit on everything else. Lists which selective arc52 sections the doc covers.

Front-matter does **not** duplicate what the content already signals — content signals by section structure (a Reference doc reads like a schema catalog; an Explanation doc reads as discursive discussion); front-matter signals by machine-readable key. They answer different questions, so the layering is useful, not redundant.

## Mode-declaration blockquote

Immediately under the front-matter, every doc carries a blockquote of the form:

> **Diátaxis mode:** Reference. This document describes X as it is, not how to use it. The problem it solves for the reader is *look-up*: ...

State the mode and the reader problem the doc solves. This is the seam that lets a reader (or LLM) orient without parsing the content. Keep it to 2–4 lines.

## Subfolder shape

Diátaxis-shape at the top level:

- `reference/` — Reference docs, flat (no subfolders unless a topic grows large).
- `explanation/` — Explanation docs. The `architecture/` subfolder groups arc52-shaped docs, because architecture is a category (future "Why Hexagonal?" docs land alongside `overview.md`).
- `tutorials/` and `how-to/` — **do not create these directories until content earns its place.** The audit found zero Tutorials and only one How-to (`DEBUGGING.md`) in the existing tree; the pilot surfaced no new Tutorial or How-to content. Empty quadrant dirs are noise. When the bulk-writing survey (ticket 06) finds content that clearly belongs to these modes, create the dirs then.

`adr/` and `plans/` stay in their current locations under `docs/` after cutover — they are out of scope for this effort.

## Writing posture

The new tree is *written*, not *migrated*. Old docs are source material. When approaching an existing doc, decide for each piece of content:

- **Keep and re-express** — content fits its Diátaxis mode cleanly; rewrite it deliberately from the new frame anyway.
- **Split** — content is mixed-purpose (e.g. `system/game_flow.md` was Reference + a "Two State Channels" Explanation aside). Split by mode into separate docs; cross-reference between them so neither stands alone.
- **Re-frame** — content changes Diátaxis mode from what the old doc typed (e.g. an "Explanation" file that's actually Reference).
- **Cross-reference** — content is mechanism (e.g. the 8-tier module map); don't re-list it, point at its source.
- **Drop** — content doesn't solve a reader problem under the compass test. Apply the test honestly; if nothing registers, drop it.

Nothing should be force-fit. If a piece of content fights the frame, that's a signal — either split, cross-reference, or drop. Record the decision in `_PILOT_NOTES.md` §6 as you go.

## Reference defers to source

Reference docs do **not** restate values that live in code as the authoritative source. This is an explicit carve-out of the no-code-indexer rule (below). Specifically: no column-level DDL, no struct field lists, no function signatures, no migration version numbers, no constants, no caps — when those are the authoritative source in code.

Restating these in markdown is drift-prone duplication. Example: the original `reference/data_layer.md` had 5 of 11 tables' column tables; `db.rs` creates 11. Drift was already real.

What Reference docs **do** carry:

- What each thing is *for* (prose, one paragraph per table/component/endpoint).
- The load-bearing invariants the code doesn't say directly ("messages are not stored in the snapshot JSON", "one message history per game", "settings is a singleton row").
- Relationships and aggregate structure (see "Relationships diagrams" below).
- Cross-references to ADRs and mechanism docs.

If a reader needs the exact column list, field type, or function signature, they open the source file — one hop. The doc's value is what the source *doesn't* say.

## Relationships diagrams

Where aggregate structure isn't obvious from reading the source sequentially, a relationships diagram earns its place. Use Mermaid `flowchart`, not `erDiagram`, not C4.

- **`flowchart`** — boxes for tables/components, edges for relationships. Cardinality as edge labels (`"1 → ∞, cascades"`). Non-FK invariants as dashed edges (`-.->|"optional, not FK"|`) with prose labels. This is the only diagram type flexible enough to express both FK edges and the load-bearing non-FK invariants (e.g. `message_swipes.snapshot_id → game_state_snapshots.id`, which is deliberately not a SQL FK).
- **Not `erDiagram`** — its strict cardinality markers (`||--o{`) handle FK edges but not non-FK invariants or independent tables cleanly, and its `{ column TYPE }` body blocks tempt re-adding the column duplication we just decided to drop.
- **Not C4** — C4 primitives describe software components with tech stacks, not data tables. Using `Component()` for a table is a category error.

**One diagram per cluster**, not everything in one. A diagram covering 11 tables in one frame is too dense to read — split by logical cluster (game state, world catalogue, standalone) and carry cross-cluster relationships as prose below the diagrams. Each cluster diagram should fit on one screen.

## Mode-specific notes

### Reference

Austere, neutral, authoritative — like a map. Describes *what is*, not *how to use it* and not *why it's that way*. The last two belong in How-to and Explanation respectively. If a Reference doc's content starts answering "why", split the why into an Explanation doc and cross-reference (the `game_flow.md` ↔ `two-state-channels.md` split is the canonical pattern).

### Explanation

Discursive, understanding-oriented. Answers "why?" — design rationale, tradeoffs, system connections. Can take perspectives and be read away from the product. Do not bloat Explanation into mechanism docs — cross-reference mechanism docs (`architecture/rust_technical.md`, `architecture/guardrails.md`) rather than re-listing their content.

The architecture overview (`explanation/architecture/overview.md`) is Explanation mode — it answers "why is it structured this way" by showing the structure and the quality tradeoffs.

### Tutorial

None exist yet. If the bulk-writing survey (ticket 06) surfaces content that earns Tutorial mode, the `tutorials/` directory gets created then. Anticipated candidate: "Getting Started: Your First Game Session" — a learning-by-doing walkthrough that builds a mental model of the engine, not a goal-achievement guide.

### How-to

Only `diagnostics/DEBUGGING.md` in the existing tree. Goal-oriented directions for already-competent users. Written from the user's goal, not from the machinery. If the bulk-writing survey surfaces more How-to content, the `how-to/` directory gets created.

## Diagrams

Mermaid only. The toolchain is already in place; do not introduce a new diagram tool.

- **C4 directives** (`C4Context`, `C4Container`, `C4Component`, `C4Deployment`) — for the architecture overview's §3/§5/§7. Use the `UpdateLayoutConfig($c4ShapeInRow="4", $c4BoundaryInRow="2")` directive for readability.
- **`flowchart`** — for relationships diagrams (see above) and for runtime/process diagrams (phase sequences, retry flows). Existing `game_flow.md` and `action_pipeline.md` use this style.
- **`erDiagram`** — avoid; see "Relationships diagrams" above.
- **`sequenceDiagram`** / **`stateDiagram-v2`** — not yet used in the pilot; available if a doc genuinely needs them.

One diagram per cluster, not one diagram for everything. Keep each diagram small enough to read at a glance.

## No negative-disclaimer paragraphs about what the diagrams don't show

The C4 diagrams and the Out-of-scope list are the source of truth for what's external to the engine. If something isn't in the diagram, it isn't in scope — **don't** add parallel paragraphs saying "X is not an external system" or "X is not a runtime dependency." That framing is defensive and keeps the very thing it's disclaiming in the reader's mind. (Analogy: a diagram showing AWS, GCP, and Postgres doesn't need a paragraph saying "Note: Microsoft Azure is not an external system here.")

Inspirations belong in the explanation doc for the thing they inspired, not in the architecture overview.

## No code-indexer docs (inherited rule, with a carve-out)

Inherited from `chronicler_engine/docs/AGENTS.md`: docs should not be code indexers. The code is self-documenting; the docs are a layer on top, existing because they are more concise and easier to curate than AI-generated code comments. Docs should not explain *how the code works* by exhaustively referencing modules, classes, methods, and types.

**Carve-out** (refined during pilot review): schema column tables, struct field lists, function signatures, migration version numbers, and constants are code-indexing and are **not** restated in Reference docs — see "Reference defers to source" above. What Reference docs *do* carry (purpose, invariants, relationships, cross-references) is not code-indexing.

XML/domain markups (e.g. `<ConversationHistory>`, `<PlayerInput>`) are domain tags, not code references — they don't trigger the code-indexer test.

## Plan-adherence

Per the workspace `AGENTS.md`: do not silently deviate from an agreed plan. If you discover a problem, opportunity, or edge case not addressed in the current plan, **stop**, report it with the two prescribed options (A: fix this now with explicit approval; B: add to plan and continue as written), and wait for direction before proceeding.
