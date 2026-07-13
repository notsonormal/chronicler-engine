# Remove Dev Container Infrastructure

## Summary

Strip all devcontainer artifacts from the repository. Devcontainer tooling is incompatible with the user's WSL-inside-Windows environment, so the layer is pure overhead. The `scheduler` service currently builds from `.devcontainer/Dockerfile.scheduler`; those files move into `docker/` so the AI stack keeps working. Documentation gets a full scrub — no references to devcontainers, sandbox user, or docker-proxy-mediated workflows remain.

## Key Changes

- Delete `.devcontainer/` directory (8 files).
- Delete `devcontainer-cli.bat` (Windows-only launcher, useless on WSL host).
- Move scheduler build assets into `docker/`:
  - `.devcontainer/Dockerfile.scheduler` → `docker/Dockerfile.scheduler`
  - `.devcontainer/crontab` → `docker/crontab`
  - `.devcontainer/logrotate.conf` → `docker/logrotate.conf`
- Update `docker/docker-compose.yml` `scheduler` service build context from `../.devcontainer` → `.` (resolves to `docker/`).
- Clean ancillary files: `.dockerignore` (drop `.devcontainer` ignore), `.gitattributes` (drop `.devcontainer/*` rule).
- Documentation scrub: delete `docs/devcontainers.md`; update `docs/docker.md`, `docs/env.md`, `README.md`, `AGENTS.md`, `scripts/issue_tracker/cli.py` example text.

## Implementation

### Phase 1: Remove Devcontainer Layer and Relocate Scheduler

- [x] #### Task 1.1: Delete devcontainer files and move scheduler assets (3 SP)
  - Delete `.devcontainer/` directory entirely (8 files: `Dockerfile`, `Dockerfile.scheduler`, `crontab`, `logrotate.conf`, `devcontainer.json`, `opencode-wrapper.sh`, `wrapper-start.sh`, `test-write.txt`).
  - Delete `devcontainer-cli.bat`.
  - Move scheduler build files to `docker/`:
    - `.devcontainer/Dockerfile.scheduler` → `docker/Dockerfile.scheduler`
    - `.devcontainer/crontab` → `docker/crontab`
    - `.devcontainer/logrotate.conf` → `docker/logrotate.conf`
  - Edit `docker/docker-compose.yml` `scheduler` service: change `build.context` from `../.devcontainer` to `.` and keep `dockerfile: Dockerfile.scheduler`. The `COPY crontab` / `COPY logrotate.conf` lines in `Dockerfile.scheduler` resolve relative to the new context (`docker/`).
  - Edit `.dockerignore`: remove the `.devcontainer` line and its preceding `# Ignore the devcontainer config itself from the context` comment.
  - Edit `.gitattributes`: remove the `.devcontainer/* text eol=lf` line.
  - Verify: `git status` shows expected deletions, moves, edits. `ls .devcontainer devcontainer-cli.bat` returns "No such file or directory". `ls docker/Dockerfile.scheduler docker/crontab docker/logrotate.conf` succeeds.

- [x] #### Task 1.2: Documentation scrub (3 SP)
  - Delete `docs/devcontainers.md`.
  - Edit `docs/docker.md`:
    - Drop the `Dev Container (Security Sandbox)` subgraph from the architecture diagram.
    - Replace "3. Scheduler Sidecar" subsection so it reads:
      - **Config**: `docker/crontab` defines the schedule.
      - **Image**: Built from `docker/Dockerfile.scheduler`.
    - Replace "Relationship to Dev Container" section with one sentence noting the workspace no longer uses a dev container; the host runs VS Code directly on Windows/WSL and uses `docker compose` from the host shell.
  - Edit `docs/env.md`:
    - Drop the "Dev Container Integration" subsection.
    - Update the mermaid diagram to show `Host[Host Machine]` directly loading `.env` for both shell sessions and docker services (no Dev Container node).
    - Rewrite "Security Implementation" to drop the sandbox user + read-only mount bullets; keep Git exclusion.
  - Edit `README.md`: remove the `- **[Dev Container Manual](docs/devcontainers.md)**: ...` bullet from the Technical Documentation list.
  - Edit `AGENTS.md`: in the workspace tree comment, change `docs/ # Workspace documentation (docker, models, devcontainer)` to drop `devcontainer`.
  - Edit `scripts/issue_tracker/cli.py` epilog example: change `"Fix devcontainer network"` to a neutral example like `"Fix scheduler log rotation"`.
  - Verify: `grep -rni devcontainer --exclude-dir=.git --exclude-dir=node_modules .` returns zero matches. `python -c "import scripts.issue_tracker.cli"` confirms argparse still parses.

## Test Plan

- `docker compose -f docker/docker-compose.yml config` — compose parses; `scheduler` build context points at `docker/` and references `Dockerfile.scheduler`.
- `docker compose -f docker/docker-compose.yml build scheduler --no-cache` — image builds from relocated `docker/Dockerfile.scheduler` with `docker/crontab` and `docker/logrotate.conf` copied in.
- `grep -rni devcontainer --exclude-dir=.git --exclude-dir=node_modules .` — returns no matches.
- Visual review of `docs/docker.md` and `docs/env.md` — no remaining references to "Dev Container", "sandbox user", "docker-proxy from dev container", or `.devcontainer/` paths.

## Per Task/Sub Task Validation Steps

- **Task 1.1**: After edits, run `docker compose -f docker/docker-compose.yml config | grep -A2 scheduler` to confirm build context and dockerfile path. Run `docker compose -f docker/docker-compose.yml build scheduler --no-cache` to catch context-resolution errors.
- **Task 1.2**: After edits, run `grep -rni devcontainer --exclude-dir=.git .` from workspace root. Any match must be investigated and removed. Run `python -c "import scripts.issue_tracker.cli"` to confirm argparse still parses.

## Assumptions

- User has Docker Desktop on Windows with WSL integration enabled and runs `docker compose` from either Windows PowerShell or WSL shell.
- WSL environment already has Python 3, Rust toolchain, OpenCode installed (or will be installed manually); repo does not need to document or provision those tools.
- `scripts/ObsidianVaultExport/vault_to_single_file.py` remains the cron job source of truth; path inside container (bind-mounted from `docker-compose.yml`) does not change.
- `docker/` `.gitignore` rules already cover `*_data` volumes; relocating scheduler files into `docker/` does not conflict.
- Scheduler service relative paths (`../logs`, parent script path) remain valid since `docker/docker-compose.yml` location does not move.
- No CI workflows reference devcontainer assets (confirmed: no `.github/` directory or other CI configs in repo).
