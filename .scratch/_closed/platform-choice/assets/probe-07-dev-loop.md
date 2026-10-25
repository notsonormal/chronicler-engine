# Probe 07: the dev-build loop, measured

Evidence for [Prototype: decide the crate split and the dev-build loop](../issues/07-prototype-crate-split-and-dev-build-loop.md).
The owner decided the ticket; this file holds the numbers and how they were taken.

Host: 3-CPU quota, 8 GiB cgroup cap, Rust 1.88.0, LLVM backend, warm `target/`.
Measured 2026-10-16 ~22:07-22:15 (`tmp/probe/` scratch, now gone).

## Method

One marker comment appended to a source file, then one timed build step, then the file restored
byte-exact from a backup and `cmp`-verified. The restore is itself an edit, so a settle build
follows before the next measurement.

Marker: `\n// probe: dev-loop measurement marker\n` appended to
`src/domain/model/state/game_state.rs` (bottom layer, 1,840 lines including its unit tests).

Peak memory: `tmp/probe/peakmem.py` samples the summed RSS of `cargo`, `rustc`, the linker and the
C compiler every 0.25 s and prints the peak. The container has no `/usr/bin/time`, and
`/sys/fs/cgroup/memory.peak` is read-only.

## 1. The loop today (all nine test binaries) against a selected target

`build.py test-pattern` runs `cargo nextest run --no-fail-fast <pattern>`, which builds every test
target in the package. A targeted test therefore pays for the eight binaries it never runs.

| Shape | Command | Wall time |
|---|---|---|
| Warm, no edit | `python build.py test-pattern "<name>"` | 2.1 s |
| Bottom-layer edit | `python build.py test-pattern "<name>"` | 20.45 s |
| Revert of that edit | same | 11.8 s |
| Bottom-layer edit, lib unit-test binary only | `cargo nextest run --lib "<name>"` | 4.83 s |
| Bottom-layer edit, one test target | `cargo nextest run --test http "<name>"` | 5.60 s |

Test names: `test_wrap_xml_preserves_single_empty_content_line` (lib unit test),
`test_nonexistent_room_sets_error_status_http` (http tier).

Selection is the lever: the same edit costs 5.60 s when only the needed target builds, against
11.8-20.45 s when all nine do. nextest supports the selection flags already
(`--test <TARGET>`, `--lib`); `build.py test-pattern` does not pass one today.

The all-binaries figures here are single runs from one load window; §4 re-measured that shape six
times per backend and got 13.16 s median for LLVM. Treat 11.8-20.45 s as the spread and 13.2 s as
the median.

## 2. Where the all-binaries after-edit time goes

Two staged runs of the same marker edit, raw cargo with the build.py linker env set:

| Stage | Command | Time |
|---|---|---|
| lib with `cfg(test)` + its unit-test binary | `cargo test --lib --no-run` | 5.77 s |
| lib plain (for the integration binaries) + 8 test binaries, compile and link | `cargo nextest run --no-run` | 8.55 s |

nextest reported `Finished test profile in 19.86s` and `Starting 1 test across 9 binaries`.
The lib is compiled twice per edit: once with `cfg(test)` for the unit-test binary, once plain for
the integration binaries.

Peak summed RSS: 1.79 GB after an edit, 118 MB for a warm no-edit run.

## 3. What a crate split could reach

- The nine test binaries depend on the package's lib, so any edit rebuilds them whatever the crate
  layout. They are about 8.5 s of the 20 s, plus roughly 4 s of the lib-plain stage that is theirs.
- The rest is the lib itself, about 6 s, and rustc's incremental compilation already isolates that
  work per changed module and its dependents. A split would rename the boundary, not remove the
  work, and it would make an edit *below* the boundary rebuild the crates above it as well.
- The crate that would gain most from being below the boundary is `domain` (7,557 lines, 10 outward
  `crate::` references in 3 files, 7 `pub(crate)` items). The churn sits in `adapters/` and
  `application/` (36,625 of the 49,032 lines under `src/`), which a split would leave as the top
  crate, so those edits would still rebuild almost everything.

Cost side of a split, if it were taken:

- `arch-lint.toml` has `root = "./src"` and one scope per `src/...` path prefix, and arch-lint
  0.4.3 cannot make an exception ([research 10](research-10-storage-constraints.md)); a workspace
  split needs a config per crate or another tool.
