# Diátaxis folder cutover

## Summary

Complete the docs-framework cutover by (a) renaming `validate_docs_diataxis.py` to `validate_docs.py` so the new validator is canonical, (b) folding the writing-convention AGENTS.md from `docs/diataxis/AGENTS.md` into the merged `docs/AGENTS.md`, and (c) updating the auto-index generator + live references in scripts/skills so old `docs-diataxis/` paths and old doc paths point at the new `docs/diataxis/` location.

Scope decisions confirmed with the user:
- `adr/`, `external_applications/`, `plans/`, `specs/` are intentionally NOT inside `docs/diataxis/`.
- New validator scans ONLY `docs/diataxis/` (the legacy `docs/` tree is implicit — its remaining contents are ADR/template/CHANGELOG, which the validator already excludes).
- **ADR cross-refs to deleted `docs/system/*`, `docs/reference/*`, `docs/architecture/*`, `docs/diagnostics/*` paths are INTENTIONALLY NOT UPDATED** per user directive. They will remain stale; no validator coverage exists for them (validator scopes to docs/diataxis/ only).
- Live rewrite scope: `scripts/`, `.agents/skills/*`, `build.py`, engine-level `AGENTS.md`. NOT `src/ //! [DOC: ...]` markers; NOT `docs/plans/`, `docs/old-docs/`, `.scratch/`, `docs/CHANGELOG.md` historical entries; NOT ADR docs.

## Key Changes

| What | Where | Notes |
|---|---|---|
| Replace validator file | `chronicler_engine/scripts/validate_docs.py` | Overwrite with renamed/repointed content of `validate_docs_diataxis.py` |
| Delete old validator | `chronicler_engine/scripts/validate_docs_diataxis.py` | No longer needed |
| Rename + update test | `chronicler_engine/scripts/tests/test_validate_docs_diataxis.py` → `test_validate_docs.py` | Module import changes; no path fixtures to update |
| Update HTTP-routes script | `chronicler_engine/scripts/extract_http_routes.py` | OUTPUT_REL constant + docstring |
| Update HTTP-routes test | `chronicler_engine/scripts/tests/test_extract_http_routes.py` | Path fixtures + import |
| Merge AGENTS.md | `chronicler_engine/docs/AGENTS.md` | Absorbs `docs/diataxis/AGENTS.md` content; updates one `docs-diataxis/` mention |
| Delete nested AGENTS.md | `chronicler_engine/docs/diataxis/AGENTS.md` | Content moved up |
| Update auto-indexer | `chronicler_engine/scripts/generate_docs_index.py` | rglob already covers nested diataxis subtree; only docstring + cosmetic updates |
| Rewrite skills + scripts | `.agents/skills/chronicler-docs-hygiene/SKILL.md` (10 mentions), `.agents/skills/diataxis-doc-review/SKILL.md` (1), `.agents/skills/_shared/chronicler-shared.md` (3), `.agents/skills/chronicler-comment-fixer/SKILL.md` (1) | `docs-diataxis/` → `docs/diataxis/`; legacy tree paths → new diataxis paths |
| CHANGELOG entry | `chronicler_engine/docs/CHANGELOG.md` | One-line cutover entry at top |

## Implementation

### Phase 1: Rename + repoint the validator (3 SP)

- [ ] #### Task 1.1: Replace `validate_docs.py` with the diátaxis validator (3 SP)
  - [ ] ##### SubTask 1.1.1: Rework the script (2 SP)
    - Move `chronicler_engine/scripts/validate_docs_diataxis.py` to `chronicler_engine/scripts/validate_docs.py` (overwrite). Inside:
      - Change `diataxis_root = engine_root / "docs-diataxis"` → `diataxis_root = engine_root / "docs" / "diataxis"` (in `main()`).
      - Change `is_diataxis_tree_path` (line ~289) to check `parts[:2] == ["docs", "diataxis"]`. (Function is unused by `scan_file` today; harmless to keep correct, or drop.)
      - Change `scan_file`'s gate `if rel_to_engine.parts[0] != "docs-diataxis"` → match `docs/diataxis/` (parts[0] == "docs", parts[1] == "diataxis"). ADR lookup remains `engine_root / "docs" / "adr"` (ADRs stay where they are).
      - Update `--path` argument check (line ~1093) and the error message accordingly.
      - Update module docstring (line 1): `"Validate markdown docs under chronicler_engine/docs/ and docs-diataxis/."` → `"Validate markdown docs under chronicler_engine/docs/diataxis/."`. Drop the "legacy `docs/` tree…" reference in the docstring + class `STANDARD` comment.
      - Update CLI usage examples (lines 53-56): `python scripts/validate_docs.py --path docs/diataxis/reference/…`.
      - Drop the "and `docs/`" arc from the docstring; the script is now single-tree.
    - File becomes the canonical validator: keeps YAML front-matter checks, mode vocabulary, link/ADR-ref checks, body-reference checks, mode-vs-content heuristic.
  - [ ] ##### SubTask 1.1.2: Rename the test file and update imports (1 SP)
    - Move `chronicler_engine/scripts/tests/test_validate_docs_diataxis.py` to `chronicler_engine/scripts/tests/test_validate_docs.py` (no path fixtures inside the test refer to `docs-diataxis/` — only the module docstring does).
    - Update import line `import validate_docs_diataxis as vd` → `import validate_docs as vd`.
    - Update module docstring line 1 + line 3: "Regression tests for `scripts/validate_docs.py`." — generic.
    - Run `python -m unittest discover scripts/tests -v` from `chronicler_engine/`. Confirm 14 fixtures pass.

