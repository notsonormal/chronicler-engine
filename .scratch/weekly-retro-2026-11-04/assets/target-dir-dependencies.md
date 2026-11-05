# Target dir dependencies

Ticket: [Find what depends on the target dirs inside the checkout](../issues/02-research-target-dir-dependencies.md). Read on 2026-11-04 at `HEAD` `20d6d23c`. Nothing was changed, built or run (no `build.py`, no cargo). Status labels: known (read or measured here), inferred, guessed.

## Answer

- **Move the target dirs to `~/.cache/chronicler-engine/target/<checkout key>/<name>`.** The key is the checkout path with `/` replaced by `-`, for example `workspace-chronicler-engine`. Known: `build.py` already keeps a machine-wide cache dir there (`~/.cache/chronicler-engine/lld-linker.sh`).
- **The move edits 12 existing files and adds 1 module:** 5 code files, 3 test files, and 4 files with text only (2 skills, `ENVIRONMENT.md`, 1 plan). The table lists 13 code sites, 12 test sites and 20 text sites. One new module holds the location, so the rule has one home.
- **The one change that is easy to miss:** `_cargo_env_for` (`build.py:727`) sets `CARGO_TARGET_DIR` only for `--target-dir`. If only `_target_paths` changes, cargo still builds in `target/`, and `build.py run`, seeding and the pruning look in the wrong place.
- **`/workspace` and `/home/node` are two mounts, so a move between them is a copy, not a rename.** Known: `/proc/self/mountinfo`. It is the same device (`259:2`), but `rename` and `link` between two mount points fail with `EXDEV` (inferred from the Linux manual; not tested). Nothing in the repo needs a cross-mount link.
- **The move makes 2 of the 5 permission rules unnecessary:** `grep * . *` and `grep * ./ *`. The other 3 stay (reasons below). Only the owner edits the config.
- **Four statements of the ticket, the map and the report need a correction** (section "Corrections").

## File system facts

| Fact | Value | Status |
|---|---|---|
| Device of `/workspace` and `/home/node` | `/dev/nvme0n1p2`, ext4, device id `259:2`, 87 GB free, 11 % of inodes used | known (`df`, `stat`, `df -i`) |
| Mount points | Two bind mounts of two host dirs (`.../ai-dev/workspace`, `.../ai-dev/home`), mount ids 709 and 713 | known (`/proc/self/mountinfo`) |
| Rename or hard link between them | Fails with `EXDEV`. `mv` falls back to copy and delete | inferred (Linux manual, `rename(2)`) |
| `/tmp` | The container overlay root. A different file system. Local to the container | known |
| Size of `target/` | 28.3 GB in 65,748 files: `rf-coverage` 6.2, `test-police` 6.2, `crap01` 6.1, `debug` 6.0, `ui-investigator` 1.4, `lt11` 1.2, `lt15` 1.2, `nextest` 56 KB | known (`du`) |
| The rest of the checkout | 3.4 GB. `tmp/` 2.7 GB, `.scratch/` 654 MB, `.git` 93 MB, `logs/` 25 MB. 38,179 files outside `.git` | known (`du`, `rg --files`) |
| Space for the move | Peak of about 34 GB: the old 28 GB, and a seeded copy of about 6 GB (inferred from the size of `target/debug`). A copy of everything would need 28 GB more. 87 GB is free | known (free space); inferred (seed size) |
| Hard links | Cargo hard-links uplifted files inside one target dir (`remove_worktrees.py` `dir_size`). Seeding copies with `shutil.copy2` and recreates symlinks. No script calls `os.link` | known (`rg`) |
| Relative paths into `target/` | `tests/test_utils/server.rs` (fallback), `scripts/trial.py:40`, `build.py:1054`, `scripts/parse_coverage.py`, 3 skills | known (table) |
| Absolute paths inside `target/` | Seeded build outputs of 4 C crates hold `-L` paths. A relink ignores them. Measured 2026-10-03 (`ENVIRONMENT.md:46`) | known (cited) |
| Cargo can move the intermediate files alone | `build.build-dir` is stable since cargo 1.91. The repo pins 1.88.0 (`rust-toolchain.toml`) | known |
| Linked worktrees now | None. `git worktree list` shows the main tree. `/workspace/ce-wt` is empty | known |
| `~/.cache` now | `sccache` 11 GB, `chronicler-engine/lld-linker.sh`. `XDG_CACHE_HOME` is unset | known |
| A walk of the checkout without `target/` | `rg -c` over the 3.4 GB took 14 s (one run, warm cache unknown). `grep -r` was not run | known for `rg`; guessed for `grep` |
| Does a rebuilt container keep `/home/node`? | `ENVIRONMENT.md:56` says a rebuilt container drops `~/.cargo/bin`. The mount table shows `/home/node` as a host dir | I don't know. The two disagree |

## Earlier decisions

I found no decision to keep the target dirs inside the checkout. I searched `.scratch/_closed/`, `old-docs/`, `plans/` and the git log (`git log -S`, `--grep=target`). The `--target-dir target/<name>` form is already in the first root-level commits (`1feb7a1c`, `d8644394`, which only moved files). I did not find its origin. Four records bear on the move:

- **`.scratch/_closed/agent-workflow-friction` (T13).** A `--target-dir` tree is rebuildable debris. Cleanup stays at the target dir. The move keeps this: `--cleanup` still removes only a target dir.
- **The same map, ticket 07 (isolated probe server).** The probe DB is `<exe dir>/chronicler_<port>.db`, so a separate `--target-dir` gives the probe its own DB. This still holds under the new root. Only the path in the skill and in `trial.py` changes.
- **`.scratch/_closed/ui-verification-redesign`, ticket 13.** Worktrees are "not a supported workflow"; the sanctioned mechanism is `--target-dir`. Later commits (`93d160e9` seeding, `d067420b` pruning) added worktree code anyway. There are no worktrees now. The move keeps that code working and does not extend it.
- **`75919da6` (2026-10-09).** It added the four `grep` rules as a workaround for the size of `target/`. Its message holds no decision about where the dirs live.

## Proposal

### Location

| Option | Move cost | Same mount as the checkout | Verdict |
|---|---|---|---|
| **A.** `~/.cache/chronicler-engine/target/<key>/<name>` | Copy, not rename | No | **Recommended.** It reuses a cache root that `_lld_linker_env` already computes. A repo-root `grep`, `rg` and `find` never enter it |
| B. `/workspace/.chronicler-target/<key>/<name>` | Instant rename | Yes | Works. The gain (a rename) is small because the plan below does not move the old dirs. A search of `/workspace` still walks 28 GB |
| C. `/tmp/...` | Copy | No (overlay) | Rejected. It is lost when the container restarts |
| D. Cargo `build.build-dir` | None | n/a | Not possible. It needs cargo 1.91, and the repo pins 1.88.0 |
| E. A symlink `target -> ~/.cache/...` | None in `build.py` | n/a | Rejected. `.gitignore` `target/` does not match a symlink, so git shows it as new. `grep -R` and `find -L` follow it. `remove_worktrees.py` refuses a symlink |

