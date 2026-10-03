# `build.py run`: one command for the dev server

> **Status:** Accepted — decisions settled in a grilling session on 2026-10-03. Not implemented.
> **Scope:** ~3 SP — one new `build.py` step, removal of the Windows-only port helpers in
> `build.py` and the server, Python and Rust test updates, doc updates.
> **Depends on:** WP2 (build tooling) of `docs/plans/dashboard-ui-updates-followups-plan.md`
> landing first. WP2 edits `build.py` and `scripts/target_seed.py` in the same regions.
> Branch the implementation worktree after WP2 is committed.
> **Blocks:** nothing.
> **Findings owned:** the raw-`cargo` fingerprint hazard recorded in `ENVIRONMENT.md`;
> `.scratch/ponytail-audit-cuts/issues/03-decide-port-killing.md` (resolved by this plan).

## Summary

The dev server starts with raw `cargo run`. Raw cargo does not export the linker
variable that `build.py` exports. Cargo hashes the linker value into the fingerprint of
every unit. Therefore each switch between a `build.py` step and the raw server command
recompiles the whole tree, in both directions.

Add a `run` step to `build.py`. The step builds the binary with the `build.py`
environment inside the machine-wide build slot, releases the slot, then replaces its own
process with the binary. To the caller it behaves like `cargo run`. The switch cost
becomes zero, and the server links with lld.

The same change removes the port-freeing helpers in `build.py` and in the server. They
call Windows tools (`netstat`, `Select-String`, `tasklist`, `taskkill`) and do nothing on
Linux. The server binds once and fails with a clear error on a busy port. `build.py run`
checks the port before it starts.

## Evidence

### Measurement 1 — the mechanism, isolated

A throwaway workspace in `tmp/linkfp2/` has two crates: `dep` (a library) and `app` (a
binary that uses `dep`). Four builds ran against one target directory.

| Build | `CARGO_TARGET_X86_64_UNKNOWN_LINUX_GNU_LINKER` | Units compiled |
|---|---|---|
| First build | not set | `dep`, `app` |
| Same build again | not set | none |
| Build with the variable | `/usr/bin/cc` | `dep`, `app` |
| Build with the variable unset again | not set | `dep`, `app` |

The variable alone invalidates the whole tree. Nothing else changed between the builds.

`dep` is an rlib. An rlib never calls the linker. Cargo still recompiles it, because
cargo computes one configuration hash for each unit and does not separate linking units
from non-linking units. `scripts/lld-linker.sh` already records this rule.

### Measurement 2 — the repository, one target directory

The target directory was in raw-cargo state. Then, on 2026-10-03:

| Command | Wall time | Cause |
|---|---|---|
| `python build.py check` | 97s | Restored the lld fingerprints; check-mode units |
| `python build.py check` again | 1.3s | Warm |
| `cargo build --bin chronicler_engine` with the lld variable | 133s | Build-mode units had been made with GNU ld |
| `python build.py check` again | 1.0s | The build-mode compile did not disturb the check-mode units |
| The same lld `cargo build --bin` again | 0s | Warm |

The last two rows are the important ones. A run-shaped build and a check-shaped build
can share one target directory once both use the same linker value.

### Measurement 3 — the earlier raw run

From `ENVIRONMENT.md`: raw `cargo run` after a gate recompiled 243 crates in 2m11s. The
immediate second raw build was a no-op in 0.85s.

### Facts found during the grilling session (known, 2026-10-03)

- **`argparse.REMAINDER` keeps the `--`.** Tested on Python 3.13.5:
  `run -- --world x --port 9` parses to `['--', '--world', 'x', '--port', '9']`.
  `run --target-dir t/a -- --port 9` sets `target_dir` correctly, so `--target-dir`
  works before or after `run`. `run --world x` without `--` fails with
  "unrecognized arguments".
- **The server has its own defaults.** `src/utils/cli.rs` declares
  `--world redmist_estate`, `--persona julian`, `--port 3000`, `--host 0.0.0.0`.
- **`build.py kill_port()` does nothing on Linux.** It runs
  `netstat -ano | Select-String ':3000'`. Here that exits 127
  (`netstat: not found`, `Select-String: not found`). `kill_by_name()` uses
  `tasklist | findstr` and `taskkill`, with the same result.
- **The server's port freeing does nothing on Linux.** `find_process_on_port()` runs
  `netstat -ano`, which is not installed here, so it returns `None`. `kill_process()` runs
  `taskkill`. The real server passes `bind_attempts: None` (`bootstrap/run.rs`), so
  `bind_with_retry()` retries every 500 ms forever on a busy port. A second server hangs.
  It does not fail.