### Phase 2: AGENTS.md merge + auto-index + live path rewrites (5 SP)

- [ ] #### Task 2.1: Fold `docs/diataxis/AGENTS.md` into `docs/AGENTS.md` (3 SP)
  - Merge strategy:
    - Keep `docs/AGENTS.md`'s top heading `# Chronicler Engine Documentation` (single source of truth).
    - Keep its preamble (`For general engine principles…` reference to `../AGENTS.md`) and `## Keeping Documentation Clean` section as-is.
    - Append the former `docs/diataxis/AGENTS.md` body as a new top-level section `## Writing Conventions` (Diátaxis modes, front-matter, subfolder shape, mode-specific notes, diagrams, no code-indexer, no negative explaining). The "Three-layer enforcement model" table inside that section already names `validate_docs.py`, so no edit needed on the validator reference after Phase 1 renames it.
    - Update the two `docs-diataxis/` mentions inside the absorbed content:
      - The merged body no longer has the file's own H1 (we stripped `# Chronicler Engine Documentation (docs-diataxis/)`), so no top-of-file correction needed.
      - Line 69: `` `adr/` and `plans/` live under `docs/`, not under `docs-diataxis/`. `` → `` `adr/`, `external_applications/`, `plans/`, and `specs/` live under `docs/`, alongside `diataxis/`. `` (per user's NOTE).
  - Update `docs/CHANGELOG.md` with one-line top entry: `**Diátaxis folder cutover** — moved writing-convention AGENTS.md content into \`docs/AGENTS.md\`, renamed validator, rewrote live references. See plan at \`docs/plans/diataxis-folder-cutover.md\`.`
  - Delete `chronicler_engine/docs/diataxis/AGENTS.md`.
- [ ] #### Task 2.2: Auto-index generator discovers `docs/diataxis/` (1 SP)
  - `scripts/generate_docs_index.py` already does `docs_dir.rglob("*.md")` and groups by `rel.parent`. No code change needed; verify with a dry-run. Cosmetic edits:
    - Update docstring (line 1) to mention the `docs/diataxis/` subtree is included.
    - Re-run auto-indexer: `python chronicler_engine/scripts/generate_docs_index.py` — new sections `### docs/diataxis/explanation/`, `### docs/diataxis/reference/`, `### docs/diataxis/how-to/` get added at the bottom of the auto-index. The legacy `docs/architecture/`, `docs/diagnostics/`, `docs/reference/`, `docs/system/` sections drop out (those folders no longer exist).
  - Confirm diataxis subtree contains no nested `AGENTS.md` (we deleted the only one) — generator's `if md_path.name.lower() == "agents.md": continue` continues to work.
- [ ] #### Task 2.3: Rewrite `docs-diataxis/` → `docs/diataxis/` + legacy tree paths in live scripts + skills (1 SP)
  - `chronicler_engine/scripts/extract_http_routes.py`:
    - Line 56: `OUTPUT_REL = "docs-diataxis/reference/frontend/http_routes.md"` → `OUTPUT_REL = "docs/diataxis/reference/frontend/http_routes.md"`.
    - Lines 1, 17, 355 (docstrings): `docs-diataxis/` → `docs/diataxis/`.
  - `chronicler_engine/scripts/tests/test_extract_http_routes.py`:
    - Lines 13, 15, 320 (docstring), 326, 338 (import), 352, 353, 355 (`fake_engine_root / "docs-diataxis" / …`), 358, 384, 431 — all `docs-diataxis/` → `docs/diataxis/`; `import validate_docs_diataxis as vd` → `import validate_docs as vd`.
  - `.agents/skills/chronicler-docs-hygiene/SKILL.md`: 10 mentions — lines 3, 10, 24, 31, 35, 36, 48, 50, 70, 108. Each `docs-diataxis/` → `docs/diataxis/`.
  - `.agents/skills/diataxis-doc-review/SKILL.md`: line 17 — `docs-diataxis/` → `docs/diataxis/`.
  - `.agents/skills/chronicler-comment-fixer/SKILL.md` line 76: example `//! [DOC: docs/system/startup.md]` (illustrative snippet). Update to `//! [DOC: docs/diataxis/reference/startup.md]`.
  - `.agents/skills/_shared/chronicler-shared.md`: lines 8-10 — rewrite the 3 doc-pointer bullets (legacy `docs/architecture/system.md`, `docs/system/*.md`, `docs/reference/*.md` are gone):
    - `docs/architecture/system.md` → `docs/diataxis/explanation/architecture.md`
    - `docs/system/*.md` → `docs/diataxis/reference/` (cluster: action pipeline, narration, message model, triggers, agent system)
    - `docs/reference/*.md` → `docs/diataxis/reference/coding_standards/` (test standards)

## Test Plan

Run from `chronicler_engine/`:

1. `python scripts/tests/test_validate_docs.py` (renamed test file): all 14 fixtures pass.
2. `python scripts/validate_docs.py` (renamed validator, default + `--strict`): passes against the live `docs/diataxis/` tree.
3. `python scripts/validate_docs.py --links --adr-refs --plan-links --body-refs`: passes individually.
4. `python scripts/extract_http_routes.py` against `src/adapters/driving/http/router.rs`: regenerates `docs/diataxis/reference/frontend/http_routes.md` with no path errors.
5. `python scripts/tests/test_extract_http_routes.py`: passes after import + path rewrites.
6. `python scripts/generate_docs_index.py`: regenerates `docs/AGENTS.md` AUTO-INDEX block; three new sections appear at the bottom (explanation/, how-to/, reference/ under `docs/diataxis/`).
7. `python build.py`: full pipeline green — cargo clippy + cargo tests + Python unittests + `scripts/validate_docs.py`.

## Per Task/Sub Task Validation Steps

| SubTask | Specific check |
|---|---|
| 1.1.1 | `python scripts/validate_docs.py --help` shows new help; `python scripts/validate_docs.py` against tree PASS. |
| 1.1.2 | `python -m unittest discover scripts/tests -v` shows 14 fixtures passing from the renamed file. |
| 2.1 | `ls chronicler_engine/docs/diataxis/AGENTS.md` returns no such file; merged `docs/AGENTS.md` has `## Writing Conventions` section. |
| 2.2 | `python scripts/generate_docs_index.py --check` returns 0. |
| 2.3 | `grep -rn "docs-diataxis" chronicler_engine/docs chronicler_engine/scripts chronicler_engine/AGENTS.md chronicler_engine/build.py .agents/skills 2>/dev/null` returns zero hits. |
| Final | Full `python build.py` green. |

## Assumptions

- The user accepts that the `--path` CLI flag for the renamed validator now expects a path under `docs/diataxis/`. Any non-diátaxis path passed to `--path` errors out.
- The validator inside is a C&P of the diátaxis version, not a re-implementation. Legacy-classify rules (architecture/system/reference/diagnostics) get dropped because they describe directories that no longer exist.
- **ADR cross-refs to deleted doc paths (`docs/system/dashboard.md`, `docs/system/prompt_system.md`, `docs/reference/testing.md`, etc.) are INTENTIONALLY NOT UPDATED.** They will render as broken links to a reader; no validator coverage exists for them because the new validator scopes to `docs/diataxis/` only. Out of scope per user direction. A future ticket can chase them down if the breakage becomes reader-visible.
- `chronicler-docs-hygiene` skill and `diataxis-doc-review` skill descriptions mention `docs-diataxis/` in their front-matter (description + body). Updating them is needed for skill discovery to stay accurate.
- `tests/AGENTS.md` already discussed in issue 04 as a separate home; out of scope for this ticket.
- `_PILOT_NOTES.md` (if any exists under docs/diataxis/) is not touched — already classified EXCLUDED.
- `src/ //! [DOC: ...]` markers are out of scope per the user's "Live docs/scripts/skills/build.py" decision. They will continue pointing at now-deleted doc paths but won't break the build (the validator doesn't scan src/).