### Design

1. **One new module** (`scripts/target_dirs.py`, name to decide) holds the rule: `target_root(checkout)` and `resolve(checkout, name)`. An environment variable `CHRONICLER_TARGET_ROOT` moves the root (for tests). Four callers use it: `build.py` `_target_paths`, `target_seed._warm_candidates`, `remove_worktrees.prune_target`, and `trial.py`.
2. **Layout:** `<root>/<key>/default/{debug,release}` for a run with no flag. `<root>/<key>/<name>/...` for `--target-dir <name>`.
3. **The flag:** a bare name is a dir under the root. `target/<name>` is an alias of `<name>`, with a one-line note. Any other path is used as given. The alias keeps the 2 skills and every habit working, and it means the old text does not break at once. Trade-off: the alias hides the real location from the agent that types it. So `build.py` must log the resolved dir on every run (`build.py:961`).
4. **`CARGO_TARGET_DIR` is set on every run.** This also makes `tests/test_utils/server.rs` find the binary with no change.
5. **`target/` stays, but small.** It keeps `target/nextest/` (56 KB), because nextest ignores `CARGO_TARGET_DIR` for the JUnit file. Known: no `nextest` dir exists inside any named target dir. So `.gitignore` and `arch-lint.toml` stay.
6. **`--cleanup` changes.** Today it removes `target/`, so it also removes every named dir. After the move it removes only `default`. This is closer to T13.

### Steps

1. **Task (one session):** the module, the 4 callers, `_cargo_env_for`, the tests, the docs. The table lists each place.
2. **Add tests:** the alias, the key, the variable always set, the old-dir scan in seeding, the removal of an external dir with its tree.
3. **Owner, after the green gate:** run one gate in the new dir. Seeding copies the third-party files from `target/debug` (about 344 s; cold is about 728 s, `ENVIRONMENT.md`).
4. **Owner:** delete the old dirs. They are idle since 2026-10-31 to 2026-11-02 and are rebuildable. Check `python scripts/build_slot.py --status` first, and check that no cargo process runs. This frees 28 GB.
5. **Check:** `du -sh target` is under 1 MB. Watch it for a week. A raw `cargo` or an editor build regrows it. I did not find a running `rust-analyzer` or cargo process (`ps`, 2026-11-04). I do not know whether an editor starts one.
6. **Owner:** edit the permission config (next section).

### What breaks if only `_target_paths` changes

| Place | Result |
|---|---|
| `build.py:727` | Cargo keeps building in `target/`. `build.py run` execs a binary that does not exist. The stamp goes to an empty dir |
| `target_seed.py:129` | No warm sibling is found. Each cold build takes about 728 s, not 344 s |
| `remove_worktrees.py:239` | Nothing is pruned. A removed tree leaves its external dir |
| `scripts/trial.py:40` | The default DB is not found |
| `ui-investigator` skill, line 29 | It gives a wrong DB path |
| `test_build_cli.py:333,338,639` | 3 tests fail |
| `test_remove_worktrees.py`, `test_target_seed.py` | 18 hit lines of fixtures build `<tree>/target`. They fail or test only the old dir |

## Permission rules

Counts come from `assets/local-only/data/tool_errors.jsonl` (the extract of 2026-10-28 to 2026-11-04). The tooling findings report 42 and 23 for two of them. I do not know the cause of the gap (guessed: a different window).

| Rule (exact glob) | Denials | After the move | Reason |
|---|---|---|---|
| `grep * . *` | 39 | **Unnecessary. Remove.** | Its harm was a walk of `target/`. A parse of 42 found 0 that walk the repo root (`grep -c . file`). The glob was wrong before the move. It stays wrong after |
| `grep * ./ *` | 0 | **Unnecessary. Remove.** | Same reason. It never fired |
| `grep * /workspace/chronicler-engine *` | 27 (26 walk the root) | **Stays.** Change the reason text | A root walk is 3.4 GB, not 31 GB. `tmp/` (2.7 GB) is most of it. A shorter walk is slow, not a hang, but I did not time `grep`. Revisit when R1 shrinks `tmp/` |
| `grep * /workspace/chronicler-engine/ *` | 0 | **Stays** with the rule above | Same hazard |
| `find / *` | 14 | **Stays.** Change the reason text | The move does not remove the hazard. `find /` would walk `~/.cache/chronicler-engine/target` (28 GB) and `~/.cache/sccache` (11 GB). Allow `-maxdepth` (tooling findings, §2) |
| `timeout`, `timeout *` | 21 | Stays. Not related | The reason is a preference, not the size of `target/` |
| `*.env`, `*.env.*` | 7 | Stays. Not related | By design |

## Corrections

- **The ticket says the 5 rules "denied 108 calls".** The 108 includes `timeout *` (21) and `*.env` (7). The 5 target rules denied 80 calls in the extract: 39, 27 and 14. Two of the five never fired.
- **The report (R6) says "then the grep rules can go".** The move makes 2 of the 4 grep rules unnecessary. The 2 absolute-path rules guard a root that still holds 3.4 GB. The `find` rule stays.
- **The map says `/workspace` and `/home/node` share one device.** True, but they are two mounts. A rename or a link between them fails.
- **The rule reason says "~10 GB".** The size is 28.3 GB. The reason text needs a change for every rule that stays.

## Follow-ups proposed

- **Task (AFK, one session):** implement the move (module, 4 callers, tests, docs). Blocked by the owner's decision in ticket 04.
- **Owner action:** the permission edits above, after the `du -sh target` check.
- **Fog:** does `/home/node` survive a container rebuild? `ENVIRONMENT.md:56` and the mount table disagree. It decides how much the cache dir is worth keeping.
- **Fog:** `tmp/` is 2.7 GB and is the largest dir left in the checkout. R1 decides its retention. That decision also decides when the two absolute-path `grep` rules can go.

## Reference table

196 hits of `rg -n -i` over `build.py`, `scripts/`, `tests/`, `.config/`, `.cargo/`, `.gitignore`, the root docs, `docs/`, `.agents/skills/`, `.pi/extensions/pi-permission-system/`, `.pi/agents/` and `arch-lint.toml`. The pattern is:

```
target/|target[-_ ]dirs?\b|target directory|CARGO_TARGET|_target_paths|_NEXTEST_DIR|"target"|--target-dir|llvm-cov/
```

