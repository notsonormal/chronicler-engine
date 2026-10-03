# Build environment

Why the build and test tooling is set up the way it is, and what to check when a build is slow.
How to run builds is in `AGENTS.md` ("Development Loop", "Concurrent Builds"). Numbers were
measured on 2026-10-02 as single runs on a shared host; treat gaps under ~10 % as noise.

## Limits

| limit | value |
|---|---|
| CPU quota (cgroup `cpu.max`) | 3 CPUs, on a 4-thread host (`nproc` says 4) |
| Memory cap (cgroup `memory.max`) | 8 GiB |

- Cargo reads the quota, so it runs 3 jobs, not 4. nextest's thread count is set by hand.
- The memory cap is what bites. Three overlapping builds ran 2.1x slower than the same builds
  one after another, and `memory.events` shows `oom_kill 8` (one was GNU `ld`; the rest are
  undated).
- The host is shared and often saturated by work outside the container, so wall times are noisy.
  `nr_throttled` stays low even then, so it does not show whether CPU is free.

## Build setup

Each piece is a small script; read the file for switches and edge cases.

| piece | file | what and why |
|---|---|---|
| Build slot | `scripts/build_slot.py` | A machine-wide lock so one heavy cargo step runs at a time across all checkouts, because overlapping builds exceed the memory cap. Waiters print the holder. After 30 minutes (`CHRONICLER_BUILD_SLOT_WAIT`) a waiter gives up and runs anyway. `fmt` and the Python checks skip it. |
| Target seeding | `scripts/target_seed.py` | A cold target dir gets third-party artifacts copied from a git-worktree sibling with the same build signature. Workspace crates, path deps and `[patch]` entries are never copied, so another checkout's code can't be served. A seeded new-worktree gate took 344 s against 728 s unseeded. A miss logs why. |
| Build signature | `target_seed.py:build_signature` | Hash of `rustc -vV`, `.cargo/config.toml`, `scripts/lld-linker.sh`, `rust-toolchain.toml` and the `RUSTFLAGS` variables. Only dirs with the same stamp seed each other. `Cargo.toml` is deliberately left out: a profile or dependency change rebuilds the affected units but still seeds. |
| lld linker | `build.py:_lld_linker_env`, `scripts/lld-linker.sh` | Linking is about 4x faster than GNU ld, and a warm edit-rebuild fell from ~52 s to ~20 s. The wrapper is copied to `~/.cache/chronicler-engine/` and passed through `CARGO_TARGET_<triple>_LINKER`. It is not in `.cargo/config.toml`, because cargo hashes the linker's absolute path into every unit and a per-checkout path would defeat seeding. |
| sccache | `scripts/sccache-wrapper.sh` | Shares Rust compiles across worktrees (~84 % hits on a cold build). It unsets `CARGO_TARGET_DIR` for sccache, which keys on its environment. C from build scripts (`aws-lc-sys`, `libsqlite3-sys`) is not cached, because `cc` only uses a wrapper whose file name is exactly `sccache`. Inferred from the `cc` source, not re-measured. |
| Check and build compiles | cargo | Clippy compiles dependencies in check mode and the tests compile them again in build mode, so on a cold dir `aws-lc-sys`'s ~117 s C build runs twice. Cargo has no setting to share them; seeding is the fix. |

## Test setup

`.config/nextest.toml`:

- `test-threads = 3`, matching the CPU quota. Integration tests stop speeding up past 3 threads.
- The browser binary has `threads-required = 1`, so **3 browser tests run at once**
  (`test-threads / threads-required`). Change the two values together. Lowering `test-threads`
  alone silently reduces browser concurrency.
- Browser trial, 4 interleaved runs each, medians: 2 at once 75 s, 3 at once 66 s, 4 at once
  65 s. Three beat two with no overlap in ranges. Four was within noise of three and hit the
  8 GiB cap in every run, so it stays at 3. No failures or OOM kills in any run.
- nextest runs each test in its own process, so an in-process cache never carries across tests.
  The Harper text-check tests each build a dictionary, taking ~3.9 s unoptimised and ~0.4 s
  with `opt-level = 2` on `harper-core` and `fst` in `Cargo.toml`. `syn` was tried and dropped:
  it saved ~3 s per gate but cost a ~4 minute rebuild of `syn` and every proc-macro crate in
  each cold target dir.
- The integration step excludes the `architecture` and `guardrails` binaries, which have their
  own gate steps. `--coverage` keeps them in.

## Typical gate cost

| scenario | total | clippy | arch | guardrails | integration | browser |
|---|---|---|---|---|---|---|
| Warm, no Rust change | 97 s | 1 | 4 | 7 | 19 | 60 |
| Warm, after a source edit (before the test-speed changes) | ~208 s | 18 | 15 | | 78 | 81 |
| Seeded new worktree (before the test-speed changes) | 344 s | 52 | 37 | | 136 | 98 |
| Seed miss (before the test-speed changes) | 728 s | 239 | 242 | | 146 | 81 |

Warm, the browser tier is most of the time. On a seed miss it is compiling dependencies twice.

## What to check

| signal | meaning |
|---|---|
| `oom_kill` in `/sys/fs/cgroup/memory.events` (baseline 8) | A rise means builds exceeded the memory cap. |
| `some avg300` in `/sys/fs/cgroup/cpu.pressure` | Rising means the host is squeezing the container. |
| `Waiting for the build slot` | Another checkout holds the slot. Normal. |
| `Target seeding: no warm sibling ...` | This dir builds from scratch. The line lists the dirs it saw and their signatures. |

## What forces a rebuild

| change | effect |
|---|---|
| Toolchain bump; edit to `.cargo/config.toml`, `scripts/lld-linker.sh`, `rust-toolchain.toml`; `RUSTFLAGS` | New signature, so every seed misses once. |
| `Cargo.toml` profile or dependency change | Seeds still copy; cargo rebuilds the affected units. |
| `--target-dir` switched mid-task | The new dir is cold. Use one target dir per checkout. |
| Raw `cargo build` or IDE build in a `build.py` target dir | Raw cargo uses GNU ld, a different hash space from the lld builds, so each side rebuilds the other's units. Inferred, not measured. |
