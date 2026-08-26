# Assess SwarmForge as multi-agent development methodology for Chronicler Engine

Type: research
Status: resolved
Assignee: pi
Blocked by: 02

## Question

Could SwarmForge's role-based, worktree-isolated, handoff-driven workflow be used to coordinate multiple AI agents developing Chronicler Engine's Rust codebase? What would need to change in the repository or workflow to support it, and which roles or pack configurations (two-pack, four-pack, six-pack) would fit the project's size and quality gates?

## Answer

### Summary of SwarmForge's dev methodology

SwarmForge coordinates multiple AI agents through a small set of fixed roles.

- **Role topology in config.** `swarmforge.conf` declares windows as `window <role> <backend> <worktree>`. The canonical roles are `specifier`, `coder`, `refactorer`, `architect`, `hardender`, and `QA`.
- **Worktree isolation.** Each role works in its own git worktree so agents do not overwrite each other's files.
- **Layered constitution prompts.** A shared `constitution.prompt` combines with branch-local articles and per-role `.prompt` files to shape behavior.
- **File-based handoff protocol.** Agents drop handoffs in `.swarmforge/handoffs/outbox/`. A `handoffd` daemon copies them to the recipient `inbox/new/`, sends a generic tmux wake-up, and moves the original to `sent/`.
- **Two message types.** `git_handoff` carries a commit SHA and a stable task name; `note` carries a one-line message. `git_handoff` forwards down the chain until the terminal broadcast.
- **Queue helpers.** `ready_for_next.sh` and `done_with_current.sh` move files through `new/`, `in_process/`, and `completed/`.
- **Audit trail.** Handoff headers (`id`, `from`, `to`, `recipient`, `priority`, `type`, timestamps) and filesystem location replace a logbook.

Evidence: ticket `02-assess-swarm-forge.md` documents these mechanics from the `swarm-forge` repository and the `experiment-htw-go-swarm` / `experiment-htw-clj-swarm` examples.

### Comparison with Chronicler Engine's current workflow

Chronicler Engine already enforces a multi-stage quality pipeline before any code is considered good.

`build.py` (repo root) runs the following gates in order:

1. `cargo fmt`
2. `scripts/validate_data.py`
3. `cargo clippy --all-targets --all-features -- -D warnings`
4. `scripts/check_test_structure.py`
5. `scripts/check_python_docstrings.py`
6. `python -m unittest discover scripts/tests`
7. `scripts/extract_http_routes.py --check`
8. `scripts/generate_guardrails_doc.py --check`
9. `scripts/validate_docs.py`
10. `cargo nextest run --no-fail-fast` (or `cargo llvm-cov nextest` with `--coverage`)

Evidence: `build.py:552-649`.

Additional guardrails live outside `build.py`:

- `arch-lint.toml` declares layer rules (model, application, server, storage, ports, adapters) and runs through `cargo nextest run --test architecture`.
- `tests/infrastructure/guardrails/` contains custom `syn`-based convention walkers for enum docs, free-function location, inherent-impl locality, layer boundaries, import ordering, file length, and doc-anchor standards.
- `scripts/check_test_structure.py` forbids inline `#[cfg(test)]` blocks and requires every `*_tests.rs` file to be declared in its module root.
- `tests/AGENTS.md` describes the integration-test mirror convention and the spec-to-test traceability rules.
- `AGENTS.md` and `docs/AGENTS.md` define the documentation conventions: Diátaxis modes, doc anchors, and the prohibition on code-indexer docs.

Evidence: `arch-lint.toml:1-142`, `docs/diataxis/reference/coding_standards/guardrails.md`, `tests/AGENTS.md`.

### Mapping SwarmForge roles to Chronicler development

| SwarmForge role | Chronicler equivalent | How well it maps |
|---|---|---|
| `specifier` | Specs in `docs/specs/` plus integration-test skeleton | Strong. Specs already define observable behavior, and tests mirror specs. |
| `coder` | Any agent implementing a feature against the spec | Strong. This is the normal mode of work. |
| `refactorer` | `cargo fmt`, `cargo clippy`, custom style guardrails | Weak. The automated checks already enforce the refactorer's concerns; a separate role would mostly re-run `build.py`. |
| `architect` | `arch-lint.toml` + `tests/infrastructure/guardrails/layers.rs` | Weak. Layer rules are declarative and tested automatically. A human or agent architect is useful for *changing* the rules, not for gatekeeping every commit. |
| `hardender` | Error handling, `clippy::unwrap_used`, `clippy::expect_used`, panic lints | Weak. These concerns are already denied at the crate root in `src/lib.rs` and verified by clippy. |
| `QA` | Integration tests, browser tests, LLM tests, coverage report | Medium. A QA step is real, but it is already expressed as `cargo nextest run` and `scripts/parse_coverage.py`. |

The mapping shows that most SwarmForge roles overlap with existing automated checks. The only genuinely additive role is `specifier`, and that function is already covered by the project's spec-first testing convention.

### Assessment of pack configurations

| Pack | Roles | Fit for Chronicler Engine |
|---|---|---|
| **Two-pack** (`specifier` + `coder`) | Small task splitting: one agent writes the spec/tests, another implements. | Partial fit. The split is reasonable, but worktree isolation is unnecessary. A single branch with a spec commit followed by an implementation commit achieves the same thing. |
| **Four-pack** (+ `refactorer` + `QA`) | Adds style/refactoring review and explicit QA. | Low fit. `refactorer` duplicates clippy and guardrails. `QA` duplicates the integration-test suite. The overhead of four worktrees and handoffs is not justified by the value added. |
| **Six-pack** (+ `architect` + `hardender`) | Full role specialization. | Very low fit. `architect` and `hardender` concerns are already machine-enforced. Six agents in six worktrees would multiply build time and merge friction for no quality gain. |