Excluded: the cargo-metadata fixtures under `scripts/tests/fixtures/build_output/` (their `"target":` is a cargo field), `.pi/skills/impeccable/` (third-party; an earlier broad search found only the words "target directly"), and `plans/` (a symlink to `docs/plans/`, so each hit repeats). `.cargo/config.toml` has no hit. It sets only the rustc wrapper and the profiles. The home-level agent config has no live reference: only logs and old delegate results match.

Kinds: `code` must change in code. `test` must change in a test. `doc` must change in text. `perm` is a permission rule (owner only). `none` needs no change. `false` is a hit that is not about a cargo target dir.

Totals: 13 code sites, 12 test sites, 20 doc sites, 5 permission rules. A site is one row group of the same cause.

### `build.py` (67 hits)

| Line | Kind | Hit | What it assumes | Change |
|---|---|---|---|---|
| 13 | none | `asset copy, no SQLite cleanup. --target-dir and --fail-...` | Docstring names the flag. | None. The flag stays. |
| 27 | none | `(lld linker, chosen target dir) and switching between a...` | Docstring: the chosen target dir. | None. |
| 54 | none | `from different checkouts run one at a time. A cold targ...` | Docstring: a cold dir is seeded from a warm sibling. | None. Still true. |
| 58 | doc | `Each script's docstring lists its CHRONICLER_* switches...` | Docstring: one dir per checkout, example `target/agent2`. | Change the example to `--target-dir agent2`. Say where the dir lives. |
| 60 | doc | `python build.py --target-dir target/agent2` | Docstring: one dir per checkout, example `target/agent2`. | Change the example to `--target-dir agent2`. Say where the dir lives. |
| 63 | doc | `live process inside (scripts/remove_worktrees.py). It a...` | Docstring: the gate prunes the stale `target/` of linked worktrees. | Reword. The prune reads the new root. |
| 67 | none | `--cleanup deletes this checkout's whole cargo target di...` | Docstring: `--cleanup` deletes this checkout's cargo target dir. | None. Behavior shifts at line 1562. |
| 71 | none | `A full gate takes about 2 minutes on a warm target dir,...` | Docstring: gate time on a warm dir. | None. |
| 306 | none | `"Removing safe worktrees and pruning stale target dirs....` | Step label. `test_build_cli_gate.py` pins it. | None, unless the label is reworded. |
| 311 | doc | `" target dirs of the linked trees that survive, which i...` | Step help: 'multi-GiB target/'. The main worktree's dir is never pruned. | Reword the first part. The main-worktree rule stays. |
| 312 | doc | `" multi-GiB target/ around; the main worktree's target ...` | Step help: 'multi-GiB target/'. The main worktree's dir is never pruned. | Reword the first part. The main-worktree rule stays. |
| 523 | none | `Step subcommands share the top-level --target-dir flag;...` | Docstring: the shared `--target-dir` flag. | None. |
| 553 | code | `"--target-dir",` | The flag takes a path that is relative to the cwd. | Read a bare name as a dir under the new root. Read `target/<name>` as the alias of `<name>`. Use any other path as given. |
| 554 | code | `dest="target_dir",` | The flag takes a path that is relative to the cwd. | Read a bare name as a dir under the new root. Read `target/<name>` as the alias of `<name>`. Use any other path as given. |
| 556 | doc | `help="Custom cargo target directory for isolated builds...` | Help text: 'e.g., target/agent2'. | Change the example to `agent2`. |
| 579 | none | `help="Delete this checkout's target dir and the machine...` | Help text of `--cleanup`. | None. |
| 599 | none | `"--target-dir",` | The same flag, copied to each step parser. | None. |
| 600 | none | `dest="target_dir",` | The same flag, copied to each step parser. | None. |
| 683 | code | `def _target_paths(args) -> tuple[Path, Path]:` | `_target_paths` is the one function that names the default dir. | Call the new module. Fix the docstring. |
| 684 | code | `"""Return (cargo_target_dir, profile target dir) for th...` | `_target_paths` is the one function that names the default dir. | Call the new module. Fix the docstring. |
| 685 | code | `cargo_target_dir = (` | The default is `Path("target")`, relative to the cwd. | Default: `<root>/<checkout key>/default`. A name maps to `<root>/<checkout key>/<name>`. |
| 686 | code | `Path(args.target_dir) if getattr(args, "target_dir", No...` | The default is `Path("target")`, relative to the cwd. | Default: `<root>/<checkout key>/default`. A name maps to `<root>/<checkout key>/<name>`. |
| 689 | code | `return cargo_target_dir, cargo_target_dir / build_profile` | Returns `(dir, dir/<profile>)`. | None. The profile suffix stays. |
| 701 | false | `var = "CARGO_TARGET_" + re.sub(r"[^A-Z0-9]", "_", host....` | `CARGO_TARGET_<host>_LINKER` is the linker variable. | None. |
| 727 | code | `if getattr(args, "target_dir", None):` | `CARGO_TARGET_DIR` is set only when `--target-dir` is given. With no flag, cargo uses `target/`. | **Breaks.** Set `CARGO_TARGET_DIR` on every run. If not, cargo builds in `target/`, and `build.py`, seeding and the tests look in the new dir. |
| 728 | code | `env["CARGO_TARGET_DIR"] = str(Path(args.target_dir).res...` | `CARGO_TARGET_DIR` is set only when `--target-dir` is given. With no flag, cargo uses `target/`. | **Breaks.** Set `CARGO_TARGET_DIR` on every run. If not, cargo builds in `target/`, and `build.py`, seeding and the tests look in the new dir. |
| 732 | none | `def _warn_if_target_locked(cargo_target_dir: Path, cust...` | Takes a path and probes `<dir>/<profile>/.cargo-lock`. | None. It works for any path. |
| 733 | none | `"""Warn when cargo holds the target directory (another ...` | Takes a path and probes `<dir>/<profile>/.cargo-lock`. | None. It works for any path. |
| 734 | none | `if not is_target_locked(cargo_target_dir):` | Takes a path and probes `<dir>/<profile>/.cargo-lock`. | None. It works for any path. |
| 738 | none | `f"WARNING: Target directory {cargo_target_dir} appears ...` | Warning for a custom dir. | None. |
| 744 | doc | `"WARNING: Default target directory (target/) appears to...` | Warning text: 'Default target directory (target/)'. | Change the text. The default is the new dir. |
| 748 | doc | `"         Use --target-dir to build in a unique folder ...` | Advice: `--target-dir target/<unique-name>`. | Change to `--target-dir <unique-name>`. |
| 750 | doc | `both_print("         python build.py --target-dir targe...` | Advice: `--target-dir target/<unique-name>`. | Change to `--target-dir <unique-name>`. |
| 878 | none | `def is_target_locked(target_dir: Path) -> bool:` | `is_target_locked(path)` probes the lock of a path. | None. |
| 879 | none | `"""Check if cargo holds a build lock on a profile dir u...` | `is_target_locked(path)` probes the lock of a path. | None. |
| 885 | none | `handle = target_seed.probe_shared_lock(target_dir / pro...` | `is_target_locked(path)` probes the lock of a path. | None. |
| 954 | none | `"""Check the toolchain, warn about a busy target dir, a...` | Docstring. | None. |
| 958 | none | `cargo_target_dir, _ = _target_paths(args)` | Calls `_target_paths`. `custom` only picks the text of the warning. | None. |
| 959 | none | `custom = bool(getattr(args, "target_dir", None))` | Calls `_target_paths`. `custom` only picks the text of the warning. | None. |
| 961 | doc | `log_status(f"Using custom target directory: {cargo_targ...` | Logs the dir only for a custom one. | Log the resolved dir on every run. An agent needs the path of the probe DB. |
| 962 | none | `_warn_if_target_locked(cargo_target_dir, custom=custom)` | Calls the lock warning. | None. |
| 967 | none | `"""Seed a cold target dir once per run, inside the slot...` | Docstring. | None. |
| 971 | none | `cargo_target_dir, profile_dir = _target_paths(ctx.args)` | Passes `cargo_target_dir.resolve()` and `Path.cwd()` to `seed_if_cold`. | None here. `target_seed._warm_candidates` finds the sources (line 129). |
| 973 | none | `target_seed.seed_if_cold(Path.cwd(), cargo_target_dir.r...` | Passes `cargo_target_dir.resolve()` and `Path.cwd()` to `seed_if_cold`. | None here. `target_seed._warm_candidates` finds the sources (line 129). |
| 979 | none | `_, profile_dir = _target_paths(ctx.args)` | Stamps the profile dir. | None. |
| 1049 | none | `# nextest writes its JUnit report under the workspace r...` | nextest writes the JUnit report to `<repo>/target/nextest/`, whatever `CARGO_TARGET_DIR` says. | None. The path stays in the checkout. Known: no `nextest` dir exists inside any named target dir (checked 2026-11-04). |
| 1050 | none | `# --target-dir says (also under cargo-llvm-cov), and bu...` | nextest writes the JUnit report to `<repo>/target/nextest/`, whatever `CARGO_TARGET_DIR` says. | None. The path stays in the checkout. Known: no `nextest` dir exists inside any named target dir (checked 2026-11-04). |
| 1051 | none | `# target dirs of one checkout share one report file per...` | nextest writes the JUnit report to `<repo>/target/nextest/`, whatever `CARGO_TARGET_DIR` says. | None. The path stays in the checkout. Known: no `nextest` dir exists inside any named target dir (checked 2026-11-04). |
| 1054 | none | `_NEXTEST_DIR = Path("target") / "nextest"` | nextest writes the JUnit report to `<repo>/target/nextest/`, whatever `CARGO_TARGET_DIR` says. | None. The path stays in the checkout. Known: no `nextest` dir exists inside any named target dir (checked 2026-11-04). |
| 1059 | none | `return _NEXTEST_DIR / profile / "junit.xml"` | `_junit_path` joins `_NEXTEST_DIR`. | None. |
| 1302 | none | `_, target_dir = _target_paths(args)` | `_copy_deployment_assets` writes into the profile dir that `_target_paths` returns. | None. The path is longer and absolute. R20 removes the copy. |
| 1303 | none | `target_dir.mkdir(parents=True, exist_ok=True)` | `_copy_deployment_assets` writes into the profile dir that `_target_paths` returns. | None. The path is longer and absolute. R20 removes the copy. |
| 1309 | none | `dest = target_dir / src_name` | `_copy_deployment_assets` writes into the profile dir that `_target_paths` returns. | None. The path is longer and absolute. R20 removes the copy. |
| 1315 | none | `(target_dir / "logs").mkdir(exist_ok=True)` | `_copy_deployment_assets` writes into the profile dir that `_target_paths` returns. | None. The path is longer and absolute. R20 removes the copy. |
| 1318 | none | `log_status(f"  Package ready in {target_dir}/")` | `_copy_deployment_assets` writes into the profile dir that `_target_paths` returns. | None. The path is longer and absolute. R20 removes the copy. |
| 1319 | none | `log_status(f"  Deployment: copy {target_dir}/ folder to...` | `_copy_deployment_assets` writes into the profile dir that `_target_paths` returns. | None. The path is longer and absolute. R20 removes the copy. |
| 1322 | none | `clean_sqlite_dbs(target_dir / "data")` | `_copy_deployment_assets` writes into the profile dir that `_target_paths` returns. | None. The path is longer and absolute. R20 removes the copy. |
| 1326 | none | `cargo_target_dir, _ = _target_paths(ctx.args)` | The coverage JSON goes to `<dir>/llvm-cov/coverage.json`. | None. It follows `_target_paths`. |
| 1327 | none | `json_path = cargo_target_dir / "llvm-cov" / "coverage.j...` | The coverage JSON goes to `<dir>/llvm-cov/coverage.json`. | None. It follows `_target_paths`. |
| 1388 | none | `_, target_dir = _target_paths(args)` | The tail reads `<dir>/data/chronicler.db`. Nothing writes it (dead code, R20). | None for the move. |
| 1389 | none | `db_path = target_dir / "data" / "chronicler.db"` | The tail reads `<dir>/data/chronicler.db`. Nothing writes it (dead code, R20). | None for the move. |
| 1535 | none | `_, profile_dir = _target_paths(args)` | `run_server` builds the `argv` of the binary from the profile dir. | None. `os.execv` takes an absolute path. |
| 1562 | code | `cargo_target_dir, _ = _target_paths(args)` | `--cleanup` deletes `_target_paths` dir. Today that is `target/`, so it also deletes every named dir and `target/nextest`. | No code change. Behavior shifts: it deletes only `default`. Say so in the help text. |
| 1563 | code | `if cargo_target_dir.exists():` | `--cleanup` deletes `_target_paths` dir. Today that is `target/`, so it also deletes every named dir and `target/nextest`. | No code change. Behavior shifts: it deletes only `default`. Say so in the help text. |
| 1564 | code | `log_status(f"Removing build directory: {cargo_target_di...` | `--cleanup` deletes `_target_paths` dir. Today that is `target/`, so it also deletes every named dir and `target/nextest`. | No code change. Behavior shifts: it deletes only `default`. Say so in the help text. |
| 1565 | code | `shutil.rmtree(cargo_target_dir)` | `--cleanup` deletes `_target_paths` dir. Today that is `target/`, so it also deletes every named dir and `target/nextest`. | No code change. Behavior shifts: it deletes only `default`. Say so in the help text. |
| 1567 | code | `log_status(f"Build directory does not exist: {cargo_tar...` | `--cleanup` deletes `_target_paths` dir. Today that is `target/`, so it also deletes every named dir and `target/nextest`. | No code change. Behavior shifts: it deletes only `default`. Say so in the help text. |