- **No port tools exist in the container.** `ss`, `lsof`, `fuser`, and `netstat` are all
  missing. Only `/proc/net/tcp` and `/proc/net/tcp6` exist.
- **The server reads a copy of `data/`.** `resolve_engine_data_path()` uses
  `<binary dir>/data` when it exists. The full gate's copy step fills
  `target/debug/data`. A dev server therefore sees the `data/` of the last full gate.
  Raw `cargo run` has the same behaviour.
- **`TMPDIR` is empty here.** `build_slot.lock_path()` falls back to `/tmp`.

### What the measurements rule out

- **Feature unification is not a factor.** `Cargo.toml` declares
  `default = ["testing"]`, `diagnostics = []`, and `testing = ["diagnostics"]`. The
  default feature set therefore equals `--all-features`, so `cargo build` and
  `cargo check --all-features` agree on features.
- **Target selection is not a factor.** `cargo build --bin chronicler_engine` did not
  disturb the check-mode units (Measurement 2, row 4).
- **The first build in a mode is not cheaper.** The 133s build-mode compile costs about
  the same as the 2m11s raw one. The gain is that this cost is paid once, not on every
  switch between the two commands.

## Mechanism

Two independent causes:

1. **The linker variable.** `build.py` exports
   `CARGO_TARGET_X86_64_UNKNOWN_LINUX_GNU_LINKER=<cache>/chronicler-engine/lld-linker.sh`
   through `_lld_linker_env()`. Raw cargo exports nothing. Cargo hashes the value into
   every unit fingerprint, so the two commands invalidate each other.
2. **Check mode and build mode keep separate artifacts.** `cargo check` never feeds
   `cargo build`. The gate already pays this: clippy compiles the tree in check mode,
   then nextest compiles it again in build mode. `ENVIRONMENT.md` records it as "Two
   compiles per dependency".

The `run` step removes cause 1. Cause 2 is a cargo property that no `run` step can
remove, and the gate pays it with or without this plan.

## Decisions

| Topic | Decision |
|---|---|
| Command | `python build.py run [-- <server flags>]`. `build.py` declares no server flags. With no flags, the binary uses its own defaults |
| Passthrough | `argparse.REMAINDER` on the `run` subparser. Strip exactly one leading `--` before the exec. `--target-dir` works before or after `run` |
| Build vs. run | `cargo build --bin chronicler_engine` inside `_cargo_slot`, then `os.execv` on the binary. This is what `cargo run` does internally, with the slot covering only the build |
| Exec position | After `main()` has run its `finally` (summary, banner, log close, history line). The `__main__` block calls `os.execv`, not `run_step` |
| History | One `logs/build_history.txt` line for the build phase. Exit code 0 when the build passes |
| Busy port | A bind test in `build.py`, with `SO_REUSEADDR` set (as tokio sets it on Unix — inferred, confirm during implementation). It runs before the build and again just before the exec. On failure: stop with "port N is in use" and a hint to stop that process or pass `-- --port N`. No PID lookup |
| Port/host source | Read `--port N`/`--port=N` and `--host H`/`--host=H` from the passthrough flags. Else 3000 and `0.0.0.0`, copied from `src/utils/cli.rs` with a comment that points there. Skip the check when the flags contain `--list-worlds`, `--help`, or `-h` |
| Working directory | The repo root. `main()` already changes to it, and it matches the `cargo run` line in `AGENTS.md` |
| Stale `target/debug/data` | Unchanged, the same as `cargo run`. Recorded in Out of scope and in the UI-investigator troubleshooting row |
| `build.py` helpers removed | `kill_port()` and its call in `_gate_prelude`. `kill_by_name()` and the "kill lingering processes" step in `run_cleanup`. `--cleanup` keeps the port-lock and target-dir removal |
| Server helpers removed | `src/adapters/driving/http/utils/port_utils.rs` (the whole module) |
| Server bind | Delete `bind_with_retry()` and `ServerConfig.bind_attempts`. The server binds once with `TcpListener::bind` and returns the existing `EngineError::Config("Failed to bind to port N: …")` |
| Kept | `terminate_pid()` in `tests/test_utils/server.rs`. Its Windows branch sits behind `#[cfg(target_os = "windows")]`, and the Linux branch uses `libc::kill` |
| Hint text | `tests/test_utils/browser.rs` drops "Check with: netstat -ano \| Select-String". The new hint names no tool, because this container has none |
| Tracker | Issue 03 is resolved by this plan. The followups plan DC5 bullet is owned by its own session, and that session has already pointed it here |
| `CONTEXT.md` / ADR | No change. These are build-tool terms, not domain terms, and the decision is easy to reverse |