- The guardrails read `src/`-relative paths: `tests/infrastructure/guardrails/layers.rs` (`path.contains("src/")`),
  `scripts/check_test_structure.py` (`src.rglob`), `scripts/generate_structure_index.py` (emits
  `- **src/**`), and the test-mirror convention in `tests/AGENTS.md`.
- 543 `crate::domain::` paths in 151 files outside `domain` would keep resolving through a
  `pub use chronicler_domain as domain;` re-export, so the mechanical churn is smaller than the raw
  count suggests.

## 4. Cranelift (`rustc_codegen_cranelift`)

Measured by a delegated implementer, 22:12-22:49, against the same marker edit and the same
commands. Toolchain `rustc 1.101.0-nightly (4ebd25825 2026-10-15)` with
`RUSTFLAGS="-Zcodegen-backend=cranelift"`, target dir `target/cranelift-probe`. No fallback flag
was needed. The backend was proven active with a throwaway crate. Full report:
[probe 07: the Cranelift run](probe-07-cranelift.md).

| Shape (bottom-layer edit) | LLVM today (n=6) | cranelift (n=6) | Reading |
|---|---|---|---|
| All nine binaries | 13.16 s | 11.07 s | −15.9 %, at the top of the host noise floor (three interleaved pairs: +18 %, +5 %, +23 %) |
| All nine binaries, revert | 11.67 s | 10.75 s | −7.9 %, noise |
| lib unit binary only | 4.83 s (n=1) | 4.63 s (n=3) | −4.2 %, noise |
| One test target (`--test http`) | 5.60 s (n=1) | 5.04 s (n=3) | −10.0 %, noise floor |
| Cold codegen build, uncached | 238.19 s | 198.60 s | −16.6 % |
| Cold codegen build, `build.py` command as written | 174.84 s (287 sccache hits) | 207.87 s | cranelift 19 % *slower* |
| Peak RSS, all nine after an edit | 1.99 GB (n=6) | 2.04 GB (n=6) | no advantage |
| `clippy` after an edit | 12.72 s (n=2), exit 0 | exit 101 | 30 first-party lints break under nightly 1.101's `-D warnings`; none is codegen-related |

The single 20.45 s figure in §1 was a load spike: six later LLVM runs of the same shape gave
13.16 s median (10.93-20.88). Sixteen cranelift runs on that shape span 10.4-17.7 s, so the two
backends overlap. The honest reading is a 1-2 s cranelift gain on the shape that target selection
already removes.

Every cranelift number also carries a compiler-version difference (1.88.0 against
1.101.0-nightly), because cg_clif is nightly-only. That confound cannot be removed inside this
probe.

## 5. Ruling the measurements support

- **No crate split.** The nine test binaries depend on the package lib, so no layout reaches their
  ~8.5 s. The lib part that a split isolates is already isolated by rustc's incremental
  compilation, and the churn sits in the top two layers. The split also costs the `arch-lint` root
  and the `src/`-relative guardrail tooling in §3.
- **No Cranelift.** It does not help the targeted loops (within noise), and the cold-build gain
  applies only to an empty compiler cache: the repository's seeded and sccache-warm path is 19 %
  faster than cranelift, and those are the mechanisms that make a cold worktree affordable. Its
  listed price: pinning a nightly toolchain, a failing `clippy` until 30 lints are addressed, a
  preview codegen backend, and no memory gain.
- **The build plan takes target selection instead.** `build.py test-pattern` gains an optional test
  target (`cargo nextest run --test <TARGET>` / `--lib`), measured at 5.0-5.6 s against 12-13 s.
  The trade-off to record: a narrow loop does not compile the other eight test binaries, so a
  shared-API change surfaces in the full gate instead.

There is no measured crate-split experiment: §3 bounds one instead. The owner asked for one only if
it earns its cost; the bound is what settles it.

## Raw evidence

`logs/build_history.txt` rows 2026-10-16T22:07:51 (warm), 22:08:17 (edit), 22:08:36 (revert);
logs `logs/build_20261016_220749.log`, `logs/build_20261016_220756.log`,
`logs/build_20261016_220825.log`.