### `scripts/remove_worktrees.py` (17 hits)

| Line | Kind | Hit | What it assumes | Change |
|---|---|---|---|---|
| 1 | none | `"""Remove linked git worktrees, but only ones that are ...` | Docstring title. | None. |
| 21 | code | `files, so a tree's own target/ goes with it.` | 'A tree's own `target/` goes with it': the dir is inside the tree. | **Breaks.** After the move, `git worktree remove` leaves the external dir. Delete `<root>/<tree key>/` when a tree is removed. |
| 25 | doc | `survivors are the trees that keep a multi-GiB target di...` | Docstring: `<tree>/target`. The main worktree's dir is kept. | Reword to the external dir. The main-worktree rule stays. |
| 28 | doc | `- it belongs to the main worktree. That checkout is lon...` | Docstring: `<tree>/target`. The main worktree's dir is kept. | Reword to the external dir. The main-worktree rule stays. |
| 37 | doc | `Tree-level vetoes do not carry over: a target dir ages ...` | Docstring: 'Only `<tree>/target` is considered. A dir outside the tree is out of scope.' | Delete the second sentence. Outside is now the rule. |
| 39 | doc | `CARGO_TARGET_DIR or --target-dir outside the tree is ou...` | Docstring: 'Only `<tree>/target` is considered. A dir outside the tree is out of scope.' | Delete the second sentence. Outside is now the rule. |
| 41 | none | `--apply extends the closing count with the pruned targe...` | Docstring: the closing count. | None. |
| 190 | none | `None when the target dir cannot be listed.` | `target_idle_hours(target)` reads the entries of any dir. | None. |
| 194 | none | `# An empty target dir has no entry to time it by; its o...` | `target_idle_hours(target)` reads the entries of any dir. | None. |
| 234 | doc | `A target dir is kept while it is the main worktree's, i...` | Docstring of `prune_target`. | Reword. |
| 236 | doc | `min_age_hours. Any other target dir is idle and unused,...` | Docstring of `prune_target`. | Reword. |
| 237 | doc | `it was pruned and how many bytes that freed; it prints ...` | Docstring of `prune_target`. | Reword. |
| 239 | code | `target = tree.path / "target"` | `target = tree.path / "target"`. Later checks use `target.parent` as the tree (`is_inside(cwd, target.parent)`). | **Breaks.** Without a change, nothing is pruned. Read the dir from the new module. Use `tree.path` in the two `is_inside` checks. |
| 255 | none | `reason = "cannot read target dir"` | Message text. | None. |
| 284 | none | `"refuse trees created or checked out more recently, and...` | Help text. | None. |
| 291 | none | `help="also prune the surviving trees' stale target dirs...` | Help text. | None. |
| 330 | none | `summary += f", {pruned} target dir(s) pruned (freed {si...` | Summary text. | None. |

