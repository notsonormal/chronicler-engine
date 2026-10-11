# Cover JavaScript in the comment finder

Type: grilling (HITL)
Status: open
Blocked by: —

## Question

`comment_finder.py` never reads `.js` files, so the comment-fixer pass skips every comment in `assets/dashboard-*.js`. Should the finder cover JavaScript, and should the pass then run again on the two JS files the dashboard UI review phase 2 changed?

## Context

- Source: `/retro` 2026-10-11 on the dashboard UI review phase 2 run (Manager session `01a12b7d`; comment-fixer session `01a12c59-8ad9`).
- `.agents/skills/chronicler-comment-fixer/scripts/comment_finder.py:36` — `UNCOMMITTED_EXTENSIONS = {".rs", ".py", ".html", ".css"}`; every mode filters on it (`:53`, `:64`, `:83`, `:102`).
- `comment_finder.py:93` — the `--all` glob has no `assets/*.js`.
- `comment_finder.py:167` — `_comment_lines` has a branch per suffix; JS needs `//` and `/* … */` handling (the `.rs` branch is close, the `.css` branch handles block comments).
- Evidence: the phase-2 pass ran `--branch c739932f`. Its per-file table lists 28 files and no `.js` file, while the diff changes `assets/dashboard-panels.js` and `assets/dashboard-errors.js`. The longest new comment blocks of the run are in `dashboard-panels.js` (for example the 8-line htmx timing block above `focusedBeforeSwap`).
- After adding `.js`: run `/chronicler-comment-fixer` on `assets/dashboard-panels.js` and `assets/dashboard-errors.js` (`--files`).
- Related: [Cover Python in the comment finder's full sweep](29-comment-finder-python-coverage.md) — the same gap for another language; decide the two together. [Audit the bloated comments in the dense files](38-audit-bloated-comments.md).
