# Build one golden-image test and run it five times

Type: task (AFK)
Status: resolved
Blocked by: —

## Question

Is one golden-image test of one dashboard part stable enough and cheap enough? Who or what reviews a changed image?

## Context

- **Covers** C-15 (golden-image UI tests). See `assets/candidates.md`, theme C, and checks K-18, K-28 and K-29.
- **The tool exists (known, checked in ticket 08).** `playwright-rs` 0.9.0 is already in `Cargo.toml`. It has `to_have_screenshot` (`src/assertions.rs:667` and `:1048` in the crate). Its `ScreenshotAssertionOptions` hold `max_diff_pixels`, `threshold`, `animations`, `mask` and `update_snapshots`.
- **Hex (known).** Hex keeps 122 PNGs in `tests/golden/`, with a tolerance of 12 per colour channel and of `width*height/500` pixels. `HEX_UPDATE_GOLDENS` rewrites them. No record proves that a person looks at each changed image.
- **Chronicler (known).** Browser screenshots are failure diagnostics only (`tests/test_utils/browser.rs:477-484`). The closed `ui-verification-redesign` map made the browser tier free of flakes by construction. Tier 3 holds 7 tests, each behind a settle-gate helper. Read `tests/STRATEGY.md` and `tests/browser/` before you start.
- **The risk in an AI-only flow.** If the agent that changes the UI also updates the baseline images, the test catches nothing unless something reviews the image diff.
- **Do.**
  1. Work in a scratch git worktree under `tmp/`. Nothing lands in this checkout.
  2. Pick one dashboard part in a fixed state, for example the status display of an idle Game. Pick a part that the stub server renders through the real Askama templates, not a canned fragment from `tests/test_utils/stub_fixtures/`. An image of a canned fragment would freeze the fixture, not the template (`tests/STRATEGY.md`, "Stub tier's accepted tax").
  3. Write one test with `to_have_screenshot`, as one more `check_*` in `tests/browser/stub/layout.rs` under the shared `StubRunner`. Turn animations off, and mask any text that changes between runs, such as dates. Run it with `python build.py test-pattern "<name>" --test-target browser`.
  4. Run the test 5 times. Record each result, the diff pixels and the run time.
  5. Change one CSS rule on purpose, and confirm that the test fails.
  6. Propose who reviews a changed image: the owner, a `reviewer` agent with vision, or nobody. Propose how an update is approved, so that the agent that changes the UI cannot accept its own diff without a review.
  7. Remove the worktree when you finish.
- **Placement (known).** The test belongs in tier 2. `tests/browser/stub/layout.rs` already holds 12 layout and computed-style checks with no spec scenario (`tests/STRATEGY.md`, "Placement rule"). The asset must say what the image catches that those 12 checks do not.
- **Not in scope.** Nothing lands in this checkout. The test, any CSS change and the baseline image stay in the worktree.
- **What already exists.** `StubRunner`, the readiness gates and the convergence check in `tests/test_utils/`. `build.py` seeds the cold target dir of a worktree from a warm sibling (`scripts/target_seed.py`).
- **Failure modes.**
  - A run fails with a small diff: record the diff pixels of each run. Do not raise the tolerance until the test passes, because that hides the flake that this ticket measures.
  - The convergence check fails: the check is wrong, not the image. Fix the check first.
  - A Chromium update changes fonts or antialiasing: record the Chromium version beside the baseline.
- **Output.** Write `assets/task-15-golden-image.md`. Copy at most 2 small PNGs into `assets/`: the baseline and one failing diff. `.scratch/` already tracks 17 PNGs. Keep the other images in `tmp/hex-compare/golden/`. The `## Answer` says whether the test is stable, what it costs, and whether to keep or drop the practice. It adds at most 2 questions for ticket 16.
- **When done.** Add `## Answer` and set `Status: resolved`.


## Answer

**Gist (known).** One golden-image test of the idle action area ran as the 14th check of `run_layout` in a scratch worktree. The worktree is removed. At zero tolerance it was not stable: 2 of 5 runs passed, and 3 failed with the same 17 pixels that differ by 1 level in one channel. With a per-pixel threshold of 0.05 it passed 5 of 5 runs, with 0 allowed pixels and with Hex's `width*height/500` allowance. Two deliberate CSS edits (status column width, "Ready" colour) failed the image, and all 12 older layout checks passed both times. Hex's pixel allowance (163 px here) hid the colour edit (153 px). The test costs one 7 KB PNG and about 1 s per run.

**Two problems found (known).** The convergence check failed after the screenshot call, because Playwright leaves `style=""` on inputs. Fix: skip an empty `style` attribute in `tests/test_utils/convergence.js` (2 lines, in the patch). A missing baseline passes and is written as the new baseline, so a deleted PNG hides a regression. The host font also decides the image (`font-family` falls back to `sans-serif`; the host's font was not pinned).

**Recommendation: keep, in a small form.** Use `to_have_screenshot` with threshold about 0.05 and no pixel allowance. Start with 2 to 4 states. Land three guards first: a gate step that fails on a missing baseline, one named update command that also writes a side-by-side review sheet, and a review rule (a `reviewer` agent with vision first; the owner for "unintended" or "cannot tell"). If the owner will not look at images or accept a reviewer verdict, drop the practice.

**Questions for ticket 16.**
1. Who must look at a changed image: the owner for every change, or a `reviewer` agent with vision first and the owner only for doubtful cases?
2. Is one more AFK ticket wanted to measure the font risk (second host or pinned font) before more than one image is added?

Asset: [task-15-golden-image.md](../assets/task-15-golden-image.md). Images: [baseline](../assets/task-15-golden-baseline.png), [failing diff](../assets/task-15-golden-failing-diff.png).
