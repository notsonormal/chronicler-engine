# Chronicler Engine Documentation (`docs-diataxis/`)

This folder is the documentation tree for the Chronicler Engine. Each doc is written deliberately in its Diátaxis mode (see "Writing posture") rather than lifted from elsewhere; the compass test ("What problem does this solve for the reader?") governs what stays, what splits, what cross-references, and what drops.

This file is the **writing-convention layer** of a three-layer enforcement model:

| Layer | Where | What it checks |
|---|---|---|
| Machine | `chronicler_engine/scripts/validate_docs.py` | Front-matter presence, required keys, mode vocabulary, link integrity, ADR references |
| Convention | **this file** | How to write a doc in the new frame |
| Semantic | `chronicler-docs-hygiene` skill | Declared-mode vs. actual-content consistency |

When in doubt about *how* to write something, read this file. When in doubt about *framing*, invoke `/grilling` and `/domain-modeling`. When in doubt about *mode classification*, ask the Diátaxis compass: "What problem does this solve for the reader?"

## The three frameworks

These are the writing perspectives the tree operates under.

### Diátaxis — doc modes

Every doc in this tree is in exactly one of four Diátaxis modes. The mode is declared in front-matter and in a blockquote at the top of the body.

| Mode | Reader problem | Orientation | Chronicler examples |
|---|---|---|---|
| **Tutorial** | Learn from zero | Learning-oriented (study) | None exist yet — see "Tutorials" below |
| **How-to** | Achieve a goal | Goal-oriented (work) | `diagnostics/DEBUGGING.md`; none here yet |
| **Reference** | Look up a fact | Information-oriented (work) | `reference/data_layer.md`, `reference/game_flow.md` |
| **Explanation** | Understand why | Understanding-oriented (study) | `explanation/two-state-channels.md`, `explanation/architecture/overview.md` |

**Compass test** (apply when classifying): "What problem does this solve for the reader?"

- Learn from zero → Tutorial
- Achieve a goal → How-to
- Look up a fact → Reference
- Understand why → Explanation

For the framework's first principles (two-axis compass, adjacent-mode boundaries, workflow), see `explanation/diataxis.md`.

If the content doesn't solve a reader problem under any of these, drop it. If it mixes modes, split it by mode. Canonical splits in this tree: `game_flow.md` ↔ `two-state-channels.md`, `agent_system.md` ↔ `agent_system_design.md`, `message_model.md` ↔ `message_swipe_model.md`.

### arc52 — architecture doc sections

The architecture overview is structured as a single document (`explanation/architecture/overview.md`) with arc52 sections as H2 headings. The full 12-section arc52 template is **not** used — only the selective subset §3, §5, §7, §10. Each section answers a different question:

| Section | Question | C4 level |
|---|---|---|
| §3 Context & Scope | What is this and what touches it? | L1 (System Context) |
| §5 Building Block View | How is it built inside? | L2 (Container) + L3 (Component) |
| §7 Deployment View | How do I run it? | Infrastructure topology (`C4Deployment`) |
| §10 Quality Requirements | What guarantees does it make? | n/a (textual) |

ADRs (in `chronicler_engine/docs/adr/`) satisfy arc52 §9 in their own frame — they are not retrofitted to Diátaxis. Plans (in `chronicler_engine/docs/plans/`) are time-capsule content in their own frame. Both live under `docs/`, not under `docs-diataxis/`.

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
- `tutorials/` and `how-to/` — **do not create these directories until content earns its place.** Empty quadrant dirs are noise; create `tutorials/` and `how-to/` only when a doc earns the mode.

`adr/` and `plans/` live under `docs/`, not under `docs-diataxis/`.

## Writing posture

Docs in this tree are written deliberately in their Diátaxis mode, not lifted from elsewhere. When writing or revising a doc, decide for each piece of content:

- **Keep and re-express** — content fits its Diátaxis mode cleanly; rewrite it deliberately in the frame anyway.
- **Split** — content is mixed-purpose. Split by mode into separate docs; cross-reference between them so neither stands alone.
- **Re-frame** — content's actual Diátaxis mode differs from where it currently lives (e.g. an "Explanation" file that's actually Reference).
- **Cross-reference** — content is mechanism (e.g. a module map); don't re-list it, point at its source.
- **Drop** — content doesn't solve a reader problem under the compass test. Apply the test honestly; if nothing registers, drop it.

Nothing should be force-fit. If a piece of content fights the frame, that's a signal — either split, cross-reference, or drop.

## Reference defers to source

Reference docs do **not** restate values that live in code as the authoritative source. This is an explicit carve-out of the no-code-indexer rule (below). Specifically: no column-level DDL, no struct field lists, no function signatures, no migration version numbers, no constants, no caps — when those are the authoritative source in code.

Restating these in markdown is drift-prone duplication — the source changes, the doc rots.

What Reference docs **do** carry:

