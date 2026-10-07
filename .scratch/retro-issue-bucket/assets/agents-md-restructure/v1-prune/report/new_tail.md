
## Your Responsibility

The repository must stay healthy: a passing build outranks your task's success. Never delete or revert an unknown or unexpected file, especially an untracked one — it may belong to work in progress elsewhere in the repository. Leave it in place even when it blocks your task.

During code reviews, make no code changes. Report the problems instead.

## Communication

Label epistemic status when it matters: known, inferred, or guessed.

When you answer user feedback or an analysis, say whether you agree or disagree before you say what you changed.

### Decision Making

Before you implement (during planning, for example):

- **State your assumptions explicitly.** If uncertain or something is unclear, stop, name what's confusing, and ask.
- **If an instruction contradicts what you see, say so before acting.** Do not silently work around the mismatch or proceed as if the instruction were accurate.
- **If multiple interpretations exist, present them** — Don't pick silently unless obvious.
- **Hold a reasoned position.** Push back when a simpler approach exists, and if the user pushes back while your reasoning still holds, say why.
- **Surface hidden trade-offs**: When generating code with architectural implications the user did not ask about (introducing a dependency, choosing an async pattern, picking a data structure with different complexity), name the trade-off in the response.

### Progress updates
Before your first tool call, one sentence on what you're about to do. While working, update only when you find something important or change direction, not before each tool call. When done, lead with the outcome: first sentence answers "what happened" or "what did you find", detail after.

## The Test-First Philosophy

Read a component's tests in `tests/` before its source; before fixing a bug, find or create a failing test case — and never rationalize a failure away. Integration tests go in `tests/`; the failure-handling protocol lives in `tests/AGENTS.md`.

## Documentation Index

`docs/AGENTS.md` catalogues the docs under `docs/diataxis/`; `tests/AGENTS.md` catalogues the integration test files. Read them when you search docs or tests.

A pre-commit hook regenerates and stages every generated index — those two files, the Structure section above, and the guardrails doc. Never hand-edit a generated block. Without the hook, regenerate with the `scripts/generate_*_index.py` scripts.

`CODING_STANDARDS.md` holds the implementation, editing, code-comment, code-review, and testing rules. Read it before you write, edit, or review code.

## Development Loop

`build.py` is the only build entry point. Every run writes `logs/build_*.log`, stamped with the pi session id, and prints that log to standard output.

Use the Pi bash tool with a timeout of 1200 seconds when you call `build.py`. Read the result from the run's log file, not from piped stdout — tail its last 10 lines:

```bash
tail -n 10 "$(ls -t logs/build_*.log | head -1)"
```

The tail prints the test result, for example `nextest: 1482 passed, 0 failed, 2 skipped`.

### Commands

Run `python build.py --help` for the step list and the flags; it names every step and states its cost. Almost every full-gate step is also a subcommand. Packaging, the test suite, and coverage stay gate-internal. `--target-dir` works on either side of the subcommand; all other top-level flags are full-gate only.

#### Iteration (use these while fixing)
Run `python build.py clippy` first and fix every warning. Then run the tier your change touches: `unit`, `architecture`, `guardrails`, or `integration` (every test binary except browser, architecture and guardrails). `browser` runs only the Playwright binary. `test-pattern "<substring>"` runs the tests whose name matches, across all test binaries. `validate-docs` validates the markdown. `run` starts the dev server (see `ENVIRONMENT.md`).

#### Final Validation (run once before considering done)

```bash
python build.py # Full gate: fmt + clippy + guardrails + tests (~2 min warm, far longer cold)
```

The browser tests dominate the gate's runtime. Run one targeted step instead of the full suite, or run the gate directly.

## Concurrent Builds

Use one target dir per checkout, and never switch mid-task: a new dir starts cold. In a git worktree, run `python build.py` with no extra flags. Agents that share one checkout run `python build.py --target-dir target/<name> --no-fmt`. The `build.py` docstring has the details.

`ENVIRONMENT.md` covers slow builds, the build slot, and out-of-memory kills.

Cold worktree builds compile the whole dependency tree into a fresh target dir. `scripts/sccache-wrapper.sh` (wired as `rustc-wrapper` in `.cargo/config.toml`) routes that through sccache, so the second build onward reuses the artifacts. Check it with `sccache --show-stats`.

## Agent Skills

### Issue tracker

Local markdown under `.scratch/` (per `docs/agents/issue-tracker.md`). No `gh` CLI dependency.

### Triage labels

Five canonical role strings, used as `Status:` lines in local-markdown files (per `docs/agents/triage-labels.md`).

### Domain docs

Single-context: one `CONTEXT.md` glossary at the repo root (per `docs/agents/domain.md`).

## Permissions System

Read `.pi/extensions/pi-permission-system/config.json` to see the allowed permissions. Never circumvent them, and never edit that config without explicit user approval — you may recommend a change at the end of a task.

Never commit without explicit approval, even when the config allows commits. Normal git commands are fine; avoid destructive git commands.

## Subagents and delegation extra rules

Prefer one long-running subagent over several short ones: loading the context twice costs more. Some workflows use several subagents by design — code reviews, and tickets that implement several wayfinder tickets at once.

Only create `scout` subagents when the current model is Anthropic (e.g. Opus or Sonnet). With any other model, read the information you need in the current session. This rule applies to `scout` subagents only.