# Cover Python in the comment finder's full sweep

Type: grilling (HITL)
Status: resolved
Blocked by: —

## Question

`comment_finder.py --all` globs only `src/**/*.rs`, `tests/**/*.rs`, `assets/*.html`, `assets/*.css`. Python is reachable only through `--files`/`--pattern`. A "full sweep" that reports clean has silently skipped every comment in `build.py` (170 pre-existing docstrings) and `scripts/*.py`. The skill labels the mode "full codebase scan; skips Python" but offers no Python mode.

Should the full sweep cover Python?

- **Extend `--all`.** Add `scripts/**/*.py` and root-level Python to the glob.
- **Add `--all-languages`.** Keep `--all` as-is for Rust/HTML/CSS and add a mode that includes Python.
- **Leave it.** Document `--pattern 'scripts/**/*.py'` as the Python route.

## Context

- Source: `/reflect` 2026-10-07, tooling reviewer; sessions `01a11774`, `01a11745`.
- Evidence: `assets/reflect-2026-10-07/synthesis.md` (Backlog "Comment-finder Python coverage").
- `.agents/skills/chronicler-comment-fixer/scripts/comment_finder.py:90` — `get_all_source_files`.
- `.agents/skills/chronicler-comment-fixer/SKILL.md:16` — "full codebase scan; skips Python"; `:23` — `--files ... scripts/bar.py`.
- Related: [Scope the comment pass to the lines the diff adds](12-comment-pass-scope-to-the-diff.md).

## Answer

2026-10-21, decided with the owner inside [37](37-comment-finder-javascript-coverage.md) (Q3). **Extend `--all`.** The glob now adds `*.py` (root, so `build.py`) and `scripts/**/*.py`. `SKILL.md` Mode 2 no longer says "skips Python", and its Carve-outs keep the module docstrings that `scripts/check_python_docstrings.py` requires. Python comment detection itself is unchanged (line-start `#` and `"""`).
