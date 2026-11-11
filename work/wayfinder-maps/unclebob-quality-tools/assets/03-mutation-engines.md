# Research: which mutation engine works here, and what it shows

Date: 2026-10-31. Ticket: [03](../issues/03-research-mutation-engines.md).
Scratch evidence: `tmp/mutation-03/` (driver, site lists, run output), `tmp/mutation-wt/` (linked
worktree, removed after the runs). Both are git-ignored.

Claims carry a label: **known** (read in a source, or measured in this session), **inferred** (a
conclusion from the measurements), **guessed** (a belief with no evidence yet).

## Where the runs happened, and what that limits

**All runs were in a linked git worktree**, `tmp/mutation-wt`, at `HEAD bf3b87a5`, locked, detached.
Other agents build and test in the main checkout at the same time, and one of them runs
`python build.py --coverage`. **No mutant ever touched a file of the main checkout**: the main
`src/` was clean at the start, and the two files that are modified at the end of the session
(`src/lib.rs`, `src/domain/model/state/generation_status.rs`) are another agent's work, not a
mutation result. Every write of this session went to `tmp/` and `.scratch/` (**known** from the
commands run).

- The worktree target was seeded from the main target: `Seeded 6771 files (2.2 GiB)` (known).
- The main checkout advanced from `bf3b87a5` to `6ea6bf6a` while the runs were going on (62 files,
  `docs/` and `scripts/` mostly). Neither target file changed in that range, so the results describe
  both commits for these two files (**known**: `git diff --name-only bf3b87a5..HEAD`). Every result
  below names `bf3b87a5`, the commit of the worktree.
- Every cargo step took the machine-wide build slot, either through the worktree's own `build.py` or
  through a `python scripts/build_slot.py` wrapper around `cargo-mutants` (**known**).
- **What this changes for a real in-place run in the shared main tree:** the two hazards below do
  *not* show up in this session, because the worktree hides them.
  1. `cargo-mutants --in-place` edits tracked files of the tree it runs in. In the main tree every
     other agent reads a mutated `src/` during the run, and a crash leaves it mutated (see *Safety*).
  2. `cargo-mutants` runs bare `cargo`, without the environment `build.py` sets. The lld linker
     variable is part of every cargo fingerprint (`ENVIRONMENT.md`, *What forces a rebuild*), so a
     bare cargo run recompiles every unit of the target dir. Measured here at **251 s** for the first
     baseline, then 12 s for the same baseline once `CARGO_TARGET_X86_64_UNKNOWN_LINUX_GNU_LINKER`
     pointed at `~/.cache/chronicler-engine/lld-linker.sh` (**known**). In the shared main tree that
     rebuild would hit every agent, and the next `build.py` run would rebuild it back.

## Verdict

Both engines run on this codebase, and both need help to be safe here.

**cargo-mutants 27.1.0** is the richer engine: it mutates whole function bodies, `match` arms and
operators, and it reports `unviable` mutants separately. It is also the more expensive one: about
24 s per mutant on `gate.rs`, and **43 % of its `gate.rs` mutants cannot build** (37 of 87), so
nearly half of its build time is spent on mutants that never reach a test. It has no build-slot
support, no tree proof in its output, and it leaves a mutated file behind after `SIGKILL`.

**A throwaway driver over `build.py test-pattern`** (mutator's operator site set, in-place, hash
checked, restore checked) costs about 12 s per mutant, holds the slot per run, restores from its own
backup, and records the tree it ran on. It cannot make body-replacement or `match`-arm mutants.

Neither engine found a real test gap in the two files. The ten surviving mutant instances (6 from
cargo-mutants, 4 from the driver) are all equivalent (**inferred**, argument per mutant below).

