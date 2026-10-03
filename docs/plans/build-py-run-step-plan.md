# `build.py run`: one command for the dev server

> **Status:** Implemented 2026-10-03, uncommitted. The full gate is green. Log:
> `logs/build_20261003_154929.log`.

## Summary

The dev server started with raw `cargo run`. Raw cargo does not export the linker
variable that `build.py` exports, and cargo hashes the linker value into the fingerprint
of every unit. Each switch between a `build.py` step and the raw server command
therefore recompiled the whole tree, in both directions.

This plan adds a `run` step to `build.py`. The step builds the binary with the `build.py`
environment inside the machine-wide build slot, releases the build slot, and then
replaces its own process with the binary. To the caller the step behaves like
`cargo run`. The switch cost becomes zero, and the server links with lld.

The same change removes the port-freeing helpers in `build.py` and in the server. Those
helpers call Windows tools (`netstat`, `Select-String`, `tasklist`, `taskkill`) and do
nothing on Linux. The server binds once and fails with a clear error on a busy port.
`build.py run` checks the port before it starts.

## Key measurements (2026-10-03)

| Measurement | Result |
|---|---|
| Isolated linker-variable test (`tmp/linkfp2/`) | Setting or unsetting `CARGO_TARGET_X86_64_UNKNOWN_LINUX_GNU_LINKER` recompiled both crates, even the linker-free rlib. Unchanged, the second build compiled none |
| Repository switch test, one target directory | `python build.py check` restored the lld fingerprints in 97s, then 1.3s warm. A lld `cargo build --bin chronicler_engine` took 133s, then 0s warm. The build-mode compile did not disturb the check-mode units |
| Earlier raw `cargo run` after a gate | 243 crates in 2m11s, then a no-op in 0.85s |

## Mechanism

Two independent causes:

1. **The linker variable.** `build.py` exports
   `CARGO_TARGET_X86_64_UNKNOWN_LINUX_GNU_LINKER=<cache>/chronicler-engine/lld-linker.sh`
   through `_lld_linker_env()`. Raw cargo exports nothing. Cargo hashes the value into
   every unit fingerprint, so the two commands invalidate each other.
2. **Check mode and build mode keep separate artifacts.** `cargo check` never feeds
   `cargo build`. The gate already pays this cost. Clippy compiles the tree in check
   mode, and then nextest compiles the tree again in build mode. `ENVIRONMENT.md` records
   this as "Two compiles per dependency".

The `run` step removes cause 1. Cause 2 is a cargo property that no `run` step can
remove, and the gate pays that cost whether or not this plan lands.

## Decisions

| Topic | Decision |
|---|---|
| Command | `python build.py run [-- <server flags>]`. `build.py` declares no server flags. With no flags, the binary uses its own defaults |
| Passthrough | `argparse.REMAINDER` on the `run` subparser. Strip exactly one leading `--` before the exec. `--target-dir` works before or after `run` |
| Build and run | `cargo build --bin chronicler_engine` inside `_cargo_slot`, then `os.execv` on the binary. This is what `cargo run` does internally, with the build slot covering only the build |
| Exec position | After `main()` has run its `finally` (summary, banner, log close, history line). The `__main__` block calls `os.execv`, not `run_step` |
| History | One `logs/build_history.txt` line for the build phase. Exit code 0 when the build passes |
| Busy port | A bind test in `build.py`, with `SO_REUSEADDR` set (confirmed for Unix: tokio 1.51.1 through mio 1.2.0 `src/net/tcp/listener.rs` sets reuseaddr under `#[cfg(not(windows))]`). It runs before the build and again just before the exec. On failure: stop with "port N is in use" and a hint to stop that process or pass `-- --port N`. No PID lookup |
| Source of the port and host | Read `--port N` or `--port=N`, and `--host H` or `--host=H`, from the passthrough flags. Otherwise use 3000 and `0.0.0.0`, copied from `src/utils/cli.rs` with a comment that points there. Skip the check when the flags contain `--list-worlds`, `--help`, `-h`, `--version`, or `-V` |
| Working directory | The repo root. `main()` already changes to the repo root, and that directory matches the run line in `AGENTS.md` |
| Stale `target/debug/data` | Unchanged, the same as `cargo run`. Recorded in Out of scope and in the UI-investigator troubleshooting row |
| `build.py` helpers removed | `kill_port()` and its call in `_gate_prelude`. `kill_by_name()` and the "kill lingering processes" step in `run_cleanup`. `--cleanup` keeps the port-lock and target-dir removal |
| Server helpers removed | `src/adapters/driving/http/utils/port_utils.rs` (the whole module) |
| Server bind | Delete `bind_with_retry()` and `ServerConfig.bind_attempts`. The server binds once with `TcpListener::bind` and returns the existing `EngineError::Config("Failed to bind to port N: …")` |
| Kept | `terminate_pid()` in `tests/test_utils/server.rs`. The Windows branch sits behind `#[cfg(target_os = "windows")]`, and the Linux branch uses `libc::kill` |
| Hint text | `tests/test_utils/browser.rs` drops "Check with: netstat -ano \| Select-String". The new hint names no tool, because this container has none |
| Tracker | Issue 03 is resolved by this plan. The followups plan DC5 bullet is owned by its own session, and that session has already pointed it here |
| `CONTEXT.md` and the ADR | No change. These are build-tool terms, not domain terms, and the decision is easy to reverse |

