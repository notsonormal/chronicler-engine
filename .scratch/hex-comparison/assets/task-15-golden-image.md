# Ticket 15: one golden-image test, run five times

Date: 2026-11-01. Covers C-15 (`candidates.md`, K-18, K-28, K-29).
Labels: **known** = measured in this ticket or read in a file. **inferred** = reasoned from known facts.

## Result in short

- **The tool works (known).** `playwright-rs` 0.9.0 `to_have_screenshot` ran as one more `check_*` in `tests/browser/stub/layout.rs`, under `StubRunner`. It needed no new dependency.
- **Stability depends on the tolerance (known).** At zero tolerance the test failed 3 of 5 runs, each time with 17 pixels that differ by 1 level in one channel. With a per-pixel threshold of 0.05 and zero allowed pixels it passed 5 of 5 runs.
- **The image caught two edits that none of the 12 layout checks caught (known).** A width change and a colour change of the status column.
- **The Hex pixel allowance hides a text-colour edit (known).** A budget of `width*height/500` pixels is 163 pixels for this part. The colour edit changed 153 pixels, so it passed.
- **Cost is low for one part (known).** One 7 KB PNG. About 1 s more in a 10 to 14 s `run_layout` run.
- **Two things need work before the practice is safe (known).** The convergence check failed after the screenshot call (fixed in the worktree). A person or an agent must look at each changed image, and no tool can force that.
- **Recommendation: keep, with conditions.** See "Recommendation".

## What was built

All work ran in the scratch worktree `tmp/wt-task15-golden` (detached at `1a0c9162`). It is removed. The patch is in `tmp/hex-compare/golden/task15.patch` (git-ignored). Nothing changed in the main checkout.

| Item | Value | Source |
|---|---|---|
| Part | `.action-area` of an idle Game: command input, Send, Autoplay cycles, Autoplay, status display "Ready" | `assets/index.html:115-170` |
| Why this part | The stub server serves the real shell `assets/index.html`. It renders the three controls through the real functions (`idle_answer`, `send_control_answer`, `autoplay_controls_answer`), not from `tests/test_utils/stub_fixtures/` (only `visual_sidebar.html` is canned there). The image freezes the real template. | `tests/test_utils/stub_server.rs:1456-1466`, `src/adapters/driving/http/layout/handlers/endpoints.rs:70-72` |
| Test | `check_idle_action_area_matches_its_golden_image`, the 14th run in `run_layout` | worktree `tests/browser/stub/layout.rs` |
| Viewport | 1280 x 720, set in the check | same |
| Image | `tests/golden/action_area_idle.png`, 1280 x 64 px, 7088 bytes | worktree |
| Copied here | `task-15-golden-baseline.png` (baseline), `task-15-golden-failing-diff.png` (diff of edit M1) | this folder |
| Animations | `Animations::Disabled` (the crate adds a style tag with zero durations) | `playwright-rs-0.9.0/src/assertions.rs:706-713` |
| Masks | None. The part has no date, no counter and no random text in an idle state. | inferred from the markup, held by 5 equal runs |
| Retry | 1.5 s poll for a match, then it fails. The crate writes `-actual.png` and `-diff.png` beside the baseline. | `assertions.rs:1128-1196` |
| Update switch | `update_snapshots(true)` when `GOLDEN_UPDATE=1` (my own switch; the crate has none) | worktree |
| Chromium | Google Chrome 156.0.8078.47, channel `chrome` | `google-chrome --version`, `tests/test_utils/browser.rs:84` |

Command: `python build.py test-pattern "run_layout" --test-target browser`. The filter cannot name one check. The test unit is the runner `run_layout`, so every run executes all 14 checks (known, `layout.rs:12-80`).

## Finding 1: the convergence check failed after a screenshot

