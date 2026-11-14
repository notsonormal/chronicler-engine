# Tools in work records after the move to `work/`

A recurring problem: a script or a command in a work-record asset names a path from before the move.

- On 2026-11-11, the chief of staff found that `memory_test.py` of the map [Long-term memory and state](../wayfinder-maps/long-term-memory-and-state/map.md) did not start. As of `4baba0a4`, it found the repo root as `parents[3]` of its own path. The move of the work records from `.scratch/` to `work/` (`ec323d14`) made the path one folder deeper, so the import of `scripts/trial.py` failed. `card_probe.py` imports `memory_test.py`, so it failed too.
- On 2026-11-11, the chief of staff changed `memory_test.py` to find the repo root by a search for `scripts/trial.py` in its parent folders. The run commands in `memory-test.md`, `card-probe.md` and the two docstrings changed from `.scratch/long-term-memory-and-state/` to `work/wayfinder-maps/long-term-memory-and-state/`. `memory_test.py report` and `check` ran after the change. The change was not committed on that day.
- On 2026-11-11, `rg` counted 1,655 lines with `.scratch/` in 188 files under `work/`. Most are records of closed efforts, and nobody runs them. The move rewrote links, and it did not rewrite paths inside code blocks or scripts.
- A rule from this: before a subagent runs a tool from a map asset, run the tool once with `--help` or its cheapest subcommand.

Related: [Long-term memory map run](long-term-memory-map-run.md).
