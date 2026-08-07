# Chronicler Engine Documentation

This folder contains all documentation for the Chronicler Engine project.

For general engine principles, workflow, and conventions, see [`../AGENTS.md`](../AGENTS.md).

Write documention using Simplified Technical English (STE).

## Keeping Documentation Clean

**Plan authoring convention:** reference doc issues by quotable phrase, never line numbers — line numbers rot. Use the exact sentence (or a quoted fragment of it) as the anchor.

### The Per-Edit Gate (for doc edits)

Docs in this repo are a **Specification**, not a conversation. They state contracts — what the system guarantees — not implementations. Code references in prose are sediment unless they pass the non-removable test.

Spec-Driven Implementation (SDI) means the code reflects the spec. It does **not** mean restating code in the docs. Symbols map 1-to-1 to concepts (SDI principle), so naming the concept in prose IS naming the symbol — no need to also quote the function/type/file.

#### Code Indexer

Documents should not be code indexers. The principle is that the code is self-documenting and the chronicler engine docs is a layer on top of that. The docs exist because they are more consise and easier to curate than AI generated code comment.

Docs should not be explaining how the code works, which is what excessive references to modules, classes, methods and types tends to be. Plans (`docs/plans`) is naturally excluded for this.

XML/domain markups (e.g. `<ConversationHistory>`, `<PlayerInput>`) are domain tags, not code references — they don't trigger this test.

Accumulated violations in existing docs: invoke the [`chronicler-docs-hygiene`](../../.agents/skills/chronicler-docs-hygiene/SKILL.md) skill.

## Folder Structure

<!-- AUTO-INDEX START -->
*Index last generated: 2026-07-19 19:52 UTC*

### `docs/diataxis/explanation/`

- [Agent System Design](./diataxis/explanation/agent_system_design.md)
- [Architecture](./diataxis/explanation/architecture.md)
- [Dashboard Design](./diataxis/explanation/dashboard_design.md)
- [Diataxis](./diataxis/explanation/diataxis.md)
- [Prompt System Design](./diataxis/explanation/prompt_system_design.md)
- [Rust Idioms](./diataxis/explanation/rust_idioms.md)
- [Storage Design](./diataxis/explanation/storage_design.md)
- [Two State Channels](./diataxis/explanation/two-state-channels.md)

### `docs/diataxis/how-to/`

- [Debugging](./diataxis/how-to/debugging.md)

### `docs/diataxis/reference/`

- [Architecture System](./diataxis/reference/architecture_system.md)
- [Game Flow](./diataxis/reference/game_flow.md)
- [Startup](./diataxis/reference/startup.md)
- [Storage](./diataxis/reference/storage.md)

### `docs/diataxis/reference/coding_standards/`

- [Guardrails](./diataxis/reference/coding_standards/guardrails.md)
- [Integration Test Standards](./diataxis/reference/coding_standards/integration_test_standards.md)
- [Testing](./diataxis/reference/coding_standards/testing.md)
- [Unit Test Standards](./diataxis/reference/coding_standards/unit_test_standards.md)

### `docs/diataxis/reference/frontend/`

- [Dashboard](./diataxis/reference/frontend/dashboard.md)
- [Http Routes](./diataxis/reference/frontend/http_routes.md)
- [Ui Design](./diataxis/reference/frontend/ui_design.md)

### `docs/diataxis/reference/narrative/`

- [Agent System](./diataxis/reference/narrative/agent_system.md)
- [Narration System](./diataxis/reference/narrative/narration_system.md)
- [Prompt System](./diataxis/reference/narrative/prompt_system.md)

<!-- AUTO-INDEX END -->

## Writing Conventions

Docs under `docs/diataxis/` are written deliberately in their Diátaxis mode, not lifted from elsewhere. The compass test ("What problem does this solve for the reader?") governs what stays, what splits, what cross-references, and what drops.

This section is the **writing-convention layer** of a three-layer enforcement model:

| Layer | Where | What it checks |
|---|---|---|
| Machine | `chronicler_engine/scripts/validate_docs.py` | Front-matter presence, required keys, mode vocabulary, link integrity, ADR references |
| Convention | **this section** | How to write a doc in the new frame |
| Semantic | `chronicler-docs-hygiene` skill | Declared-mode vs. actual-content consistency |

When in doubt about *how* to write something, read this section. When in doubt about *framing*, invoke `/grilling` and `/domain-modeling`. When in doubt about *mode classification*, ask the Diátaxis compass: "What problem does this solve for the reader?"

### The two frameworks

These are the writing perspectives the diátaxis tree operates under.

