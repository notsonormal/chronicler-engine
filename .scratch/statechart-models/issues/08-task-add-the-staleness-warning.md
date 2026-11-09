# Add the staleness warning of the verify skill to build.py

Type: task
Status: resolved
Blocked by: —

## Question

Add a check to `build.py` that warns when a function that `verify-gate` covers is no longer the text that the TLA+ model was last checked against. [Ticket 09](09-grilling-decide-the-gaps-and-the-verify-skill.md) keeps `.agents/skills/verify-gate/` (Q3).

## The decisions (ticket 04)

- **A warning, not a failure** (Q5). The owner chose a warning.
- **In the end summary of the build log** (Q14). Agents read only the last lines of the newest log (`AGENTS.md`, Development Loop: `tail -n 10`). The closing banner is four lines (`build.py:1750-1753`). So the warning prints last in `_print_epilogue` (`build.py:1672`), just before the banner, in at most three lines. A failure of the check prints one line and never raises: an exception in the `finally` of `main` skips the banner and the `build_history` append.
- **The warning names `/maintain-formal-verification-skill`**, and the agent that sees it runs the maintain pass before it ends its task (Q10). Write that rule into the warning text, in the words of ticket 09, Q3: the agent that changed a listed function runs the pass. The warning names the changed functions, so an agent can see whether the change is its own.
- **By function, not by file** (Q6). The coverage record of `verify-gate` (ticket 07) names functions. An edit elsewhere in a covered file gives no warning.
- **Every `build.py` run shows the warning** (ticket 09, Q8), the narrow `test-pattern` loops too. The check reads the files on disk, so uncommitted work counts.

## The decisions (plan review, 2026-11-08)

- **No git and no JDK. Python only** (owner). The check compares hashes, not commits.
- **A hash for each function** (owner, decision 1, Option A). `coverage.toml` stores a hash of the text of each listed function, next to its name. The check hashes each listed function as it is on disk and compares. A different hash, or a listed function that the check cannot find (renamed, moved, deleted), gives the warning.
- **The maintain pass writes the hashes.** After it brings the TLA+ model along, it records the new hashes with one command. The warning then stops at once, with no commit. Before this design, the warning stayed until the owner committed and `sync_commit` moved.
- **`sync_commit` stays, as a note for the maintain pass only** (owner, Option A). The agent of a maintain pass reads `git diff <sync_commit>` to see the old code, as `/maintain-formal-verification-skill` (steps 1, 3 and 6) expects. The warning never reads it.
- **New functions that touch `Holder` are out of scope** (owner, decision 2). The maintain pass already looks for them (`verify-gate/SKILL.md:58`).

## Context

- The method (known, a review prototype, 0.03 s): read each covered file. Find each `impl <Type>` or `impl <Trait> for <Type>` block at depth 0, and each `fn` at depth 1, by brace count. Key each function as `coverage.toml` names it (`Type::fn`, `Drop for Type`). All 44 names resolve today. No `.gitattributes` change is needed.
- `git log -L` was the first idea, and it does not fit (known). It reads commits only. Its `:<funcname>:` is a regex on a source line, so `fn hold` (`gate.rs:190`, `:339`, `:796`) gives the first match. Without `*.rs diff=rust` in `.gitattributes`, it finds no Rust function.
- The TLA+ model is in sync with the code today (known: `git diff --stat e6715c60` on the four covered files is empty). So the first hashes can be recorded from the working tree.
- Ticket 14 adds a second check of the same kind: a generated diagram that is older than the TLA+ model. It compares a stored hash too, so the two checks share the hash code and the epilogue print.

## The work

- A script in `scripts/` with two modes: check (used by `build.py`) and record (used by the maintain pass). It reads each `.agents/skills/verify-*/coverage.toml`, so a later verify skill gets the warning with no change.
- The hash field in `coverage.toml`, recorded for the 44 functions. Update the comment at the top of the file.
- `.agents/skills/verify-gate/SKILL.md`: line 18 names the `build.py` warning as the trigger; step 6 records the hashes with the new command, and moves `sync_commit` only to a commit that holds the change.

## Checks

- Python tests in `scripts/tests/`. Cases: a changed function body; an edit outside every listed function (no warning); a renamed function; same-named methods in two `impl` blocks; `Drop for <Type>`; a brace in a string literal; a function with no hash yet; record, then check, gives no warning. One test in `test_build_cli_summary.py`, with the check mocked: the warning prints after the counts and before the banner.
- The `build.py` module docstring adds the warning to its list of stdout signals (`build.py:41-43`).
- `python build.py` (full gate) passes.
- Do not commit.

## Not in scope

- Failing the build. The check only warns (ticket 04, Q5).
- Functions that `coverage.toml` does not list, including new functions that read `Holder`.
- Running TLC or `check.py`.
- Changing the TLA+ model.
- Changing the vendored `/maintain-formal-verification-skill`.
- The diagram staleness check of ticket 14.
- A `.gitattributes` change.

## What already exists

- `.agents/skills/verify-gate/coverage.toml`: `sync_commit` and the 44 function names. Read it with `tomllib`, as `check.py:160` does.
- `build.py:1672` `_print_epilogue`: called from the `finally` of `main` (`build.py:1748`) for every command.
- `build.py:1662` `_epilogue_baselines`: the best-effort pattern (`try`/`except`, then show less).
- `build.py:103-107`: how `build.py` imports a module from `scripts/`.
- `scripts/tests/test_build_cli_summary.py:318`: a test of the epilogue order to copy.
- `scripts/check_python_docstrings.py`: a new script needs a module docstring.

## Failure modes