| Run | Mutants | Caught | Missed | Unviable | Timeout | Wall |
|---|---|---|---|---|---|---|
| cargo-mutants, `parser.rs` (1 run) | 19 | 12 | 4 | 3 | 0 | 222 s |
| cargo-mutants, `gate.rs` (3 shard runs, all 87) | 87 | 48 | 2 | 37 | 0 | 2,131 s |
| driver, `parser.rs` (mutator sites) | 2 | 2 | 0 | 0 | 0 | 230 s* |
| driver, `gate.rs` (mutator sites) | 24 | 17 | 4 | 3 | 0 | 301 s |

\* 217 s of the 230 s is the first-run rebuild of the worktree against the lld linker state; the
steady-state cost is **13 s** for the second site.

Per-mutant cost, baseline subtracted (**known**, from `outcomes.json` phase durations):

| Engine / file | build | test | total | note |
|---|---|---|---|---|
| cargo-mutants / `parser.rs` | ~10 s | ~1 s | ~11 s | filter: lib tests only |
| cargo-mutants / `gate.rs` | ~9 s | ~15 s | ~24 s | filter: lib + `http` binaries |
| driver / `parser.rs` | ~7 s | ~2 s | ~9 s | lib tests, fail-fast |
| driver / `gate.rs` | ~9 s | ~3 s | ~13 s | `http` binary, fail-fast |

The driver is faster because `build.py` adds `--fail-fast`: a killed mutant stops at the first red
test. cargo-mutants runs the whole filter for every mutant.

## Engine A: cargo-mutants

**Install** (**known**): `cargo install cargo-mutants --locked` → 27.1.0, about 3 min of compile plus
a 66 s wait for the slot. Binary lands in `~/.cargo/bin`.

**Command actually used** (**known**):

```bash
# parser.rs, lib unit tests only
cargo mutants --in-place --test-tool nextest -C --lib \
  -f src/application/agents/quantifier/utils/parser.rs \
  --minimum-test-timeout 120 --output ../mutation-03/cm-parser.out \
  -- quantifier::utils::parser

# gate.rs, lib + http binaries, all 87 mutants in three runs
cargo mutants --in-place --test-tool nextest -C --lib -C --test=http \
  -f src/application/generation/gate.rs --shard 0/3 \
  --minimum-test-timeout 180 --output ../mutation-03/cm-gate-shard0.out
```

The environment `CARGO_TARGET_X86_64_UNKNOWN_LINUX_GNU_LINKER=~/.cache/chronicler-engine/lld-linker.sh`
was exported for every run, to match `build.py` and to keep the target warm (**known**).

**Facts about the tool that decide whether it fits** (**known**, from `--help` and from the runs):

- `--in-place` cannot be combined with `--jobs`. One mutant at a time, by construction.
- Extra args after `--` go to the test tool; `-C` args go to cargo. `-C --lib -C --test=http` was
  the way to keep the browser binary out of the loop.
- The test timeout is set from the baseline run: 120 s for `parser.rs`, 180–224 s for `gate.rs`.
- `mutants.out/` holds `outcomes.json` (per-phase durations), `diff/`, `log/`, and the
  caught/missed/unviable/timeout lists. `.gitignore` line 252 already ignores `mutants.out/*`.
- `mutants.out/lock.json` records **version, start time, hostname and username only**. There is no
  git commit and no source hash anywhere in the output.

**Mutant kinds** (**known**, from `--list`):

| File | body → `Ok(<default>)` | operator | body → constant/empty | body → `Default::default()` | delete `!` | delete `match` arm |
|---|---|---|---|---|---|---|
| `parser.rs` | 6 | 2 | 6 | 2 | 1 | 2 |
| `gate.rs` | 35 | 19 | 15 | 15 | 2 | 1 |

**Why 37 `gate.rs` mutants are unviable** (**known**): most of them replace a function body with
`Ok(Default::default())` or `Default::default()`, and the types have no `Default` impl:
`AcceptedJob`, `AutoplayAccept`, `JobPlan`, `Step`, `MovementJson`. The mutant fails to compile, and
cargo-mutants counts it as unviable, not as a test signal.

