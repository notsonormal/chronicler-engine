# Research how a Hex change goes from branch to installed app

Type: research (AFK)
Status: resolved
Blocked by: —

## Question

Which path does a Hex change take from a branch to the app on the owner's machine, and which checks run on the way? Where does this differ from `build.py` and Chronicler's commit rules?

## Context

- **Hex sources.**
  - `.github/workflows/`: 15 files, among them `check-linux.yml`, `check-macos.yml`, `check-windows.yml`, `test-builds.yml`, `build-desktop.yml` and `agent-policy.yml`
  - `.github/release/` and `.github/CODEOWNERS`
  - `scripts/`: `check`, `build-app.sh` and `install-app.sh`
  - `.hivemind.toml`: a land queue with `batch_max = 4`, a `verify` list, and an install to `/Applications`
  - the branch rules in `AGENTS.md`, `## Private Kit Tool`: `dev` and `production` branches, and agents merge their own PRs when the checks pass
  - `docs/test-builds.md` and `docs/releasing.md`
- **What to find.**
  - what runs on each PR and on each push, and how long it takes
  - which checks are required
  - what "land" does
  - how a bad change is caught after the merge, for example through dev-channel builds and test builds
  - how merges and reverts show in the git history
- **Chronicler side.**
  - `build.py` and `scripts/build_history.py`
  - `scripts/install_git_hooks.py` and `scripts/git-hooks/`
  - `ENVIRONMENT.md`
  - `.pi/extensions/pi-permission-system/config.json`
  - the `AGENTS.md` rule "Don't commit without explicit approval"
- **Overlap.** The ticket "Research the Hex issue pipeline from its public PRs and issues" covers how the pipeline uses the agent policy. This ticket covers where the policy sits in the path.
- **Candidates this touches** ([findings asset](../assets/findings.md)): 1 (risk gate) and 4 (small units that land automatically).

## Answer

Hex lands one squashed commit on `dev` per pull request. The agent merges its own PR when the checks pass. Four checks gate a PR: a deterministic policy gate run from the base branch, `check-linux` with a Nix job, `check-macos`, and `check-windows`. Three are path-filtered. A push to `dev` repeats the platform checks, publishes a dev-channel build, and makes downloadable test builds. `.hivemind.toml` then runs three `verify` commands, builds `Hex.app`, installs it to `/Applications`, and stamps the commit. Ten `fix(ci)` commits on 2026-10-31 show that this post-merge path catches cross-platform breakage that a local loop misses.

Chronicler has no remote gate: no `.github/`, `build.py` on one host, and owner approval for each commit. Nine practices: 3 partly covered, 1 covered differently, 5 gaps. Six are new (not in the findings asset). Durations and required checks are unknown, because both live outside the clone.

Asset: [research-06-branch-to-installed-app.md](../assets/research-06-branch-to-installed-app.md)