| New code path | Failure | Handled? |
|---|---|---|
| Read of `coverage.toml` | File missing or malformed | One notice line, no raise. |
| Read of a covered file | File missing | Warns "not found" for its functions. |
| Function bounds by brace count | A `{` in a string, char or raw string, or a comment, moves the bounds | False warning or missed change. Strip literals and comments before the count. A test covers it. |
| Name lookup | A listed function is renamed or deleted | Warns "not found". |
| Name lookup | Two `impl` blocks give the same key (generics, `cfg`) | Warns about the duplicate, and picks neither. |
| Hash compare | A rustfmt-only or comment-only edit | Gives a warning. Accepted. |
| Hash compare | A listed function has no hash yet | Warns "no hash". |
| Record mode | Run before the TLA+ model follows the code | The warning stops while the TLA+ model is stale. `SKILL.md`, step 6, records only after the check passes. |
| Any exception in the check | It skips the banner and the history append | The whole check is wrapped. |

## Answer

The staleness warning is in place. `build.py` prints it last before the banner on every run. The check is Python only: no git, no JDK.

**Files (known, I wrote them):**
- `scripts/verify_coverage.py` (new). Two modes: `check` (prints the warning, exit 0) and `record [--function NAME ...]` (writes the hashes). It reads each `.agents/skills/verify-*/coverage.toml`, so a later verify skill gets the warning with no change. The hash is the first 16 hex digits of SHA-256. The hashed text runs from the start of the `fn` line to the closing brace. Attributes and doc comments above the `fn` are not hashed. The brace count runs on source with comments, strings, raw strings and char literals blanked. A lifetime is not read as a char literal.
- `.agents/skills/verify-gate/coverage.toml`: an entry is now either `"Name"` (no hash yet) or `{ name = "Name", hash = "..." }`. The header comment says so. `record` rewrites those lines only and keeps the rest of the file. `sync_commit` stays as a note for the maintain pass.
- `.agents/skills/verify-gate/SKILL.md`: "When to use it" third bullet (the `build.py` warning is the trigger) and step 6 (`record` after the check passes; `sync_commit` moves only to a commit that holds the change). Both were small exact edits.
- `build.py`: import, `_staleness_lines()` (a failed check prints one line and never raises), a call at the end of `_print_epilogue`, and the module docstring.
- Tests: `scripts/tests/test_verify_coverage.py` (new, 23 tests); two tests in `test_build_cli_summary.py` (the warning prints after the counts and before the banner; a failed check prints one line and keeps the banner); `build_cli_support.run_main` has a `staleness` parameter that mocks the check, so the other epilogue tests do not read the repo.

**Warning text (three lines for one skill):** `WARNING verify-gate: the TLA+ model may not match the code (changed: ...; not found or not unique: ...; no hash: ...).` Then the rule: the agent that changed a listed function runs `/maintain-formal-verification-skill` before it ends its task. Then the record command: `python scripts/verify_coverage.py record`. A list shows at most four names and "and N more".

**Key rules (known):** `Type::fn` for an `impl Type`; `Trait for Type::fn` for one function of a trait impl; `Trait for Type` (for example `Drop for JobWriter`) for the whole trait impl block; plain `fn` for a free function. All 44 names of `coverage.toml` resolved to one definition each when I checked.

**The seed (known).** I compared each listed function at `sync_commit` (`git show`, one time) with the working tree. 41 are the same, and I recorded their hashes. These three differed, so they have no hash: `GameCatalogue::delete_game`, `GameCatalogue::reset`, `GameCatalogue::switch_game`. Ticket 10 has since added more plain entries to `coverage.toml` (the check now also names `GameCatalogue::new` and `GameCatalogue::create_game`). I did not hash those. The coordinator runs `python scripts/verify_coverage.py record` after ticket 10 ends, once its TLA+ model follows the code.

**Checks:**
- `python -m unittest discover scripts/tests`: 572 tests, OK (known).
- `python build.py py-tests`: pass. The warning shows as the last lines before the banner (known, read from the log).
- `python build.py` (full gate): every step before `Linting prose with Vale` passed. That step fails with `AGENTS.md:336:151 STE 6.3: this sentence has more than 25 words`. It is not my file. `git status` shows `AGENTS.md` unchanged, and the sentence comes from commit `25db6ee5`. Earlier in the run it also failed on `docs/diataxis/reference/storage.md` (ticket 10). That one cleared after a few minutes. So the later steps (clippy, guardrails, integration, browser, packaging) did not run in my full-gate run. My change touches Python and docs only (known). I did not run those steps.
- `guardrails.md` was regenerated by the gate (`ActiveGame::lock` in one row). That comes from ticket 10, not from me.

**Left open:**
- The AGENTS.md Vale error blocks the full gate. The owner or the coordinator must fix it. I did not edit AGENTS.md.
- A rustfmt-only or comment-only edit inside a listed function gives a warning. Accepted in the ticket.
- `record` before the TLA+ model follows the code hides the warning. Step 6 of `SKILL.md` says to record after the check passes. No code enforces this (inferred: nothing can).
- Ticket 14 can reuse `hash_text` and `warning_lines`. Its own warning needs its own lines in `_staleness_lines` (guessed).

### Coordinator check (2026-11-08)

- After ticket 10 ended, the coordinator ran `python scripts/verify_coverage.py record`. `check` then says that every listed function matches its hash (known).
- The Vale failure on `AGENTS.md:336` is fixed: the owner approved a split of the sentence.
- The full gate passes (`logs/build_20261108_224117.log`: guardrails 309, integration 1653 passed and 3 skipped, browser 44). Before that run, the coordinator excluded `**/.scratch/**` in `arch-lint.toml` and regenerated `guardrails.md`. The cause: arch-lint read the git-ignored `.rs` copies under `.scratch/weekly-retro-2026-11-04/assets/local-only/` (2,490 violations), from 2026-11-04.