## Engine B: throwaway driver over `build.py test-pattern`

**Design** (**known**; `tmp/mutation-03/driver.py`, about 180 lines, not in `scripts/`):

1. Read the site list and the original file; hash the original (sha256, 16 hex chars).
2. Write `<file>.orig` and `<file>.pending`, apply the mutant at mutator's byte offsets, then assert
   the applied file's hash equals the expected mutant hash.
3. Take the build slot and run `python build.py test-pattern "<pattern>" --test-target <bin>` in the
   worktree. Outranks: exit 0 = survived, `error[E` / `could not compile` = unviable, other non-zero
   = killed, timeout = timeout.
4. Restore from memory, assert the restored hash equals the original, delete `.orig` and `.pending`,
   and record the result with `seconds`, the log tail and the tree fingerprint.
5. `--recover` puts `<file>.orig` back and cleans the markers, for a killed run.

The site set comes from **mutator's own parser**: `mutator.sites.discover_raw(source, "rust", path)`
in the checkout at `tmp/unclebob/mutator`, run with the venv's `tree_sitter`. That is mutator's Rust
rule set, not a reimplementation (**known**).

**Site set** (**known**): `parser.rs` 2 sites (`delete !` at 49, `+`→`-` at 119); `gate.rs` 24 sites
(12 constant `0`/`1`, 6 equality, 2 boolean, 2 unary, 1 arithmetic, 1 comparison).

**Cost** (**known**): `gate.rs` 24 sites in 301 s (12.5 s each, `http` binary filter, fail-fast);
`parser.rs` 13 s per site once warm.

## The surviving mutants, annotated

cargo-mutants left 6 missed mutants, and the driver left 4 surviving sites. **None is a real test
gap.** The 4 `parser.rs` mutants are equivalent and reveal dead code. The other 6 are equivalent with
no consequence.

| # | Mutant | Class | Annotation |
|---|---|---|---|
| 1 | `parser.rs:115` `extract_json_from_code_fence` → `None` | equivalent | `try_parse_json_full` falls through to the brace scan (`find('{')`..`rfind('}')`), which parses the same fenced JSON. |
| 2 | `parser.rs:115` → `Some(String::new())` | equivalent | Empty content fails to parse, so the brace scan runs and gives the same answer. |
| 3 | `parser.rs:115` → `Some("xyzzy".into())` | equivalent | Same as 2: bad content, then the brace scan. |
| 4 | `parser.rs:119` `start_idx + start_marker.len()` → `start_idx * start_marker.len()` | equivalent | A wrong slice start makes the fence parse fail, so the brace scan runs again. For a fence at index 0 the product is 0 and the slice starts on the marker, which also fails to parse. |
| 5 | `gate.rs:791` `JobWriter::job_id` → `0` and → `1` (2 mutants) | equivalent | Every caller passes the id to a `tracing` span or a log line (`job.rs:124,157,206,382,388,395,404`). The safety check in `JobWriter::hold` uses `self.id`, not the accessor, so behaviour is unchanged. |
| 6 | `gate.rs:328,329,336,347` `AtomicU64::new(0)`→`1` and `wrapping_add(1)`→`wrapping_add(0)` (driver only, 4 sites) | equivalent | Job ids and Autoplay run ids only need to differ per call and to match in the holder. Nothing tests for a zero id, and `AutoplayRunId` is compared for equality only. |

**What survives says more about the code than about the tests:**

- Mutants 1–4 show that `extract_json_from_code_fence` and its branch are **redundant**: the brace
  scan covers every input the fence branch accepts. `test_parse_json_with_markdown_code_fence` passes
  with the whole fence branch deleted. That is Uncle Bob's target case — code that tests do not need
  — not a missing test. **Recommendation: delete the fence branch (about 13 lines) instead of adding
  a test.**