#### Diátaxis — doc modes

Every doc in `docs/diataxis/` is in exactly one of four Diátaxis modes. The mode is declared in front-matter.

| Mode | Reader problem | Orientation | Chronicler examples |
|---|---|---|---|
| **Tutorial** | Learn from zero | Learning-oriented (study) | None exist yet — see "Tutorials" below |
| **How-to** | Achieve a goal | Goal-oriented (work) | `diataxis/how-to/debugging.md` |
| **Reference** | Look up a fact | Information-oriented (work) | `diataxis/reference/storage.md`, `diataxis/reference/game_flow.md` |
| **Explanation** | Understand why | Understanding-oriented (study) | `diataxis/explanation/two-state-channels.md`, `diataxis/explanation/architecture.md` |

**Compass test** (apply when classifying): "What problem does this solve for the reader?"

- Learn from zero → Tutorial
- Achieve a goal → How-to
- Look up a fact → Reference
- Understand why → Explanation

For the framework's first principles (two-axis compass, adjacent-mode boundaries, workflow), see `diataxis/explanation/diataxis.md`.

If the content doesn't solve a reader problem under any of these, drop it. If it mixes modes, split it by mode. Canonical splits in this tree: `game_flow.md` ↔ `two-state-channels.md`, `agent_system.md` ↔ `agent_system_design.md`, `storage.md` ↔ `storage_design.md`.

### Front-matter

Every doc under `docs/diataxis/` carries YAML front-matter at the top of the file. Required keys:

- `diataxis: <mode>` — one of `tutorial`, `how-to`, `reference`, `explanation`. The declared Diátaxis mode.
- `title: <H1 mirror>` — the doc's title, mirroring the H1. Lets downstream indexes consume front-matter without parsing markdown.

Front-matter does **not** duplicate what the content already signals — content signals by section structure (a Reference doc reads like a schema catalog; an Explanation doc reads as discursive discussion); front-matter signals by machine-readable key. They answer different questions, so the layering is useful, not redundant.

### Subfolder shape

Diátaxis-shape at the top level of `docs/diataxis/`:

- `reference/` — Reference docs.
- `explanation/` — Explanation docs.
- `tutorials/` and `how-to/` — **do not create these directories until content earns its place.** Empty quadrant dirs are noise; create `tutorials/` and `how-to/` only when a doc earns the mode.

`adr/`, `external_applications/`, `plans/`, and `specs/` live under `docs/`, alongside `diataxis/`. They are not part of the diátaxis tree.

### Writing posture

Docs in this tree are written deliberately in their Diátaxis mode, not lifted from elsewhere. When writing or revising a doc, decide for each piece of content:

- **Keep and re-express** — content fits its Diátaxis mode cleanly; rewrite it deliberately in the frame anyway.
- **Split** — content is mixed-purpose. Split by mode into separate docs; cross-reference between them so neither stands alone.
- **Re-frame** — content's actual Diátaxis mode differs from where it currently lives (e.g. an "Explanation" file that's actually Reference).
- **Cross-reference** — content is mechanism (e.g. a module map); don't re-list it, point at its source.
- **Drop** — content doesn't solve a reader problem under the compass test. Apply the test honestly; if nothing registers, drop it.

Nothing should be force-fit. If a piece of content fights the frame, that's a signal — either split, cross-reference, or drop.

### Reference defers to source

Reference docs do **not** restate values that live in code as the authoritative source. This is an explicit carve-out of the no-code-indexer rule (below). Specifically: no column-level DDL, no struct field lists, no function signatures, no migration version numbers, no constants, no caps — when those are the authoritative source in code.

Restating these in markdown is drift-prone duplication — the source changes, the doc rots.

What Reference docs **do** carry:

- What each thing is *for* (prose, one paragraph per table/component/endpoint).
- The load-bearing invariants the code doesn't say directly ("messages are not stored in the snapshot JSON", "one message history per game", "settings is a singleton row").
- Relationships and aggregate structure (see "Relationships diagrams" below).
- Cross-references to ADRs and mechanism docs.

If a reader needs the exact column list, field type, or function signature, they open the source file — one hop. The doc's value is what the source *doesn't* say.

### Relationships diagrams

Where aggregate structure isn't obvious from reading the source sequentially, a relationships diagram earns its place. Use Mermaid `flowchart`, not `erDiagram`, not C4.

