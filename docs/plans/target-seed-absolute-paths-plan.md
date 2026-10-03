# Plan: Seeded target dirs reference the source checkout

**Date:** 2026-10-03
**Status:** Parked, not started, not scheduled. This plan is where the seeded-`build/` item lives now; it is **not part of the `dashboard-ui-updates` change set** and nothing in that branch depends on it. The two decisions below (D1, D2) belong to whoever picks this up, not to the dashboard work. Nothing here blocks other work.
**Sources:** measurements taken in this checkout against a warm `target/debug` on 2026-10-03; the "Known limitation" paragraph in `ENVIRONMENT.md`; item B1 of `docs/plans/dashboard-ui-updates-followups-plan.md`, which this plan takes over.
**Goal:** Decide whether a seeded target dir must be self-contained, and make it so only if the cost is small.

## Summary

`scripts/target_seed.py` copies a warm sibling checkout's build artifacts into a cold target dir so that only workspace crates compile. Some of the copied files carry **absolute paths into the source checkout**, so the seeded dir works only while that checkout's target dir exists. The measured surface is small: 10 files, from four C-building crates. The fix has one trap that decides everything — cargo compares the mtime of a build script's `output` file, so rewriting its content invalidates the fingerprint unless the rewrite also restores the mtime. Without that, a rewrite re-runs the very C builds the seeding exists to avoid, and is therefore pointless.

## What already exists

- `scripts/target_seed.py`: `_COPIED_DIRS` (`.fingerprint`, `build`, `deps`), `_shareable_hashes` (registry and git sources only), `_keep` (per-package filter), `copy_third_party` (walks and copies; `copy2` keeps mtimes, which cargo compares), and the staging rename `<target>/<profile>.seeding-<pid>` → `<target>/<profile>`.
- `build.py` seeds once per run for a cold target dir, and `probe_shared_lock` (shared with `is_target_locked`) keeps the source idle during the copy.
- `ENVIRONMENT.md` documents the limitation and names the rewrite as the open option.

## Measured state

Taken on 2026-10-03 against a warm `target/debug`:

| Thing | Count |
| --- | --- |
| `build/*/output` files | 81 |
| `build/*/root-output` files | 81 |
| `output` + `root-output` files containing an absolute path into the source checkout | 99 |
| ... of those, files whose link directives carry such a path | 10 |
| `root-output` files with link directives | 0 |
| Crates emitting those link directives | `aws-lc-sys` (4), `zstd-sys` (2), `ring` (2), `libsqlite3-sys` (2) |
| `deps/*.d` files containing an absolute target path | 882 |
| `.fingerprint` files containing an absolute source path | 0 |

The shape of a stale directive, in `build/aws-lc-sys-<hash>/output`:

```
cargo:rustc-link-search=native=<source-checkout>/target/debug/build/aws-lc-sys-<hash>/out
```

## Mechanism

- The seed copies the whole `build/<pkg>-<hash>/` subtree, `out/` included. The C libraries **do** exist at the mirrored path in the destination; only the recorded path is stale.
- Cargo does not re-run a fresh build script. It replays the directives in `output`, so a seeded dir links against the source checkout's `out/` directories.
- Cargo's build-script fingerprint stores `output` as a path **relative** to the target dir and compares its mtime: `local: [{"RerunIfChanged": {"output": "debug/build/<pkg>-<hash>/output", "paths": ["build.rs"]}}]`. `copy2` preserves that mtime, which is why a copied `output` stays fresh.
- Any rewrite changes the mtime, so cargo re-runs that build script — recompiling the C library — unless the rewrite restores the original mtime with `os.utime`.

## When it actually breaks

Inferred, not yet observed: the seeded dir fails once the source checkout's target dir disappears (a `cargo clean` in the sibling, a deleted worktree, a disk cleanup) and the seeded dir next relinks. The linker error names a path that no longer exists, which is hard to trace back to the seeding. Nothing else is affected, and `ENVIRONMENT.md` already warns about it.

## The experiment that decides it

Do this first; it is cheap and it chooses the option.

1. Seed a scratch target dir from a disposable copy of a warm profile, not from the shared one.
2. Rewrite the 10 `output` files to the destination prefix, restoring each file's original mtime.
3. Build in the seeded dir and check whether the four C crates stay fresh (no `Compiling aws-lc-sys` and friends).
4. Delete the source copy, force a relink (a trivial edit plus a build), and confirm the link succeeds.
5. Separately, measure the alternative: exclude `build/` and time the four C rebuilds on a cold dir.

Steps 3 and 5 produce the numbers the decision needs.

## Options

| Option | Change | Cost |
| --- | --- | --- |
| A. Rewrite and restore mtime | rewrite in `copy_third_party` using the final profile path (not the staging path), then `os.utime` | Keeps the C builds fresh, so seeding keeps its full benefit. About 15 lines. Needs the experiment to confirm freshness. |
| B. Exclude `build/` | drop one entry from `_COPIED_DIRS` | Every cold dir re-runs all 81 build scripts, four of them C compiles that sccache does not cover. |
| C. Exclude `build/` for the four crates | filter by package in `_keep` | Same C cost as B. Keeps the other 77 build-script outputs, which are cheap to re-run anyway. |
| D. Do nothing | none | The limitation stands with its `ENVIRONMENT.md` warning, and a sibling cleanup gives a confusing link error. |

## Recommendation

Run the experiment, then decide. If a rewritten `output` with a restored mtime keeps the C crates fresh, take A: the seeded dir becomes self-contained at no build-time cost. If it does not, take C: the same C cost as B, the cheap artifacts kept, and no path rewriting to maintain.

## Acceptance

- The chosen option lands with a reproducible check: seed a scratch dir, delete the source, relink successfully.
- `ENVIRONMENT.md`'s "Known limitation" paragraph is updated either way — removed if A lands, reworded with the measured cost if B or C lands, or left alone if D is chosen.

## Not in scope

- sccache behaviour, the C crates' own build scripts, and the choice of seeding source (`_SEEN_LIMIT`, `build_signature`).
- The 882 `.d` files carrying absolute artifact paths. They are a second class of stale reference. Whether cargo cares is unknown; the experiment above will show it.

## Failure modes

| Failure | Mitigation |
| --- | --- |
| A wrong prefix points a cached directive at nothing | The experiment's relink step. Fall back to B or C. |
| Restoring the mtime hides a script that genuinely needed re-running | Restore the mtime only for files whose content this code changed, and only in the seeded copy. |
| Excluding `build/` slows every cold build | Measure the four C rebuilds before choosing, and keep A if the rewrite works. |

## Unresolved decisions

| # | Question | Recommendation |
| --- | --- | --- |
| D1 | Fix the seeding, or keep the documented limitation? | Fix if the experiment shows A keeps the C builds fresh; otherwise fix with C, which is one filter. |
| D2 | If fixed, which option? | A, falling back to C. |