- **Known.** The first run failed in `assert_screen_converged`, not in the image compare: `options-dock differs from a fresh GET /screen?parts=options-dock: ... attribute style "" vs "undefined" on "<input type=\"hidden\" name=\"command\" value=\"/options\" style=\"\">"`. Source: `logs/build_20261101_200427.log` of the worktree.
- **Known.** A probe of `[style]` elements showed none before the screenshot and three `INPUT` elements with `style=""` after it. Setting `animations` was not the cause: the run with no animation switch failed the same way, and the run that skipped the screenshot passed.
- **Inferred.** The Playwright screenshot code sets and clears an inline style on inputs. An empty `style` attribute has the same meaning as none.
- **Fix in the worktree.** `tests/test_utils/convergence.js`, function `comparableAttributes`: skip a `style` attribute whose value is empty. Two lines. Per the ticket, the check was wrong, not the image. This change belongs to every screenshot test, so it must land with the first one.

## Finding 2: five runs at three tolerances

Run time is the `nextest` summary of the whole `run_layout` test (14 checks). A failed image check takes 2.2 to 2.6 s, and 1.5 s of that is the poll before it gives up. Source: logs `build_20261101_200905` to `build_20261101_201432` in the worktree.

The crate compares each pixel pair by a normalised distance over four channels (`dist_sq = (dr²+dg²+db²+da²)/4`, a pixel differs if `dist_sq > threshold²`; `assertions.rs:1266-1285`). So a threshold of 0.05 is about 12.75 levels, which is close to Hex's 12 per channel (inferred; Hex's own rule is in K-18).

| Setting | Threshold | Max pixels | Runs 1 to 5 (differing pixels; run time of `run_layout`) | Result |
|---|---|---|---|---|
| A: zero tolerance | 0.0 | 0 | pass 0 px, 13.4 s; **fail 17 px**, 13.7 s; **fail 17 px**, 14.4 s; **fail 17 px**, 12.6 s; pass 0 px, 11.2 s | **2 of 5 pass** |
| B: Hex-like | 0.05 | 163 (`1280*64/500`) | pass 11.9 s; pass 12.2 s; pass 12.5 s; pass 9.9 s; pass 10.0 s | 5 of 5 pass |
| C: tight | 0.05 | 0 | pass 10.0 s; pass 10.8 s; pass 10.9 s; pass 12.7 s; pass 16.2 s | 5 of 5 pass |

- **Known.** In A, the 17 failing pixels are the same in the three failing runs. A pixel compare of baseline and actual showed 17 pixels, each off by 1 level in one channel, in rows 51 and 52 (the bottom corners of the input and the two buttons). Columns 18 to 1005.
- **Inferred.** Sub-pixel layout moves the rounded corners by a fraction of a pixel in some runs. The cause was not found.
- **Known.** A pass in B or C does not show the exact differing pixel count. The crate prints a count only on failure. The pass is the only evidence that the count is within the limit.
- **Known.** The three failures in A came in a row (runs 2 to 4). I do not know whether load on the host or the order explains this.
- **Known.** The run times show noise from a shared host (`ENVIRONMENT.md`), so a gap under about 10 % is noise. The pass and fail times in A overlap.

**Reading.** A zero tolerance is not stable on this part. A threshold of 0.05 with no pixel allowance was stable in 5 of 5 runs (inferred rate: at least "most", as 5 runs cannot show a low flake rate). This ticket did not raise a tolerance to make a failing test pass for a hidden flake: A's flake is recorded above, and C is the setting chosen by that evidence.

## Finding 3: two deliberate CSS edits

Each edit was one rule in `assets/styles.css`, restored after the run. All 12 older layout checks ran in each case.

| Edit | Rule | Setting B (163 px allowed) | Setting C (0 px allowed) | The 12 older layout checks |
|---|---|---|---|---|
| M1: width | `#status-display, #action-status`: `flex` and `width` 240 px to 200 px | **fail**, 8045 px (9.82 %) | not run (B already failed; C is stricter) | all 12 pass |
| M2: colour | `.status.ready`: `color: var(--color-accent-ok)` to `var(--color-accent-yellow)` | **pass** (edit not seen) | **fail**, 153 px (0.19 %) | all 12 pass |