### `scripts/target_seed.py` (8 hits)

| Line | Kind | Hit | What it assumes | Change |
|---|---|---|---|---|
| 1 | none | `"""Seed a cold cargo target dir with dependency artifac...` | Docstring title. | None. |
| 129 | code | `target = root / "target"` | `target = root / "target"`: it scans `<tree>/target/*/<profile>` of each worktree. | **Breaks.** Without a change, no warm sibling is found and each cold build takes about 728 s, not 344 s. Scan `<root>/<tree key>/*/<profile>`. Scan the old `<tree>/target` too until the first build in the new dir. |
| 158 | none | `return "no warm target dir in any worktree"` | Message text. | None. |
| 249 | none | `target_dir: Path,` | `seed_if_cold(target_dir)` takes the dir as an argument. The staging dir sits inside it, so the `os.rename` stays on one mount. | None. |
| 253 | none | `"""Seed target_dir/<profile> from the best warm sibling...` | `seed_if_cold(target_dir)` takes the dir as an argument. The staging dir sits inside it, so the `os.rename` stays on one mount. | None. |
| 256 | none | `destination = target_dir / profile` | `seed_if_cold(target_dir)` takes the dir as an argument. The staging dir sits inside it, so the `os.rename` stays on one mount. | None. |
| 283 | none | `staging = target_dir / f"{profile}.seeding-{os.getpid()}"` | `seed_if_cold(target_dir)` takes the dir as an argument. The staging dir sits inside it, so the `os.rename` stays on one mount. | None. |
| 286 | none | `log(f"Cold target dir: seeding third-party artifacts fr...` | `seed_if_cold(target_dir)` takes the dir as an argument. The staging dir sits inside it, so the `os.rename` stays on one mount. | None. |

### `scripts/trial.py` (1 hit)

| Line | Kind | Hit | What it assumes | Change |
|---|---|---|---|---|
| 40 | code | `DEFAULT_DB = "target/ui-investigator/debug/chronicler_3...` | `DEFAULT_DB = "target/ui-investigator/debug/chronicler_3001.db"`, relative to the cwd. | **Breaks.** The default DB is not found. Compute the path with the new module. |

### `scripts/parse_coverage.py` (5 hits)

| Line | Kind | Hit | What it assumes | Change |
|---|---|---|---|---|
| 6 | doc | `python scripts/parse_coverage.py --json target/llvm-cov...` | Usage line with `target/llvm-cov/coverage.json`. | Change the example path. |
| 21 | code | `help="Path to JSON coverage file (default: target/llvm-...` | Default `--json` is `target/llvm-cov/coverage.json`. | `build.py` always passes `--json`. Only a hand run uses the default. Read the root from the new module, or make `--json` required. |
| 51 | code | `"target/llvm-cov/coverage.json",` | The list of paths to try, relative to the cwd. | Same as line 21. |
| 52 | code | `"target/llvm-cov/codecov.json",` | The list of paths to try, relative to the cwd. | Same as line 21. |
| 152 | code | `json_path = find_json_file("target/llvm-cov/coverage.js...` | Falls back to `target/llvm-cov/coverage.json`. | Same as line 21. |

### `scripts/build_report.py` (2 hits)

| Line | Kind | Hit | What it assumes | Change |
|---|---|---|---|---|
| 5 | none | `- the nextest JUnit report (target/nextest/<profile>/ju...` | Docstring: the JUnit report is at `target/nextest/`; nextest ignores `CARGO_TARGET_DIR`. | None. Still true. |
| 6 | none | `nextest ignores CARGO_TARGET_DIR for it): the failing t...` | Docstring: the JUnit report is at `target/nextest/`; nextest ignores `CARGO_TARGET_DIR`. | None. Still true. |

### `scripts/sccache-wrapper.sh` (3 hits)

