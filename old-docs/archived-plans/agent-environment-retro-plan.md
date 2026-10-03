# Plan: Agent-environment retro fixes

**Date:** 2026-10-03
**Status:** Implemented — WP1–WP4 landed and the full gate passed (2026-10-03). A three-axis review (Standards, Spec, Markdown) then found two bundle-script data-loss paths and several stale-claim defects; see the review-fix entries under "Decisions". WP4 was narrowed to the `tests/STRATEGY.md` rule, no validator change
**Sources:** the 2026-10-03 retro on the `dashboard-ui-updates` session (its findings 1, 2, 4 and 5; findings 3, 6 and 7 were closed by the maintainer, see "Not in scope")
**Goal:** close the four environment gaps that session exposed — a build slot with no way to ask whether it is free, a `commit-and-push` skill that describes a pre-commit hook that no longer exists, review diffs materialized by hand before every review, and specs that quote markup their covering tests assert differently.

## What already existed at planning time

This section records the pre-change state. The work packages below superseded several of its claims — the build-slot entry point, the `commit-and-push` hook description, the code-review diff instruction — so read it as history, not as current fact.

- `scripts/build_slot.py` is a library with no entry point (`hold`, `maybe_hold`, `is_heavy`, `_acquire`; no `__main__`, no `argparse`). `scripts/target_seed.py` already carries `probe_shared_lock(lock)`, a single non-blocking `LOCK_SH` probe that `build.py`'s `is_target_locked` shares — the probe WP1 needs is written and tested. Reuse it, but mind the dependency direction: `build_slot` imports nothing repo-local today, while `target_seed` pulls in `subprocess`, `hashlib` and `shutil`. There is no cycle (`target_seed` does not import `build_slot`), so the import is safe; lifting `probe_shared_lock` into neutral plumbing is the cleaner option if the coupling is unwanted.
- `ENVIRONMENT.md:19` states the slot's purpose and `:67` lists the `Waiting for the build slot` symptom. Neither says how to check the slot. The sibling `.holder` JSON is written for the log message only, so neither the lock file's existence nor that JSON is a liveness signal.
- `.git/hooks/pre-commit` regenerates four indexes — `AGENTS.md`, `tests/AGENTS.md`, `docs/AGENTS.md`, `docs/diataxis/reference/coding_standards/guardrails.md` — and stages them into the in-progress commit, precisely so the author does not commit twice. It aborts only when one of those files already had unstaged changes before it ran. `.agents/skills/commit-and-push/SKILL.md` describes the opposite: a `docs/README.md` abort and a two-commit flow. `docs/README.md` does not exist.
- `.agents/skills/code-review/SKILL.md` step 4 tells the parent to include "the full diff command and commit list" in each sub-agent prompt, but both sub-agents run under the `reviewer` definition, which has no shell — the command is unusable to them. `.agents/skills/chronicler-after-plan-workflow-plus-review/SKILL.md` step 12 names the same gap ("the read-only three also need the branch diff (`git diff` saved under `tmp/`)").
- `scripts/validate_feature_spec.py` links every scenario to its covering test through the `// [spec] SCENARIO: N.N` annotation, and enforces tag and observation-surface rules. It never compares text: 16 stale `<span class='error'>…</span>` literals in `docs/specs/prompt_presets.md` and `docs/specs/settings.md` passed it while their covering tests asserted `<div class="error-message">`. Those 16 literals are already repaired by hand — `docs/specs/` now carries `class="error-message"` and no `span class='error'` — so WP4 removes the drift surface, it does not fix the current one.

## How to execute

Work packages are independent and can run in any order; each is small enough for one agent. Run `python build.py py-tests` for WP1 and WP3 (no cargo). No gate step covers `.agents/` — `validate_docs.py` scopes to `docs/`, and the guardrails do not reference the skills tree — so WP2 needs no build at all; its acceptance is a read-back against `.git/hooks/pre-commit`.

