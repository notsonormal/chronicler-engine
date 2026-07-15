---
title: Pilot Notes — docs-diataxis/ writing pattern
---

# Pilot Notes

Findings from writing the three pilot docs that establish the writing pattern for `chronicler_engine/docs-diataxis/`. Short on purpose — this file graduates the open questions on the wayfinder map.

## 1. Mode declaration per doc

| Pilot doc                                      | Mode         | Why it fits                                                                                                                                                                                |
|------------------------------------------------|--------------|--------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------|
| `reference/data_layer.md`                      | Reference    | Pure schema description. Column tables, foreign-key relationships, index list. Reader problem: "what tables exist and what columns do they have?" — look-up.                                |
| `reference/game_flow.md`                       | Reference    | Phase sequence, status-phase table, retry flowchart, error contract. Reader problem: "when the engine is in state X, what happens next?" — look-up. The "why" content is split out.       |
| `explanation/two-state-channels.md`            | Explanation  | Discursive treatment of the tradeoff the dual-channel design encodes. Reader problem: "why are there two generation-state signals and what is each for?" — understand.                       |
| `explanation/architecture/overview.md`         | Explanation  | System context, building blocks, deployment topology, quality guarantees. Reader problem: "how is the engine structured and what does it promise?" — understand.                            |

None of the pilots emerged as Tutorial or How-to. See §7.

## 2. Front-matter decision + reasoning

**Verdict: keep YAML front-matter, but only `diataxis:` + `title:` (and `arc52:` on the architecture doc).**

Findings while writing:

- `diataxis:` is **load-bearing**. The folder name `reference/` vs `explanation/` already signals the mode by path, but a reader scanning a list of files (search results, an index, a flat export) needs the mode without parsing the path. The cost is one line; the benefit is grep-ability across the whole tree.
- `title:` is **load-bearing**. The H1 may diverge from the title the file is referred to by in conversation (e.g. the architecture doc's H1 is "Architecture Overview" but cross-references want the bare title). Mirroring the H1 in `title:` keeps both stable and lets a downstream index use the front-matter value without parsing markdown.
- `arc52:` is **load-bearing on the architecture doc only**. It tells a reader which selective arc42 subset a given overview covers. Not useful on other docs — they are not arc52-shaped.

Front-matter does **not** duplicate what the content signals, because the content signals by *section structure* (a Reference doc reads like a schema catalog; an Explanation doc reads as a discursive discussion) while the front-matter signals by *machine-readable key*. They answer different questions. If we later add a `validate_docs.py` check that asserts the declared mode matches an automated content check, the front-matter becomes the canonical answer and the content check is a backstop — useful layering, not redundancy.

## 3. arc52 section scoping

`§3 / §5 / §7 / §10` covered the architecture story cleanly, **with one note about §7**.

- **§3 Context & Scope** — clean. The C4Context diagram plus a short "out of scope" list answers "what is this and what touches it?" without leaking into building-block detail.
- **§5 Building Block View** — clean. L2 + L3 are both needed and earn their place; see §4. The 8-tier tier map from `architecture/system.md` was re-cast as the L3 Component diagram rather than re-listed; readers who need the tier map can follow the cross-reference to `architecture/system.md`.
- **§7 Deployment View** — **scope corrected during pilot**. The ticket instruction said to pull §7 from `docs/docker.md`, but `docs/docker.md` is top-level workspace documentation, explicitly out of scope per the map. The 03 grilling's §7 example summary (engine containerized, HTTPS on 443) described the workspace topology, not the engine's own contract. The pilot's §7 instead describes the **engine's deployment contract** — process boundary, DB file, FS directories, outbound HTTPS — and explicitly defers workspace-level topology (Caddy, `no-internet` network, sibling AI-stack containers) as out of scope. No discrepancy remains open; the scope error was in the ticket, not the resolution.
- **§10 Quality Requirements** — clean and novel content. No section wanted to become a fifth arc42 heading; the four-section shape carried the architecture story end-to-end.

No section wanted to burst into its own file. Each one is a self-contained "answer to one question" within the overview.

## 4. C4 level fit

**Verdict: L1 + L2 are required; L3 is required for the Component view of the Application Core, but redundant in many other deployments.**

The test was: does L3 earn its place, or is it redundant with L2?

- **L1 (Context)** is required. There is no other diagram that captures "the engine is one system among several external systems." This is the orienting diagram for a new reader.
- **L2 (Container)** is required. The four containers (HTTP server, application core, SQLite, outbound LLM clients) are a stable, durable vocabulary — every ADR and every system doc implicitly refers to them. Without L2 the rest of the architecture story is ungrounded.
- **L3 (Component)** earned its place **inside the Application Core**, where the domain/application/adapters/bootstrap decomposition is load-bearing (it's the dependency-invariant story). Outside the Application Core — for the SQLite container or the HTTP server — L3 would have been redundant: those containers are atomic from the engine's perspective.

So the L3 diagram in the pilot zooms into one container (Application Core). This is the standard C4 usage pattern and the right call. L3 is not "needed at the architecture-overview level" in general; it is needed because the engine has an internal hexagonal layout that the reader needs to see.

If the engine ever grew a second complex container (e.g. a separate scheduler service), that container would warrant its own L3 diagram. Today, one L3 for the application core is enough.

## 5. Naming and organization

The top-level shape that felt natural: `reference/` + `explanation/`, with no `tutorials/` or `how-to/` until the pilot surfaced content for them (none did — see §7).

Within `explanation/`, `architecture/` as a subfolder for the arc42-shaped overview felt right because:

- Architecture is a category, not a single document. Future explanation docs on "Why Hexagonal Architecture?" or "Why the LLM Provider Port?" would naturally sit alongside `architecture/overview.md`.
- The arc42 shape is specific enough that mixing architecture docs with discursive explanation docs at the same level would dilute the signal. `explanation/architecture/` keeps the arc42-shaped docs findable.

Within `reference/`, no subfolders emerged. Each reference doc is small enough (the data-layer schema is ~5 KB rendered) to live flat. If a future reference doc grows large, it can split by topic without disturbing siblings.

What felt forced: nothing structural. The only naming friction was inside the architecture doc — "Building Block View" is arc42 vocabulary that a reader unfamiliar with arc42 will not parse on first read. Acceptable because the arc52 section is identified explicitly in the front-matter and the H2 heading is self-describing in context.

## 6. What old-docs content got dropped

Audit of the source docs and the new pilots:

- **From `system/game_flow.md`:** the entire "Two State Channels" aside (~150 words) was moved to `explanation/two-state-channels.md`. It did not fit Reference — it answered "why two signals?" not "what is the phase sequence?" — so it was a clean drop from the reference file. Nothing was force-fit.
- **From `architecture/system.md`:** the tier map (8-tier breakdown) is not re-listed in the architecture overview. The L3 Component diagram covers the high-level structure (domain / application / adapters / bootstrap), and a reader who wants the exact module path for a tier can follow the cross-reference to `architecture/system.md`. The tier map is reference content; the L3 component view is explanation content; they coexist.
- **From `architecture/rust_technical.md`:** the cross-cutting Rust idioms (sync services, `spawn_blocking`, `Arc<RwLock<AppSettings>>`, poison recovery, `Arc<AtomicBool>`, `CancellationToken`) are **not** re-listed in §10. They are mechanism docs; §10 is a quality-attribute list. The cross-reference at the bottom of the architecture overview and inside §10's source-of-truth column points readers to `architecture/rust_technical.md` for the mechanism. This is a deliberate separation: §10 promises the guarantee; `rust_technical.md` shows how it is delivered.
- **From `docs/docker.md`:** **not pulled into the architecture overview.** `docs/docker.md` is top-level workspace documentation (out of scope per the map), not a Chronicler Engine doc. The pilot's §7 Deployment View describes the engine's own deployment contract (process, DB file, FS directories, outbound calls) and explicitly defers workspace-level topology as out of scope. This corrects a scope error in the ticket, which had said to pull §7 from `docs/docker.md`.
- **From `reference/data_layer.md`:** the per-column tables (one Markdown table per SQL table, listing every column with type and notes) were **dropped**. The column-level DDL is already the authoritative source in `src/adapters/driven/storage/db.rs`; restating it in markdown was drift-prone duplication — the existing doc had 5 tables documented while `db.rs` creates 11, so the drift was already real. The pilot's `data_layer.md` instead carries prose per table (what it's for, the load-bearing invariants) plus a Mermaid `flowchart` of all 11 tables' relationships. **Convention**: Reference docs defer column/type details to source; a relationships diagram is permitted where the aggregate structure isn't obvious from the DDL. This carve-out is recorded in the map's Notes.
- **Nothing was force-fit.** If the source content did not fit the new frame, it was either dropped (no replacement yet) or split (the Two State Channels aside) or cross-referenced (the tier map, the Rust idioms, the broader Docker topology).

## 7. Tutorial / How-to candidates

The pilot **did not surface Tutorial or How-to content that needs to be written now**. The audit predicted this might happen; it did not. Specifically:

- **Tutorial candidates:** none surfaced. A "getting started" walkthrough is plausible (audit recommendation 7.3.2), but it would be net-new content not derivable from the existing docs, and writing it is out of scope for this pilot.
- **How-to candidates:** none surfaced from the pilots. Existing `diagnostics/DEBUGGING.md` is the only current How-to in the workspace; the audit recommendation 7.3.1 (flag mixed-purpose files for mode-split) was the closest actionable signal, and the pilots acted on it (splitting `game_flow.md` and reframing the architecture docs).

If Tutorials are to be written at all, they should be charted as separate tickets, not implied by the pilot.

## 8. Framings that didn't fit cleanly

Three honest frictions:

1. **The §7 scope correction.** The ticket (and the 03 grilling's example summary) described §7 as workspace Docker topology pulled from `docs/docker.md`. That was a scope error: `docs/docker.md` is top-level workspace documentation, explicitly listed out of scope in the map. The pilot's §7 was rewritten to describe **only the engine's deployment contract** — the process boundary and what it binds, reads, and calls out to — and defers workspace-level topology (Caddy, `no-internet` Docker network, sibling AI-stack containers) as out of scope. The architecture doc no longer links to `docs/docker.md`. This corrects the discrepancy cleanly; no current-vs-target split is needed.

2. **The `is_generating` cache versus the per-game registry.** ADR-030 distinguishes three layers — persisted `GenerationStatus`, per-game registry, atomic projection — but the pilot's `two-state-channels.md` describes only two channels (persisted + atomic), with the per-game registry implicit in "the registry claim/release path". This was a deliberate simplification: the two-channel frame is the architectural tradeoff the engine makes; the registry is the mechanism the atomic is a projection of. A reader who needs the full three-layer story can follow the ADR-030 cross-reference. If this confuses readers, the explanation doc can grow a third paragraph distinguishing the registry from the atomic, but the pilot kept it short on the principle that explanation should not bloat into mechanism docs.

3. **The LLM HTTP timeout is reported as 180s in §10.** INV-004 cites the 180s figure. The figure is also restated in ADR-010 ("the LLM transport enforces only a 180-second HTTP timeout"). No friction here, but it is worth noting that §10 cites a number that lives in two places (guardrails INV-004 and ADR-010 §Cooperative cancellation only). If the figure ever changes, both citations need updating. A future ticket could centralize timeout/limit values in one config-style reference; for the pilot, the dual citation is acceptable.

4. **Negative-disclaimer paragraphs about what the diagrams don't show.** The first draft of the §3 architecture doc carried paragraphs saying "SillyTavern is not a runtime system" and "Harper is not an external system". This was defensive framing — it kept the very things it was disclaiming in the reader's mind, and read like apologizing for ticket 03's summary errors (which the reader never saw). **Convention**: the C4 diagrams and the Out-of-scope list are the source of truth for what's external. If something isn't in the diagram, it isn't in scope — no parallel negative paragraphs. Inspirations belong in the explanation doc for the thing they inspired (e.g. SillyTavern's prompt-system inspiration lives in the prompt-system doc, not the architecture overview). Harper appears only in the places where its in-process nature is a positive statement of fact (§5 Adapters description mentions `harper_core`; §7 deployment contract has an "In-process text check" bullet); it is not editorialized about.

5. **Three slop patterns to avoid in bulk-writing** (found in pilot audit, 2026-07-15):
   - **Negative-disclaimer prose** — paragraphs insisting that X is "not an external system" or "not in scope". Diagrams and Out-of-scope lists are the source of truth; if something isn't in the diagram, it isn't in scope. Don't editorialize about absences.
   - **Tables-of-contents-as-prose** — a table that maps the doc's own sections to "Question answered" / "C4 level" / "Source of truth". The sections are coming right up; the reader doesn't need a preview. H2 headings + their prose are the TOC.
   - **Dual citations / vague guarantees** — citing both INV-004 and ADR-010 for one value, or "O(1)" when the guarantee is "no storage round-trip per poll". Cite the single authoritative source; describe the guarantee concretely.
   These apply to all four Diátaxis modes, not just architecture docs.

## 9. `game_flow.md` split verdict

**The Two State Channels split held.** The reference doc reads as a clean phase/phase-table/retry/error-model contract; the explanation doc reads as a coherent essay on the dual-channel tradeoff. No content from the original aside had to be force-fit into either file, and no new content had to be invented to bridge them.

What the split revealed:

- The reference doc's "Error Model" section has a one-line cross-reference back to `explanation/two-state-channels.md` at the end of "Granular Status Phases". Without that cross-reference, a reader of the reference doc would be left wondering why there are two signals. The cross-reference is essential — it is the seam between the two docs.
- The reference doc's "Stale-Generating recovery" sentence still names the atomic flag and the persisted status by their concrete shapes. This is borderline — it is factual description of what happens, not "why". Acceptable because it describes observable behavior of the engine; the "why" lives in the explanation doc.
- The retry-flow diagram and status-phase table are pure Reference and stayed where they were.

If a future ticket introduces a third channel (e.g. for cross-process coordination, mentioned in `two-state-channels.md`'s "What this design does not address"), the explanation doc will need a small extension but the reference doc will not. The split ages well.

## Summary

The pattern works. Front-matter is justified; the architecture doc's arc52 shape carried the story; L3 was earned by the application-core decomposition but not needed elsewhere; the Two State Channels split held; no Tutorial or How-to content surfaced from the pilots; the deployment story needs a human decision.

Outstanding for human review: none. The §7 scope error was corrected during the pilot; the two minor judgment calls (two-channels vs three-layers in `two-state-channels.md`; dual 180s timeout citation) are noted above and do not block charting the bulk-writing tickets.
