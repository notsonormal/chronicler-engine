# What the delegated agents hit, 2026-10-28 to 2026-11-04

Sources: `delegate_final_reports.md` (188 delegate sessions), `tool_errors.jsonl` (387 errors), `sessions_with_effort.json`, `delegates.jsonl`, `analyse_out.txt`. Session ids are the file prefix under `~/.pi/agent/sessions/--workspace-chronicler-engine--/`. Counts are **known** unless marked.

Delegate spend was **$238.51 of $503**. `reviewer` took $112.85 (47%), `strong-implementer` $82.78, `researcher` $23.30, `implementer` $18.79, `scout` $0.79.

## Ranked findings

### 1. Reviewers keep finding tests that cannot fail. Mechanical.
**9 instances, 8 efforts, in 8 of 67 reviewer reports.** Three shapes: an assert the surrounding code makes inevitable, a check reduced to `is_ok()`, and a guard no test drives.

| Session | Quote |
|---|---|
| `2026-10-31T13-31-25` cr94-standards | "Tautological assert (`CODING_STANDARDS.md`, Testing: "Tautological tests are considered harmful") … so this assert cannot fail." |
| `2026-10-31T23-52-36` pf19-review-standards | "The first assertion proves `!user.contains("<PlayerInput>")` … So `!user.contains("You enter the inn.\n</PlayerInput>")` cannot fail." |
| `2026-10-29T00-57-31` aa-stage5-review | "no check fails if `returnFocusAfterClose` … does nothing" |
| `2026-10-30T21-27-43` pr-stage3-review | "`test_list_games_empty` now checks only `is_ok()`." |
| `2026-11-01T23-45-49` lt09-standards | "only checks that each layer is present (`contains`), so the new order in that list asserts nothing." |

Also `2026-10-28T21-34-49` ("Two count assertions cannot fail as written"), `2026-10-28T22-53-10` (44.21), `2026-10-30T19-08-18` (39.9). **Cost:** find → fix → re-review, 2-4 sessions each. **Mechanical:** a one-line revert or mutation run per changed assertion catches all nine. Adds evidence to tickets **23** and **87**.

### 2. A new test duplicates an existing test's behaviour. Judgement, with a mechanical part.
**8 instances, 6 efforts; 26 of 67 reviewer reports raise overlap.** The rule is documented (`tests/STRATEGY.md`, *Overlap*: "one behaviour has one test") and is quoted back each time.

- `2026-10-29T19-38-17` aa-fixpass-review: "three tier-2 checks that pin the same behaviour"
- `2026-10-30T00-43-47` cr-standards: "`assert_busy_refusal` (16.42/42.6) and `reports.rs::assert_reported_in_the_notice` now assert the same rule"
- `2026-11-02T21-00-07` lt15-standards: "CASE `chat-completions-single-user-message-marks-roles` already pins that rule"
- `2026-10-28T19-21-39` aa-stage1-review: "The engine-failure row of 45.18 repeats 25.12."

Adds evidence to ticket **98**. **Judgement:** is one assertion one behaviour? **Mechanical part:** repeated assertion lines inside one test binary are findable; `healthcheck.py duplicates --ref` runs on code only.

### 3. A change lands that the plan and its Gate log do not describe.
**11 reviewer reports, 5 efforts**, mostly the staged action-answers plan.

- `2026-10-28T19-21-39`: "Section 6 of the plan does not list them, which is what stop condition 3 covers."
- `2026-10-30T19-08-18`: "The plan's stage-1 exit check also names the wrong file: the real matches are in `router.rs`."
- `2026-10-29T19-38-17`: "the spec now says focus is on the body, which is what the owner rejected."

Ticket **86** verbatim. New evidence: the *stage* review reports the drift, not only the final plan check.

### 4. The permission rules deny whole compound commands. Mechanical, one config file.
**108 denials, 90 sessions, one turn each; 4 lost a build.** `config.json` uses shell globs, and `*` crosses `;`.

- `grep * . *` **39×**: matches any command with `grep`, a standalone `.`, and more. `2026-10-28T20-38-23`: `python build.py --no-browser …; git status --short | grep -v "^ M .scratch"` was refused whole, build included. Also `…; grep -c . tmp/followup-plan/cites2.out`.
- `grep * /workspace/chronicler-engine *` **27×**: fires on any command naming that path.
- `timeout *` **21×**: `timeout 120 python scripts/healthcheck.py duplicates` is the coreutils wrapper, not a repo walk.
- `find / *` **14×**, `*.env` **7×** (by design).

**Fix:** narrow the two grep rules to a path-terminated form and allow `timeout`. Not in the Won't do list.

### 5. Doc or spec text left false by the change.
**15 reviewer reports, 8 efforts.**

