# Ticket 07 probe: `rustc_codegen_cranelift` against the LLVM loop

Measured 2026-10-16 22:12-22:49 by the implementer for
[Prototype: decide the crate split and the dev-build loop](../../.scratch/platform-choice/issues/07-prototype-crate-split-and-dev-build-loop.md),
executing `tmp/probe/T07-cranelift-protocol.md` with the three lead additions
(clippy is a smoke test, cold-codegen is the real cold comparison, plus an
apples-to-apples LLVM clippy-after-edit run) and one correction.

Baseline compared against: `tmp/probe/baseline-llvm.md` (Rust 1.88.0, warm `target/`, main checkout).

Every number below is **measured** unless it is explicitly labelled **inferred**.
"Δ" is `(cranelift − LLVM) / LLVM`; negative means cranelift is faster.
Host noise: `/proc/loadavg` was recorded around every run (1.6-4.7 on this 3-CPU
quota, shared with ~13 other checkouts). Per `ENVIRONMENT.md`, gaps under 10 % are
noise on this host. Raw per-run values are in the appendix.

## Environment and exact flags

| | cranelift side | LLVM side |
|---|---|---|
| toolchain | `rustc 1.101.0-nightly (4ebd25825 2026-10-15)`, `cargo 1.101.0-nightly (3960fa69b 2026-10-12)` | `rustc 1.88.0 (6b00bc388 2025-06-23)`, `cargo 1.88.0` (from `rust-toolchain.toml`) |
| switch | `RUSTUP_TOOLCHAIN=nightly` (env only; no file edited) | default |
| backend | `RUSTFLAGS="-Zcodegen-backend=cranelift"` | none |
| linker | `CARGO_TARGET_X86_64_UNKNOWN_LINUX_GNU_LINKER=$HOME/.cache/chronicler-engine/lld-linker.sh` (both sides) | same |
| target dir | `target/cranelift-probe` | warm `target/`, plus fresh `target/llvm-cold-probe*` for the cold runs |

Component installed: `rustc-codegen-cranelift-x86_64-unknown-linux-gnu`
(requested as `rustc-codegen-cranelift-preview`), dylib at
`/usr/local/rustup/toolchains/nightly-x86_64-unknown-linux-gnu/lib/rustlib/x86_64-unknown-linux-gnu/codegen-backends/librustc_codegen_cranelift-1.101.0-nightly.so`.

**No cranelift fallback flag was needed.** `CARGO_PROFILE_TEST_DEBUG=0`,
`CARGO_PROFILE_DEV_DEBUG=0` and `-C debuginfo=0` were never required: every
codegen build and every test run succeeded with the plain `RUSTFLAGS` above.
No cranelift diagnostic ("debug info unsupported" or similar) appeared in any log.

**The backend was verifiably active** (measured): a throwaway crate
`tmp/probe/tiny` built with the same env prints a rustc invocation containing
`-Zcodegen-backend=cranelift` under `cargo build -v`, and the resulting binary ran
and printed `14850`. The host-side verbose line confirms the flag reaches rustc
through `scripts/sccache-wrapper.sh`.

**Confound to keep in mind for every comparison:** cg_clif is nightly-only, so
"cranelift vs LLVM" here also carries a **compiler-version difference**
(1.101.0-nightly vs 1.88.0), not just a backend difference. That cannot be
removed inside this protocol.

## 1. The loop shapes (wall seconds, medians over the run count)