## Measured after implementation (2026-10-03)

The full gate passed after the change. The first post-change gate took 113 s with
py-tests 223 (`logs/build_20261003_153706.log`). The final gate, after the review fixes,
took 87.1 s with py-tests 224, one more than the first
(`logs/build_20261003_154929.log`). Both runs: architecture 1, guardrails 137,
integration 1522 passed, 2 skipped, browser 30, exit 0.

- `python build.py run` served `GET /` on port 3000 with HTTP 200 in ~45 s the first
  time. The engine binary unit compiled in 42.6 s. A second `run` reuses the binary.
- `python build.py run -- --port 3099` served HTTP 200.
- With port 3000 already bound, `python build.py run` exited 1 in 0.1 s with
  `ERROR: port 3000 is in use.` and ran no cargo command.
- With port 3000 already bound, the raw binary exited 1 in 0 s with
  `Failed to bind to port 3000: Address already in use (os error 98)`. The raw binary
  does not hang.
- While the server ran, `python build.py check` took 7.1 s with zero units compiled and
  no slot wait, and `flock -n /tmp/chronicler-engine-build-slot-1000.lock` succeeded.
  The server holds no lock.
- Running `cargo check`, then `cargo build --bin chronicler_engine`, then `cargo check`
  again, each directly with the lld environment that `build.py` sets, reported zero units
  and ~0.3 s each way.
- `python build.py --release run` exited 2 (gate-only flag). `python build.py run --
  --list-worlds` printed the worlds while a server was up.
- `python build.py integration` then a direct `cargo build --bin chronicler_engine` under
  the lld environment reported zero units in 0.29 s, but only because an earlier `run`
  had already built the binary. Test tiers build their harness and dependency units, not
  `target/debug/chronicler_engine`. The first `run` after the gate compiled the binary in
  42.6 s. Once the binary is warm, there is zero churn in either direction.

`build.py` strips cargo `Compiling` and `Checking` lines from the logs
(`_CARGO_PROGRESS_RE`), so you cannot read a "zero units" claim from a build-log line
count. For a measurement, use the wall time (`Finished … in ~0.3 s`) or run cargo
directly under the lld environment that `build.py` sets.

## Why not a separate target directory

A separate target directory for `run` would cold-start at 344 s seeded or 728 s unseeded,
and a second copy of the build tree is 35 GB on this checkout, so `run` shares the
existing target directory.

## Known limitation

The server holds no lock, so a concurrent `build.py` gate can replace the binary while
the server runs. On Linux the running process keeps its own inode, so the server
continues to serve the old binary. The bind test and the server's own bind are separate
events. Another process can take the port between them, and the server then fails with
its bind error.

## Touched areas

`build.py` gains the `run` step and loses the port helpers, `scripts/tests/test_build_cli.py`
gains cases, the server's `port_utils` and `bind_with_retry` are deleted, the browser hint
loses its tool-specific text, and the docs that named the server command are updated.
`AGENTS.md`'s AUTO-STRUCTURE block still lists `port_utils.rs` in the working tree. The
pre-commit hook regenerates the block at commit, so that line goes then.

## Out of scope

- A release-profile `run`.
- A watcher or hot reload.
- A PID lookup for the busy-port message. The container has no `ss`, `lsof`, `fuser`, or `netstat`, so a lookup would need a scan of `/proc/net/tcp` and `/proc/*/fd`.
- Making the server read the live `data/` instead of `target/<profile>/data`.
- Changing the linker configuration in `.cargo/config.toml`. The `scripts/lld-linker.sh`
  header records why the environment variable is used instead.
- Removing the check-mode and build-mode duplication. That is a cargo property, and the
  gate pays that cost either way.
- `terminate_pid()` in `tests/test_utils/server.rs`.
- Historical mentions in `docs/CHANGELOG.md`.
