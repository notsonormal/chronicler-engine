# Development Loop

Reach this doc when you build, format, lint, test, or start the engine.

## Working rules

Write temporary files into a `tmp` folder, for example `tmp/`.

Re-read the exact target region immediately before every file edit. Edit from the file's current content, never from remembered or truncated output. Read back a multi-block edit before you run further commands.

Give a file-read tool an exact file name. When the name is unconfirmed, list the directory first.

## Build logs

`build.py` writes logs to standard output and to `logs/build_*.log`. Each log carries the pi session id, so the `mrn-context` extension attributes the log to the session that ran it.

Tail the last 10 lines of the run's log file — not piped stdout — to read the test results:

```bash
tail -n 10 "$(ls -t logs/build_*.log | head -1)"
```

The tail holds a line such as `nextest: 1482 passed, 0 failed, 2 skipped`.

Give the Pi bash tool a 1200-second timeout when you call `build.py`.

## Commands

All build actions go through `build.py`.

### Iteration (use these while fixing)

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

Almost every full-gate step is also a subcommand. Run `python build.py --help` for the list. Only the packaging, test-suite, and coverage-report phases stay gate-internal. `--target-dir` works on either side of the subcommand. All other top-level flags are full-gate only.

## Final validation

Run this command once before you consider the task done:

```bash
python build.py # Full gate: fmt + clippy + guardrails + tests (~2 min warm)
```

Most of the gate time is the browser tests. A full suite run just before `build.py` repeats that work. Instead, run a targeted step (`python build.py test-pattern <pattern>`) or run `build.py` straight away.

## Concurrent builds

Use one target dir per checkout, and keep using it. A new dir starts cold.

In a git worktree, run `python build.py` with no extra flags. When several agents share one checkout, each agent runs `python build.py --target-dir target/<name> --no-fmt`.

The `build.py` docstring holds the remaining detail.

Read `ENVIRONMENT.md` when a build or test run is slow, waits for the build slot, or runs out of memory. It holds the reason for the build slot, target seeding, and sccache.