## WP1 — Build-slot status probe

**Depends on:** nothing.
**Done when:** `python scripts/build_slot.py --status` answers free or held without taking the slot (exit 0 free, 1 held, 2 error), ENVIRONMENT.md points at it, and the cases are covered by tests.

Steps:

1. `scripts/build_slot.py` gains a status path (`--status`, exit 0 free, exit 1 held, exit 2 error) that reuses `probe_shared_lock(lock_path())` — a single non-blocking `LOCK_SH` probe released immediately — and prints the holder's pid, cwd, label and age from the `.holder` JSON when it is readable.
2. The verdict comes from the flock alone. `probe_shared_lock` returning `None` means an exclusive holder has the lock, and the kernel releases that lock when the holder dies, so a held flock always means a live holder. Treat the `.holder` JSON as informational metadata only: never let a dead recorded pid flip a held verdict to stale, because the pid is written once at acquire and can lag the true holder, and the file is never deleted on release. When the probe says free, exit 0 and, if a `.holder` file is left behind, note it as stale; when it says held, exit 1 and print the recorded holder, annotating a dead recorded pid as "flock held by an unrecorded process" without changing the verdict.
3. Any unexpected exception on the status path exits 2 rather than 1, so a crash cannot be misread as "held" by a caller that tests the exit code.
4. `ENVIRONMENT.md`'s build-slot bullet names `python scripts/build_slot.py --status` and carries the one caveat the script's docstring does not spell out: a build that gave up waiting (`CHRONICLER_BUILD_SLOT_WAIT`) or runs with the slot disabled holds no flock, so `--status` reports free while it compiles. It does not restate the `.holder`-is-informational note — a reader meets that in the docstring first.
   Import note: `build.py` imports `build_slot` and `target_seed` as top-level modules (`scripts/` is on `sys.path`), and running `python scripts/build_slot.py` puts `scripts/` first on the path, so `import target_seed` resolves in both entry modes. The status path must not create the lock file (`hold` uses `O_CREAT`; `probe_shared_lock` does not) — a missing file reads as free.
5. `scripts/tests/test_build_slot.py` gains cases for free, held by a live process, a free slot with a leftover `.holder` naming a dead pid (asserting free-with-stale-note, not a stale verdict), a missing or unreadable holder, and a failure injected on the status path that exits 2.

## WP2 — Correct the commit-and-push skill

**Depends on:** nothing.
**Done when:** every statement in the skill matches `.git/hooks/pre-commit`, and no line mentions `docs/README.md` or a mandatory second commit.

Steps:

1. Rewrite "Pre-commit Hook Behavior": name the four generated files, state that the hook regenerates and stages them into the in-progress commit, and give the one abort case (a generated file already had unstaged changes) with its fix (`git add <file>`, then commit again).
2. Replace the "Two-Commit Flow" section with that abort case — the hook's own comment says it stages the files so the author does not commit twice.
3. Fix every other line that repeats the stale claim, not only two rows. The skill mentions `README.md` or a second commit in the intro list of hook behaviour, the pre-flight step ("If this updates `docs/README.md`, stage it now" and its "Why do this first?" note), the hook-failure recovery snippet (`git add docs/README.md`), the status-output comment ("updated README.md"), the Verification Checklist ("or README.md staged for second commit") and the Common Mistakes row about two commits. Find them all with `grep -n "README\|second commit\|Two-Commit" .agents/skills/commit-and-push/SKILL.md`; the done-when grep must return nothing.
4. Keep step 1's manual generator run as an optional pre-flight, not as the thing that avoids a second commit.

## WP3 — Review bundle script

**Depends on:** nothing.
**Done when:** one command produces the bundle this session assembled by hand, both review skills name it, and the output stays under gitignored `tmp/`.

Steps:

1. New `scripts/prepare_review_bundle.py` with two modes: `--uncommitted` (default; `git diff HEAD` plus untracked and deleted lists) and `--ref <fixed-point>` (three-dot `git diff <ref>...HEAD`, matching the code-review skill). It writes `stat.txt`, one patch per top-level area (`src/`, `tests/`, `docs/` + `assets/`, tooling), `untracked.txt`, `deleted.txt`, and a `README.md` naming the fixed point and how to page the files.
2. In `--uncommitted` mode the patch set must carry untracked file *contents*, not just names: `git diff HEAD` omits untracked files, so a new source file would otherwise reach the reviewer as a bare filename. List them with `git ls-files --others --exclude-standard` and emit each as `git diff --no-index -- /dev/null <file>` (exit code 1 means "differs", not failure) into the patch for its area. Do **not** use `git add -N`: it rewrites the index, which the project's git-supervision rules reserve to the user and which races with other agents on a shared checkout. The script must never write to `.git`. `--ref` mode needs no such handling, because committed files appear in the three-dot diff.
   Clear the run directory at the start of each run so a stale patch from an earlier fixed point cannot reach a reviewer — but only a directory carrying this script's in-progress marker. The run writes the marker while it works and drops it on success, so a finished bundle carries none: a second run at the same default path refuses (exit 2) and names the path instead of clearing the bundle a reviewer is reading. A directory with no marker to find — `tmp` itself, another tool's directory — is refused by the same check, so `--out tmp` cannot wipe the shared scratch tree. Give the script a module docstring (`python build.py docstrings` checks `scripts/`).
3. `--out` defaults to `tmp/review/<short HEAD sha>-<mode>/` (under gitignored `tmp/`) and the script prints the path; the script refuses to write outside a gitignored path so a bundle cannot ride into a commit, and refuses to clear a directory it did not create (step 2).
4. `.agents/skills/code-review/SKILL.md` step 4 and `.agents/skills/chronicler-after-plan-workflow-plus-review/SKILL.md` step 12 tell the parent to run the script and pass the bundle paths instead of a diff command.
5. A test in `scripts/tests/` covers the mode selection, the untracked-content handling and the path-splitting logic against a fixture repository.
6. In `--ref` mode the script also writes `commits.txt` (`git log <ref>..HEAD --oneline`).

## WP4 — Spec-markup rule (decided: narrow C)

**Depends on:** nothing.
**Decision (maintainer, 2026-10-03):** option C, narrowed. No check lands in the gate. Reason: the retro drift came from specs quoting markup their covering tests asserted differently, and a check that couples spec text to test text (A or B) adds pressure on the long "renders the full surface" scenarios (189 scenarios; median 3 `And`/`Then` lines, 12 with 8 or more, up to 16 in `prompt_presets.md` 21.1). The maintainer does not want to make the tests more fragile. Removing the drift surface is cheaper than policing it.
**Done when:** `tests/STRATEGY.md` carries the rule below, and nothing else changes.

Steps:

1. Add a short paragraph to `tests/STRATEGY.md`, under "Spec scenarios and HTTP E2E" after "Spec completeness is mandatory": a scenario describes behaviour in words, and quotes markup only as an element's `id` or `class` that the frontend or a browser test selects on.
2. Apply the rule when a spec is next edited. No rewording sweep over the existing scenarios, and no change to `validate_feature_spec.py`.
3. Residual drift is handled in review (a stale literal is one review comment, not a gate failure).

Options considered:

| Option | Outcome |
|---|---|
| A. Extract `<tag …>` literals from specs and require them in the covering test file | Rejected: false positives, and it pushes authors to thin specs or lengthen tests |
| B. Same check on `class="…"` tokens | Rejected: same coupling, even though it is cheaper to audit (17 class attributes today) |
| C. Stop quoting markup in specs | **Chosen**, narrowed to a `tests/STRATEGY.md` rule with no rewording sweep |
| D. Do nothing | Folded in for residual drift only |