## Design

### Command surface

```bash
python build.py run                                        # binary defaults: redmist_estate, port 3000
python build.py run -- --world other_world --port 3099     # override server flags
python build.py --target-dir target/agent2 run -- --port 3099
```

- `run` enters `REGISTRY`, so `parse_args` creates the subparser and
  `python build.py --help` lists it.
- `run` does not enter `GATE_ORDER`.
- `--release` stays a gate-only flag, so `run` is dev-profile only. The existing
  `_reject_gate_flags_with_step` rejects `--release run`.

### Flow

1. `main()` parses arguments and changes to the repo root, as today.
2. `run_server(args)` parses port and host from the passthrough flags, and runs the bind
   test. On a busy port it exits non-zero with the message. No build runs.
3. `run_server` builds under `_cargo_slot("cargo build --bin chronicler_engine", …)` with
   `_cargo_env_for(args)`. `_cargo_slot` seeds a cold target directory and stamps it.
4. `run_server` leaves the slot and returns the exec argv:
   `[<cargo_target_dir>/debug/chronicler_engine, *server_flags]`, built from
   `_target_paths(args)`.
5. `main()` stores the argv, then runs its `finally` as today: summary, banner, log close,
   history line.
6. The `__main__` block sees the stored argv and an exit code of 0. It runs the bind test
   again, flushes stdout and stderr, and calls `os.execv`.

No lock is held while the server runs. Ctrl-C, stdin, and stdout behave as with
`cargo run`, because the server replaces the `build.py` process.

### Test seams

No test may start a server. The seams are pure helpers and patched calls in the existing
`mock.patch.object` style of `scripts/tests/test_build_cli.py`:

- the passthrough split (strip one `--`);
- the port/host parser (both `--port N` and `--port=N`, defaults, skip flags);
- the bind test (a test can hold a socket on an ephemeral port and expect "in use");
- the exec argv (target dir before and after `run`);
- the `__main__` exec path, with `os.execv` patched.

### Why not a separate target directory

| Option | Cost |
|---|---|
| A separate target directory for `run` | Cold start: 344s seeded, 728s unseeded. A second copy of the build tree — 35 GB on this checkout. It still needs the lld variable to link fast. |
| The shared target directory with the shared environment | 0s to switch, once built. Faster relinks through lld. sccache reuse. |

The second row is better on every axis. `--target-dir target/<name>` already exists for
an agent that needs isolation, and the `run` step inherits it.

### Known limitation

The server holds no lock, so a concurrent `build.py` gate can replace the binary while
the server runs. On Linux the running process keeps its own inode, so the server
continues to serve the old binary. After this plan, the gate prelude no longer tries to
kill port 3000. Before this plan the kill did nothing on Linux, so behaviour on Linux
does not change.

The bind test and the server's own bind are separate events. Another process can take
the port between them. The server then fails with its bind error, which is the correct
result.

## Steps

1. **Remove the `build.py` port helpers.** Delete `kill_port()`, its call and comment in
   `_gate_prelude`, `kill_by_name()`, and the kill step in `run_cleanup`. Update the
   module docstring ("port-3000 kill", "removes lingering build processes"), the
   `run_step` docstring, the `_gate_prelude` docstring, and the `--cleanup` help text.
   Done when `grep -n "kill_port\|kill_by_name\|taskkill\|Select-String\|tasklist" build.py`
   returns nothing.
2. **Remove the server port helpers.** Delete `port_utils.rs` and its `pub mod` line.
   Delete `bind_with_retry()`, `port.rs`, and `port_tests.rs` (or keep `port.rs` only if
   something else remains in it). Delete `ServerConfig.bind_attempts` and update
   `bootstrap/run.rs`, `ServerConfig::default`, and `http/mod_tests.rs`. Update the
   `test-police` skill line that names `port_utils.rs`.
   Done when `python build.py check` passes and a second server on a busy port exits
   with "Failed to bind to port".
3. **Fix the browser hint.** Replace the `netstat -ano | Select-String` hint in
   `tests/test_utils/browser.rs` with tool-neutral text.
4. **Add the passthrough argument.** In the `parse_args` loop over `REGISTRY`, give the
   `run` subparser a `REMAINDER` argument for the server flags.
   Done when `python build.py run -- --world x --port 9` parses and the namespace holds
   the step name and `['--world', 'x', '--port', '9']` after the strip.
5. **Add the port check.** Add the port/host parser and the bind test as module-level
   helpers.
6. **Add the dispatcher and the exec.** Add `run_server(args)`, call it from `main()`
   before `run_step()`, store the returned argv, and exec it from the `__main__` block
   after the bind test.
   Done when the server starts and `GET /` returns 200.