- **`flowchart`** — boxes for tables/components, edges for relationships. Cardinality as edge labels (`"1 → ∞, cascades"`). Non-FK invariants as dashed edges (`-.->|"optional, not FK"|`) with prose labels. This is the only diagram type flexible enough to express both FK edges and the load-bearing non-FK invariants (e.g. `message_swipes.snapshot_id → game_state_snapshots.id`, which is deliberately not a SQL FK).
- **Not `erDiagram`** — its strict cardinality markers (`||--o{`) handle FK edges but not non-FK invariants or independent tables cleanly, and its `{ column TYPE }` body blocks tempt re-adding column duplication.
- **Not C4** — C4 primitives describe software components with tech stacks, not data tables. Using `Component()` for a table is a category error.

**One diagram per cluster**, not everything in one. A diagram covering 11 tables in one frame is too dense to read — split by logical cluster (game state, world catalogue, standalone) and carry cross-cluster relationships as prose below the diagrams. Each cluster diagram should fit on one screen.

### Mode-specific notes

#### Reference

Austere, neutral, authoritative — like a map. Describes *what is*, not *how to use it* and not *why it's that way*. The last two belong in How-to and Explanation respectively. Canonical Diátaxis imperative: **describe and only describe; when tempted to explain, link to an Explanation doc** (per diataxis.fr/reference/). Do not absorb rationale into Reference prose; do not point at ADRs as the home for current-behaviour rationale — ADRs are frozen decision records, possibly Superseded, not living descriptions of how the system works today.

If a Reference doc's content starts answering "why", split the why into an Explanation doc and cross-reference (the `game_flow.md` ↔ `two-state-channels.md`, `agent_system.md` ↔ `agent_system_design.md`, and `storage.md` ↔ `storage_design.md` splits are the canonical patterns in this tree).

#### Explanation

Discursive, understanding-oriented. Answers "why?" — design rationale, tradeoffs, system connections. Can take perspectives and be read away from the product. Per diataxis.fr/explanation/: admit opinion and perspective; consider alternatives; provide background and context. Do not bloat Explanation into mechanism docs — cross-reference mechanism docs rather than re-listing their content.

Explanation carries **current understanding**. ADRs capture decisions as they were made (frozen at decision time, possibly Superseded, possibly deleted per the ADR README). The two overlap on tradeoffs but differ in frame: an Explanation doc can be revised as the system evolves; an ADR cannot. Cite ADRs in Document References as historical decision records; do not duplicate their Consequences prose verbatim — re-frame as current understanding.

The architecture overview (`diataxis/explanation/architecture.md`) is Explanation mode — it answers "why is it structured this way" by showing the structure and the quality tradeoffs.

##### Register

Explanation prose states design choices, reasons, and tradeoffs directly. Discursive, not performed. The canonical Diátaxis site's example sentences are the register target: "_The reason for x is because historically, y…_", "_W is better than z, because…_", "_An x in system y is analogous to a w in system z._"

Forbidden registers:

- **Narrated reader experience** — "A reader opening X is confronted with…", "A reader will reasonably ask…", "The fair question is…". State the design; do not narrate someone encountering it.
- **Dramatic contrast framing** — "What this design is not" sections, "This is not a plugin system", "The model declines, deliberately, to…". Name alternatives where they are load-bearing context, not as strawman dramatic contrasts.
- **Editorializing perspective** — "the perspective the design takes is…", "whether that tradeoff is worth it is outside the scope of this document", "that cost is invisible". State the tradeoff and the comparison point; let the reader evaluate.
- **Speculative color** — "a hypothetical prose guardian tomorrow". Drop speculative examples that add narrative color without carrying fact.

Discursive means prose that explores design rationale across connected sections. It does not mean keynote address. The opinion-and-perspective Diátaxis permits is stated directly ("the design chose X over Y because Z"), not performed as a rhetorical event.

##### Explanation unfolds; it does not justify

Explanation docs in this tree explain **what is happening** — they unfold and illuminate their subject. They do not justify the design to a skeptical reviewer; that framing belongs in ADRs.

The single most common framing failure: an Explanation doc whose sections read as an apologia — "Why an abstraction at all", "Why two and not one", "Why trait objects". Each of those is a defense of a choice against an imagined alternative. The apologia answers "should the design have been different?" — that is the ADRs' question. Explanation answers "what is going on here?".

The test: a section title phrased as "Why X?" or "Why X instead of Y?" is a justification title; rephrase it as what the section explains ("How X works", "What X does", "The moving parts of X") and rewrite the body to unfold the subject rather than defend it. Comparisons to alternatives still appear where load-bearing — they become "X differs from Y on..." statements inside the unfolding, not the section's reason for existing.

#### Tutorial

