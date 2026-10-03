# Build environment

Read this when a build or test run is slow, waits, or runs out of memory. `AGENTS.md` says how to
run builds. Numbers are single runs from 2026-10-02 on a shared host, so treat gaps under 10 % as noise.

## Limits

- **CPU:** the container has a 3-CPU quota on a 4-thread host, and `nproc` says 4. Cargo reads the
  quota and runs 3 jobs. nextest's thread count is set by hand in `.config/nextest.toml`.
- **Memory:** 8 GiB. This limit bites first. Three overlapping builds ran 2.1 times slower than the
  same builds one after another, and `oom_kill` in `/sys/fs/cgroup/memory.events` stands at 8.
- **Host load:** other work on the host often saturates it, so wall times vary. `nr_throttled` in
  `cpu.stat` stays low even then, so it does not show whether CPU is free.

## Why the build works this way

Each script's docstring has its details. These are the reasons and measurements the code can't show.

- **Build slot** (`scripts/build_slot.py`): one heavy cargo step runs at a time, because overlapping
  builds exceed the memory limit.
- **Target seeding** (`scripts/target_seed.py`): a cold target dir copies dependency artifacts from
  a worktree with the same build signature. A seeded new-worktree gate took 344 s, against 728 s
  unseeded.
- **lld linker** (`build.py`, `scripts/lld-linker.sh`): it links about 4 times faster than GNU ld.
  It is set through an environment variable, not `.cargo/config.toml`, because cargo hashes the
  linker's absolute path into every unit.
- **sccache** (`scripts/sccache-wrapper.sh`): it shares Rust compiles across worktrees, with about
  84 % hits on a cold build. It does not cache the C code that `aws-lc-sys` and `libsqlite3-sys`
  compile, because the `cc` crate only uses a wrapper whose file name is exactly `sccache`. This is
  inferred from the `cc` source, not measured.
- **Two compiles per dependency:** clippy compiles dependencies in check mode, and the tests compile
  them again in build mode. On a cold dir, the ~117 s C build in `aws-lc-sys` runs twice. Cargo has
  no setting to share them. Seeding avoids both.
- **Browser concurrency:** nextest runs `test-threads / threads-required` browser tests at once. In a
  trial of four runs each, 2 at once took 75 s, 3 took 66 s, and 4 took 65 s. Four hit the memory
  limit in every run, so the setting is 3.
- **Per-test cost:** nextest runs each test in its own process, so an in-process cache never carries
  across tests. Each Harper text-check test builds its own dictionary, which `Cargo.toml` speeds up.

## Typical gate cost

| Scenario | Total | clippy | architecture | guardrails | integration | browser |
|---|---|---|---|---|---|---|
| Warm, no Rust change | 97 s | 1 | 4 | 7 | 19 | 60 |
| Seeded new worktree, before the test speed-ups | 344 s | 52 | 37 | n/a | 136 | 98 |
| Seed miss, before the test speed-ups | 728 s | 239 | 242 | n/a | 146 | 81 |

## Diagnose a slow build

| Signal | Meaning |
|---|---|
| `Waiting for the build slot` | Another checkout holds the slot. This is normal. |
| `Target seeding: no warm sibling ...` | This dir builds from scratch. The line lists the dirs it saw and their signatures. |
| `oom_kill` rising | Builds exceeded the memory limit. |
| `some avg300` rising in `/sys/fs/cgroup/cpu.pressure` | The host is squeezing the container. |

## What forces a rebuild

| Change | Effect |
|---|---|
| Toolchain bump, or an edit to `.cargo/config.toml`, `scripts/lld-linker.sh`, `rust-toolchain.toml`, or `RUSTFLAGS` | New build signature, so every seed misses once. |
| `Cargo.toml` profile or dependency change | Seeds still copy. Cargo rebuilds the affected units. |
| Switching `--target-dir` mid-task | The new dir is cold. |
| A raw `cargo build` or IDE build in a `build.py` target dir | Raw cargo uses GNU ld, so it and `build.py` rebuild each other's units. Inferred, not measured. |