### Worktree isolation: help or hindrance?

Worktree isolation would hinder this Rust codebase more than it would help.

- **Cargo target contention.** Each worktree would need its own `target/` directory or agents would fight over Cargo locks. `build.py` already supports `--target-dir` for concurrent agents, but that is a practical workaround for occasional parallel builds, not a daily workflow.
- **Longer build times.** A full `cargo nextest run` takes 1-2 minutes on the existing target. Running it six times across six worktrees, even with shared incremental artifacts, is wasteful.
- **Merge conflicts between worktrees.** Handing off via git commits does not remove the need to merge. If the `coder` and `refactorer` both touch the same file in separate worktrees, the `hardender` still has to reconcile them.
- **State duplication.** SwarmForge's `.swarmforge/handoffs/` directory, `constitution.prompt`, per-role prompts, and `handoffd` daemon are additional state that must be maintained. Chronicler already has enough moving parts: specs, tests, Diátaxis docs, guardrails, and build scripts.

Worktree isolation makes sense when agents are unreliable and would overwrite each other's files. Chronicler's guardrails and review model make that risk low.

### Handoff artifacts for Rust

SwarmForge's `git_handoff` could carry artifacts that make sense for Rust development:

- Commit SHA and task name (already in git).
- Test report (`cargo nextest` output).
- Clippy report (`cargo clippy --all-targets`).
- Coverage summary (`cargo llvm-cov report --json` + `scripts/parse_coverage.py`).
- Architecture review notes (violations from `arch-lint` or `tests/infrastructure/guardrails/`).

However, all of these can be produced by a single `python build.py` run and attached to a pull request or commit message. A file-based handoff daemon adds indirection without adding information.

### Does `build.py` already cover the six-pack?

Yes, for the parts that matter.

| Six-pack concern | Chronicler coverage |
|---|---|
| Spec | `docs/specs/` + integration test mirror convention |
| Code | Implementation by agent |
| Style/refactor | `cargo fmt`, `cargo clippy`, style guardrails |
| Architecture | `arch-lint.toml`, layer guardrails |
| Hardening | Panic/unwrap/expect lints, error propagation patterns |
| QA | Integration tests, browser tests, LLM tests, coverage |

The missing piece is not a role; it is a coordination protocol. `build.py` verifies quality but does not assign tasks, track dependencies between agents, or hand off context. That coordination currently happens in plans (`docs/plans/` or `.scratch/`) and through explicit agent instructions.

### What would need to be added to run SwarmForge here?

Adopting SwarmForge literally would require:

1. A `.swarmforge/` directory with `swarmforge.conf`, `constitution.prompt`, and per-role prompts.
2. Git worktrees for each role.
3. A `handoffd` daemon or a port of its queue helpers to Linux/WSL.
4. `ready_for_next.sh` / `done_with_current.sh` adapted for the project.
5. Documentation telling agents which worktree to use and how to hand off.
6. A decision about how to merge worktrees back into the main branch.

That is a non-trivial tooling investment. It would also create a second workflow alongside `build.py` rather than replacing it, because the quality gates would still need to run.

### Lighter-weight alternatives

The project already has lighter ways to capture the value of role specialization:

- **Skills.** Existing skills such as `code-review`, `code-consistency-check`, `chronicler-comment-fixer`, and `chronicler-docs-hygiene` act as focused reviewers without requiring worktrees or daemons.
- **Pre-commit hooks.** `scripts/install_git_hooks.py` already installs hooks. These could be extended to run `validate_docs.py` or `check_test_structure.py` before commit, though `build.py` already covers the full gate.
- **CI jobs.** A CI job that runs `python build.py --coverage` on every pull request would enforce the same quality bar across multiple agents without any local workflow change.
- **Target-dir isolation.** For the rare case where two agents build concurrently, `python build.py --target-dir target/<agent-name> --no-fmt` already avoids Cargo lock contention.
- **Plan-based role tags.** A plan can label tasks as `specifier`, `coder`, `reviewer`, etc., and assign them to the same agent sequentially or to different agents on the same branch.

These alternatives give most of the coordination benefit without the worktree overhead.

### Practical adoption path or recommendation

**Do not adopt SwarmForge as a development methodology or tooling layer for Chronicler Engine.**

The repository's existing quality infrastructure already enforces what SwarmForge's six-pack tries to guarantee through role specialization. Adding worktree isolation, a handoff daemon, and per-role constitution prompts would introduce complexity and build-time cost without improving the quality signal.

The better path is to keep using the current workflow and make coordination explicit through plans and skills:

- Use specs and integration tests as the `specifier`-to-`coder` contract.
- Rely on `build.py`, `arch-lint.toml`, and the custom guardrails for the `refactorer`, `architect`, and `hardender` roles.
- Use the `code-review` skill or a CI job for the `QA` role.
- If multiple agents ever need to build concurrently, use `build.py --target-dir` with `--no-fmt`.

No new ticket is needed. The conclusion from ticket `02` (SwarmForge is not applicable as an application-LLM pattern) extends to the code-development angle as well.