- What each thing is *for* (prose, one paragraph per table/component/endpoint).
- The load-bearing invariants the code doesn't say directly ("messages are not stored in the snapshot JSON", "one message history per game", "settings is a singleton row").
- Relationships and aggregate structure (see "Relationships diagrams" below).
- Cross-references to ADRs and mechanism docs.

If a reader needs the exact column list, field type, or function signature, they open the source file — one hop. The doc's value is what the source *doesn't* say.

## Relationships diagrams

Where aggregate structure isn't obvious from reading the source sequentially, a relationships diagram earns its place. Use Mermaid `flowchart`, not `erDiagram`, not C4.

- **`flowchart`** — boxes for tables/components, edges for relationships. Cardinality as edge labels (`"1 → ∞, cascades"`). Non-FK invariants as dashed edges (`-.->|"optional, not FK"|`) with prose labels. This is the only diagram type flexible enough to express both FK edges and the load-bearing non-FK invariants (e.g. `message_swipes.snapshot_id → game_state_snapshots.id`, which is deliberately not a SQL FK).
- **Not `erDiagram`** — its strict cardinality markers (`||--o{`) handle FK edges but not non-FK invariants or independent tables cleanly, and its `{ column TYPE }` body blocks tempt re-adding column duplication.
- **Not C4** — C4 primitives describe software components with tech stacks, not data tables. Using `Component()` for a table is a category error.

**One diagram per cluster**, not everything in one. A diagram covering 11 tables in one frame is too dense to read — split by logical cluster (game state, world catalogue, standalone) and carry cross-cluster relationships as prose below the diagrams. Each cluster diagram should fit on one screen.

## Mode-specific notes

### Reference

Austere, neutral, authoritative — like a map. Describes *what is*, not *how to use it* and not *why it's that way*. The last two belong in How-to and Explanation respectively. Canonical Diátaxis imperative: **describe and only describe; when tempted to explain, link to an Explanation doc** (per diataxis.fr/reference/). Do not absorb rationale into Reference prose; do not point at ADRs as the home for current-behaviour rationale — ADRs are frozen decision records, possibly Superseded, not living descriptions of how the system works today.

If a Reference doc's content starts answering "why", split the why into an Explanation doc and cross-reference (the `game_flow.md` ↔ `two-state-channels.md`, `agent_system.md` ↔ `agent_system_design.md`, and `message_model.md` ↔ `message_swipe_model.md` splits are the canonical patterns in this tree).

### Explanation

Discursive, understanding-oriented. Answers "why?" — design rationale, tradeoffs, system connections. Can take perspectives and be read away from the product. Per diataxis.fr/explanation/: admit opinion and perspective; consider alternatives; provide background and context. Do not bloat Explanation into mechanism docs — cross-reference mechanism docs (`architecture/rust_technical.md`, `architecture/guardrails.md`) rather than re-listing their content.

Explanation carries **current understanding**. ADRs capture decisions as they were made (frozen at decision time, possibly Superseded, possibly deleted per the ADR README). The two overlap on tradeoffs but differ in frame: an Explanation doc can be revised as the system evolves; an ADR cannot. Cite ADRs in Document References as historical decision records; do not duplicate their Consequences prose verbatim — re-frame as current understanding.

The architecture overview (`explanation/architecture/overview.md`) is Explanation mode — it answers "why is it structured this way" by showing the structure and the quality tradeoffs.

#### Register

Explanation prose states design choices, reasons, and tradeoffs directly. Discursive, not performed. The canonical Diátaxis site's example sentences are the register target: "_The reason for x is because historically, y…_", "_W is better than z, because…_", "_An x in system y is analogous to a w in system z._"

Forbidden registers:

- **Narrated reader experience** — "A reader opening X is confronted with…", "A reader will reasonably ask…", "The fair question is…". State the design; do not narrate someone encountering it.
- **Dramatic contrast framing** — "What this design is not" sections, "This is not a plugin system", "The model declines, deliberately, to…". Name alternatives where they are load-bearing context, not as strawman dramatic contrasts.
- **Editorializing perspective** — "the perspective the design takes is…", "whether that tradeoff is worth it is outside the scope of this document", "that cost is invisible". State the tradeoff and the comparison point; let the reader evaluate.
- **Speculative color** — "a hypothetical prose guardian tomorrow". Drop speculative examples that add narrative color without carrying fact.

Discursive means prose that explores design rationale across connected sections. It does not mean keynote address. The opinion-and-perspective Diátaxis permits is stated directly ("the design chose X over Y because Z"), not performed as a rhetorical event.

#### Explanation unfolds; it does not justify

Explanation docs in this tree explain **what is happening** — they unfold and illuminate their subject. They do not justify the design to a skeptical reviewer; that framing belongs in ADRs.

