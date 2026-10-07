# Documentation

Two generated catalogues index the documentation:

- `docs/AGENTS.md` — the catalogue of `docs/diataxis/`.
- `tests/AGENTS.md` — the catalogue of the integration test tree.

Read them when searching docs or tests, and read `docs/AGENTS.md` when writing or reviewing documentation: it owns the writing conventions.

## Generated indexes and the pre-commit hook

The pre-commit hook regenerates and stages four generated files on every commit:

- `STRUCTURE.md`
- `tests/AGENTS.md`
- `docs/AGENTS.md`
- `docs/diataxis/reference/coding_standards/guardrails.md`

Never hand-edit their generated blocks. Without the hook, regenerate them with the `scripts/generate_*_index.py` scripts; `scripts/precommit_regenerate.py` runs all four and stages the result.
