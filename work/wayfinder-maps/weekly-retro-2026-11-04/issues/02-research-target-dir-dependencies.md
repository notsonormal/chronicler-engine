# Find what depends on the target dirs inside the checkout

Type: research (AFK)
Status: resolved
Blocked by: —

## Question

Which tools, configs, docs and skills depend on the cargo target dirs being under `target/` in the checkout? What must change, and what breaks, if `build.py` `_target_paths` puts them outside the checkout?

## Context

- **Why (R6 of [the report](../assets/README.md)).** `target/` is 28 GB inside the checkout. Four `grep` rules and one `find` rule in `.pi/extensions/pi-permission-system/config.json` exist so that a command does not walk it. This week they denied 108 calls. A parse of the 42 denials of the rule `grep * . *` finds 0 that walk the repo root ([the tooling findings](../assets/findings/tooling-env.md), section 2; [the delegate findings](../assets/findings/sessions-delegates.md), section 4). If the target dirs are outside the checkout, the rules may not be necessary. Only the owner edits the permission config.
- **Known places to check:**
  - `build.py` `_target_paths`, and the `--target-dir` flag
  - `scripts/target_seed.py`, which seeds a cold target dir from a warm sibling checkout
  - `scripts/remove_worktrees.py`, which prunes stale target dirs
  - `.config/nextest.toml`, which writes the JUnit report under `target/nextest/`
  - `.cargo/config.toml` and `scripts/sccache-wrapper.sh`
  - `ENVIRONMENT.md`, `AGENTS.md`, and the skills that pass `--target-dir target/...`, for example `test-police`
  - the linked worktrees, and the probe server of `build.py run` and `scripts/trial.py`
  - the tests that find the engine binary through `CARGO_TARGET_DIR` (`start_server_with_env`, per `work/_closed/ui-verification-redesign/issues/13-branch-temporary-code-sweep.md`)
  - `.gitignore`
- **Earlier decisions.** No record of a decision to keep the target dirs inside the checkout was found in the plan review (2026-11-04). Search `work/_closed/`, `old-docs/` and the git log for one before you propose a move.
- **Read only.** Do not change any file. Do not run `build.py` or cargo.

## Steps

1. List every reference to `target/`, to a target dir or to `CARGO_TARGET_DIR` in `build.py`, `scripts/`, `tests/`, `.config/`, `.cargo/`, `.gitignore`, the root docs, `docs/` and `.agents/skills/`. Use `rg`.
2. For each reference, record what it assumes about the location, and the change that a location outside the checkout needs.
3. Check the file system: is a candidate location such as `~/.cache/` on the same file system as the checkout? Is there space for 28 GB? Does anything use hard links or relative paths into `target/`?
4. Propose a new location and the steps of the move. Name each permission rule that the move makes unnecessary, and each rule that stays, with the reason.

## Output

One asset: `../assets/target-dir-dependencies.md`, with the reference table, the file system facts, and the proposal.

## Done when

Each `rg` hit of step 1 has a row in the table, with its assumption and the change it needs.

## Answer

Resolved 2026-11-04. Put the target dirs at `~/.cache/chronicler-engine/target/<checkout key>/<name>`, where the key is the checkout path with `/` replaced by `-`. `build.py` already keeps a machine-wide cache dir there.

- **Places that change:** 12 existing files and one new module. The 196 hits of the `rg` pattern each have a row: 13 code sites, 12 test sites, 20 text sites, 5 permission rules. 114 hits need no change.
- **What breaks if only `_target_paths` changes:** `_cargo_env_for` sets `CARGO_TARGET_DIR` only for `--target-dir`, so cargo keeps building in `target/`. Seeding, pruning, `trial.py` and 3 tests then look in the wrong place.
- **File system:** `/workspace` and `/home/node` are two bind mounts of one device. A move between them is a copy. Nothing in the repo uses a cross-mount link.
- **Permission rules:** the move makes `grep * . *` and `grep * ./ *` unnecessary. `grep * /workspace/chronicler-engine *`, its `/` twin and `find / *` stay. Only the owner edits the config.
- **Earlier decisions:** none found that keeps the dirs inside the checkout.

Full table, facts and proposal: [target dir dependencies](../assets/target-dir-dependencies.md).