The single most common framing failure: an Explanation doc whose sections read as an apologia — "Why an abstraction at all", "Why two and not one", "Why trait objects". Each of those is a defense of a choice against an imagined alternative. The apologia answers "should the design have been different?" — that is the ADRs' question. Explanation answers "what is going on here?".

The test: a section title phrased as "Why X?" or "Why X instead of Y?" is a justification title; rephrase it as what the section explains ("How X works", "What X does", "The moving parts of X") and rewrite the body to unfold the subject rather than defend it. Comparisons to alternatives still appear where load-bearing — they become "X differs from Y on..." statements inside the unfolding, not the section's reason for existing.

### Tutorial

None exist yet. Tutorials are learning-by-doing walkthroughs that build a mental model of the engine, not goal-achievement guides.

### How-to

Only `diagnostics/DEBUGGING.md`. Goal-oriented directions for already-competent users, written from the user's goal, not from the machinery.

## Diagrams

Mermaid only. The toolchain is already in place; do not introduce a new diagram tool.

- **C4 directives** (`C4Context`, `C4Container`, `C4Component`, `C4Deployment`) — for the architecture overview's §3/§5/§7. Use the `UpdateLayoutConfig($c4ShapeInRow="4", $c4BoundaryInRow="2")` directive for readability.
- **`flowchart`** — for relationships diagrams (see above) and for runtime/process diagrams (phase sequences, retry flows). Existing `game_flow.md` and `action_pipeline.md` use this style.
- **`erDiagram`** — avoid; see "Relationships diagrams" above.
- **`sequenceDiagram`** / **`stateDiagram-v2`** — available if a doc genuinely needs them.

One diagram per cluster, not one diagram for everything. Keep each diagram small enough to read at a glance.

## No code-indexer docs

Docs should not be code indexers. The code is self-documenting; the docs are a layer on top, existing because they are more concise and easier to curate than AI-generated code comments. Docs should not explain *how the code works* by exhaustively referencing modules, classes, methods, and types.

**Carve-out**: schema column tables, struct field lists, function signatures, migration version numbers, and constants are code-indexing and are **not** restated in Reference docs — see "Reference defers to source" above. What Reference docs *do* carry (purpose, invariants, relationships, cross-references) is not code-indexing.

**Seam identifiers vs. mechanics leaks.** Not every code reference is code-indexing. The test: *would a reader grep for the name to find the contract the prose is describing?* If yes, the name is a **seam identifier** — keep it. If no, it's a **mechanics leak** — drop it.

- **Keep** (seam identifiers): type names, enum variants, and method references that name the contract the prose is describing — `LlmCallRecorder::complete()`, `assembler.assemble()`, `QuantifierAgent`, `PhaseError`, `state.scene.npcs_in_area`, `AppSettings.response_length`, `<ConversationHistory>`.
- **Drop** (mechanics leaks): impl-detail references a reader wouldn't navigate to — bare free-function names that label mechanics (`run_migrations()`, `execute_freeaction_impl`, `spawn_pipeline_task`), struct field dumps (a bulleted list of every field on a struct), variant payload type syntax (`Variant(String)` → keep `Variant`), Rust-type leaks in prose (`Option<String>`), code syntax (`chars.div_ceil(4)`, `max_attempts = 2` assignment), constructor forms (`AppSettings::default()` — rephrase to "the engine's default settings").

XML/domain markups (e.g. `<ConversationHistory>`, `<PlayerInput>`) are domain tags, not code references — they don't trigger the code-indexer test.

## No negative explaining

Don't describe a thing by what it isn't, and don't editorialize about absences in body prose. State the positive; if the positive is already stated elsewhere (in another section, a diagram, or an Out-of-scope list), the negative version is tautology — drop it.

Two forms, both banned:

- **Tautological negative definition** — e.g. "`Message` carries no `text`, `location_header`, `event_header`, or `snapshot_id` field" when the `Swipe` bullet just said `Swipe` holds those fields. The reader learns the same fact twice and the negative is the weaker copy. State what a thing *is*; the complement's own description carries the rest. A dedicated section that exists only to restate an invariant the Overview already established fails the compass test ("What problem does this solve for the reader?") — drop the section, keep the one-line positive where the reader first encounters the thing.
- **Defensive scope disclaiming** — e.g. "X is not an external system", "X is not in scope", "State mutation via LLM function calling is out of scope for this reference". Diagrams and Out-of-scope lists are the source of truth for what's external; if something isn't in the diagram, it isn't in scope. Don't keep the disclaimed thing alive in the reader's mind with parallel negation. Inspirations belong in the explanation doc for the thing they inspired, not in negative asides on unrelated docs.

If the negative is genuinely load-bearing — a constraint the reader must know — state it as a **positive constraint**, not a disclaimer. "The LLM cannot call back into the engine" becomes "state mutation is the engine's job, run through the action pipeline after the LLM has spoken." The reader gets the same fact as an assertion about how the system behaves, not as an apology about what it doesn't do.