If a check is wanted later, it must warn rather than fail: `validate_feature_spec.py` is a gate step, so a failing check would inherit its exit-code contract, and it must compare class tokens, not quoted markup (tests assert `body.contains("error-message")`, and Rust escapes quotes).

## Not in scope

- **The generated STRUCTURE block in `AGENTS.md`** (retro finding 3, 304 of 429 lines). Left as-is deliberately: it works well, and agents rarely have to list or search for files.
- **The `rg` search pointer** (finding 6) and **the shared-checkout sentence** (finding 7): skipped by the maintainer.
- **The `reviewer` agent definition**: it stays read-only. WP3 exists so that read-only reviewers can still receive a diff; granting them a shell is a pi-config change outside this repo and would give up the property that makes the definition useful.
- **Re-running the retro's other findings**: closed above, not deferred.

## Failure modes

| Risk | Mitigation |
|---|---|
| The status probe itself takes the slot | Reuse `probe_shared_lock`'s non-blocking `LOCK_SH` and release the handle before printing |
| The status probe reads a stale `.holder` as a live build | The verdict comes from the flock; a leftover `.holder` is reported stale only when the probe says free, and a dead recorded pid never flips a held verdict |
| A probe crash reads as "held" | Unexpected exceptions exit 2, distinct from the held exit 1 |
| An untracked file reaches the reviewer as a name only | `--uncommitted` includes untracked contents in the patch set |
| A bundle is huge for a large fixed point | Split by top-level area, keep `stat.txt` as the index, and let reviewers page the patches |
| The skill text drifts from the hook again | WP2's acceptance is line-by-line agreement with `.git/hooks/pre-commit`, which is the source of truth |
| A literal check pushes authors to weaken specs | WP4 adds no check; the rule lives in `tests/STRATEGY.md` |
| A bundle is written outside `tmp/` and committed | The script refuses a non-gitignored `--out` (checked with `git check-ignore -q`) |
| The bundle script alters the user's index or working tree | It only reads git state; untracked contents come from `git diff --no-index`, never `git add -N`. The test asserts `git status --porcelain` is unchanged after a run |
| Two runs at the same HEAD and mode overwrite each other's bundle, or `--out tmp` wipes shared scratch | A run clears only a directory carrying its in-progress marker, and refuses (exit 2, path named) every directory without one — a finished bundle, `tmp`, or another tool's directory |

## Decisions

- **WP4:** narrow C (a `tests/STRATEGY.md` rule, no check). See WP4.
- **Bundle output location and ownership:** `--out` defaults to `tmp/review/<short HEAD sha>-<mode>/` and the script prints the path. A run holds an in-progress marker inside the run directory and drops it on success, so a finished bundle carries none. Two runs at the same HEAD and mode therefore do not clobber each other — the second finds no marker, refuses (exit 2) and names the path — while a marker left behind means an interrupted run whose directory the next run may reclaim. Every directory without the marker, `tmp` itself included, is refused, so `--out tmp` cannot wipe the shared scratch tree.
- **Review fix — reviewer diff acquisition:** the after-plan workflow's step 12 states that the bundle supersedes each reviewer skill's own diff-acquisition step. `antipattern-checker` and `thermo-nuclear-code-quality-review` keep their shell instructions, which are correct when the skill is invoked standalone by an agent that has a shell.
- **Review fix — pre-commit abort case:** one home in the `commit-and-push` hook section; step 5 and the Common Mistakes row point at it instead of repeating the fix command.
- **Review fix — code-review bundle file list:** skill step 4 lists bundle file names once, by pointing at the bundle's generated `README.md`, so the list cannot drift from the script's `AREAS`.
- **Commit list:** the bundle script emits `commits.txt` (`git log <ref>..HEAD --oneline`) in `--ref` mode, so the reviewers' brief is identical in both modes.

## Unresolved decisions

None.