| Line | Kind | Hit | What it assumes | Change |
|---|---|---|---|---|
| 10 | none | `# CARGO_TARGET_DIR is dropped for the sccache child bec...` | The wrapper drops `CARGO_TARGET_DIR` for the sccache child, because sccache hashes its environment. | None. The variable is now always set, so the drop matters on every run. It already holds for named dirs. |
| 15 | none | `if [ -x "$SCCACHE_BIN" ]; then exec env -u CARGO_TARGET...` | The wrapper drops `CARGO_TARGET_DIR` for the sccache child, because sccache hashes its environment. | None. The variable is now always set, so the drop matters on every run. It already holds for named dirs. |
| 16 | none | `if command -v sccache >/dev/null 2>&1; then exec env -u...` | The wrapper drops `CARGO_TARGET_DIR` for the sccache child, because sccache hashes its environment. | None. The variable is now always set, so the drop matters on every run. It already holds for named dirs. |

### `scripts/refine_character_json.py` (2 hits)

| Line | Kind | Hit | What it assumes | Change |
|---|---|---|---|---|
| 51 | false | `target_dir = Path(__file__).resolve().parent.parent / "...` | `target_dir` is `data/characters`. | None. |
| 52 | false | `for file in target_dir.glob("*.json"):` | `target_dir` is `data/characters`. | None. |

### `scripts/tests/build_cli_support.py` (7 hits)

| Line | Kind | Hit | What it assumes | Change |
|---|---|---|---|---|
| 42 | none | `target_dir=None,` | A test argument named `target_dir`. | None. |
| 49 | none | `def make_ctx(testcase, target_dir=None, **args) -> buil...` | A test argument named `target_dir`. | None. |
| 52 | none | `nextest writes its JUnit report under target/ of the re...` | Docstring: nextest writes the report under `target/`. | None. |
| 53 | none | `target dir is, so the report directory is a temp dir an...` | Docstring: nextest writes the report under `target/`. | None. |
| 54 | none | `environment, as --target-dir does.` | Docstring: nextest writes the report under `target/`. | None. |
| 58 | none | `patcher = mock.patch.object(build, "_NEXTEST_DIR", Path...` | Patches `_NEXTEST_DIR` to a temp dir. | None. |
| 61 | none | `env = {"CARGO_TARGET_DIR": target_dir} if target_dir el...` | Puts `CARGO_TARGET_DIR` in the test environment. | None. |

### `scripts/tests/test_build_cli.py` (19 hits)

| Line | Kind | Hit | What it assumes | Change |
|---|---|---|---|---|
| 152 | none | `args = build.parse_args(["--target-dir", "target/agent2...` | Parse tests of `--target-dir target/agent2`. | None. Add one test for the alias. |
| 153 | none | `self.assertEqual(args.target_dir, "target/agent2")` | Parse tests of `--target-dir target/agent2`. | None. Add one test for the alias. |
| 156 | none | `args = build.parse_args(["clippy", "--target-dir", "tar...` | Parse tests of `--target-dir target/agent2`. | None. Add one test for the alias. |
| 157 | none | `self.assertEqual(args.target_dir, "target/agent2")` | Parse tests of `--target-dir target/agent2`. | None. Add one test for the alias. |
| 209 | none | `args = build.parse_args(["--target-dir", "t/a", "run", ...` | Parse tests of `--target-dir t/a`. | None. |
| 210 | none | `self.assertEqual(args.target_dir, "t/a")` | Parse tests of `--target-dir t/a`. | None. |
| 214 | none | `args = build.parse_args(["run", "--target-dir", "t/a", ...` | Parse tests of `--target-dir t/a`. | None. |
| 215 | none | `self.assertEqual(args.target_dir, "t/a")` | Parse tests of `--target-dir t/a`. | None. |
| 315 | none | `def _args(flags, target_dir=None):` | Test helper. | None. |
| 317 | none | `command="run", server_flags=flags, target_dir=target_di...` | Test helper. | None. |
| 333 | test | `argv, ["target/debug/chronicler_engine", "--world", "x"...` | Expects the default binary at `target/debug/chronicler_engine`. | **Breaks.** Expect `<root>/<key>/default/debug/chronicler_engine`. Point the root at a temp dir with an environment variable. |
| 338 | test | `argv, _ = self._serve(self._args(["--port", "3099"], ta...` | Expects `target/agent2/debug/chronicler_engine`. | **Breaks.** Expect the mapped path. Keep one test for the alias and one for a literal path. |
| 339 | test | `self.assertEqual(argv[0], "target/agent2/debug/chronicl...` | Expects `target/agent2/debug/chronicler_engine`. | **Breaks.** Expect the mapped path. Keep one test for the alias and one for a literal path. |
| 406 | none | `SimpleNamespace(command="run", server_flags=[], target_...` | Test argument. | None. |
| 635 | none | `env = build._cargo_env_for(SimpleNamespace(target_dir="...` | `--target-dir` sets `CARGO_TARGET_DIR`. | None. |
| 636 | none | `self.assertIn("CARGO_TARGET_DIR", env)` | `--target-dir` sets `CARGO_TARGET_DIR`. | None. |
| 639 | test | `env = build._cargo_env_for(SimpleNamespace(target_dir=N...` | `test_no_target_dir_keeps_env_clean` asserts that the variable is absent. | **Breaks.** Invert it. The variable is now always set. |
| 640 | test | `self.assertNotIn("CARGO_TARGET_DIR", env)` | `test_no_target_dir_keeps_env_clean` asserts that the variable is absent. | **Breaks.** Invert it. The variable is now always set. |
| 650 | false | `VAR = "CARGO_TARGET_X86_64_UNKNOWN_LINUX_GNU_LINKER"` | The linker variable. | None. |

### `scripts/tests/test_build_cli_gate.py` (2 hits)

| Line | Kind | Hit | What it assumes | Change |
|---|---|---|---|---|
| 46 | none | `"Removing safe worktrees and pruning stale target dirs....` | Pins the step label. | None. |
| 57 | none | `"Removing safe worktrees and pruning stale target dirs....` | Pins the step label. | None. |

### `scripts/tests/test_build_cli_summary.py` (3 hits)

| Line | Kind | Hit | What it assumes | Change |
|---|---|---|---|---|
| 136 | none | `"""nextest ignores --target-dir for the report, so buil...` | Pins that the JUnit path does not follow `--target-dir`. | None. |
| 137 | none | `ctx = make_ctx(self, target_dir="/elsewhere/target/agen...` | Pins that the JUnit path does not follow `--target-dir`. | None. |
| 146 | none | `self.assertEqual(build._junit_path("default"), build._N...` | Pins that the JUnit path does not follow `--target-dir`. | None. |

### `scripts/tests/test_remove_worktrees.py` (15 hits)

