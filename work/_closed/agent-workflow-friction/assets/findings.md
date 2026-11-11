# Findings catalogue — 2026-10-04 reflect sweep

Every finding the sweep produced, with a stable ID and the ticket that owns it, grouped by **lever**: what actually changes when a decision is made.

- **Source.** A three-lens reflect sweep over chronicler-engine sessions from 2026-10-01T20:20Z to 2026-10-04T20:20Z: 162 session files, 112 MB. See [sweep-method.md](sweep-method.md).
- **Status of the "Sweep proposal" column.** These are the synthesizer's proposals. They are hypotheses, not decisions. The owning ticket's grilling decides each one: adopt, change, or drop with a reason.
- **"Was" column.** `A<n>` = row *n* of the Accepted list as presented on 2026-10-04. `B<nn>` = the misfiled ticket `NN`, now kept in [mechanism-notes.md](mechanism-notes.md). `S` = a synthesis row that the presented list dropped by mistake.
- **"Checked" column.** ✔ means the parent session confirmed the cited repo fact against the live tree on 2026-10-04. The proposal itself is still open.

## Ticket 01 — your own process

Lever: what you ask for, approve, or invoke. These change because you do them, not because a skill says so.

| ID | Problem | Sweep proposal | Was | Checked |
|---|---|---|---|---|
| F16 | Each finding becomes its own ticket. Per-ticket overhead (claim, build slot, full gate, two-axis review) dwarfs ten-minute work. The window saw two retroactive consolidations, 12→6 and 13→5. A full gate on an idle box took 662.96 s. | Price a ticket at authoring time. Batch same-theme findings into one ticket, with sub-findings as evidence lines. | A7 | |
| F17 | A claimed ticket is stranded when the delegating session ends. Tickets 44 and 46 needed transcript archaeology to close. | The merge step records the implementing session and resolves each claimed ticket with an `## Answer`. Document an `issues/_resolved/` move. | A9 | |
| F25 | The UI investigator ran once in three days, by hand. DOM stuck-state bugs, which stub fixtures don't model, went unverified while "full gate green" stood as proof. See also F05 for the test-side proof. | Make the description fire it as routine verification after UI-touching tickets. | A19 | ✔ |

## Ticket 02 — mechanisms to build

Lever: a script, hook, gate, fixture, or ignore rule. It runs whether or not a model reads anything.

| ID | Problem | Sweep proposal | Was | Checked |
|---|---|---|---|---|
| F07 | `chronicler-comment-fixer` judges every comment in a touched file. Each pass re-argues untouched comments. One pass isolated ~227 diff lines by hand. | Judge only the comment lines the current diff adds or changes. | A14 | |
| F08 | `chronicler-comment-fixer` ran 6+ times in three days against the same classes (AI_SLOP narration, paraphrase docs, stale cross-file refs). The rules live where slop is found, not where comments are written. | Encode the mechanical classes in a lint or pre-commit check. | B04 | |
| F09 | A sub-agent's effective config merges the bundled definition and the user override. Only `researcher` lacks `noExtensions`. No skill or tool shows the merged result. | A command that prints the effective definition per agent. | B03 | |
| F10 | Every delegated prompt retypes the worktree-per-ticket contract and the serialize-mutating-passes rule. Worktree homes vary: `/workspace/ce-wt/t34`, `/workspace/ce-t43`, `/workspace/wt65`. No tooling runs `git worktree add`. The build-slot lock guards compiles, not edit state. | A "parallel tickets and worktrees" section in `wayfinder`: one worktree per ticket, a list of forbidden git commands, and serialized passes that edit the same tree. | A8 | |
| F11 | `commit-and-push` stages with `git add -A`. On a hook collision it says "stage the file", which sweeps in a concurrent agent's edits. | Stage explicit paths and never use `-A`. Prescribe the shared-tree collision path. | A12 | ✔ lines 73, 161 |
| F12 | The pre-commit hook aborts when one of its 4 generated files has unstaged edits, which may be another agent's. Sessions answered with `--no-verify`. | Give the hook a non-destructive path for that case. | B07 | |
| F13 | Nothing safely prunes worktrees. A careless prune can delete a live agent's tree. | A wrapper that refuses dirty trees, trees with live processes, or unpushed work. | B02 | |
| F15 | Chromium leaves `core.<pid>` dumps (~27 MB each) at the repo root. An untracked one confused two commit sessions. | Add `core.*` to `.gitignore`. | B05 | ✔ no rule present |
| F18 | Nothing lists tickets that are `Status: claimed` with no `## Answer`. | A sweep script, or a wayfinder pre-frontier check. | B01 | |
| F21 | `reflect` assumes one transcript, and its output has no tracker shape. A multi-day sweep had to invent digests on the spot, then filed 8 `needs-triage` AFK tickets with no map. That filing is how this effort started. | Step 1 builds per-session digests for a date-range sweep. *Open:* what shape reflect output takes on the tracker. | A27 + this run | |
| F24 | Commits land on a chain of self-reported greens. Nothing records where the last green came from. | The commit report names which gate ran, where, and when. | A13 | |
| F26 | The UI investigator drives the user's live game. One probe ran a real LLM turn that had to be removed by hand. | A no-mutation contract: use a throwaway game/world, or record and restore every change. | A17 | |
| F29 | `scripts/diagnostic_benchmark.py` reports omit the contention condition and the effect size. | Record both in the report. | B08 | |

## Ticket 03 — facts placed where they are read

Lever: one specific fact put in a brief template, a skill description, or a doc line that is loaded when the action happens. Moderately reliable, so a yes/no per item is enough.