- Mutants 5–6 show that ids are used as opaque uniqueness tokens, not as numbers with meaning. That is
  a fair design; no test can distinguish the mutants (**inferred**).
- No surviving mutant hides a behavioural gap in `GameWriter` or `JobWriter`, including the Autoplay
  and Job-writer paths that the 2026-10-22 incidents touched. The gate is well covered (**inferred**
  from 48 caught of 50 viable `gate.rs` mutants, and 17 of 21 viable driver sites).

## Safety check against the ticket's four requirements

| Requirement | cargo-mutants `--in-place` | Throwaway driver |
|---|---|---|
| Restore after a crash | **No.** `SIGKILL` during a mutant left `M src/.../parser.rs` with the marker `/* ~ changed by cargo-mutants ~ */`, and nothing else. A normal exit does restore. | **Yes.** `SIGKILL` left the mutated file plus `<file>.orig` and `<file>.pending`; `--recover` restored the file to sha `040675a0524df036`, the value recorded before the run. |
| Proof of the tree it ran on | **No.** `lock.json` holds version, time, hostname, username. `outcomes.json` holds commands and durations. No commit, no source hash. Incident 2026-10-22 #1 (a stale green result) is therefore still possible. | **Yes.** The run record holds `git rev-parse HEAD`, the sha256 of `git status --porcelain` before and after every mutant, the source sha before and after, and the restored sha per mutant. Both fingerprints were identical (`e3b0c44298fc1c14`) at the start and the end of both runs. |
| No `git stash` | **Yes**, by design: it edits the working file and reverts it. | **Yes**: it writes the file, then writes the original back from memory and from `.orig`. |
| No collision with another agent's build | **No, not on its own.** It runs bare cargo, takes no build slot, and changes the linker fingerprint of the target dir it runs in. A wrapper (`scripts/build_slot.py`) plus the lld variable is required. | **Yes**, built in: every test run takes the slot; `--hold-whole-run` takes it once for a batch. |
| Interrupt behaviour | Leaves the mutated file (above). The build-slot flock is released by the kernel, so the next build is not blocked. | Leaves the file plus recovery markers, and a clean `--recover` path. |

Both engines write only inside the tree they run in, so a mismatch between engine and tree cannot
happen — but a mismatch between a run's *result file* and the current tree can, for cargo-mutants.

## What this implies for a tool in `build.py`

1. **Run it in a worktree, or not at all in a shared tree.** A worktree costs one seed (344 s
   measured in `ENVIRONMENT.md`) and removes both shared-tree hazards.
2. **cargo-mutants needs four settings here**: `--in-place`, `--test-tool nextest`, the lld linker
   variable, and a `-C --lib -C --test=<bin>` filter that keeps the browser binary out. A fifth is
   `--output` outside the tree, or the existing `mutants.out/*` ignore is relied on.
3. **The slot must be held around the whole run**, because cargo-mutants cannot be told about it.
   A run of one file takes 4–12 min, so a hold stays under the 1800 s fail-open limit only while the
   run stays on one file.
4. **A tree stamp is still needed.** `build.py` could print `git rev-parse HEAD` and the
   `git status --porcelain` hash into the step log and the report, which closes incident #1 for both
   engines at no cost.
5. **Value is thin at this scope.** 106 mutants and 26 sites on 2 files, 10 surviving instances, all
   equivalent. The one real finding (the redundant code-fence branch) is a simplification, not a
   defect.

## Open questions for the value decision

- Is "all survivors are equivalent" a good enough result to justify 24 s per mutant on `gate.rs`, and
  4–12 min per file?
- Which mutant kinds earn their cost here: operator-only (cheap, thin) or body replacement (rich, 43 %
  unviable)?
- Where does a mutation run belong: an on-demand subcommand, a report-only step scoped to changed
  functions (cargo-mutants `--iterate`, `--in-diff`), or nowhere?
- Does the shared-tree safety question stay open even if the tool is never a gate?