| Line | Kind | Hit | What it assumes | Change |
|---|---|---|---|---|
| 43 | none | `# The real repo ignores target/, so a worktree that has...` | The fixture ignores `target/` in the test repo. | None. |
| 46 | none | `(info / "exclude").write_text("target/\n")` | The fixture ignores `target/` in the test repo. | None. |
| 63 | test | `target = tree / "target"` | Fixture helper creates `tree/target/debug/artifact`, and ages it. | Create the dir under a temp root. Inject the root with an environment variable. |
| 70 | test | `"""Backdate every entry under target, deepest first, th...` | Fixture helper creates `tree/target/debug/artifact`, and ages it. | Create the dir under a temp root. Inject the root with an environment variable. |
| 111 | none | `(self.repo / ".gitignore").write_text("target/\n")` | The fixture writes `.gitignore`. | None. |
| 116 | test | `(tree / "target").mkdir()` | Creates `tree/target/artifact`. | Same as line 63. |
| 117 | test | `(tree / "target" / "artifact").write_text("x")` | Creates `tree/target/artifact`. | Same as line 63. |
| 272 | test | `target = self.repo / "target"` | Main-worktree and current-checkout cases use `repo/target`. | Same as line 63. |
| 281 | test | `main_target = self.repo / "target"` | Main-worktree and current-checkout cases use `repo/target`. | Same as line 63. |
| 317 | test | `(tree / "target").symlink_to(elsewhere)` | A symlinked `tree/target` is refused. | Keep for the old dir. Add a case for the new dir. |
| 337 | test | `self.assertNotIn(str(tree / "target"), out)` | Asserts on `tree/target` in the output. | Assert the new path. |
| 373 | test | `self.assertIn(", 1 target dir(s) pruned (freed 4.0 KiB)...` | Asserts the prune lines for `tree/target`. | Same as line 63. |
| 378 | test | `target = tree / "target"` | Asserts the prune lines for `tree/target`. | Same as line 63. |
| 393 | test | `self.assertIn(f"keep    {target}: cannot read target di...` | Asserts the prune lines for `tree/target`. | Same as line 63. |
| 402 | test | `self.assertIn("0 target dir(s) pruned", out)` | Asserts the prune lines for `tree/target`. | Same as line 63. |

### `scripts/tests/test_target_seed.py` (7 hits)

| Line | Kind | Hit | What it assumes | Change |
|---|---|---|---|---|
| 170 | test | `self.src = self.root / "target" / "debug"` | Fixture sources are `root/target/debug` and `root/target/t1`. | Place them under the injected root. Keep one old-dir case. |
| 172 | test | `self.target = self.root / "target" / "t1"` | Fixture sources are `root/target/debug` and `root/target/t1`. | Place them under the injected root. Keep one old-dir case. |
| 211 | test | `poor = self.root / "target" / "poor" / "debug"` | Ranking fixtures under `root/target/<name>/debug`. | Same as line 170. |
| 219 | test | `newer = self.root / "target" / "n1" / "debug"` | Ranking fixtures under `root/target/<name>/debug`. | Same as line 170. |
| 224 | test | `unstamped = self.root / "target" / "raw" / "debug"` | Ranking fixtures under `root/target/<name>/debug`. | Same as line 170. |
| 262 | test | `check_only = self.root / "target" / "check" / "debug"` | Check-only fixture under `root/target/check`. | Same as line 170. |
| 279 | none | `self.assertIn("no warm target dir in any worktree", mis...` | Asserts the miss message. | None. |

### `scripts/tests/test_validate_docs.py` (1 hit)

| Line | Kind | Hit | What it assumes | Change |
|---|---|---|---|---|
| 552 | false | `"'<target-dir>/status/x', 'http/fragment/y'.\n"` | A `<target-dir>` placeholder in a doc sample. | None. |

### `tests/test_utils/server.rs` (7 hits)

| Line | Kind | Hit | What it assumes | Change |
|---|---|---|---|---|
| 199 | none | `// Respect CARGO_TARGET_DIR for concurrent builds with ...` | Reads `CARGO_TARGET_DIR`, else `target`. Runs `<dir>/debug/chronicler_engine`, else `cargo run`. | None for runs through `build.py`: nextest passes the variable. A raw run with no variable runs `cargo run`, which rebuilds into `target/`. Optional: panic with a message. |
| 200 | none | `let target_dir = std::env::var("CARGO_TARGET_DIR").unwr...` | Reads `CARGO_TARGET_DIR`, else `target`. Runs `<dir>/debug/chronicler_engine`, else `cargo run`. | None for runs through `build.py`: nextest passes the variable. A raw run with no variable runs `cargo run`, which rebuilds into `target/`. Optional: panic with a message. |
| 202 | none | `format!("{target_dir}/debug/chronicler_engine.exe")` | Reads `CARGO_TARGET_DIR`, else `target`. Runs `<dir>/debug/chronicler_engine`, else `cargo run`. | None for runs through `build.py`: nextest passes the variable. A raw run with no variable runs `cargo run`, which rebuilds into `target/`. Optional: panic with a message. |
| 204 | none | `format!("{target_dir}/debug/chronicler_engine")` | Reads `CARGO_TARGET_DIR`, else `target`. Runs `<dir>/debug/chronicler_engine`, else `cargo run`. | None for runs through `build.py`: nextest passes the variable. A raw run with no variable runs `cargo run`, which rebuilds into `target/`. Optional: panic with a message. |
| 261 | none | `let db_path = std::path::PathBuf::from(&target_dir)` | DB path is `<dir>/debug/chronicler_<port>.db`. | None. It follows the variable. |
| 558 | none | `let target_dir = std::env::var("CARGO_TARGET_DIR").unwr...` | Same read of the variable, for the DB path. | None. |
| 559 | none | `let db_path = std::path::PathBuf::from(&target_dir)` | Same read of the variable, for the DB path. | None. |

### `.config/nextest.toml` (2 hits)

| Line | Kind | Hit | What it assumes | Change |
|---|---|---|---|---|
| 33 | none | `# <workspace root>/target/nextest/<profile>/junit.xml w...` | Comment: the report is at `<root>/target/nextest/` whatever `CARGO_TARGET_DIR` says. | None. Still true. |
| 34 | none | `# --target-dir says, so the target dirs of one checkout...` | Comment: the report is at `<root>/target/nextest/` whatever `CARGO_TARGET_DIR` says. | None. Still true. |

### `.gitignore` (1 hit)

| Line | Kind | Hit | What it assumes | Change |
|---|---|---|---|---|
| 76 | none | `target/` | `target/` is ignored. | None. Keep it: `target/nextest/` stays. A symlink named `target` is not matched by `target/`. |

### `.pi/extensions/pi-permission-system/config.json` (5 hits)

