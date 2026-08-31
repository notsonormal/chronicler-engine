# Plan: Replay blob storage shape investigation

**Date:** 2026-08-31
**Status:** Draft — investigation only, no code changes
**Goal:** Decide whether steering-replay data should stay a JSON blob in `message_swipes.replay`, or move to structured columns. Record the decision with evidence.

## Origin

During Q&A triage of the CodeRabbit PR #2 review (recorded in `tmp/pr2_review_issues.md`), the question surfaced: why does the engine keep a dedicated column for "data we might need if we retry" (`message_swipes.replay TEXT`, migration v15, `plumbing.rs:277-278`), instead of modelling guide/impersonate steering as separate fields on the message or swipe?

The immediate defect found in the same review (guide-only retry rejection) is fixed separately under the M3 Option A decision. This plan deliberately does **not** revisit that fix; it examines the storage shape underneath it.

## Current state (verified 2026-08-31)

- `message_swipes.replay` is a nullable `TEXT` column holding a serde-serialized `GenerationReplay` (`swipes.rs:21-29`, read back at `swipes.rs:133`).
- `GenerationReplay` has four fields: `guide: Option<String>`, `impersonate: bool`, `impersonate_direction: Option<String>`, `impersonate_preset_id: Option<String>` (`narration_generation.rs:170-183` writes it).
- The blob is sparse: only swipes from guided or impersonated turns carry one. Normal retries on plain narrations leave it `NULL`.
- The blob is always read whole (`parse_swipe_replay`, `swipes.rs:133`); no SQL filters or joins on its fields.
- Corrupt JSON degrades to `None` with a log line, by design ("replay is auxiliary steering metadata, not structural").
- The codebase already stores other enums as JSON text (`message_type_json`), so the blob extends an existing pattern rather than introducing a new one.
- The retry flow consults the blob in two places: `run_from_input` (`core.rs:224-230`) re-applies guide/impersonate on redo, and `push_message` (`game_state.rs:126-143`) inherits the blob onto a retry-appended swipe.

## Investigation questions

### Q1. Query pressure — will anything ever need SQL over replay fields?
Scan the roadmap / specs for features that would query replay fields (e.g. "list all guided turns", "analytics on impersonate use"). If none exist, columns buy queryability nobody will use.

- Search `docs/specs/`, `.scratch/` issue maps, and `docs/plans/` for replay-aware read queries.
- Check whether any report, export, or UI surface would need per-field filtering.

### Q2. Extensibility cost — what do future steering surfaces actually cost under each shape?
Estimate the marginal cost of adding a fifth steering field under (a) the blob vs (b) columns.

- Blob: add field to `GenerationReplay` + serde default; no SQL migration (verified cheap in the M3 discussion, `tmp/pr2_review_issues.md`).
- Columns: `ALTER TABLE` + mapper + insert/load changes + backfill.
- Rate how often steering fields are plausibly added (see ticket backlog for narrator modes, perspective/tense).

### Q3. Scope — should the blob pin the impersonate preset id at all?
The blob currently records `impersonate_preset_id`, pinned at entry time (`action.rs` — "the Swipe records the preset that produced the generation") so a redo re-voices with the original preset even if the user changed the active preset in between. Review feedback (2026-08-31 triage) flags this as possibly overaggressive: arguably a redo should resolve the *current* active preset like any fresh impersonate, and the blob should carry only steering the user explicitly typed (guide text, direction), not a frozen settings snapshot.

- Evaluate the two semantics: "redo = faithful retake of the original generation" (pin) vs "redo = same user intent, current settings" (resolve at redo time).
- Check what swiping implies: the swipe list is presented as alternative takes of one turn — does mixing voices across swipes break that presentation?
- Note the interaction with PR #2 finding N1 (`tmp/pr2_review_issues.md`): if pinning stays, N1's fix (store the resolved id) is required; if pinning is dropped, N1's hole disappears by construction.
- Decision here changes the blob's field set, so it precedes the blob-vs-columns question.

### Q4. Failure-mode tolerance — is "log and discard" acceptable for replay?
Decide whether replay data is truly auxiliary enough to survive silent loss.

- Consequences of a lost blob: retry falls back to current settings (`active_impersonate_preset_id`) or drops the guide. The generation still succeeds; the turn just re-runs un-steered.
- Compare with `message_type`: corruption there is structural and fails the row. Replay explicitly accepts graceful loss.
- **Decision input:** if silent replay loss is tolerable, the blob's weakest property (no DB-level integrity) is unimportant.

### Q5. Schema legibility and debugging cost
Quantify how much harder the blob makes ad-hoc database inspection.

- How often does an operator open `sqlite3` and inspect `message_swipes`? (Evidence: debugging sessions, docs, test helpers.)
- Does the blob hide state during incident triage? (Example: "why did this retry use the wrong preset?" — the blob must be parsed to answer.)
- Compare against the cost of four mostly-NULL columns cluttering `PRAGMA table_info`.

### Q6. Consistency with the structural/payload dividing line
Check whether `replay` fits the codebase's existing split: columns for structural data (`message_type`, `snapshot_id`, headers), blobs for payload data.

- Verify no structural decision (anchor lookup, event detection, rendering) reads a replay field directly from SQL.
- If some structural logic starts needing replay fields, that is a signal to migrate.

### Q7. Recommendation
- **GO** — proceed to a migration plan that splits replay into columns (with a backfill strategy for existing rows).
- **NO-GO** — keep the blob, document the rationale in `docs/diataxis/explanation/storage_design.md` (or the storage reference) so the design decision is discoverable.
- **PARTIAL** — e.g. keep the blob for now but add a `json_extract`-based debug view or a `PRAGMA`-friendly note.

## Out of scope

- The M3 guide-only retry fix — decided separately (Option A), this plan does not reopen it.
- `GenerationReplay` field semantics (what guide/impersonate mean) — settled by the steering-and-guided-generation epic.
- `message_type_json` or other JSON columns — only the replay column is under review.
- `Backend::InMemory` — owned by `in-memory-storage-investigation-plan.md`.

## Deliverables

1. `tmp/replay-blob-findings.md` with Q1–Q6 write-ups and a Q7 recommendation.
2. If GO: a follow-up migration plan in `docs/plans/`.
3. If NO-GO: a short note in the storage docs recording why the blob is retained.

## Verification

- `python build.py` stays green; the investigation commits no code.
- Any temporary inspection queries stay in `tmp/`.
