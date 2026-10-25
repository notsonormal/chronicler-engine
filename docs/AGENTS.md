# Chronicler Engine Documentation

This folder contains all documentation for the Chronicler Engine project.

For general engine principles, workflow, and conventions, see `AGENTS.md`.

Write documentation using Simplified Technical English (STE).

## Documentation Layers

There are three documentation layers:

1. The code and the unit tests (`src/`)
2. The integration tests (`tests/`) and the specs (`docs/specs`)
3. The Diátaxis docs (`docs/diataxis`)

They might not traditionally be considered documentation, but reading the code and the unit tests is the most accurate way to understand how the application works.

The specs describe the behaviour a client observes through HTTP and browser interactions, and are enforced by the integration tests (every spec has an associated integration test).

The code and the specs are the complete record. The Diátaxis docs are selective: they carry what the code, tests and specs cannot.

### Where a fact lives

Put each fact in exactly one layer:

| A fact that is… | Lives in |
|---|---|
| The purpose of a part, its name, or an invariant: an always-true relation between named parts — what owns what, what makes what current, how many of a part exist, where a part shows ("a swipe switch makes the target Swipe active", "role health is engine-wide", "settings is a singleton row") | Diátaxis Reference |
| Why the system is that way | Diátaxis Explanation |
| An event rule at feature level ("when X, then Y"), or a property that every request to one route keeps | A spec scenario, a spec section intro, or a spec's Properties section |
| Fine-grained behaviour (a rule inside one part, such as how a function reads its input), edge cases, values, copy text, icons, markup, implementation steps | The code and its tests |

In `docs/diataxis/`, a fact has one home per subject: the Reference doc states it, and its Explanation doc gives only the reason. A doc on another subject may restate it in one sentence when its reader needs it.

An invariant stays true through the next fix, restyle or copy edit in its area. That is the test for a Reference sentence.

**Direction of edits.** A behaviour change edits its spec and its test. A doc in `docs/diataxis/` changes only when a change adds, removes or renames a part, an invariant or a name that the doc states. A Diátaxis doc that needs an edit for most changes in its area copies the code. Rewrite it.

**Document References.** List only the paired Reference or Explanation doc of the same subject, and any doc that the body hands a point to. The Folder Structure index below lists every other doc, and each test names its spec. Cite no spec scenario numbers in a Diátaxis doc.

## Folder Structure

<!-- AUTO-INDEX START -->
*Index last generated: 2026-10-25 13:11 UTC*

### `docs/diataxis/explanation/`

- [Agent System Design](./diataxis/explanation/agent_system_design.md)
- [Architecture](./diataxis/explanation/architecture.md)
- [Dashboard Design](./diataxis/explanation/dashboard_design.md)
- [Diataxis](./diataxis/explanation/diataxis.md)
- [Prompt System Design](./diataxis/explanation/prompt_system_design.md)
- [Rust Idioms](./diataxis/explanation/rust_idioms.md)
- [Storage Design](./diataxis/explanation/storage_design.md)

### `docs/diataxis/reference/`

- [Architecture System](./diataxis/reference/architecture_system.md)
- [Game Flow](./diataxis/reference/game_flow.md)
- [Startup](./diataxis/reference/startup.md)
- [Storage](./diataxis/reference/storage.md)

### `docs/diataxis/reference/coding_standards/`

- [Guardrails](./diataxis/reference/coding_standards/guardrails.md)
- [Integration Test Standards](./diataxis/reference/coding_standards/integration_test_standards.md)

### `docs/diataxis/reference/frontend/`

- [Dashboard](./diataxis/reference/frontend/dashboard.md)
- [Http Routes](./diataxis/reference/frontend/http_routes.md)

### `docs/diataxis/reference/narrative/`

- [Agent System](./diataxis/reference/narrative/agent_system.md)
- [Narration System](./diataxis/reference/narrative/narration_system.md)
- [Narrator Mode](./diataxis/reference/narrative/narrator_mode.md)
- [Prompt System](./diataxis/reference/narrative/prompt_system.md)

<!-- AUTO-INDEX END -->

## Writing Conventions

Docs under `docs/diataxis/` are written deliberately in their Diátaxis mode, not lifted from elsewhere. The compass test ("What problem does this solve for the reader?") governs what stays, what splits, what cross-references, and what drops.

`scripts/validate_docs.py` checks the front-matter, the mode vocabulary and the links. The `chronicler-docs-hygiene` skill checks that the content of a doc matches its declared mode. When in doubt about *framing*, invoke `/grilling` and `/domain-modeling`.

