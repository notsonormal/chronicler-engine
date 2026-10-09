# Close the four-review findings on `dashboard-ui-issues-2`

Type: task (AFK)
Status: resolved
Blocked by: 64, 68, 69, 70

## Question

The four reviews of `main...HEAD` at `b291dd82` (code-review, thermo-nuclear,
antipattern, test-police) found one regression, one design question and a set of
structural and test-hygiene defects. Fix every verified finding.

## Context

- Findings: `tmp/review/b291dd82-ref/findings/{code-review,thermo-nuclear,antipattern,test-police}.md`.
- Plan: `docs/plans/close-the-four-review-findings-on-dashboard-ui-issues-2.md`.
- User decisions: role health stays engine-wide and the UI labels it so; the
  three failure paths that ticket 64 deferred are fixed now; a generation
  failure persists a typed kind plus the raw text; no compatibility shim for old
  snapshots.

## Done when

- Every finding in the plan is fixed or recorded as excluded with a reason.
- `python build.py` is green. Commit after user approval.

## Answer

Resolved, uncommitted on `dashboard-ui-issues-2`.

- **Failure surfaces.** A failed Text Check save answers 500 into an inline slot,
  so the card stays (settings 20.22; 20.23 and 20.24 cover the panel and the
  edit-form load failures). Preset duplicate, preset activate and world create
  answer non-2xx; `error_response` is gone (prompt presets 21.22, 21.24, 21.25;
  worlds 25.9).
- **Role health.** One domain `Role`; the label and failure effect live in one
  HTTP mapping; the stub server renders the production header; the header and
  the Settings marker say "engine-wide role health".
- **Generation failures.** `GenerationStatus::Error` carries a
  `GenerationFailure` with a kind classified from the typed `EngineError` and
  the raw text; the HTTP layer picks the sentence from the kind. The substring
  matcher is gone. A missing preset now shows the generic sentence.
- **State ownership.** The dock and `/debug` read the live generation registry
  (dashboard 39.3); `NewGame` derives the display name; one
  `EngineError → ApplicationError` refusal conversion; the boot heal reuses the
  pipeline; one `llm_message` fixture.
- **Options presets.** `AppSettings` owns every active-preset slot; one card
  builder and a branch-free card template replace the parallel Options path.
- **Dead code and hygiene.** Unused CSS, `data-subtab` attributes, world-form
  fields and `get_generating_status` removed; the discarded browser assertion,
  the bare sleep, the parity gap and the hidden swipe query error fixed; the
  connection-edit failure has its own scenario 16.35; the guardrail registry
  scanner uses `syn` visitors.
- **File size.** `tests/browser/stub/dashboard.rs` and `story_log.rs` split into
  modules; the shell's inline script moved to `assets/dashboard.js` and then
  into five per-surface scripts (`assets/dashboard-errors.js`,
  `dashboard-action-area.js`, `dashboard-story-log.js`, `dashboard-slash-menu.js`,
  `dashboard-panels.js`); `scripts/tests/test_build_cli.py` split into three
  modules.
- **Second pass over the leftovers.** The preset-order test asserts the exact
  descending `updated_at` order its name promises (SQLite, explicit timestamps,
  no sleeps) and the in-memory half pins insertion order instead, because
  `PromptPreset` carries no timestamps — the panel's card order therefore
  differs between the two backends, left for a ticket. `set_mode_and_auto_check`
  assigns once through one `normalised_auto_check`. A missing active preset is
  a typed `EngineError::PresetNotFound` classified as
  `GenerationFailureKind::PresetMissing` and clamped to its own line (failure
  display 38.5). The Options slot folds into one validation loop over
  `PresetSelection::slots()` with `Game`/`PresetType` slot accessors. The
  per-agent LLM message query is deleted: role health folds the whole retained
  window, whose size is one `LLM_MESSAGE_RETENTION_LIMIT`.
- **Licences.** `assets/LICENSE-htmx.txt` (htmx 1.9.10, BSD 2-Clause) and
  `assets/LICENSE-idiomorph.txt` (0BSD) carry the upstream licence text.
- **Excluded.** The connection-test surface note, which the reviewer did not
  raise as a finding.