7. **Add Python unit tests** in `scripts/tests/test_build_cli.py` for the seams above.
   Add `run` to `RegistryTests.test_expected_subcommands_exist`. Test the rejection of
   `--release` next to `run`.
   Done when `python build.py py-tests` passes.
8. **Update the documents.** `AGENTS.md` (the run line in the Iteration code block), the
   root `ENVIRONMENT.md` (the raw-cargo row of the build-signature table), and both
   `chronicler-ui-investigator` files (the Serve step, the busy-port note, and the "Engine
   won't start" troubleshooting row, which now has a clear `build.py` error; add a row for
   stale `target/debug/data`). Check `docs/diataxis/` for text about port retry or port
   killing.
   Done when `grep -rn "cargo run" AGENTS.md ENVIRONMENT.md .agents/skills/` returns only
   deliberate mentions of raw or IDE builds.

## Acceptance criteria

- `python build.py run` starts the engine with the binary defaults, and `GET /` on port
  3000 returns 200.
- `python build.py run -- --port 3099` starts on port 3099.
- With port 3000 already bound, `python build.py run` exits non-zero with "port 3000 is
  in use" and runs no cargo command.
- With port 3000 already bound, the raw binary exits with "Failed to bind to port 3000".
  It does not hang.
- A `build.py` step after `run` compiles zero units, and a `run` after a `build.py` step
  compiles zero units.
- A test tier and `run` share their build-mode units: `python build.py integration` then
  `python build.py run` compiles near zero units. Measure it and record the result in
  this plan.
- While the server runs, a second `build.py` step acquires the build slot without
  waiting. The server holds no lock.
- `logs/build_history.txt` gets one line for the `run` build phase, and the run's
  `logs/build_*.log` ends with the "Build Complete" banner.
- No Windows-only port helper remains in `build.py` or `src/`.
- `python build.py py-tests` passes, including the new tests.
- `python build.py` passes the full gate.
- No test starts a server process through `run`.
- The documents name `python build.py run` as the way to start the dev server.

## Verification

Run every command in a separate git worktree. Other sessions share this checkout.

```bash
python build.py run &                                        # in the background
curl -s -o /dev/null -w '%{http_code}\n' http://127.0.0.1:3000/   # expect 200
python build.py run; echo "exit=$?"                          # expect "port 3000 is in use", exit != 0
python build.py check                                        # expect 0 units compiled
# while the server still runs, prove the slot is free:
flock -n "${TMPDIR:-/tmp}/chronicler-engine-build-slot-$(id -u).lock" -c 'echo slot free'
tail -n 3 logs/build_history.txt
python build.py integration && python build.py run -- --port 3099   # record units compiled
python build.py py-tests
python build.py
```

## Blast radius

| File | Change |
|---|---|
| `build.py` | `run` registry entry, passthrough argument, port helpers, `run_server`, `main()` and `__main__` exec path; remove `kill_port`, `kill_by_name`, and their calls; docstring and help text |
| `scripts/tests/test_build_cli.py` | New cases; the expected-set list gains `run` |
| `src/adapters/driving/http/utils/port_utils.rs`, `utils/mod.rs` | Module deleted |
| `src/adapters/driving/http/bootstrap/port.rs`, `port_tests.rs`, `mod.rs` | `bind_with_retry` deleted |
| `src/adapters/driving/http/bootstrap/server.rs`, `src/bootstrap/run.rs`, `src/adapters/driving/http/mod_tests.rs` | Single bind; `bind_attempts` removed |
| `tests/test_utils/browser.rs` | Tool-neutral port hint |
| `AGENTS.md` | The run line in the Iteration code block (the structure block regenerates) |
| `ENVIRONMENT.md` | The raw-cargo row of the build-signature table |
| `.agents/skills/chronicler-ui-investigator/SKILL.md`, `ENVIRONMENT.md` | Serve step and troubleshooting rows |
| `.agents/skills/test-police/SKILL.md` | The `port_utils.rs` line |

## Out of scope

- A release-profile `run`.
- A watcher or hot reload.
- A PID lookup for the busy-port message.
- Making the server read the live `data/` instead of `target/<profile>/data`.
- Changing the linker configuration in `.cargo/config.toml`. The `scripts/lld-linker.sh`
  header records why the environment variable is used instead.
- Removing the check-mode and build-mode duplication. That is a cargo property, and the
  gate pays it either way.
- `terminate_pid()` in `tests/test_utils/server.rs`.
- Historical mentions in `docs/CHANGELOG.md`.
