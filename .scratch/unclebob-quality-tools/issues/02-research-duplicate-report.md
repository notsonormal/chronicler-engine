# Research: what a structural duplicate report shows beside jscpd

Type: research (AFK)
Status: open
Blocked by:

## Question

What does a function-level structural duplicate report (dryer: normalized syntax tree, Jaccard score over subtree fingerprints) find in Chronicler Engine that jscpd does not, and what would the user need to see to judge its value?

## Context

- First run (see the [research asset](../assets/research-crapper-mutator-dryer.md), section *dryer*): 61 pairs in 4.2 s with defaults (score ≥ 0.82, ≥ 4 lines, ≥ 20 nodes). 49 do not overlap any jscpd fragment in `report/jscpd-report.json`. About 30 of the 49 look like idiom noise (builder setters, `enum` → `&'static str` matches, `from_row` column lists); some are real near-clones (`assembler.rs` `render_game_state_layer` / `render_world_info_layer`).
- That run used a copy of `src/` with `*_tests.rs` removed. `src/test_support/` was still in; exclude it.
- jscpd runs as `python build.py duplicates` (`scripts/healthcheck.py`), report-only, scoped to files changed against `main`; `--all` covers the whole repo. Its report had 2,201 fragments.
- dryer is installed in `tmp/unclebob/.venv`.

## Work

1. Run dryer on production code only, at the defaults and at one or two other settings (for example `--min-nodes 40`, `--threshold 0.9`).
2. Run jscpd `--all` on the same scope for a fair comparison.
3. Sort every dryer pair into: real duplicate jscpd misses; real duplicate jscpd also finds; idiom noise. For each idiom, say whether a simple ignore rule removes it.
4. Note whether test code (`*_tests.rs`) is worth a separate run: test duplication is a known pain point in this repo's retros (inferred; check).

## Expected output

- An asset under `assets/` with the classified pairs, the settings comparison, and a side-by-side count against jscpd: written so the user can judge value without reading the tool's source.
- On resolution, graduate a grilling ticket: "Does the duplicate report have value here, which settings and ignore rules, and what happens to jscpd?"
