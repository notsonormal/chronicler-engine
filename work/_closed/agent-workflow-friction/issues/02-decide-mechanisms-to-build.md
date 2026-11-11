# Decide which mechanisms to build

Type: grilling (HITL)
Status: resolved

## Question

These frictions can be fixed by something that runs whether or not a model reads it: a script, a hook, a gate step, a fixture, or an ignore rule. For each, is it worth building, and what shape should it take?

1. **Shared checkout.**
   - A worktree-create script with one standard home (F10).
   - A safe prune that refuses dirty trees, trees with live processes, or unpushed work (F13).
   - Commit staging that cannot sweep in another agent's files (F11).
   - A pre-commit hook path for when its generated files carry foreign edits (F12).
   - A `core.*` ignore rule (F15).
2. **Tracker.**
   - A reflect digest builder: `tmp/reflect/extract.py`, promoted out of `tmp/` (F21).
3. **Proof.**
   - A last-green stamp written by `build.py` and read by the commit report (F24).
   - A throwaway-game fixture so the UI investigator never touches your live game (F26).
   - Contention and effect-size fields in `scripts/diagnostic_benchmark.py` reports (F29).
4. **Comments and config.**
   - A diff-only mode for the comment finder (F07).
   - A lint for the comment-slop classes that are mechanically detectable (F08).
   - A command that prints each agent's effective definition (F09).

Then the default-drop findings marked `[02]` in [findings.md](../assets/findings.md#default-drop--judgement-advice): F20, F23, F28 and F30. Does any of them have a mechanism worth building? One example is a required evidence-class field in the review output template (F23). If none does, it is dropped.

## Context

- Findings are in [findings.md](../assets/findings.md). Old tickets 01–08 in [mechanism-notes.md](../assets/mechanism-notes.md) hold the gathered evidence and first-draft designs. Treat those designs as inputs, not decisions.
- **Facts to dispatch, not to ask you.** Send a `scout` for each before its question comes up:
  - Can `scripts/git-hooks/pre-commit` tell its own regeneration diff from a foreign edit? (F12)
  - Which comment-slop classes in past `chronicler-comment-fixer` reports are mechanically detectable without false positives? (F08)
  - What does `build.py` already record about a finished gate run? (F24)
- Facts already known:
  - No repo tooling runs `git worktree add`.
  - `commit-and-push` uses `git add -A` at lines 73 and 161 (✔).
  - `.gitignore` has no `core.*` rule (✔).
  - The `build_slot.py` `.holder` file is only a note.
- Other efforts edit this checkout at the same time. `dashboard-ui-review` has uncommitted edits. Treat F10–F12 as a real case, not a hypothetical.
- Graduated work is cut case by case. Ticket 01 dropped the sizing rule (F16) and dropped F18.
- Ticket 01 decided that UI correctness comes from automated tests, not the UI investigator (F25). F26 only protects your live game during ad-hoc investigator runs, so weigh it on that basis.

## Done when

- Each of the 12 mechanisms has a decision: build (with its shape), or drop with a reason.
- Each `[02]` default-drop finding is marked absorbed-as-mechanism or dropped.
- Adopted mechanisms are graduated into implementation tickets. A one-line change, such as F15, can be applied by this session instead.

## Answer

Resolved 2026-10-04 by grilling. Four mechanisms graduate into implementation tickets and one is a one-line change applied in this session. Everything else is dropped.

### Mechanisms

| # | Finding | Decision | Reason |
|---|---|---|---|
| 1 | F10 worktree-create script | **Dropped** | `git worktree list` already finds every tree regardless of home, so a fixed home buys nothing. The varied homes (`/workspace/wt63`, `/workspace/ce-wt`) cause no measured harm. |
| 2 | F13 safe worktree prune | **Build** → [Prune worktrees safely](05-safe-worktree-prune.md) | The only item that can destroy a live agent's tree. Enumerate via `git worktree list`; refuse a tree that is dirty, has unpushed commits, or is held by a live process. No home convention, no prototype. |
| 3 | F11 explicit staging in `commit-and-push` | **Dropped** | The fix is skill-body text, the lever Q7 rules out. |
| 4 | F12 pre-commit hook non-destructive path | **Build** → [Give the pre-commit hook a non-destructive path](06-pre-commit-hook-non-destructive-path.md) | The hook cannot tell its own leftover output from a foreign edit, so it aborts and blames the operator, and sessions escape with `--no-verify`. Snapshot, regenerate, compare: stage its own output, restore and abort on a foreign edit. |
| 5 | F15 `core.*` ignore | **Build** — applied in this session as `/core.[0-9]*` | The finding's literal `core.*` matches at any depth, so it would ignore all three `core.rs` files. Root-anchored and digits-only is the fact it meant. |
| 6 | F21 reflect digest builder | **Dropped** | The script is disposable; the next sweep can rebuild it. Reflect's tracker output shape is a reflect-skill decision, not this map. |
| 7 | F24 last-green stamp | **Dropped** | `commit-and-push` not building is deliberate: building is an implementation gate, not a commit gate. |
| 8 | F26 throwaway-game fixture | **Build** → [Isolate the UI investigator's probe server](07-isolated-probe-server.md) | The server DB is `<exe dir>/chronicler_<port>.db`, so a separate `--target-dir` gives the probe its own DB and the live game is untouched. |
| 9 | F29 benchmark contention + effect size | **Build** → [Record contention and effect size in benchmark reports](08-benchmark-contention-effect-size.md) | A single run on a contended host reads as signal; a finding was retracted late for exactly this. |
| 10 | F07 diff-only comment finder | **Dropped** | Deliberate: full-file review is wanted, because the agent does not remove comments aggressively enough. |
| 11 | F08 comment-slop lint | **Dropped** | The mechanically-clean classes have zero current violations; the classes that recur are the judgement ones a lint cannot reach. |
| 12 | F09 effective-definition command | **Dropped** | The session-start snapshot already prints model, thinking, `noExtensions` and both source paths for every agent. |

### Default drops (`[02]`)

| Finding | Disposition | Reason |
|---|---|---|
| F20 report → plan/tickets | **Dropped** | Extraction from prose is judgement; "at report time" is a process habit, not a mechanism. |
| F23 evidence-class label | **Dropped** | Both review briefs already distinguish hard violations from judgement and require the spec quote; a label is text, which Q7 says does not bind. |
| F28 retro measurement | **Absorbed** | Same theme as F29; the contention and effect-size fields answer it. The minimum-effect-size threshold is judgement and is not built. |
| F30 hazard-site warning | **Dropped** | Already built: `scripts/build_slot.py:112-115` and `:122-125` report a stale `.holder` in both branches. |

F18 was already removed from this ticket by [Decide what you will do differently](01-decide-your-process-changes.md).

### Dropped findings and the skills that carry their like

No dropped finding is reworded into a skill body. That is Q7's rule, and ticket 01 already applied it to the `[01]` rows.