### Diátaxis — doc modes

Every doc in `docs/diataxis/` is in exactly one of four Diátaxis modes. The mode is declared in front-matter.

| Mode | Reader problem | Orientation | Chronicler examples |
|---|---|---|---|
| **Tutorial** | Learn from zero | Learning-oriented (study) | None yet |
| **How-to** | Achieve a goal | Goal-oriented (work) | None yet |
| **Reference** | Look up a fact | Information-oriented (work) | `diataxis/reference/storage.md`, `diataxis/reference/game_flow.md` |
| **Explanation** | Understand why | Understanding-oriented (study) | `diataxis/explanation/architecture.md`, `diataxis/explanation/storage_design.md` |

Drop content that solves no reader problem in this table. Split content that mixes modes. `diataxis/explanation/diataxis.md` holds the two-axis compass and the adjacent-mode boundaries.

### Front-matter

Every doc under `docs/diataxis/` carries YAML front-matter at the top of the file. Required keys:

- `diataxis: <mode>` — one of `tutorial`, `how-to`, `reference`, `explanation`.
- `title: <H1 mirror>` — the doc's title, mirroring the H1.

### Subfolder shape

- `reference/` — Reference docs.
- `explanation/` — Explanation docs.
- `tutorials/` and `how-to/` — create these directories only when a doc earns the mode.

`external_applications/`, `plans/`, and `specs/` live under `docs/`, alongside `diataxis/`. They are not part of the diátaxis tree.

### Cite symbols, not line numbers

Cite a symbol (`Storage::get_settings`), not a `path:line`. The next edit breaks a line number. `scripts/validate_docs.py` enforces this (`LINE_CITATION`).

### Diagrams

Use Mermaid only. Draw one diagram per cluster, not one diagram for everything. Each diagram fits on one screen.

- **`flowchart`** — for relationships diagrams and for runtime diagrams (phase sequences, retry flows).
- **`erDiagram`** — do not use. Its cardinality markers cannot show a non-FK invariant or an independent table, and its `{ column TYPE }` blocks invite column lists.
- **`sequenceDiagram`** / **`stateDiagram-v2`** — available when a doc needs them.

A relationships diagram earns its place where the aggregate structure is not clear from the source. Boxes are tables or components, and edges are relationships. Put the cardinality in the edge label (`"1 → ∞, cascades"`). Draw a non-FK invariant as a dashed edge with a prose label (`-.->|"optional, not FK"|`), for example `games.world_key → worlds.key`, which is deliberately not a SQL FK. Split the diagrams by logical cluster (game state, world catalogue, standalone), and state the cross-cluster relationships in prose below them.

### Mode-specific notes

#### Reference

Describe and only describe, like a map. When a Reference doc starts to answer "why", move the why into its Explanation doc and link the two (`storage.md` ↔ `storage_design.md`).

#### Explanation

State the design, its reason and its tradeoff directly: "the design chose X over Y because Z". Name an alternative only where the reader needs it as context.

- Name a section by what it explains ("How X works"), not as a defence ("Why X?", "Why X instead of Y?").
- Do not narrate a reader ("A reader will ask…"), stage a contrast ("What this design is not"), editorialize ("that cost is invisible"), or speculate ("a hypothetical guardian tomorrow").
- Do not re-list mechanism. Link the Reference doc instead.

### No code-indexer docs

The code documents itself. A doc does not explain how the code works by listing its modules, types and methods.

**Values stay in the code.** Column lists, struct field lists, function signatures, migration version numbers, constants and caps stay in the source file, one hop away. Purpose, names, invariants and relationships ("Where a fact lives" above) are not code-indexing.

**Seam identifiers vs. mechanics leaks.** Keep a name when a reader would grep for it to find the contract that the prose describes (`GameWriter`, `LlmCallRecorder::complete()`, `<ConversationHistory>`). Drop a name that only labels mechanics: a free-function name, a field dump, variant payload syntax (`Variant(String)`), a Rust type in prose (`Option<String>`), or code syntax.

Plans (`docs/plans` and `.scratch`) are exempt.

### No negative explaining

State what a thing is. Do not describe it by what it is not, and do not disclaim scope in body prose: a diagram or an Out-of-scope list says what is external.

When the reader must know a negative constraint, state it as a positive one. "The LLM cannot call back into the engine" becomes "state mutation is the engine's job, run by the Job after the LLM has spoken."
