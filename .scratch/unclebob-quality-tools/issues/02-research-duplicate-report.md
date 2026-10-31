# Research: what a structural duplicate report shows beside jscpd

Type: research (AFK)
Status: resolved
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

## Answer

Asset: [02 — a structural duplicate report beside jscpd](../assets/02-duplicate-report.md). Evidence in `tmp/unclebob/` (copies, `out/` reports, join scripts).

**Gist.** On production code at the defaults, dryer reports 52 pairs: 10 real duplicates and 42 idiom pairs. Only 2 of the 10 are ones jscpd misses (`assembler.rs` `render_game_state_layer`/`render_world_info_layer` at 1.00, and the two `Debug` impls of the options and quantifier agents). jscpd finds 184 clone fragments on the same scope and 5.2% duplicated lines, so dryer complements jscpd rather than replacing it — but at the defaults it needs rules to be readable.

The 42 noise pairs fall into seven idioms, each removable by one structural predicate (builder setter, `from_row` column list, enum → text `match`, `FromStr`/`parse_or_default`, variant-payload accessor, `map(..).unwrap_or(..)`, thin delegation). The seven rules leave 9 pairs, all real, and cost one real pair (`from_config`). dryer has no ignore mechanism at all, so every rule is a feature of a port or a vendored patch: that is the map's open *Form of each kept tool*.

Settings: `--threshold` is the wrong knob (the noise scores 1.00 because the normalizer drops the literal, while the real near-clones sit at 0.84–0.97); `--min-nodes 40` changes nothing; `--min-nodes 100` gives 14 pairs, 7 real; `--min-lines 8` gives 21 pairs, 7 real. Cost is 7.8 s on production, against 0.2 s for jscpd.

**Test code.** dryer reports 409 pairs for `*_tests.rs` + `src/test_support/` and 1,016 for `tests/`, against 52 for production, and 92.5% of the 2,427 clones in the gate's own jscpd report have a test side. The documented pain is real (`.scratch/_closed/test-strategy-execution/issues/05-retry-http-e2e.md:172,179-183`). The defaults on that scope are dominated by browser `page.evaluate` wrappers (286 call sites), so a test run needs its own rules. Verdict: production first.

**Addendum: the gate's own jscpd run** (coordinator, 2026-10-31). `python build.py duplicates --all` (2.10 s, `logs/build_20261031_143959.log`): 2,427 clones over the whole repo, 17.81 % duplicated lines. Of the 10 real dryer pairs, the gate's report has no clone for 3: A1, A2 and B8 (`list_games`/`list_personas`; the class "jscpd also finds" above was wrong for it). Four real pairs sit in one file, and the printed summary drops same-file clones. The summary shows only the top 25 file pairs, and test files fill 24 of them. So a reader of the build output sees 1 of the 10 real pairs (B5). Detail: [asset, section *The gate's own jscpd run*](../assets/02-duplicate-report.md#the-gates-own-jscpd-run-added-by-the-coordinator-2026-10-31).

**Graduated:** [Grilling: does the duplicate report have value here, which settings and ignore rules, and what happens to jscpd?](./04-grilling-duplicate-report-value.md)
