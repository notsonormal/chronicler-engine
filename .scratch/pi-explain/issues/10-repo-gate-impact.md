# 10 — Repository gate and hygiene impact

Type: research
Status: open
Blocked by: 01
Assignee: (unclaimed)

## Question

What does putting a pi extension package inside chronicler-engine cost that
repository, and what must the package avoid?

This opens only after ticket 01 fixes the location.

## What to find out

1. Which parts of the repository gate touch a new `.pi/extensions/pi-explain/`
   tree. Known gate members: `python build.py`, the pre-commit hook that
   regenerates four generated index files, `scripts/check_test_structure.py`,
   `scripts/validate_docs.py`, `scripts/vale_lint.py`, and the guardrails tests.
2. Whether the generated indexes scan `.pi/`. The AGENTS.md structure index is
   built by `scripts/generate_structure_index.py`, and the tests index by
   `scripts/generate_tests_structure_index.py`.
3. What `.gitignore` already covers. Known: `.pi/extensions/**/*.js`,
   `.pi/extensions/**/*.d.ts`, `.pi/extensions/**/node_modules/`, and
   `.pi/extensions/**/package-lock.json`.
4. Whether the repository's permission extension at
   `.pi/extensions/pi-permission-system/config.json` interferes with the new
   extension's file writes or its child-process spawning.
5. Whether the extension's own dependencies must be committed, given that
   `.pi/extensions/**/node_modules/` is ignored. A local package is not
   installed by pi, so its dependency tree is the author's responsibility.

## Deliverable

A markdown summary linked as an asset, plus any small `.gitignore` or settings
change the package needs. Do not change the repository gate.

## Background

- `.scratch/` is tracked by this repository. This map's own files are part of
  that cost, and charting accepted it.
- The repository's AGENTS.md makes the agent responsible for the repository's
  overall health, so a new package must not break the gate.
- `docs/packages.md` states that local packages are not installed or modified by
  pi, so their dependency tree stays the author's responsibility.

## Recommendation

Keep the package self-contained. Commit sources only. Record any gate finding in
the note rather than changing the gate.
