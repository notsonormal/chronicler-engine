# The protective copy of the cited `tmp/` evidence

On 2026-11-04, the plan review of this map found that the gate deletes cited evidence. `build.py` `run_gate` runs `_gate_tail` after each green full gate and each green `--no-browser` gate. `_gate_tail` deletes each file under `tmp/` that is older than 30 days. The open maps cite the folders below. The first deletes can come with the first green gate on or after 2026-11-05.

So each cited folder was copied with `cp -a`, which keeps the file dates, to `local-only/tmp-evidence/<folder>/`. The copy is git-ignored, and it stays on this machine only. Check on 2026-11-04: 8,811 files in the sources and in the copy, and the byte size of each copy is equal to its source.

The copy is a snapshot. A file that a map writes under `tmp/` after 2026-11-04 is not in it. [Decide the changes to the build tooling, the trial harness and the target dirs](../issues/04-grilling-tooling-trial-harness-target-dirs.md) decides the final home of the evidence and the change of the citations.

## The folders

A folder is listed when an open map (not `_closed`, not this map) names it as a repo path `tmp/<folder>`. A citation of `/tmp/<folder>` is not counted.

| Folder | Cited by | Oldest file | Size | Copied |
|---|---|---|---|---|
| `tmp/agents-restructure/` | retro-issue-bucket | 2026-10-06 | 92K | yes |
| `tmp/character-art/` | character-art-from-code | 2026-11-02 | 63M | yes |
| `tmp/comment-fixer/` | retro-issue-bucket | 2026-10-06 | 148K | yes |
| `tmp/db-backup-20261031/` | prompt-flexibility | 2026-10-31 | 3.1M | yes |
| `tmp/dd-fixpass/` | retro-issue-bucket | 2026-10-29 | 140K | yes |
| `tmp/dup_now/` | unclebob-quality-tools | 2026-10-09 | 5.2M | yes |
| `tmp/final-review/` | retro-issue-bucket | 2026-10-27 | 604K | yes |
| `tmp/impeccable/` | dashboard-design | 2026-11-01 | 3.0M | yes |
| `tmp/lt01/` | layout-trials, long-term-memory-and-state, retro-issue-bucket | 2026-11-01 | 43M | yes |
| `tmp/lt02/` | layout-trials | 2026-11-01 | 136K | yes |
| `tmp/lt03/` | layout-trials, long-term-memory-and-state, retro-issue-bucket | 2026-11-01 | 43M | yes |
| `tmp/lt05/` | layout-trials | 2026-11-02 | 448K | yes |
| `tmp/lt10/` | layout-trials, retro-issue-bucket | 2026-11-01 | 44M | yes |
| `tmp/lt11/` | layout-trials, retro-issue-bucket | 2026-11-02 | 2.3M | yes |
| `tmp/lt15/` | layout-trials, long-term-memory-and-state, retro-issue-bucket | 2026-11-02 | 79M | yes |
| `tmp/ltm03/` | long-term-memory-and-state | 2026-11-03 | 464K | yes |
| `tmp/ltm04/` | long-term-memory-and-state | 2026-11-03 | 272K | yes |
| `tmp/ltm09/` | long-term-memory-and-state | 2026-11-03 | 29M | yes |
| `tmp/ltm11/` | long-term-memory-and-state | 2026-11-04 | 380K | yes |
| `tmp/maid/` | character-art-from-code | 2026-11-02 | 1.1M | yes |
| `tmp/marinara/` | layout-trials | 2026-10-20 | 545M | no (clean public clone, see below) |
| `tmp/mutation-03/` | unclebob-quality-tools | 2026-10-31 | 23M | yes |
| `tmp/pf08/` | layout-trials, prompt-flexibility | 2026-10-31 | 51M | yes |
| `tmp/pf18/` | prompt-flexibility | 2026-10-31 | 1.2M | yes |
| `tmp/pf19/` | prompt-flexibility | 2026-10-31 | 5.1M | yes |
| `tmp/pf20/` | layout-trials, prompt-flexibility | 2026-11-01 | 42M | yes |
| `tmp/pf2021/` | retro-issue-bucket | 2026-11-01 | 68K | yes |
| `tmp/pstack/` | code-concepts | 2026-10-20 | 7.8M | yes |
| `tmp/reflect3/` | retro-issue-bucket | 2026-10-07 | 2.7M | yes |
| `tmp/review/` | layout-trials, retro-issue-bucket | 2026-10-06 | 28M | yes |
| `tmp/stage6/` | prompt-flexibility | 2026-10-26 | 86M | yes |
| `tmp/statechart/` | statechart-models | 2026-10-29 | 1.4M | yes |
| `tmp/tla-ts-run/` | statechart-models | 2026-10-31 | 4.0M | yes |
| `tmp/unclebob/` | unclebob-quality-tools | 2026-10-09 | 52M | yes |

## Not copied

`tmp/marinara/` is a clone of `https://github.com/Pasta-Devs/Marinara-Engine` with no local change (`git status` was empty on 2026-11-04, `HEAD` `c56501495`). A clone gives it again:

```bash
git clone https://github.com/Pasta-Devs/Marinara-Engine tmp/marinara
git -C tmp/marinara checkout 457e827a3d3291be978121f8c50c98c5dd9d644d
```

The layout-trials map names `457e827` as the pinned commit.