| Shape | LLVM baseline (22:07-22:15) | LLVM re-measured today | cranelift | Δ vs LLVM today | Δ vs baseline |
|---|---|---|---|---|---|
| A warm, no edit, all 9 binaries | 2.1 (n=1) | — | **2.52** (n=1) | — | +20 % |
| A bottom-layer edit, all 9 binaries | 20.45 (n=1) | **13.16** (n=6) | **11.07** (n=6) | **−15.9 %** | **−45.9 %** |
| A revert (settle), all 9 binaries | 11.8 (n=1) | **11.67** (n=6) | **10.75** (n=6) | −7.9 % (noise) | −8.9 % (noise) |
| B bottom-layer edit, lib unit binary only | 4.83 (n=1) | — | **4.63** (n=3) | −4.2 % (noise) | −4.2 % (noise) |
| B revert (settle) | — | — | **4.78** (n=3) | — | — |
| C bottom-layer edit, one test target (`--test http`) | 5.60 (n=1) | — | **5.04** (n=3) | −10.0 % (noise floor) | −10.0 % (noise floor) |
| C revert (settle) | — | — | **5.04** (n=3) | — | — |

Reading of row 2, honestly: the 15.9 % median gap is *at* the top of the host's
noise floor, not comfortably above it. The three interleaved pairs (cranelift run
immediately followed by the LLVM run of the same shape) were **+18.2 %, +5.3 %,
+22.5 %** in cranelift's favour, so the direction is consistent but one pair is
inside the noise. The baseline's 20.45 s and today's first LLVM run (20.88 s) both
fell in a load spike (loadavg 3.7), which is why the LLVM re-measurement was done
(n=6 runs, median 13.16 s).

The "warm, no edit" row says the opposite (cranelift 2.52 s vs 2.1 s), and that is
n=1 vs n=1 from a different load window — treat it as noise, not as a 20 % regression.

Rules the shapes obey: a one-line marker comment at the bottom layer,
`src/domain/model/state/game_state.rs`; after every measurement the file is restored
and proven identical (`cmp` against `tmp/probe/game_state.rs.bak`, `git diff` on the
file empty, `git status --short src/` empty before the first and after the last run).

## 2. Cold codegen build (all 9 binaries, one `test-pattern` command)

This is the cold comparison the lead asked for: fresh target dir, unseeded, codegen.

