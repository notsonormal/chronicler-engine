# Research 01: How Hex writes and evolves its agent instructions

Status: reference

Source: [anomalyco/hex](https://github.com/anomalyco/hex), commit `7609574ca92c` (2026-10-31), clone `tmp/hex-compare/anomalyco-hex/`. Archived Swift original: [kitlangton/Hex](https://github.com/kitlangton/Hex), commit `3e300721b878`, clone `tmp/hex-compare/kitlangton-hex/`. Chronicler Engine: this repo, 2026-11-01.

## Summary

Hex keeps one agent file, `AGENTS.md`, that loads into every session. It has 1486 lines and 100,349 bytes at the source commit. The file holds product facts: a purpose, a hand-written module map, and a list of invariants with exact values. Workflow rules are short and point to a private global file.

The file grew in four phases: a 96-line start (2026-07-14), product build-out to 760 lines (2026-10-19), the multi-platform rewrite (2026-10-22), and a 1486-line end state. It shrank only when a feature died or a platform truth changed.

Agents update the file inside the change that makes it true. Only 7 of the 225 commits that touch it touch nothing else. The median such commit changes 12 files.

Rules enter with the incident or the review that caused them, and they enter with numbers. A feature's removal takes its rules out in the same commit.

Chronicler spreads the same job over four files, 19 Diátaxis docs, 31 specs and 38 skills. It also writes two rules that oppose the Hex style: values stay in the code, and docs state what a thing is, not what it is not.

## Practices

| Practice | Hex evidence | Chronicler state | Candidate |
|---|---|---|---|
| 1. One always-loaded spec that holds purpose, module map and invariants | `AGENTS.md:1-1486` (100,349 B). Sections: Purpose `:23`, Feature Map `:49`, Architecture `:72` (97 bullets), Invariants `:527` (73 bullets). Commit `7609574c`. Shape set by `1ede22ec`. **Known.** | **Partly covered.** The facts live in `AGENTS.md:1-426`, `CODING_STANDARDS.md:1-35`, `CONTEXT.md:1-189`, `tests/AGENTS.md:1-198`, 19 Diátaxis docs, 31 specs and 38 skills. `docs/AGENTS.md:23-38` decides where each fact goes. **Known.** | 2 |
| 2. Invariants stated as behavior plus exact value | `AGENTS.md:527-1188`; 31 of 73 bullets carry a number. Example `:605-606` "Captures shorter than 300 ms discard". Caps at `:747` (64 MiB) and `:781` (10,000 entries, 16 MiB). **Known.** | **Partly covered, and one written rule opposes it.** `docs/AGENTS.md:149` "Values stay in the code" sends caps and constants to the source file. `docs/diataxis/reference/coding_standards/guardrails.md:1-137` numbers rules and pairs each with a test. **Known.** | 2 |
| 3. The spec rides the change that makes it true | 225 commits touch `AGENTS.md` of 1146. Only 7 touch it alone. Median files per such commit is 12, mean 18. Example `eab8ff35` (a lock-recovery fix adds the repair rule and its tests). **Known** (measured). | **Partly covered.** `docs/AGENTS.md:38` "A behaviour change edits its spec and its test". The Structure block regenerates (`AGENTS.md:7`, `:328`) and the pre-commit hook stages it. No rule covers hand-written `AGENTS.md` prose. **Known.** | new |
| 4. A feature's removal removes its instructions in the same commit | `a8a4645d` "Remove spoken commands and Moonshine" (36+/82−) drops the module bullet, the invariants and the diagnostic steps. Also `33430157` (Voice Action) and `5adf4086` (GPUI, 71+/93−). **Known.** | **Gap for prose.** The Structure block prunes itself. `scripts/validate_docs.py` fails on a dead path. No rule tells a writer to delete the paragraph or the bullet. **Known.** | new |
| 5. A `## Current Work` block of live constraints | `AGENTS.md:1471-1486`. Introduced at `c691f749` (2026-07-23) in place of `## Follow-up`. Content is replaced, not appended: `4fdfd2d1` adds the release freeze, `7609574c` keeps it. `:1473` names `ROADMAP.md` as the one work list. **Known.** | **Gap.** No `ROADMAP.md` at the root (root docs are `AGENTS.md`, `CODING_STANDARDS.md`, `CONTEXT.md`, `ENVIRONMENT.md`, `README.md`). No live-constraints block. Nearest analogue is `Notes` in each `.scratch/<effort>/map.md`. **Known.** | the `## Current Work` habit |
| 6. The seam rule sits in the always-loaded file | `AGENTS.md:1466-1469` "Do not add hypothetical seams for roadmap items. Introduce a seam once there are two real adapters or a current test requires substitution." Also `CONTRIBUTING.md:61-63` and `ROADMAP.md:282`. Present already at `1ede22ec`. **Known.** | **Covered in substance, in a skill.** `.agents/skills/codebase-design/SKILL.md:65` "One adapter means a hypothetical seam. Two adapters means a real one." `.agents/skills/tdd/SKILL.md:18-22` agrees the seams before a test. Neither is in `AGENTS.md`. **Known.** | 7 |
| 7. The instruction file is a protected diff path | `.github/agent-policy/check.ts:30` marks `(AGENTS\|CLAUDE)\.md` as `flagged`, so a change needs the strict reviewer. Neither a `README.md` change nor a `docs/` change gets that tier. **Known.** | **Gap.** No diff policy. `.pi/extensions/pi-permission-system/config.json` limits tools, not diffs. **Known.** | 1 |
| 8. Workflow rules defer to one private global file | `AGENTS.md:3-21` states the merge rule in five lines and points to `~/code/control/AGENTS.md` (`:9`) for the rest. The public file stays short on workflow. **Known** (the pointer). The private content is **unknown**. | **Gap.** Owner approval, git rules and permissions are inline in `AGENTS.md:410-414` and in the project instructions. **Known.** | new |
| 9. The app ships agent instructions to its users | `src/personal_commands/AGENTS.md:1-55` is written by the app (`src/personal_commands.rs:31` `include_str!`, rewritten at `:1462`). Its first line says Hex owns and replaces it. `skills/hex-personal-commands/SKILL.md:1-43` is a public skill for user agents and ends with a completion criterion (`:40`). **Known.** | **Gap.** The 38 skills in `.agents/skills/` serve repo work only. No product file tells a user's agent how to drive Chronicler. **Known.** | new |
| 10. A periodic audit sweep of every guide | `9e9da53a` "docs: audit guides against current behavior" aligns `AGENTS.md`, README, ROADMAP, 11 plans and a skill in one commit. 25 of the 225 `AGENTS.md` commits use a `docs:` subject. **Known.** | **Partly covered.** `build.py validate-docs` and `build.py vale` gate every run (`AGENTS.md:388-389`). `.agents/skills/chronicler-docs-hygiene/SKILL.md` audits on demand. No sweep is scheduled. **Known.** | new |

Ten practices. Five are marked new.

## Detail

### The shape of the file

The file keeps one order from first commit to now: private workflow, purpose, feature map, architecture, invariants, development, diagnostics, future direction, current work. **Known.**

Three of those sections carry the weight. Architecture has 97 bullets. Each bullet names a module or a folder in backticks, then a colon, then a comma list of what it owns. Example: `AGENTS.md:73-76` gives `audio` its device enumeration, its input resolution order and its recovery bound. Invariants has 73 bullets of prose. Most bullets run 5 to 15 lines.

The file uses negation often. 92 lines in the Invariants range contain `never`, `do not` or `must not`. **Known** (counted).

Nine GitHub issue numbers appear in the file, for example `#216` and `#442` in the Labs list (`AGENTS.md:42-44`). **Known.** The file cites a PR number only where the number identifies the feature. It does not cite the PR that added the rule.

### How the file changed

The sampled sizes rise except at the two rewrites. **Known** (sampled at every 20th commit):

| Date | Lines | Event |
|---|---|---|
| 2026-07-14 | 96 | `1ede22ec` first commit |
| 2026-07-24 | 367 | desktop build-out |
| 2026-08-31 | 486 | |
| 2026-10-19 | 760 | Windows beta, Release service |
| 2026-10-22 | 858 | Rune becomes the only UI |
| 2026-10-27 | 1164 | release workflow rules |
| 2026-10-31 | 1486 | `7609574c` |

Total: 3068 lines added and 1582 deleted over 225 commits.

Five kinds of change started a commit. **Inferred** from the sampled diffs and subjects:

- **A new feature.** `df2edf84` (Meetings beta, +95/−18), `8efd9909` (Windows beta).
- **An incident.** `eab8ff35` adds the lock-repair rule after a stuck double-tap lock. `629c9256` adds the lid-closed microphone rule.
- **A review finding.** `cd2edb9f` "Fix the Command-Tab press resync after review" adds two sentences of conditions to one invariant.
- **A change of platform truth.** `5adf4086` "Rune is the only UI; GPUI is gone" rewrites 71 lines and deletes 93. `21fb3477` aligns the module map with the single-process design.
- **A governance decision.** `2742dbb9` "private tool, push straight to main" adds the `## Private Kit Tool` section. `a16e96a2` "AGENTS: PRs only; goldens in the same PR" changes the merge rule and adds the golden rule.

A rule keeps the file current. The written rule covers the feature map only: `AGENTS.md:57` "Update the affected map in the same change". **Known.** No sentence tells a writer to update `AGENTS.md` itself. The same-commit habit is **inferred** from the commit sizes above. `.github/agent-policy/check.ts:30` enforces review of an `AGENTS.md` change. **Known.**

### What was removed

Removals follow feature death, in the same commit. **Known.**

- `a8a4645d` (Removed spoken commands and Moonshine) deletes 24 bullets and adds none: the `moonshine`, `command_grammar`, `keys` and `commands` module bullets, about ten invariants, and the command-mode lines in Diagnostics.
- `5adf4086` (GPUI removed) rewrites the design section and deletes 93 lines.
- `6eb94c28` "docs: remove captain's log guidance" edits one Diagnostics sentence and touches nothing else in the file.

The file also drops lines that no longer bear on the work. `9e9da53a` corrected stale module descriptions in one sweep.

### The conventions the file states for itself

- The feature map tells built behavior, available checks and observed proof apart. `AGENTS.md:67-70` contains the rule "A test's existence is not a passing run".
- The feature map forbids duplication: link a shared flow, do not describe it twice (`:60-62`).
- A map does not authorize a live device test (`:69-70`).
- Work that is not built stays in `ROADMAP.md`, not in the map (`:70`).

These are the same levers that `.agents/skills/writing-for-agents/SKILL.md` describes as a single source of truth, progressive disclosure and pointer wording. **Inferred**: the Hex file does not cite that skill.

### The Swift original, and what carried over

The archived app has `CLAUDE.md` (216 lines) with `AGENTS.md` as a symlink, and two `.cursor/rules/` files: a 12-line overview and a 484-line copy of the TCA library documentation. `CLAUDE.md` history has 11 commits, from 2025-05-30 to 2025-12-30. **Known.**

Carried into the Rust rewrite:

- The Purpose section, in one sentence of the same shape.
- The module map with "Keep modules deep" as its closing line. The Swift text names "Moonshine stream handles, CoreAudio formats, AppleScript details"; the Rust text drops the Moonshine item and adds event serialization.
- The Invariants list, now longer and numeric.
- The Development command block, the Diagnostics list and the `termctrl` line.
- The seam rule, word for word.
- `## Follow-up`, which became `## Current Work`.
- The changeset duty for agents, which became the SDK release section (`AGENTS.md:1191-1219`).

Dropped:

- The symlink trick. The Rust repo has one real `AGENTS.md` and no `CLAUDE.md`.
- `.cursor/rules/`. The Rust repo has no `.cursor/` directory.
- The numbered "Important Implementation Details" list. The Rust file replaced it with the module map and the invariants.
- The release procedure, which left the instruction file for `.github/` workflows and `docs/`.

### The Chronicler side

Chronicler splits the same material four ways:

- `AGENTS.md:330-426` holds agent process: responsibility, communication, decisions, test-first, the development loop, skills, permissions, delegation.
- `CODING_STANDARDS.md:1-35` holds code and comment rules. `:11-19` makes each source file carry a `//! [DOC: …]` anchor and a one-line summary.
- `CONTEXT.md:1-189` holds the product vocabulary, with an `_Avoid_` line per term.
- `docs/diataxis/` holds 19 reference and explanation docs, and `docs/specs/` holds 31 specs whose scenarios `scripts/validate_feature_spec.py` ties to tests.

The module map is generated, not written. `scripts/generate_structure_index.py` builds `AGENTS.md:7-328` from the `//!` line 2 of each file. A guardrail test keeps the anchor rule. **Known** from `AGENTS.md:312` and `CODING_STANDARDS.md:14-15`.

Two Chronicler rules pull against the Hex style. Both are **known** text:

- `docs/AGENTS.md:149` "Values stay in the code … constants and caps stay in the source file, one hop away." Hex states the cap in the invariant, for example `AGENTS.md:781` "10,000 entries and 16 MiB".
- `docs/AGENTS.md:155-158` "No negative explaining … state what a thing is. Do not describe it by what it is not." Hex writes many invariants as prohibitions.

The seam rule appears in Chronicler, at `.agents/skills/codebase-design/SKILL.md:65`. It loads only when a session reads that skill. Hex puts the same sentence in every session.

## What I could not find or did not read

- **`~/code/control/AGENTS.md`.** Not public. The size and the content of Hex's real workflow rules are unknown. `AGENTS.md:3-21` is the only visible part.
- **The pipeline prompts.** They belong to ticket 02.
- **The reason behind each rule.** I sampled 15 diffs. The other 210 commits that touch the file were measured by size only, not read.
- **The invariant bullets in full.** I read the first 40 lines of the Invariants range, the last 40, and about 15 bullets around the numeric examples. The other bullets went unread.
- **`sdk/commands/workspace-template/AGENTS.md`.** The ticket names this file. It does not exist at `7609574c`. The user-facing instruction file in the repo is `src/personal_commands/AGENTS.md`, and the public skill is `skills/hex-personal-commands/SKILL.md`. **Known** (searched).
- **A `CLAUDE.md` in the Rust repo.** It does not exist. **Known.**
- **Vale and Pi loading behaviour.** I did not measure which Chronicler files a session actually loads. The four-file count is the repo's own claim.

## Sampling

- `AGENTS.md`: read `:1-140` and `:1189-1486` in full, plus every heading and about 55 sampled lines of the 1345-line middle. Read the whole Swift `CLAUDE.md` and both `.cursor/rules/` files.
- History: measured all 225 `AGENTS.md` commits for added and deleted lines, file counts, subjects and dates. Read the diff of 15 of them: `1ede22ec`, `b1671672`, `2742dbb9`, `a16e96a2`, `9e9da53a`, `5adf4086`, `21fb3477`, `cd2edb9f`, `eab8ff35`, `a8a4645d`, `33430157`, `4fdfd2d1`, `c691f749`, `6eb94c28`, `cb547de4`.
- Chronicler: read `CODING_STANDARDS.md` in full, `docs/AGENTS.md:1-60` and `:138-160`, `AGENTS.md:310-426`, `.agents/skills/writing-for-agents/SKILL.md` in full, and the seam and heading lines of the skills named above.