None exist yet. Tutorials are learning-by-doing walkthroughs that build a mental model of the engine, not goal-achievement guides.

#### How-to

Only `diataxis/how-to/debugging.md`. Goal-oriented directions for already-competent users, written from the user's goal, not from the machinery.

### Diagrams

Mermaid only. The toolchain is already in place; do not introduce a new diagram tool.

- **C4 directives** (`C4Context`, `C4Container`, `C4Component`, `C4Deployment`) — use the `UpdateLayoutConfig($c4ShapeInRow="4", $c4BoundaryInRow="2")` directive for readability.
- **`flowchart`** — for relationships diagrams (see above) and for runtime/process diagrams (phase sequences, retry flows). Existing `game_flow.md` and `action_pipeline.md` use this style.
- **`erDiagram`** — avoid; see "Relationships diagrams" above.
- **`sequenceDiagram`** / **`stateDiagram-v2`** — available if a doc genuinely needs them.

One diagram per cluster, not one diagram for everything. Keep each diagram small enough to read at a glance.

### No code-indexer docs

Docs should not be code indexers. The code is self-documenting; the docs are a layer on top, existing because they are more concise and easier to curate than AI-generated code comments. Docs should not explain *how the code works* by exhaustively referencing modules, classes, methods, and types.

**Carve-out**: schema column tables, struct field lists, function signatures, migration version numbers, and constants are code-indexing and are **not** restated in Reference docs — see "Reference defers to source" above. What Reference docs *do* carry (purpose, invariants, relationships, cross-references) is not code-indexing.

**Seam identifiers vs. mechanics leaks.** Not every code reference is code-indexing. The test: *would a reader grep for the name to find the contract the prose is describing?* If yes, the name is a **seam identifier** — keep it. If no, it's a **mechanics leak** — drop it.

- **Keep** (seam identifiers): type names, enum variants, and method references that name the contract the prose is describing — `LlmCallRecorder::complete()`, `assembler.assemble()`, `QuantifierAgent`, `PhaseError`, `state.scene.npcs_in_area`, `AppSettings.response_length`, `<ConversationHistory>`.
- **Drop** (mechanics leaks): impl-detail references a reader wouldn't navigate to — bare free-function names that label mechanics (`run_migrations()`, `execute_freeaction_impl`, `spawn_pipeline_task`), struct field dumps (a bulleted list of every field on a struct), variant payload type syntax (`Variant(String)` → keep `Variant`), Rust-type leaks in prose (`Option<String>`), code syntax (`chars.div_ceil(4)`, `max_attempts = 2` assignment), constructor forms (`AppSettings::default()` — rephrase to "the engine's default settings").

XML/domain markups (e.g. `<ConversationHistory>`, `<PlayerInput>`) are domain tags, not code references — they don't trigger the code-indexer test.

### No negative explaining

Don't describe a thing by what it isn't, and don't editorialize about absences in body prose. State the positive; if the positive is already stated elsewhere (in another section, a diagram, or an Out-of-scope list), the negative version is tautology — drop it.

Two forms, both banned:

- **Tautological negative definition** — e.g. "`Message` carries no `text`, `location_header`, `event_header`, or `snapshot_id` field" when the `Swipe` bullet just said `Swipe` holds those fields. The reader learns the same fact twice and the negative is the weaker copy. State what a thing *is*; the complement's own description carries the rest. A dedicated section that exists only to restate an invariant the Overview already established fails the compass test ("What problem does this solve for the reader?") — drop the section, keep the one-line positive where the reader first encounters the thing.
- **Defensive scope disclaiming** — e.g. "X is not an external system", "X is not in scope", "State mutation via LLM function calling is out of scope for this reference". Diagrams and Out-of-scope lists are the source of truth for what's external; if something isn't in the diagram, it isn't in scope. Don't keep the disclaimed thing alive in the reader's mind with parallel negation. Inspirations belong in the explanation doc for the thing they inspired, not in negative asides on unrelated docs.

If the negative is genuinely load-bearing — a constraint the reader must know — state it as a **positive constraint**, not a disclaimer. "The LLM cannot call back into the engine" becomes "state mutation is the engine's job, run through the action pipeline after the LLM has spoken." The reader gets the same fact as an assertion about how the system behaves, not as an apology about what it doesn't do.

### What are ADRs for

Up until now we've been using ADRs as a mixture of high-level design, explanation, and reference documentation. This type of multi-purpose doc does not fit in the diátaxis framework. The plan is to migrate much of the ADR content into diátaxis explanation/reference docs. Until this happens the new diátaxis/reference docs will have overlapping information with the ADRs.