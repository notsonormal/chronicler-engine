# Development Loop

How to work here: communication and decision norms, the test-first stance, the build loop, and build concurrency.

## Communication

Label epistemic status when it matters: known, inferred, or guessed.

If you don't know something, say "I don't know" instead of inventing an answer.

When responding to user feedback or an analysis, explicitly say whether you agree or disagree before saying what you changed.

## Decision making

**Don't assume. Don't hide confusion. Surface tradeoffs.**

Before implementing (e.g. during planning):

- **State your assumptions explicitly.** If uncertain or something is unclear, stop, name what's confusing, and ask.
- **If an instruction contradicts what you see, say so before acting.** Do not silently work around the mismatch or proceed as if the instruction were accurate.
- **If multiple interpretations exist, present them** — don't pick silently unless obvious.
- **Hold a reasoned position.** Push back when a simpler approach exists, and if the user pushes back while your reasoning still holds, say why.
- **Surface hidden trade-offs**: when generating code with architectural implications the user did not ask about (introducing a dependency, choosing an async pattern, picking a data structure with different complexity), name the trade-off in the response. Do not bury it.

These guidelines bias toward caution over speed. For trivial tasks, use judgment.

### Progress updates

Before your first tool call, one sentence on what you're about to do. While working, update only when you find something important or change direction — not before each tool call. When done, lead with the outcome: first sentence answers "what happened" or "what did you find", detail after.

## Test-first philosophy

A comprehensive suite of unit and integration tests is the ultimate source of truth for behavior. Read a component's tests in `tests/` before its source; before fixing a bug, find or create a failing test case. Integration tests go in `tests/`; the failure-handling protocol lives in `tests/AGENTS.md`.

Avoid **analysis paralysis**: when reasoning stops converging, act instead — read, run, or write a test, check the UI directly in the browser, or add logging and diagnostics to the production code.

## The loop

Temporary files should be written into tmp folders e.g. `tmp`.

Re-read the exact target region immediately before every file edit — edit from the file's current content, never from remembered or truncated output — and read back multi-block edits before running further commands. Never pass glob or wildcard patterns to file-read tools; if the exact name is unconfirmed, list the directory first.

`build.py` writes logs to both standard output and to the `logs/` folder. Use the Pi bash tool with a timeout of 1200 seconds when calling `build.py`. Tail the last 10 lines of the run's log file — not piped stdout — to get the results of the tests i.e. `nextest: 1482 passed, 0 failed, 2 skipped`:

```bash
tail -n 10 "$(ls -t logs/build_*.log | head -1)"
```

### Commands

All build actions go through `build.py`. Every run writes `logs/build_*.log`, stamped with the pi session id so the `mrn-context` extension attributes the log to the session that ran it.

```bash
python build.py fmt                             # Format sources (rewrites files in place)
python build.py check                           # Fast compile check across all targets (no lint)
python build.py clippy                          # ~10s — fix warnings here
python build.py unit                            # Run the unit tests
python build.py architecture                    # Run the architecture tests
python build.py guardrails                      # Run the guardrails tests
python build.py test-pattern "action_pipeline::options_tests" # Run tests whose name matches a substring, across all test binaries
python build.py integration                     # Every test binary except browser, architecture and guardrails (~20s)
python build.py browser                         # Only the browser/Playwright binary (~1 min)
python build.py validate-docs                   # Validate markdown docs
python build.py run                             # Run the dev server (see ENVIRONMENT.md)
```

Almost every full-gate step is also a subcommand — see `python build.py --help`. Only the packaging, test-suite, and coverage-report phases stay gate-internal. `--target-dir` works on either side of the subcommand; all other top-level flags are full-gate only.

### Final validation (run once before considering done)

```bash
python build.py # Full gate: fmt + clippy + guardrails + tests
```

A majority of the time taken by `build.py` is the browser tests. Running the full suite just before running `build.py` is inefficient. Either run a targeted step (`python build.py test-pattern <pattern>`) or skip them and run `build.py` straight away.

## Concurrent builds

Use one target dir per checkout and never switch, because a new dir starts cold. In a git worktree, run `python build.py` with no extra flags. Agents that share one checkout run `python build.py --target-dir target/<name> --no-fmt`. The `build.py` docstring has the details.

Read `ENVIRONMENT.md` when a build is slow, waits for the build slot, or runs out of memory: it owns the CPU and memory limits, target seeding, the lld linker, and sccache.