- **Route renamed in a spec.** `2026-10-30T19-08-18`: "43 spec lines still name a removed route, but their tests already read `/screen`." `validate_docs.py` `UNKNOWN_ROUTE` reads the generated `http_routes.md`, so it passes while that table is stale (**inferred**; a stale table still lists the removed route). **Mechanical**, and adds evidence to ticket **94**.
- **Hand-written sentence now wrong.** `2026-11-01T00-49-32`: "`agent_system_design.md:16` … This text is now stale." `2026-10-28T21-34-49`: "`worlds.md:67` … Line 97 of the same file now says the group holds no slot, so the spec contradicts itself." **Judgement.** Won't do 82 measured a lint at 0 of 10 fix rounds.

### 6. The generated indexes are not regenerated before review. Fully mechanical.
**9 reviewer reports.**

- `2026-10-30T19-08-18`: "Three lines of the generated index in `tests/AGENTS.md` are out of date. The pre-commit hook regenerates them."
- `2026-10-28T19-21-39`: "The module summary of `tests/http/worlds.rs` does not name the new test."

`scripts/precommit_regenerate.py` already exists and runs at commit only. **Fix:** one brief line.

### 7. Code left without a caller after a refactor.
**5 reports, 4 efforts.** `clippy` is blind to a `pub fn` with no caller.

- `2026-10-30T21-27-43`: "`render_fragment` … has no caller … `clippy` does not report a dead `pub` fn."
- `2026-10-28T19-21-39`: "`ApplicationError::is_user_displayable` … has no caller now."
- `2026-11-01T00-49-32`: "`HistoryCut::blocks()` has no production reader"

**Mechanical:** `rg -w` over the deleted symbols of the diff.

### 8. 27 `read` errors on image outputs.
**12 art sessions; 17 of 27 passed `offset`/`limit`.** Two **known** causes: the path is an output name guessed before the generator ran (`…/facegen/out/sheet-r7.png`, `…/r3-grid64/out/z.png`), and a text-file read form reused on a PNG. One turn each inside a $0.5-2.3 prototype; no tool change. One brief line: list the output directory first.

### 9. A linked-worktree implementer cannot get the API key.
**2 instances** (`2026-11-01T00-13-24`, `2026-11-01T00-13-25`).

- `2026-11-01T00-13-25`: "I could not read any `.env`, and `build.py run` from the worktree found none. I started the built binary from a scratch folder under `tmp/` in the main checkout."
- `2026-11-01T00-13-24`: "**Probe server ran from the main checkout's directory.** … which my policy bars me from reading"

Ticket **97** (symlink fix, `1a0c9162`) is in the tree. Adds evidence: each workaround cost turns and put the probe data folder on the wrong checkout.

### 10. Shared-checkout collisions stop work, not only commits.
**6 implementer reports.** This meets Won't do **D2**; it defeats the reason only in part: the blocked step is now a *merge*.

- `2026-11-01T00-13-24`: "Ticket 20 is done and committed, but **not merged**. The main checkout is not clean … `?? .scratch/hex-comparison/`."
- `2026-11-01T18-51-12`: "Those are another agent's Phase B work, not mine. I did not read, edit, format or stage them."

5 of the 9 `edit` failures also hit `.scratch/` map files: the coordinators' shared surface, not source.

### 11. Briefs are long, rarely wrong; the repeated brief defect is an impossible instruction.
**86 briefs, median 2360 chars, max 8501.** Medians: strong-implementer 3238, scout 3623, implementer 2395, reviewer 1972, researcher 1937.

- **4 of 188** reports call the brief wrong, each a small forced addition: `2026-10-30T20-51-21` "Beyond the brief's literal list, all forced by the change".
- **3 resumes** of 86. A resumed child buys a fresh context for a small delta.
- The recurring defect is a step the child cannot run: ticket **99**. One instance (`f6c26d0f`, layout-trials 09), named in that ticket.

### 12. Cost by role.
| Role | Sessions | Cost | Per session | Model |
|---|---|---|---|---|
| reviewer | 67 | $112.85 | $1.68 | opus 5.5 |
| strong-implementer | 29 | $82.78 | $2.85 | sonnet 5.5 |
| researcher | 32 | $23.30 | $0.73 | deepseek 20, sonnet 12 |
| implementer | 47 | $18.79 | $0.40 | deepseek 19, sonnet 28 |
| scout | 13 | $0.79 | $0.06 | deepseek |

- **Review is not over-used** (inferred): it found every defect in findings 1-7. But it is the only control for 1, 2, 6 and 7, which are mechanical, so part of its cost is a workaround.
- **`scout` is under-used:** $0.79 for 13 investigations, and a reviewer pays $1.68 for a comparable read.
- **`implementer` on deepseek is 7× cheaper per session** than `strong-implementer` and ran more tool calls (2411 vs 2379).

## Not findings
`build-or-test-fail` (5) is a misclassification: each command's *output* quoted a failing log. `nonzero-exit` (77): 27 are `exit 1` with no output, 9 are `rg` with no match. "The image viewer served stale images for a reused file name twice" (`2026-11-03T19-38-21`) is one instance; watch it, do not file it.
