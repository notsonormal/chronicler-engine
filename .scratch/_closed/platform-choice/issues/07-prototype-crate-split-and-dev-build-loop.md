# Prototype: decide the crate split and the dev-build loop

Type: prototype
Status: resolved (2026-10-16)
Blocked by: 04, 05, 11

## Question

Does a split of the crate into several crates, or the Cranelift backend for dev builds, make the edit-to-test loop (problem A) and the parallel-agent limits (problem B) acceptable? If yes, which split follows the module edges of the new design?

## Context

- **Problem A** (known, seed §1): `test-pattern` median 3.3 s, p90 30.6 s; one example took 53.2 s for the first run after a Rust edit, then 2.6 s. `clippy` median 13.0 s.
- **Problem B** (known, seed §2): one heavy cargo step at a time across all checkouts (`scripts/build_slot.py`); 3 CPUs, 8 GiB; three overlapping builds ran 2.1 times slower; a cold worktree costs 344 s seeded, 728 s not seeded.
- Neither option is measured (seed §4). Cranelift needs nightly Rust. Cranelift does not depend on the module shape, so that part can run first.
- The split follows the module edges of the new design: [sketch 25](../../turn-state-model/assets/sketch-25-new-design.md) and the result of [Research: sort the open architecture-deepening tickets against the new design](04-research-sort-architecture-deepening.md).
- **Added by ticket 04 (2026-10-16).** [Decide the storage seam of the rebuild](11-decide-storage-seam-of-rebuild.md) sets which way `application/` and `storage/` depend on each other, so a crate layout waits for it. The Cranelift measurement does not depend on it.
- [Seed: build, test and port facts](../assets/seed-build-and-port-facts.md). `ENVIRONMENT.md`.

## Done when

- Measured on the real crate, under the host limits: the first targeted test after a one-line edit, `clippy`, a cold worktree, and peak memory, for today's crate and for each option. Treat gaps under 10 % as noise.
- A proposed crate layout, if the split earns its cost.
- The owner decides what the build plan takes.

## Comments

**2026-10-16, from [Decide the storage seam of the rebuild](11-decide-storage-seam-of-rebuild.md) (resolved).** The repositories are methods on `Storage`, with no port trait (R2). So the dependency direction is `application → storage`, as today, and a crate split puts storage below the application. No arch-lint change.

## Answer

**Ruling (owner, 2026-10-16). No crate split. No Cranelift. The loop's fix is target selection:**
`build.py test-pattern` gains an optional test target, so a targeted test builds only the test
binary it needs.

Measurements, all on a one-line marker edit in `src/domain/model/state/game_state.rs`, file restored
byte-exact and `cmp`-verified after every run.

| Shape | Today (LLVM 1.88.0) | Cranelift (nightly 1.101) |
|---|---|---|
| All 9 test binaries, after an edit | 13.16 s (n=6; one earlier sample 20.45 s was a load spike) | 11.07 s (n=6) |
| Lib unit binary only, after an edit | 4.83 s (n=1) | 4.63 s (n=3) |
| One test target (`--test http`), after an edit | 5.60 s (n=1) | 5.04 s (n=3) |
| Warm, no edit | 2.1 s | 2.52 s |
| Cold codegen build, uncached | 238.19 s | 198.60 s |
| Cold codegen build, `build.py` command as written | 174.84 s (287 sccache hits) | 207.87 s |
| Peak RSS, all 9 after an edit | 1.99 GB (n=6) | 2.04 GB (n=6) |
| `clippy` after an edit | 12.72 s (n=2), exit 0 | exit 101 |

`cargo check` after an edit (front end only, LLVM, warm dir): 4.36 s. Decomposition of the 13 s:
the lib with `cfg(test)` plus its unit-test binary 5.77 s; the lib plain plus the other eight test
binaries, compile and link, 8.55 s. The lib is compiled twice per edit.

**Why no split.** About 8.5 s of the 13 s is eight test binaries that depend on the package lib, so
no crate layout reaches them. The rest is the lib, and rustc's incremental compilation already
isolates that work per changed module; the churn sits in `adapters/` and `application/` (36,625 of
the 49,032 lines under `src/`), which a split would leave as the top crate. A split also costs the
`arch-lint` root (`root = "./src"`, and 0.4.3 cannot make an exception) and the `src/`-relative
guardrail tooling. The split was **bounded, not built**: the owner declined a measured split
prototype, so no crate layout is proposed.

**Why no Cranelift.** It does not help the targeted shapes (−4.2 % and −10.0 %, inside this host's
10 % noise floor). Its 15.9 % gain on the all-binaries shape sits at the noise ceiling (three
interleaved pairs: +18 %, +5 %, +23 %) and that shape is the one target selection removes. Its cold
win holds only with an empty compiler cache: against the seeded, sccache-warm path it is 19 %
slower, and seeding and sccache are what make a cold worktree affordable. Its price: a pinned
nightly toolchain, a failing `clippy` until 30 first-party lints are addressed, a preview codegen
backend, and no memory gain. Every cranelift number also carries a compiler-version difference
(1.88.0 against 1.101.0-nightly), because cg_clif is nightly-only.

**What the build plan takes.**

1. `build.py test-pattern` gains an optional target (`cargo nextest run --test <TARGET>`, or `--lib`),
   with today's all-binaries behaviour as the default. Measured effect: 13.16 s → 5.0-5.6 s.
2. Problem B is unchanged by both options. The build slot, target seeding, per-worktree
   `--target-dir` and the browser thread limit stay; the shorter loop holds the slot for about a
   third of the time, which is the one effect on B.
3. Trade-off to carry: a narrow loop does not compile the other eight test binaries, so a change to a
   shared API surfaces in the full gate instead of the fast loop.

The loop stays cheap under the rebuild's own test rule
([Decide the test rule outside the designed scope](08-decide-test-rule-outside-designed-scope.md)):
that rule moves unit tests into seam contracts, which is the library-unit binary half of the loop.

**Assets.** [probe 07: the dev-build loop, measured](../assets/probe-07-dev-loop.md) (baselines,
decomposition, the cost side of a split);
[probe 07: the Cranelift run](../assets/probe-07-cranelift.md) (the delegated report, per-run values,
verbatim failures). The 11 GB of probe target dirs and the nightly toolchain were removed after the
ruling.