| Run | wall | build.py step | nextest "Finished" | peak RSS | sccache hits |
|---|---|---|---|---|---|
| LLVM 1.88, fresh dir, `CHRONICLER_NO_SEED=1` (protocol's command) | 174.84 | 174.7 | 2m 53s | 2.99 GB | 287 |
| LLVM 1.88, fresh dir, no compiler cache (`RUSTC_WRAPPER=`) | **238.19** | 238.0 | 3m 56s | 2.54 GB | 0 |
| cranelift, `target/cranelift-probe` cold codegen, first codegen run after the clippy run | 207.87 | 207.8 | 3m 26s | 2.79 GB | 0 |
| cranelift, fresh dir, no compiler cache (`RUSTC_WRAPPER=`) | **198.60** | 197.8 | 3m 16s | 2.59 GB | 0 |

- Apples to apples (fresh dir, empty compiler cache, one command): **cranelift
  198.60 s vs LLVM 238.19 s = cranelift 16.6 % faster.**
- Against the protocol's literal command, cranelift is 19 % *slower*
  (207.87 vs 174.84) — but that LLVM run consumed **287 sccache hits** (measured:
  `sccache --show-stats` moved by exactly 287 hits during it), i.e. it was unseeded
  but not uncached. Cranelift gets 0 hits by construction: its compiler binary
  (1.101.0-nightly) had no cache entries in this container. So that pair compares an
  uncached backend with a half-cached one; the uncached LLVM run is the honest partner.
- The two cranelift cold runs differ by 9.3 s (4.5 %) in the *unexpected* direction
  (the run with pre-built C artifacts was slower). Both are single runs, so the
  cranelift cold figure is `198.6 s ± host noise`, not a tight value.
- Peak RSS on the cold path is ~2.5-3.0 GB on both sides -- the cold build is memory
  bound on neither backend.

## 3. `clippy`

| Run | wall | result | peak RSS |
|---|---|---|---|
| cranelift `clippy`, cold `target/cranelift-probe` | 203.88 | **exit 101 — FAILED at the lint stage** | 1.03 GB |
| LLVM `clippy`, warm `target/`, after the marker edit | **12.72** (n=2: 14.39, 11.04) | exit 0 | 0.13 GB |
| LLVM `clippy`, warm `target/`, revert/settle | 11.85 (n=2: 12.05, 11.65) | exit 0 | 0.13 GB |

Per the lead's correction this run is a **smoke test, not a timing comparison**:
clippy emits metadata only, so the cranelift backend cannot speed it up. Its verdict
is that the run **fails before codegen matters** — 30 first-party lints that nightly
1.101 lints harder than 1.88 (first five verbatim in §7). Nothing in the failure
mentions cranelift, a codegen backend, or a dylib. The crate's front-end analysis
under cg_clif therefore proceeds normally; the backend itself is proven by the
codegen runs and the `tiny` crate above.

The apples-to-apples LLVM number the baseline was missing: **12.72 s median
(n=2) after a bottom-layer edit** in the warm main `target/`, 11.85 s for the settle.
The baseline's "13.0 s clippy median" is consistent with it; whether the baseline
figure was an after-edit run is now moot.

## 4. `cargo check` — the front-end-only floor (LLVM, warm `target/`)

| Run | wall | peak RSS |
|---|---|---|
| after the marker edit, first `cargo check` in this dir (check profile cold) | 39.40 | 0.87 GB |
| after the marker edit, check profile warm | **4.36** | 0.47 GB |
| warm, no edit / settle | 1.98 / 1.90 | 0.47 GB |

Note (measured): the first `cargo check` in the main `target/` dir built the whole
check profile from scratch (39.40 s) because `cargo clippy` uses a different
`RUSTC_WORKSPACE_WRAPPER` and so owns separate units. Only the second run is an
after-edit delta. The front-end floor for a bottom-layer edit is ~4.4 s, against
~11-13 s for the full all-binaries codegen shape: most of shape A is codegen + link
of the 8 test binaries the targeted test does not use.

## 5. Peak memory (summed RSS of cargo/rustc/linker/cc, sampled by `tmp/probe/peakmem.py`)

| Shape | LLVM | cranelift |
|---|---|---|
| A warm, no edit | 0.118 GB (baseline, n=1) | 0.108 GB (n=1) |
| A after edit | 1.99 GB median (n=6; 1.71-2.51) | 2.04 GB median (n=6; 1.74-2.26); baseline 1.79 GB (n=1) |
| A revert/settle | 1.98 GB median (n=6) | 1.88 GB median (n=6) |
| B after edit | not measured | 0.91 GB median (n=3) |
| C after edit | not measured | 0.96 GB median (n=3) |
| cold codegen, uncached | 2.54 GB (n=1) | 2.59 GB (n=1) |
| `clippy` after edit | 0.13 GB (n=2) | n/a (fails; cold-run figure 1.03 GB is not comparable) |
| `cargo check` after edit (warm) | 0.47 GB (n=1) | not measured |

**Cranelift does not reduce peak memory here.** The two backends are equal within
noise on every shape that both sides measured (~2 GB while a full all-binaries
after-edit run is in flight; ~0.1 GB warm). Nothing came near the 8 GiB cap;
`oom_kill` in `/sys/fs/cgroup/memory.events` stayed at 0 through the whole probe.

## 6. Method: what the numbers rest on

- Marker edit, byte-exact restore: `tmp/probe/mark_edit.sh` appends
  `\n// probe: dev-loop measurement marker\n`; `tmp/probe/mark_restore.sh` copies
  `tmp/probe/game_state.rs.bak` back and prints `RESTORE_OK cmp-identical`
  (`RESTORE_OK cmp-identical` on all 22 restores -- 21 captured in the run logs,
  1 in the warm `cargo check` repeat -- and no `RESTORE_FAILED` anywhere).
  Final state verified:
  `git diff --exit-code -- src/domain/model/state/game_state.rs` passes,
  `cmp` against `HEAD:` passes, `git status --short src/` is empty.
  `git status --short src/` was empty before the first measurement.
- Marker diff while in place: `git diff --numstat` = `2 0` (one blank line + one
  comment line) — the protocol's printf adds two lines.
- Run counts: A after-edit 6 per backend plus 6 settles each; shapes B and C 3
  after-edit + 3 settles each; clippy 2 after-edit + 2 settles (LLVM); cold shapes
  1 each (a repeat is not cheap); `cargo check` 1 cold + 1 warm after edit + 2 warm.
  Medians are stated with their n. Every "settle" is the same command after the
  byte-exact restore, so each measurement starts from a clean tree.
- One heavy cargo step at a time: every `build.py` run went through the machine-wide
  slot, and the slot reported free before each of my runs. Every `ps` check during
  the measurement window (22:12-22:41) showed only my own cargo/rustc processes; the
  ~13 sibling checkouts `ENVIRONMENT.md` mentions are the reason to distrust small
  gaps. `python scripts/build_slot.py --status` reported a stale holder for this
  checkout at ~22:49 -- i.e. **after** my last measurement -- whose recorded pid was
  already dead. Load before/after every run is recorded in `tmp/probe/timings.txt`.
- sccache: the first `sccache --show-stats` (taken after every cranelift run and
  after the LLVM `cargo check` and `clippy` runs) showed **0 hits / 808 misses** --
  this container's sccache started cold, so **every cranelift run and every LLVM
  `check`/`clippy` run above is uncached**. The only cache activity in the whole
  probe is the 287 hits the protocol's LLVM cold-probe run consumed. The cranelift
  toolchain can never hit that cache (different compiler binary hash), which is why
  the "no compiler cache" LLVM run was added as its partner.
- Raw `cargo` in `target/cranelift-probe` (shapes B, C) always ran with the same
  `CARGO_TARGET_X86_64_UNKNOWN_LINUX_GNU_LINKER` and the same `RUSTFLAGS` as the
  `build.py` runs, so no unit was fingerprinted twice.
- Scratch: everything under `tmp/probe/` (`run_timed.sh`, `shapeA.sh`, `shapeBC.sh`,
  `llvmChecks.sh`, `llvmShapeA.sh`, `extraRuns.sh`, `interleaved.sh`, `mark_edit.sh`,
  `mark_restore.sh`, `tiny/`, `*.out`, `*.mem`, `timings.txt`, `peakmem.txt`,
  `game_state.rs.bak`). No destructive git command was used.

### Deviations from the protocol, disclosed

1. **Extra target dirs.** `target/llvm-cold-probe-nocache` (sccache-free LLVM cold
   run) and `target/cranelift-cold-probe` (cranelift cold-from-nothing run). The
   second deviates from "a dedicated target dir: `target/cranelift-probe`, always":
   I created a second cranelift-only dir instead of wiping the warm one, to get a
   fresh-dir pair for §2 without destroying the dir every earlier shape was measured
   in. No cranelift artifact ever entered `target/`. Both dirs are git-ignored
   scratch; the four probe dirs total 10.8 GB and are reclaimable with `rm -rf`.
2. **`RUSTC_WRAPPER=` (empty)** for the two "no compiler cache" cold runs. Env-only;
   it bypasses `scripts/sccache-wrapper.sh` (verified: sccache stats did not move
   during the LLVM one — 287 hits before = 287 after) and changes cargo fingerprints,
   which is harmless in a fresh dir. It does not touch the toolchain or the backend.
3. **Extra LLVM re-measurement of shape A, B/C repeats, and the
   `cargo check` cold-profile repeat** — requested by the lead additions, plus the
   third `cargo check` run because the first one was not an after-edit delta.
4. `tmp/probe/tiny/` is a throwaway crate that proves the backend flag reaches rustc.

## 7. Failures, verbatim

`rustup toolchain install nightly` printed, *after* installing the toolchain and
components (non-fatal — `rustup run nightly rustc -vV` works and the cranelift
component is present):

```
error: rustup is not installed at '/home/node/.cargo'
```

(Cause: `CARGO_HOME=/home/node/.cargo` while rustup lives in `/usr/local/cargo`.)

`cranelift clippy`, exit code 101, first 5 of 30 lint errors:

```
error: consider using `sort_by_key`  (src/adapters/driven/storage/games.rs:37:17)
error: this `if` statement can be collapsed  (src/adapters/driven/text_check/harper_text_checker.rs:31:9)
error: used `unwrap()` on an `Option` value  (src/adapters/driving/http/templates.rs:25:372)
error: used `unwrap()` on an `Option` value  (src/adapters/driving/http/templates.rs:25:553)
error: used `unwrap()` on an `Option` value  (src/adapters/driving/http/templates.rs:25:1749)
```

with, e.g.:

```
   = note: `-D clippy::unnecessary-sort-by` implied by `-D warnings`
   = help: to override `-D warnings` add `#[allow(clippy::unnecessary_sort_by)]`
```

All 30 are first-party lint regressions from the newer clippy; **none is a codegen
or cranelift error**. Any future cranelift `clippy`/full-gate run on this crate must
either carry an allow-list for them or accept exit 101 for reasons unrelated to the
backend. No other command failed: every codegen build, every `cargo nextest` run
(1 passed, 1827 skipped in every shape-A run) and every LLVM step exited 0.

## 8. What this does and does not support

Supported by measurement: warm-dir all-binaries edit→test is ~11.1 s with cranelift
against 13.2 s with LLVM today (n=6 each, +18 %/+5 %/+23 % in the three interleaved
pairs) and 20.45 s in the baseline's single run; an uncached cold all-binaries
codegen build is 198.6 s against 238.2 s; peak memory is not lower.

Not supported: that cranelift helps the *targeted* loops (shapes B and C move by
−4 % and −10 %, i.e. within this host's noise floor), that it helps `clippy` at all
(it cannot — metadata only), or that it lowers peak memory. Cranelift also cannot be
used on this crate's pinned 1.88.0 toolchain, and its nightly clippy currently fails
the crate; and every number above mixes backend with compiler version.

No design conclusion is drawn here, and nothing under `.scratch/` was touched.

## Appendix: per-run values

`tmp/probe/timings.txt` (wall ms, loadavg before/after) and `tmp/probe/peakmem.txt`
(peak summed RSS) hold the raw rows. Wall seconds:

- cranelift shape A after-edit: 10.713, 11.044, 17.720, 11.336, 11.090, 10.926
  (median 11.067, mean 12.138, max under load spike 17.720)
- LLVM shape A after-edit (today): 20.878, 12.470, 10.925, 13.852, 11.710, 14.100
  (median 13.161, mean 13.989)
- cranelift shape A settle: 10.683, 10.967, 12.634, 10.815, 10.520, 10.680 (median 10.749)
- LLVM shape A settle (today): 11.413, 11.928, 10.736, 12.028, 12.132, 11.389 (median 11.671)
- cranelift shape B after-edit: 4.596, 4.628, 5.474 (median 4.628); settles 4.782, 4.763, 5.120
- cranelift shape C after-edit: 5.039, 4.902, 5.563 (median 5.039); settles 5.039, 4.822, 5.278
- LLVM `clippy` after-edit: 14.391, 11.041 (median 12.716); settles 12.054, 11.648
- cold codegen: LLVM 174.838 (287 cache hits), LLVM no-cache 238.189,
  cranelift cold-dir 207.865, cranelift fresh no-cache 198.599
- cranelift `clippy` cold: 203.879, exit 101
- LLVM `cargo check`: 39.401 (check profile cold), 4.362 (after edit, warm),
  1.982 (warm), 1.896 (settle)

`logs/build_history.txt` rows for the `build.py` runs are at 22:16:08 (cranelift
clippy, 203.8, rc 101), 22:20:24 (cranelift cold codegen, 207.8), 22:20:48-22:41:01
(shapes A and clippy), 22:27:33 (LLVM cold probe, 174.7), 22:32:18 (LLVM no-cache
cold probe, 238.0).