- **Known.** In both edits only the image check failed. The 12 older checks do not test the absolute width of the status column (the one width check, `check_command_input_width_is_stable`, compares the idle and the generating state with each other) and they do not test the colour of the "Ready" word (`check_root_design_tokens` reads the tokens, not their use).
- **Known.** M2 passes under Hex's pixel budget because 153 pixels is below 163. The budget is a share of the image area. A small part makes the budget as large as a whole word. So a share-of-area allowance is the wrong form for small parts (inferred). A threshold per pixel with no allowance caught it.
- **Known.** `task-15-golden-failing-diff.png` is the diff of M1 (the crate paints differing pixels red over a faded copy).

## What the image catches that the 12 checks do not

- **Known.** Anything that changes how the part looks without changing a measured number: a colour in use, a width, a gap, an icon, a label font, a border. The two edits above are examples.
- **Known.** Nothing outside the chosen element and state. The idle state of one part says nothing of the failed state, the busy state, or a long sentence. Each state needs its own image (inferred cost: one PNG and one `check_*` each).
- **Inferred.** A layout check states one intent ("the Details control stays inside the column"). An image states no intent. When it fails, a person must decide whether the change was meant. This is the review problem below.

## Findings that limit the practice

1. **Host fonts (known and inferred).** `assets/styles.css:49` sets `font-family` to `-apple-system, BlinkMacSystemFont, 'Segoe UI', Roboto, sans-serif`. This host has 13 fonts (`fc-list`), and `fc-match sans-serif` answers JetBrains Mono from `~/.local/share/fonts`. Which font Chrome drew for the baseline is **not known**; the image looks like a sans font. A host with other fonts will draw other glyphs. Inferred: the baseline is valid only on a host with the same fonts and the same Chrome. The ticket names the Chromium risk; fonts are the same risk and are larger. Mitigation to test: ship one font file and name it in the CSS (a change to the product), or record the font and the Chrome version beside the baseline and run the test only in one place. There is one place today (one host; `ENVIRONMENT.md`).
2. **A missing baseline passes (known).** `compare_screenshot` writes the screenshot as the new baseline and returns `Ok` when the file is absent (`assertions.rs:1100-1119`). A deleted PNG therefore hides a regression. The first run in this ticket passed for that reason. A guard is needed: a gate step that fails when a `check_*` image has no committed baseline.
3. **The check cannot be run alone (known).** `run_layout` is one test of 14 checks. The loop to refresh one image costs a full `run_layout` (9.6 to 16 s here).
4. **One part and one host (known).** The result is for one part, one state, one machine and one Chrome. Larger parts (story log with portraits, wrapped text) were not measured. 5 runs per setting do not show a rare flake.
5. **Not run (known).** `fmt`, `clippy`, the guardrails and the spec-coverage validator were not run on the spike. The test code has a long line that `cargo fmt` would split, and `layout.rs` says "Twelve checks" in its header doc (now 13 or 14). The `GOLDEN_VARIANT` and `GOLDEN_TOL` switches are spike scaffolding and would not ship.

## Cost

| Item | Value | Source |
|---|---|---|
| Disk per image | 7088 bytes for 1280 x 64 px. A full-page image would be larger; I did not measure one. | worktree |
| Added test time | A failed image check took 2.2 to 2.6 s including a 1.5 s poll. The other 13 checks took 0.55 to 0.83 s each when run. So a passing image check costs about 0.6 to 1.1 s (inferred). | logs `build_20261101_201432` |
| Browser tier | The whole browser tier takes about 92 s warm | `AGENTS.md`, `ENVIRONMENT.md` |
| Code | About 25 lines of test, 2 lines in `convergence.js`, one PNG | `task15.patch` |
| Upkeep | One `GOLDEN_UPDATE`-style run and a review for each intended visual change | inferred |

## Who reviews a changed image

Risk (known from the ticket): when the agent that changes the UI also rewrites the baseline, the test proves nothing unless a second party looks at the diff. No tool in this repo can force a look. The options:

| Option | What it gives | What it costs | Verdict |
|---|---|---|---|
| Nobody | The test stays green. The image only guards against an unplanned change that is not committed together with its baseline. | The agent that changes the UI rewrites the PNG and nothing is checked. The practice then has no value beyond a diff of a binary file. | **Reject.** |
| The owner | Strongest. The owner already approves each commit (`AGENTS.md:415`). | A PNG diff is hard to read in `git diff`. Without a side-by-side sheet the owner will skim it. | **Use for any change that was not planned in the ticket.** |
| A `reviewer` agent with vision | It can view the pair (baseline, new, diff) and compare it to the stated intent of the change. It is a different session from the agent that wrote the change. | It adds one model call per changed image. It can be wrong on small pixel changes (inferred: this ticket's M2 changed 0.19 % of the pixels). | **Use as the first reviewer.** Its verdict goes into the work report (ticket 13). |

**Proposal (inferred, not tested):**

1. A baseline update only runs through one named switch. `python build.py golden-update` (new subcommand) would set the switch, write the new PNGs, and write a review sheet to `tmp/`: baseline, new image and diff side by side, plus the pixel count.
2. The agent that changed the UI may run the update. It may not decide that the diff is right. It names each changed PNG and the intent of the change in its work report.
3. A `reviewer` agent with vision, in a fresh session, views the sheet and the stated intent. It writes one verdict per image (intended, unintended, cannot tell) in the work report.
4. The owner sees the verdicts at commit approval. Any image with the verdict "unintended" or "cannot tell" goes to the owner as an image, not as a binary diff.
5. A gate step fails when a `check_*` image has no committed baseline (finding 2) and when the update switch is set during `python build.py` (so the full gate never rewrites baselines).

**Honest limit (inferred).** Step 2 is a rule, not a lock. An agent can still run the update and say nothing. What catches it is the commit approval of the owner and the reviewer verdict that the work report must hold. If the owner skips that, the practice reduces to "nobody".

## Recommendation

**Keep, in a small form.** Do not roll it out wide.

- Adopt the tool as is (`to_have_screenshot`). It is already a dependency and it works.
- Use a per-pixel threshold near 0.05 and **no pixel allowance** (setting C). Do not copy Hex's `width*height/500` allowance for small parts.
- Start with the 2 to 4 states that have no good measured check: the action area (idle, busy), a failed status line, and one story-log entry. Each is one PNG and one `check_*`.
- Land the convergence fix with the first test.
- Do not adopt it before the three guards exist: the missing-baseline step, the fixed update command, and the review rule. Without them the image is a rubber stamp.
- The font risk (finding 1) needs one more measurement before more images are added: run the same test on a second host, or after a font change, and see how many pixels differ.

If the owner will not look at images and will not accept a `reviewer` verdict, **drop** the practice (inferred: then it adds cost and no review).

## Questions for ticket 16

1. Who must look at a changed image: the owner for every change, or a `reviewer` agent with vision first and the owner only for "unintended" and "cannot tell"?
2. Is one more AFK ticket wanted to measure the font risk (the same test on a second host or with a pinned font) before more than one image is added?

## Sources

- Worktree logs (removed with the worktree): `logs/build_20261101_200235.log` to `build_20261101_201432.log`. The numbers above come from them.
- `tmp/hex-compare/golden/`: the three images of the first failing run, `m1-actual.png`, `m1-diff.png`, `m2-diff.png`, and `task15.patch`.
- `playwright-rs-0.9.0/src/assertions.rs` lines 667-701 (`to_have_screenshot`), 706-713 (animations), 757-770 (options), 1080-1285 (compare).
- `tests/STRATEGY.md` ("Placement rule", "Stub tier's accepted tax"), `tests/browser/stub/layout.rs`, `tests/browser/stub/support.rs`, `tests/test_utils/convergence.js`, `tests/test_utils/stub_server.rs`.
- `candidates.md` C-15, K-18, K-28, K-29.