| Line | Kind | Hit | What it assumes | Change |
|---|---|---|---|---|
| 29 | perm | `"reason": "Repo-root grep walks target/ (~10 GB) and ha...` | Rule `grep * . *`: 'Repo-root grep walks target/ (~10 GB)'. | Owner: remove (see the rule table). |
| 33 | perm | `"reason": "Repo-root grep walks target/ (~10 GB) and ha...` | Rule `grep * ./ *`: same reason. | Owner: remove. |
| 37 | perm | `"reason": "Repo-root grep walks target/ (~10 GB) and ha...` | Rule `grep * /workspace/chronicler-engine *`: same reason. | Owner: keep. Change the reason text. |
| 41 | perm | `"reason": "Repo-root grep walks target/ (~10 GB) and ha...` | Rule `grep * /workspace/chronicler-engine/ *`: same reason. | Owner: keep with the rule above. Change the reason text. |
| 45 | perm | `"reason": "find / walks target/ (~10 GB) and hangs. Use...` | Rule `find / *`: 'find / walks target/ (~10 GB)'. | Owner: keep. Change the reason text. |

### `arch-lint.toml` (1 hit)

| Line | Kind | Hit | What it assumes | Change |
|---|---|---|---|---|
| 14 | none | `"**/target/**",` | Excludes `**/target/**` from the `src` scan. | None. |

### `.agents/skills/chronicler-after-plan-workflow-plus-review/SKILL.md` (1 hit)

| Line | Kind | Hit | What it assumes | Change |
|---|---|---|---|---|
| 23 | none | `11. Run the 4 review subagents — '/thermo-nuclear-code-...` | 'its own target dir (see `ENVIRONMENT.md`)'. No path. | None. |

### `.agents/skills/chronicler-ui-investigator/SKILL.md` (3 hits)

| Line | Kind | Hit | What it assumes | Change |
|---|---|---|---|---|
| 25 | doc | `python build.py run --target-dir target/ui-investigator...` | The command is `--target-dir target/ui-investigator`. | Works through the alias. Change to `--target-dir ui-investigator`. |
| 29 | doc | `'build.py run' replaces itself with the server, so star...` | The DB path is `target/ui-investigator/debug/chronicler_3001.db`. | **Breaks.** The path is wrong. Give the new path: `<root>/<key>/ui-investigator/debug/chronicler_3001.db`. |
| 112 | doc | `\| World JSON edits do not appear \| The server reads '<t...` | 'The server reads `<target-dir>/debug/data`'. | Say where `<target-dir>` is. |

### `.agents/skills/test-police/SKILL.md` (2 hits)

| Line | Kind | Hit | What it assumes | Change |
|---|---|---|---|---|
| 17 | doc | `- You ran 'python build.py --coverage --target-dir targ...` | The command is `--target-dir target/test-police`. | Same as line 25 of the ui-investigator skill. |
| 22 | doc | `- '--target-dir target/test-police' keeps this review's...` | '`--target-dir target/test-police` keeps this review's builds apart'. | Same. |

### `AGENTS.md` (2 hits)

| Line | Kind | Hit | What it assumes | Change |
|---|---|---|---|---|
| 320 | none | `- 'remove_worktrees.py' — Remove linked git worktrees, ...` | Generated lines from the first docstring line of `remove_worktrees.py` and `target_seed.py`. | None. The hook regenerates them if a docstring changes. |
| 321 | none | `- 'target_seed.py' — Seed a cold cargo target dir with ...` | Generated lines from the first docstring line of `remove_worktrees.py` and `target_seed.py`. | None. The hook regenerates them if a docstring changes. |

### `ENVIRONMENT.md` (9 hits)

| Line | Kind | Hit | What it assumes | Change |
|---|---|---|---|---|
| 23 | none | `- **Target seeding** ('scripts/target_seed.py'): a cold...` | Target seeding: a cold dir copies from a worktree with the same signature. | None. |
| 26 | doc | `- **Stale target pruning** ('scripts/remove_worktrees.p...` | Stale target pruning. | Say that the prune reads the new root. |
| 46 | none | `absolute '-L' paths into the source checkout's target d...` | Seeded outputs carry absolute `-L` paths. Removing the sibling dir leaves the seeded profile working (measured 2026-10-03). | None. This supports the move: a moved dir is no worse than a seeded one. |
| 48 | none | `A missing '-L' directory is not an error. Removing the ...` | Seeded outputs carry absolute `-L` paths. Removing the sibling dir leaves the seeded profile working (measured 2026-10-03). | None. This supports the move: a moved dir is no worse than a seeded one. |
| 55 | none | `so a coverage run is a full gate plus the coverage buil...` | Coverage: use a dedicated `--target-dir`. | None. |
| 66 | none | `\| Warm default target dir, no Rust change, '--no-browse...` | Table row: 'Warm default target dir'. | None. |
| 88 | none | `\| Switching '--target-dir' mid-task \| The new dir is co...` | 'Switching `--target-dir` mid-task': the new dir is cold. | None. |
| 89 | doc | `\| A raw 'cargo build'/'cargo run' or IDE build in a 'bu...` | A raw `cargo` in a `build.py` target dir recompiles the tree. | Add: a raw `cargo` with no `CARGO_TARGET_DIR` builds into `target/` again. |
| 95 | doc | `The dev server stores its SQLite DB next to its binary ...` | The dev server DB is `<target-dir>/debug/chronicler_<port>.db`. | Add the location of `<target-dir>`, and add the new-root paragraph. |

### `docs/diataxis/reference/coding_standards/guardrails.md` (1 hit)

| Line | Kind | Hit | What it assumes | Change |
|---|---|---|---|---|
| 33 | none | `Scan scope: 'root = "./src"', excluding '**/target/**',...` | Generated from `arch-lint.toml`: excludes `**/target/**`. | None. |

### `docs/diataxis/reference/coding_standards/integration_test_standards.md` (1 hit)

| Line | Kind | Hit | What it assumes | Change |
|---|---|---|---|---|
| 111 | none | `directory stays under '/tmp', not 'target/', so that a ...` | Port locks stay under `/tmp`, not `target/`. | None. |

### `docs/plans/narration-line-break-tags.md` (1 hit)

| Line | Kind | Hit | What it assumes | Change |
|---|---|---|---|---|
| 13 | none | `- **The source.** One Narration holds the tag: Swipe 30...` | Names the probe DB `target/ui-investigator/debug/chronicler_3001.db` as evidence. | None. The plan records a past state. |

### `docs/plans/parallel-ticket-runs-without-commits.md` (1 hit)

| Line | Kind | Hit | What it assumes | Change |
|---|---|---|---|---|
| 33 | doc | `- **Worktree scripts.** 'scripts/remove_worktrees.py' (...` | '`target_seed.py` seeds a new worktree's cold `target/` dir.' | Optional: drop `target/`. The plan records a past state. |
