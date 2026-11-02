# Research 06: how a Hex change goes from a branch to the installed app

Asset of the [map](../map.md). Subject: [anomalyco/hex](https://github.com/anomalyco/hex) at commit `7609574ca92c` (2026-10-31), clone in `tmp/hex-compare/anomalyco-hex/`. The Chronicler side is this repo.

Each claim is labelled **known** (read in a file, a command output, or a commit) or **inferred** (my reading of that evidence).

## Summary

- Hex lands one squashed commit on `dev` for each pull request. `dev` is the only integration branch, and a pull request is the only door into it (`AGENTS.md:5-11`). **Known.**
- A pull request runs `agent-policy` (a deterministic diff sort), `check-linux` with a Nix job, `check-macos`, and `check-windows`. Three of these are path-filtered. **Known.**
- A push to `dev` starts a dev-channel release and a set of downloadable test builds. Both catch a bad change after the merge. **Known.**
- The owner's machine gets the app from `.hivemind.toml`: a land queue runs three `verify` commands, builds `Hex.app`, installs it to `/Applications`, and stamps the commit in `~/Library/Caches/hex-install/installed-rev`. **Known** (config only; the queue program is private, so its behaviour is **inferred**).
- Chronicler has no remote gate at all: no `.github/`, and `build.py` runs on the local machine. The owner approves every commit (`AGENTS.md:414`). **Known.**
- Nine practices follow. Three are partly covered, one is covered differently, five are gaps. Six are **new** (not one of the eight candidates in the [findings asset](./findings.md)).

## Practices

| # | Practice | Hex evidence (file:line, commit) | Chronicler state | Candidate it touches |
|---|---|---|---|---|
| 1 | One integration branch, and a pull request as the only door. The agent merges its own PR when the checks pass. `production` moves only by a fast-forward from `dev`. | `AGENTS.md:5-11`; `docs/releasing.md:175-180`; history shape in §5 | **Partly covered.** Branches and a remote exist (`.git/config`, `git branch -a`), and 28 merge commits exist in 868. No file states who merges, or that a pull request is required. `AGENTS.md:414` says the owner approves each commit. | 4 (small units that land) |
| 2 | A deterministic risk gate in the PR path, run from the base branch so a patch cannot change its own policy. | `.github/workflows/agent-policy.yml:3-12,36-45`; `.github/agent-policy/check.ts` | **Gap.** `tests/infrastructure/guardrails/` checks structure. `.pi/extensions/pi-permission-system/config.json` limits the agent. No step sorts a diff by risk. | 1 (risk gate) |
| 3 | Path-filtered checks: a change runs only the workflows whose paths it touches. A docs-only push cuts no release. | `check-linux.yml:3-46`; `check-macos.yml:3-37`; `release.yml:55-58` (`paths-ignore`) | **Gap.** `build.py:1191-1203` lists 11 fixed checks, and `build.py:1264` runs them in one fixed order. The only reductions are step mode, `--no-browser` and `--no-fmt` (`AGENTS.md:380-386`). | **new** (change-scoped checks) |
| 4 | Every pull request builds and tests on three operating systems, plus a Nix check. Each platform has its own job. | `check-linux.yml:57-162`; `check-macos.yml:48-74`; `check-windows.yml:19-101` | **Gap.** No `.github/` in the tree, so no CI. The gate runs on one host, about 145 s warm (`AGENTS.md:387`). A macOS-only or Windows-only break cannot show. §4 shows Hex catching such breaks 10 times on 2026-10-31. | **new** (the cross-platform matrix) |
| 5 | One check-loop script for the local machine, with a filter argument and one exit code per stage. | `scripts/check:1-44` (exit 10 fmt, 11 clippy, 12 tests) | **Partly covered.** `python build.py clippy` and `python build.py test-pattern "<pat>" --test-target <name>` (`AGENTS.md:380-382`). `build.py:1581` returns 1 for any failed step, so the exit code does not name the stage; the epilogue names it. | **new** (staged exit codes) |
| 6 | A land queue: verify, build, install to `/Applications`, and stamp the installed commit. The queue holds at most 4 PRs and waits for the app to be idle. | `.hivemind.toml:1-17`; `AGENTS.md:7-8` ("After merging, update the installed copy"); `scripts/install-app.sh:1-45` | **Partly covered.** `build.py:1301-1319` copies `data/` and `assets/` into `target/<profile>/`, clears that dir's DBs, and prints "Deployment: copy the target/ folder to your target machine". `build.py run` starts the server there (`ENVIRONMENT.md:93`). There is no install script, no idle wait, and no installed-revision stamp. | 4, plus the smaller habit "installed-rev stamp" |
| 7 | After a push to `dev`, two automatic paths carry the change to a machine: a dev-channel release that installs follow, and unsigned test builds that sit beside the release app. | `release.yml:43-58`; `docs/releasing.md:130-157`; `test-builds.yml:3-11,60-99`; `docs/test-builds.md:3-30` | **Gap.** No channel, no artifact, no installable build. The owner sees a bad change in the browser or at the next gate run. `logs/build_history.txt` records the gates (`scripts/build_history.py:1-25`). | **new** (post-merge channels) |
| 8 | A release boundary: build jobs hold no secret, sign and publish jobs hold the secrets and check out only `.github/release/`, and `SHA256SUMS` digests pass between jobs. `CODEOWNERS` plus a ruleset let one person change `.github/`. | `docs/releasing.md:11-28,74-80`; `release.yml:1-42`; `.github/CODEOWNERS:1-3`; `.github/release/` (22 files) | **Gap.** No release process and no secret boundary. `origin` is `git@github.com:notsonormal/chronicler-engine.git`. | **new** (release boundary) |
| 9 | The commit path runs no generator. All checks are CI or an explicit script call, so a commit never rewrites a tracked file. | no hook config in the repo; `.git/hooks/` holds only `*.sample` files; `docs/releasing.md:30-48` names the check jobs | **Covered differently.** `scripts/git-hooks/pre-commit:1-25` runs `scripts/precommit_regenerate.py`, which regenerates four indexes, stages them, and refuses when hand-written prose would be swept in. `build.py` installs the hook (`install-hooks` step, `build.py:1264`). | **new** (no commit-time generation) |

## 1. The path in one picture

```
branch agent/<slug>  ──PR──▶  checks (4 workflows) + policy gate
                                   │
                                   ▼
     agent squash-merges into dev  ──push──▶  check-windows, test-builds
                                   │          release (dev channel)
                                   │          site, release-typescript
                                   ▼
     .hivemind.toml [land]  ── batch_max=4, verify ×3 ──▶ build Hex.app
                                   │
                                   ▼
     [land.app] install_cmd ──▶ /Applications/Hex.app, stamp installed-rev
                                   │
                                   ▼
     production ← fast-forward from dev ──▶ Release (stable) ──▶ release service
```

**Known:** the boxes are the files named in each section below. **Inferred:** the land queue runs after a merge, because `[land] branch = "dev"` and `AGENTS.md:7-8` says "After merging, update the installed copy".

## 2. What runs, on which event

All 15 workflow files, by trigger (**known**, read in each file):

| Event | Workflow | Runs |
|---|---|---|
| pull request to `dev` | `agent-policy.yml` | `bun .github/agent-policy/check.ts` on the diff, checked out at the **base** sha. Exit 2 fails the job. Exit 1 fails it unless the `agent-strict-review` label is present. Only for head branches `agent/*`. |
| pull request, path match | `check-linux.yml` | `cargo fmt --all --check`, nextest (`--profile ci`) for `hex` and `scrawl-router`, X11 grabs on Xvfb, Wayland paste into GTK, virtual-microphone capture/inference/paste, signed-installer tamper test, service IPC test, clippy. A second job runs `nix flake check` and builds the Nix package. |
| pull request, path match | `check-macos.yml` | fmt, `scripts/test-app-identity.sh`, clippy, nextest (`--profile ci`), a CA-root check on the binary. |
| pull request, any path | `check-windows.yml` | x64: fmt, clippy `--release --all-targets`, nextest `--release`, a Visual C++ Redistributable import check. ARM64: nextest and build only. |
| pull request, path match | `site.yml`, `windows-installer.yml` | Site lint/build, and the Windows installer script test. |
| push to `dev` | `check-windows.yml`; `check-linux.yml` and `check-macos.yml` on path match | The same checks as the pull request. |
| push to `dev` | `test-builds.yml` | `build-desktop.yml` (Linux x64/ARM64 with `.deb`/`.rpm`, Windows x64/ARM64), `windows-installers.yml`, and a macOS arm64 **Hex Test.app** DMG. Artifacts kept 14 days. |
| push to `dev` | `release.yml` | A dev-channel build, skipped when only `docs/**`, `site/**` or `*.md` changed (`release.yml:55-58`). Publishes `0.0.0-dev-<run>`. |
| push to `dev` | `site.yml`, `release-typescript.yml` (sdk paths), `installers.yml` (release-script paths) | Site deploy, npm SDK publish, installer publish. |

**Durations.** The repository does not record them, and this session used no GitHub API. **Known** from `docs/features/release-verification.md:88-90` and `:125-126`: the Linux job took 3m50s and 4m15s, and the Nix job 40m10s and 39m58s, in September 2026 runs of these workflow files. The Nix job dominates the wall clock. **Known:** the timeouts are 60 min for `check-linux`/`check-macos`/`test-builds` macOS, 90 min for `check-windows` and for the Nix job. **Unknown:** the current duration of `check-macos`, `check-windows`, and the dev-channel release.

**Which checks are required.** **Unknown.** Required status checks live in a GitHub ruleset, which is not in the repository. **Known:** `docs/releasing.md:74-80` names one ruleset, "release boundary (.github needs Dax)", on `dev` and `production`: pull requests only, code-owner review, Dax the one bypass actor. **Inferred:** a second ruleset lists the required checks, because `AGENTS.md:7` says "merge it yourself once its checks pass" without naming them.

**Where the policy sits.** `agent-policy.yml` is a separate workflow, not a step inside another check. It checks out the base sha, so the patch under test cannot change `.github/agent-policy/check.ts` (**known**, `agent-policy.yml:21-40`). **Inferred:** the label `agent-strict-review` is the only way a flagged diff proceeds, so the policy decides which PRs need the strict reviewer. The other research ticket covers how the pipeline uses the policy.

## 3. What "land" does

`.hivemind.toml` is 17 lines (**known**, read whole):

- `[land] branch = "dev"`, `batch_max = 4`. **Inferred:** the queue lands up to 4 pull requests per batch, in one run.
- `verify` has three commands: `cargo test --locked --workspace --lib`, a `bun install` guard for the SDK, and `HEX_CAPTURE=1 HEX_LOCAL_BUILD=1 scripts/build-app.sh`. **Known.** **Inferred:** `--lib` runs only the unit tests, so the land queue is a narrower check than `scripts/check` and narrower than CI.
- `[land.app]` names `target = "/Applications/Hex.app"`, `bundle = "target/app/Hex.app"`, `id = "anomaly.hex"`, `exe = "hex"`, `wait_idle = 30`, and `stamp = "~/Library/Caches/hex-install/installed-rev"`. **Known.**
- `install_cmd` waits until `~/Library/Caches/hex-install/install-gate.sh` prints `ready`, then runs `~/code/home/tools/hex-autobuild/install.py <head:0:8> <bundle>` when that file exists, and otherwise `rm -rf /Applications/Hex.app && ditto … && open -g`. **Known** (the text). **Inferred:** `install-gate.sh` is a private hold that keeps the install away from a busy app, and the stamp file names the commit the owner runs.

`scripts/install-app.sh` is the manual form of the same step (**known**, read whole): it builds, refuses to replace an app whose bundle ID is not `anomaly.hex` or `com.kitlangton.hex2`, sends the running app `TERM` and waits up to 30 s, then `ditto` and `open`. `build-app.sh:9-11` marks a `HEX_LOCAL_BUILD=1` build as "Local build · updates off", and `build-app.sh:57-59` stamps the short commit into the bundle. **Known.** **Inferred:** the owner's installed copy is therefore never on the dev channel; the two install paths do not overwrite each other's update behaviour.

## 4. How a bad change is caught after the merge

Three routes, all **known**:

1. **The `dev` push repeats the platform checks.** `check-windows.yml:6-7` runs on every push to `dev`. `check-linux.yml:26-46` and `check-macos.yml:21-37` run on a path match. Each run keeps its own concurrency group, so every dev commit gets a result (`check-linux.yml:51-55`).
2. **Test builds.** Every dev push makes unsigned downloads and a `Hex Test.app` DMG (`test-builds.yml:3-11`). That app has its own bundle ID `anomaly.hex.test`, its own URL scheme `hex-test` (`build-app.sh:88-91`), and its own data dirs (`docs/test-builds.md:22-30`). It never updates itself, so it cannot replace the release app.
3. **The dev channel.** A dev push publishes `0.0.0-dev-<run>`, and an install on the Dev channel follows it within minutes (`docs/releasing.md:130-157`). A rollback is a `promote` of an earlier sequence, not a revert.

**How often route 1 fires.** **Known** from the history: 11 commits have the subject prefix `fix(ci)`, and 10 of them are dated 2026-10-31. Nine are by Dax, one by Kit. The subjects name the platform: "unused `ModelChoice::model` helper that fails macOS clippy (#600)", "set the new srt and vtt fields in the macOS debug meeting show command (#649)", "detect held meeting locks on Windows and fix the OpenCode workspace header test (#656)". **Inferred:** the local loop on one Mac misses cross-platform breakage often enough that same-day follow-up PRs are normal. This is the strongest argument for practice 4.

## 5. How merges and reverts show in the git history

**Known** from `git log` on the clone:

- **Squash, not merge.** All 272 commits dated 2026-10-29 or later have one parent. The newest two-parent commit is `325a67b7`, a "Merge pull request #246" from 2026-10-28. The 38 merge commits in the repository are older, and most belong to the `rune-ui` integration (`b29c97ba`, 2026-10-22).
- **Shape of a squashed commit.** `7609574c` "Bump Rune to bb28aefe and prewarm on Metal (#673)" has the PR number in the subject and the five original commit subjects as `*` lines in the body. **Inferred:** the PR body, and its gate summary, do not reach the commit. No commit body in the last 300 contains "Gates".
- **PR coverage of the history.** 338 of the last 400 subjects end with `(#NNN)`. The commits without a number are the owner's direct pushes: the module-folding series "Fold dictation_* into dictation/" on 2026-10-31, docs updates, and two `fix(ci)` commits. **Inferred:** the owner can push to `dev` directly, and the agent path is the PR path.
- **Reverts.** Four subjects start with `Revert`: `ca7059d8` (#283) reverts #257, `22a7353f` and `c314ce5b` revert Roadmap commits, and `04ea3320` reverts a 2026-07-18 feature. `ca7059d8` landed as its own PR. A re-land follows the revert: `39f0e47c` restores the Windows caption, and `ab2e819f` re-lands #592 as #614 with a fix. **Inferred:** a revert is an ordinary change with its own PR, and a fix lands as a new change, so no force push is needed.
- **Pipeline commits.** 59 commits carry `Co-authored-by: Hex issue pipeline <noreply@anoma.ly>`. A body reads "Automated fix from the Hex issue pipeline (run 712-2026-10-31t23-48-23-441z). Fixes #712". **Known.**

## 6. The release boundary

**Known:**

- `.github/CODEOWNERS` names `@thdxr` for `/.github/`, with a comment that the release workflows hold the release secrets.
- `docs/releasing.md:11-28` splits build jobs (no environment, no secret, no id-token, unsigned files) from sign and publish jobs (the `release` environment, a sparse checkout of `.github/release/` only, artifacts treated as data).
- Each artifact carries a `SHA256SUMS`, and every job that downloads it checks the digest (`docs/releasing.md:25-28`).
- `.github/release/` holds 22 files, among them `verify-artifacts.sh`, `sign-macos.sh`, `publish-github-release.sh`, and `tag-release.sh`.
- `production` moves only by a fast-forward from `dev`, and the tag is pushed last, so a failed run leaves no tag (`docs/releasing.md:158-167,175-180`).

**Inferred:** this practice protects secrets, not code quality. It matters to Chronicler only if a release path appears; today the engine has no distributed artifact.

## 7. Where this differs from Chronicler

| Step | Hex | Chronicler |
|---|---|---|
| Where the gate runs | GitHub runners, 4 workflows | The agent's machine, `build.py`, about 145 s warm (`AGENTS.md:387`) |
| What a change must pass to land | Policy gate, Linux + Nix, macOS, Windows, and a green PR | One local gate run, then owner approval (`AGENTS.md:414`) |
| Who merges | The agent, when the checks pass (`AGENTS.md:7`) | Nobody merges: the owner approves the commit |
| History shape | One squashed commit per PR, number in the subject | Linear commits, conventional subject `type(scope): …` |
| Install | `.hivemind.toml` → `/Applications/Hex.app`, stamped | `target/<profile>/` with `data/` and `assets/`, started by `build.py run` |
| After landing | Dev channel and test builds rebuild the app | Nothing rebuilds the app; the next gate run or the owner's browser shows a break |
| Commit hooks | None in the repo | `pre-commit` regenerates four indexes and refuses hand-written prose |
| Release | Secrets split off, signed, published, rolled out by sequence | None |

**Known** for each row, from the files named above. **Inferred:** the Chronicler row "nothing rebuilds the app" is why `build.py`'s "Deployment: copy the target/ folder" line exists: the install is a manual instruction, not a step.

## What I could not find or did not read

- **Required status checks.** Not in the repository. Rulesets are GitHub settings, and this session used no API. **Unknown.**
- **The land queue program.** `hivemind` is not in either clone. `scripts/dictation-corpus.sh:33-35` calls `hivemind run --kind cargo`, and `scripts/build-app.sh:55` names "Kit's HiveMind-backed cargo". `.hivemind.toml` is the only description of the queue. **Inferred** for everything except the config text.
- **The private gate and install scripts.** `~/code/control/AGENTS.md`, `~/Library/Caches/hex-install/install-gate.sh` and `~/code/home/tools/hex-autobuild/install.py` are outside both clones. I did not read them.
- **Current CI durations.** The only durations in the repo are from September 2026 (`docs/features/release-verification.md:88-90,125-126`).
- **Job bodies of `release.yml`.** I read the 42-line header comment and the trigger block of this 712-line file, plus the job tree in `docs/releasing.md:30-48`. I did not read the 670 job lines.
- **Other workflow bodies.** I read every trigger and the full text of `agent-policy.yml`, `check-linux.yml`, `check-macos.yml`, `check-windows.yml`, `test-builds.yml` and `build-desktop.yml`. I read only the triggers of `aur-check.yml`, `installers.yml`, `legacy-appcast.yml`, `site.yml`, `site-deploy.yml`, `release-typescript.yml`, `windows-installer.yml` and `windows-installers.yml`.
- **PR bodies and review threads.** They are on GitHub, not in the clone. The findings asset quotes PR #714; I did not fetch any.
- **The issue pipeline itself.** It is the subject of another research ticket. I recorded only where its policy gate sits in this path.

## Sampling I used

- **Hex:** `git log` queries on the clone (`--merges`, `--since`, `--grep` for `Revert`, `fix(ci)`, the pipeline trailer, `(#NNN)`), `rev-list --parents`, and `wc -l` for the file sizes above. I read the six workflow files in full and every workflow's trigger block. I read `AGENTS.md:1-50` and the build and check sections around lines 1200-1300, not all 1486 lines.
- **Chronicler:** `build.py`'s module docstring and the gate plan (`_GATE_CHECKS`, `_plan_gate_steps`, `_copy_deployment_assets`), `scripts/git-hooks/pre-commit`, `scripts/install_git_hooks.py`, `scripts/build_history.py`'s docstring and header, `.pi/extensions/pi-permission-system/config.json`, `ENVIRONMENT.md` (the first 80 lines), `.config/nextest.toml`, and `git log` and `git branch -a` for the history shape.
- **Stop rule.** This ticket is one session. The Hex area is small enough to read whole; the two limits above are `release.yml`'s job bodies and the private tooling, both named rather than guessed.