| ID | Problem | Sweep proposal | Was | Checked |
|---|---|---|---|---|
| F01 | `CODING_STANDARDS.md` holds the review rules and the test-doc pointers. Nothing in always-loaded `AGENTS.md` points to it. Commit `62072d5a` removed the pointer during a deliberate trim, and a retro fix approved on 2026-10-04 never landed. | Restore a one-line pointer. | A1 | ✔ |
| F02 | `code-review` forbids building (line 15), but neither §4 sub-agent brief carries the rule. The parent retypes it, and reviewers build anyway. | Paste the rule into both briefs, with `python build.py test-pattern <name>` as the only exception. | A2 | ✔ |
| F03 | `retro` step 2 says only "searching through session logs on this machine". The user had to name `session_search` and paste JSONL paths. | Name `session_search` and the session-JSONL directory. | A4 | ✔ |
| F04 | `tests/STRATEGY.md § SCENARIO tags` omits that coverage keys on `(spec_path, scenario_id)` (`scripts/validate_feature_spec.py:246`). A non-duplicate was "fixed" by renumbering. | State per-spec scoping, and "do not renumber across specs". | S | ✔ |
| F05 | `tdd` fires only on "TDD"/"red-green-refactor". Sessions that add, strengthen or de-tautologize tests never open it, so its break-it-and-watch-it-fail proof (line 51) is lost. | Add trigger phrases to the description. | A25, A26 | ✔ |
| F06 | `chronicler-docs-hygiene` stayed silent on "is this doc used at all?" because its description doesn't cover usage evidence. | Add usage-evidence pruning to the description, plus a phase that mines sessions and git history first. | A24 | ✔ |
| F14 | The `build.py --cleanup` scope is not written down. Agents feared it would touch other worktrees. | A scope note in the `--cleanup` docstring and in `ENVIRONMENT.md`. | B06 | |
| F27 | The operator can't reach the dashboard from the host, but the skill never says the screenshot is the evidence channel. | Return shots inline. Never hand back a URL or a path. | A18 | |
| F35 | `skills-lock.json` is output of an upstream installer that is absent here, which tempts a fabricated `computedHash`. | Repo-authored skills get no lock entry. Never invent a hash. | A16 | |

## Default drop — judgement advice

Lever: advice in a skill body. This is the kind of text the sweep shows failing to bind (F02, F05). Each one is **dropped** unless ticket 01 adopts it as your process, or ticket 02 adopts it as a mechanism. The ticket named in brackets checks it. The drop reason is the same for all: *skill-body advice does not reliably change model behaviour, and no mechanism or habit carries it.*

| ID | Problem | Sweep proposal | Was | Checked |
|---|---|---|---|---|
| F19 [01] | The user had to bring up SillyTavern/Marinara twice. The repo already documents them (`docs/external_applications/marinara_engine.md`, the closed steering map's Notes). | Each option in a UX-mechanism decision shows what peer products do. Map Notes name the prior-art sources. | A10 | |
| F20 [02] | Audit and verification reports stay as prose. Their open items get lost, and the user has to ask for a plan. | Turn actionable findings into a `docs/plans/` plan or tracked tickets with stable IDs, at report time. | A22 | |
| F22 [01] | A handoff in the OS temp dir can come back with another session's review appended. Acting on the stale copy wastes work. | Re-read a handoff before acting, and look for an appended review section. | A23 | |
| F23 [02] | Review findings carry no evidence class, so "apply all" is the operator's only safe move. | Label each finding quoted-rule, quoted-spec-line or inference. Put the ticket's accepted deviations in the review bundle. | A3 | |
| F28 [02] | `retro` files measurement findings from single runs on a contended host. One framing was retracted late. | State the contention condition and an effect-size threshold. Skip measuring when the fix costs less than the measurement. | A5 | |
| F30 [02] | A hazard deferred into a plan protects nobody. An agent sat 16 min on a phantom build-slot holder whose fix was planned but not built. | Put a one-line warning at the hazard site in the same session. | A6 | |
| F31 [01] | Grilling records letter answers without the runner-up's cost, and doesn't restate the frontier after an interruption. | Give each option a one-line cost. Record why the rejected option lost. Restate the frontier after an interruption. | A11 | |
| F32 [01] | Research dispatched mid-grilling answers "how does the reference product do it". It closes a complexity objection by authority. | Research must also answer the simpler option and the do-nothing option. | A21 | |
| F33 [01] | `correct`'s "fix at the highest level that works" ratchets. A trial produced more `#[allow]` exemptions than findings. | Compare exemption count to finding count before adding a lint. Make "no change" a valid outcome. | A20 | |
| F34 [01] | A vendored skill import grew into a repo refactor until the user pulled it back. | Keep an import minimal and verbatim, record provenance, and put repo fixes in their own plan. | A15 | |

## Rejected by the synthesizer

These are out of scope unless a ticket brings new evidence. Reason codes come from the synthesizer.

| Finding | Reason |
|---|---|
| Verify each review finding before applying it | `duplicate` of `chronicler-after-plan-workflow-plus-review` step 13. Its usable part is F23. |
| Check git history and session search before declaring a capability missing | `duplicate`, absorbed by F03 and F06 |
| The effective agent definition is a merge product | `structural`, now F09 |
| Keep cleanup to `target/<name>` | `structural`, now F14 |
| Emit comment rules back into the authoring path | `structural`, now F08 |
| New `agent-config-audit` skill | `existing-skill-first`, now F09 |
| New `worktree-lifecycle` skill | `existing-skill-first`. `implement-spec`/`wayfinder` own worktree use. Now F10, F13 |
| `core.<pid>` is a Chromium dump | `structural`, now F15 |
| Pre-commit `--no-verify` policy | `structural`, split into F11 and F12 |
