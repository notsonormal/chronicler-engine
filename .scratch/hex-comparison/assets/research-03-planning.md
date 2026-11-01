# Research: How Hex plans work

Status: reference. Asset of ticket [Research how Hex plans work](../issues/03-research-planning.md), map [What Chronicler can learn from Hex](../map.md).

Sources: `anomalyco/hex` at `7609574ca92c` (2026-10-31), local clone `tmp/hex-compare/anomalyco-hex/`; Chronicler Engine at this working tree.

## Summary

- Hex keeps 18 plan documents in `docs/plans/` (30951 words) and one prioritized work list, `ROADMAP.md` (known: file counts).
- Five Hex plan parts have uneven cover in Chronicler: a Status line that names the built parts, a limits section, a manual-verification checklist, a numbered landing order, and a "Decisions for <owner>" list with a recommendation for each question.
- Hex links a plan outward to file paths, test names, commands, PR numbers, issue numbers and commits. Chronicler does the same in its plan Facts sections.
- Hex keeps a finished plan in place and rewrites its Status line. It deletes a plan only when the plan's premise dies. One case exists (known: commit `a247f2b3`).
- Chronicler moves a finished plan to `old-docs/archived-plans/` and rewrites its Status line with the build commits. Both projects keep the record.
- `design/capture_capsule.sigil` is a declarative UI notation. It names 19 scenes, and 12 of the 19 names appear word for word in the 46 golden images under `tests/golden/capture-capsule-*.png` (known: file lists). See [What `.sigil` is](#what-sigil-is).
- Chronicler has no `ROADMAP.md`, no `## Current Work` section, no `.sigil` file and no task ids. Its work list is 10 open maps with 97 open tickets (known: file counts).
- 14 practices follow. 4 are covered, 5 are partly covered and 5 are gaps. 8 rows name a new candidate, and they cluster into 6 ideas.

## Practices

Labels: **known** means a file or commit in a clone shows it. **inferred** means a reading that the file does not state.

| # | Practice | Hex evidence | Chronicler state | Candidate it touches |
|---|---|---|---|---|
| 1 | A plan's Status line names the parts that are built, not only done/not done | `docs/plans/capture-in-hex.md:3-9` ("steps 2 and 3, and the input half of step 4 … are built"); `docs/plans/rune-ui.md:3-16` ("Done and merged") | **Partly covered.** All 5 plans in `docs/plans/` carry a `**Status:**` line (`docs/plans/ste-structural-prose-gate.md:4`; `docs/plans/parallel-ticket-runs-without-commits.md:4`). The states are coarse: Draft, Done, or a branch | new: plan conventions |
| 2 | A plan states what it did not prove | `docs/plans/engine-service.md:193-205` ("What the Spike Did Not Prove"); `docs/plans/typescript-sdk.md:471-475` ("a plan-level result, not implementation approval") | **Gap.** No Chronicler plan has a limits section. `docs/plans/generic-chat-completions-connections.md` labels facts known/docs/inferred (`:28`), but it never lists the untested part | new: plan conventions |
| 3 | A manual-verification checklist, by OS, gates the merge | `docs/plans/rune-ui.md:5-16` ("Do not merge to `main` until a person has checked"); `:118-137` the list; `ROADMAP.md:19-29` restates it as the release gate | **Partly covered.** A plan records that a person ran the checks (`docs/plans/generic-chat-completions-connections.md:4`). No plan or doc holds the checklist itself. The browser tier runs by machine (`build.py browser`) | new: manual-verification list |
| 4 | A numbered landing order, with a done mark on each step | `docs/plans/capture-in-hex.md:372-393` ("Order on the branch", each step "Built"); `docs/plans/engine-service.md:179-187` (PR 1 to PR 4) | **Partly covered.** Plans have numbered Phases with steps (`docs/plans/generic-chat-completions-connections.md:154`). Maps order tickets with `Blocked by:` (`docs/agents/issue-tracker.md:27`). No per-step done mark | new: plan conventions |
| 5 | Each open question carries a recommendation | `docs/plans/transcription-quality.md:173-186` ("Decisions for Kit", each with *Recommendation*); `docs/plans/language-recommendations.md:188` | **Covered.** `## Open questions` (`docs/plans/generic-chat-completions-connections.md:150`) and the Decisions table D1 to D14 with a provenance column ("Owner, 2026-10-30", "Code, already true") at `:131-148` | covered: no candidate |
| 6 | Acceptance-gated slices, plus a record of the review that changed the plan | `docs/plans/typescript-sdk.md:211` ("four acceptance-gated slices"); `:455-475` ("Adversarial review record", one row per reviewer) | **Covered.** `docs/plans/generic-chat-completions-connections.md:291-303` holds a finding-to-fix table (F1 to F11) from `improve-ai-plan` | covered: no candidate |
| 7 | A plan links outward: file paths, test files, commands, PR and issue numbers, commits | `docs/plans/rune-ui.md:34-64` (module tree), `:66-92` (golden names), `:83-91` (commands); `docs/plans/typescript-sdk.md:59` (PR #57); `docs/plans/transcription-quality.md:55-62` (issues #139, #177); `docs/plans/linux.md:222`; `docs/plans/rune-dependency.md:14-16` | **Covered.** Plan Facts name files and symbols (`docs/plans/generic-chat-completions-connections.md:47-115`). A finished plan records the build commits (`old-docs/archived-plans/remove-unused-part-routes.md:3`) | covered: no candidate |
| 8 | A document that is only a decision record: the choice, then the rejected alternatives | `docs/plans/rune-dependency.md:3-12` ("**Decision (Dax, 2026-10-22):**"), `:18-40` ("Why", four rejected alternatives) | **Partly covered.** The ADR format lives in `.agents/skills/domain-modeling/ADR-FORMAT.md:1-12` and names `docs/adr/`. That folder does not exist (known: `ls docs/`). Durable reasons live in `docs/diataxis/explanation/` | new: decision record |
| 9 | How a plan ends | Plans stay in `docs/plans/` with the Status rewritten (`rune-ui.md:3`, `quickjs.md:3`, `shared-desktop-ui.md:3`). A plan is deleted only when its premise dies: `a247f2b3` deletes `docs/plans/swift-to-rust-migration.md` and adds `docs/plans/swift-app-handoff.md` (known) | **Covered, by another mechanism.** A finished plan moves to `old-docs/archived-plans/` (`old-docs/archived-plans/remove-unused-part-routes.md:3`, 4 files). The move writes the build commits into the Status line | covered: no candidate |
| 10 | `ROADMAP.md` is the one prioritized work list | `ROADMAP.md:1-17` states what ships; 11 `##` sections, from "Validate The Rune Desktop App" (`:19`) to "Deliberate Deferrals" (`:276-284`); `AGENTS.md:1468-1473` ("`ROADMAP.md` is the authoritative work list"); `docs/README.md` lists it as "prioritized product and engineering work"; 51 commits touch it, most under 20 changed lines (known: `git log --numstat`) | **Gap.** Chronicler has no roadmap file (known: `rg -i roadmap` finds none outside this map). The work list is 10 open maps with 97 open tickets, plus 5 plans in `docs/plans/` | "ROADMAP.md as the one work list" (findings asset) |
| 11 | A `## Current Work` section in `AGENTS.md` keeps live constraints visible | `AGENTS.md:1472-1486`: a release freeze with its date, plus 5 rules. The section exists since 2026-07-23 (`c691f749`, known) | **Gap.** `AGENTS.md` holds the development loop and permissions. It has no live-constraint section. A freeze-like note lives only in a map's Notes (this map, `## Notes`) | "`## Current Work` habit" (findings asset) |
| 12 | A prototype keeps its decisions as files next to the variants | `prototypes/app-shell/README.md:1-5` (marked historical, defers to `AGENTS.md` and `ROADMAP.md`); `DECISIONS.md:1-18` (flat decision list, then "Interaction Values"); `SYNTHESIS.md:1-22` (which variant to keep, what to borrow, which data is missing); 3 variants switched by `?variant=` | **Partly covered, with a different record.** `/prototype` builds 3 variants switched by `?variant=` (`.agents/skills/prototype/UI.md:20,42`), and rule 6 says "Delete or absorb when done" (`.agents/skills/prototype/SKILL.md:26`). The record is a facts asset: `.scratch/prompt-flexibility/assets/trial-prototype.md:1-8`; `.scratch/_closed/platform-choice/assets/prototype-02-job-state-machine.md:1-8`. The decision goes to a ticket or a map, not to a file beside the variants | new: prototype decision record |
| 13 | A design notation file whose scenes become the UI test cases | `design/capture_capsule.sigil:1-8` (`schema "sigil/v1"`, a `model` of state, 6 components), `:79-133` one artboard, `:135-153` 19 `scene` lines; commit `3d7c581b` adds the file with the capsule; 46 goldens named `tests/golden/capture-capsule-*.png`, 12 of them sharing a scene name | **Gap.** No notation file. Design intent lives in prose (`docs/diataxis/explanation/dashboard_design.md`) and in the closed map `ui-verification-redesign` | new: `.sigil` design file |
| 14 | A task id in the commit subject points at a tracker outside the repo | 31 commits carry `(tNNNN)`, for example `a07f994e … (t694)`, `eadcd5eb … (t1153)` (known: `git log`). The tracker is not in the repo (inferred) | **Gap.** Chronicler ids are paths, one numbering per effort (`.scratch/<effort>/issues/NN-slug.md`, `docs/agents/issue-tracker.md:9-10`). A commit names no ticket | new: task ids |

## Detail

### The shape of a Hex plan

A Hex plan is a long-lived document that keeps the current state of one line of work. The parts differ by plan type, but 6 parts recur (known).

| Part | Where it appears |
|---|---|
| Status line, with date and which steps are built | `capture-in-hex.md:3-9`; `language-recommendations.md:3` |
| Why the work exists | `linux.md:8-30`; `capture-in-hex.md:11` |
| Facts with a label: measured, or read from code | `engine-service.md:66-88` (a measured spike, with the machine and model named) |
| Shape: modules, contracts, seams | `rune-ui.md:34-64`; `linux.md:144-171` |
| Ordered work: phases, slices or PRs | `transcription-quality.md:33-172`; `typescript-sdk.md:211`; `engine-service.md:179-187` |
| Decisions, limits, and what a person must check | `linux.md:212`; `engine-service.md:193`; `rune-ui.md:118` |

Two plans are only a decision: `rune-dependency.md` (2401 bytes) and `swift-app-handoff.md` (5744 bytes). Both open with the decision and its author, then give the rejected alternatives (known).

**Inferred:** the plan earns its size from evidence. `engine-service.md` and `transcription-quality.md` carry measurements, and `rune-dependency.md` is short because the alternative list is the whole content.

### How a plan links to code, tests and history

Hex uses 5 kinds of link inside a plan (known):

1. A module tree that names each file and its one job (`rune-ui.md:34-64`). The tree repeats the `## Architecture` bullets of `AGENTS.md`.
2. A table of surfaces against their golden image names (`rune-ui.md:66-92`).
3. The commands that prove the work (`rune-ui.md:83-91`: `cargo run -- app -- --list-scenes`, `cargo test rune_ui`, `HEX_UPDATE_GOLDENS=1`).
4. A PR or issue number (`typescript-sdk.md:59` PR #57; `transcription-quality.md:55` issues #139 and #177; `linux.md:222` PRs #28 and #29).
5. A commit hash, for a claim that comes from another tree (`language-recommendations.md:3` cites `ebd9fc8` and `fde2bb3`; `capture-in-hex.md:13-14` cites Scrawl's `e351ddb`; `rune-dependency.md:14-16`).

`docs/README.md` indexes each plan with its state in one clause: "implemented", "implemented; Rune desktop UI port, surfaces, goldens, and remaining real-window checklist", "research on importing historical Granola meetings" (known).

### How a plan ends

Three endings exist in Hex (known):

- **Kept, Status rewritten.** The common case. `rune-ui.md:3` reads "Done and merged into `main`". `quickjs.md:3` reads "implemented". `meeting-live-transcription.md:3` reads "recommendation, not implemented". The plan stays at its path.
- **Deleted, with a replacement.** Commit `a247f2b3` deletes `docs/plans/swift-to-rust-migration.md` (253 lines) and adds `docs/plans/swift-app-handoff.md` (93 lines) in the same commit. The premise (an automatic Swift-to-Rust migration) died, and the replacement records the new decision. `git log --diff-filter=D -- docs/plans/` shows one more deletion: 6 stale screenshots in `7ff2d59b`.
- **Archived in place.** `prototypes/app-shell/README.md:1-5` marks the prototype "Historical design input" and points at the authority docs.

**Inferred:** the plan text does not need to be true for a reader who builds from it, because the Status line and `docs/README.md` tell the reader its state first. `docs/README.md:57` states the rule: "When a plan disagrees with `AGENTS.md` about shipped behavior, update the plan."

Chronicler's ending is a move: `old-docs/archived-plans/`, 4 files. The move also rewrites the Status line with the build commits and the owner's decisions (`remove-unused-part-routes.md:3`) (known). This is more detail than Hex keeps about the landing.

### How a prototype becomes a decision

`prototypes/app-shell/` holds 3 HTML variants, one server script, and 4 markdown files (known). The decision path has 3 steps:

1. `SYNTHESIS.md:6-22` picks the base ("Use Variant A's calm workspace as the shell foundation"), borrows one part from B and one from C, and lists the data that the prototype needs and the product lacks.
2. `DECISIONS.md:7-17` lists 8 decisions as single sentences, for example "Meeting recording always starts explicitly." It then adds 4 interaction values (`:19-28`).
3. `README.md:1-5` marks the whole folder as history and points a reader at `AGENTS.md` and `ROADMAP.md` for current rules.

**Inferred:** the decision survives the prototype. The prototype does not. The folder stays because it is cheap to keep and it explains the shell's origin.

Chronicler's `/prototype` skill keeps the same variant-switch pattern (`UI.md:20`), and its rule 6 says to delete or absorb the prototype (`SKILL.md:26`). It says to capture the answer "somewhere durable (commit message, ADR, issue, or a `NOTES.md` next to the prototype)" (`SKILL.md:30`). Chronicler does keep a record, but as an asset: `.scratch/prompt-flexibility/assets/trial-prototype.md:1-8` lists what the prototype made, and `.scratch/_closed/platform-choice/assets/prototype-02-job-state-machine.md:1-8` keeps the facts of a prototype whose code stayed in `tmp/` (known). Neither file lists the decisions that the prototype settled, and neither sits beside the variant code.

### What `.sigil` is

`design/capture_capsule.sigil` is 161 lines with `schema "sigil/v1"` (known). It declares:

- a `model Capsule` of 25 state fields, each with a default (`:7-33`);
- 6 `component` blocks: `Chip`, `Action`, `Key`, `KeyChip`, `Section`, `Dest` (`:35-77`);
- one `artboard capsule (w: 640, theme: "opencode.dark")` that draws the capsule from state (`:79-133`);
- 19 `scene` lines, each a state combination with a name, such as `scene "picker-area" { area: true }` and `scene "no-answer" { picking: false, done: true, why: true, why_text: "the router didn't answer", ask: true, ask_label: "Send to…" }` (`:135-153`).

The header comments say the job: "Hex's capture capsule (src/rune_ui/capture_capsule), every state as a scene" and "Kit's Scrawl capsule (sigil examples/scrawl_capsule.sigil) in the OpenCode theme" (known).

The link to tests: 12 of the 19 scene names appear word for word in `tests/golden/capture-capsule-*.png` (`picker-area`, `picker-window`, `recording`, `recording-chips`, `processing`, `saved`, `rule-sent`, `sent`, `queued`, `guess`, `no-answer`, `list-guess`). Seven scene names have no golden of their own: `failed`, `nothing`, `taken-back`, `too-late`, `picker-screen`, `list-offer`, `list-search` (known: `ls tests/golden | rg capture-capsule` gives 46 images, light and dark). `src/rune_ui/capture_capsule/tests.rs:534` collects "The goldens' states: one of each kind of capsule", and `:561-564` runs them against `tests/golden/capture-capsule-*.png`.

**Inferred:** the sigil file is the design source for the capsule, and the scene list is the case list. The file is not generated and nothing reads it: no script, workflow or Rust target names `sigil` (known: `rg sigil scripts/ packaging/ .github/ site/` finds nothing). One commit adds it with the implementation (`3d7c581b`, 2026-10-26). `AGENTS.md` and `docs/` do not mention it (known).

**Inferred:** Sigil is a tool in the author's own toolchain. Scrawl's `examples/scrawl_capsule.sigil` is the only other file that the comments name, and it is not in this repo.

### `ROADMAP.md` and `## Current Work`

`ROADMAP.md` has 284 lines and 11 sections, ordered by the next release risk: validate the Rune app, validate the macOS release, finish settings, validate Linux, harden the real-time boundary, then 5 smaller areas, then "Deliberate Deferrals" (known).

Each section states what ships now, then what remains, then the person or machine that must prove it. For example `ROADMAP.md:19-29` names the per-OS checklist and what each OS received so far (known).

The file is edited in place and stays current. 51 commits touch it, and recent edits add and remove at most 18 lines each (known: `git log --numstat` gives 0/7, 10/14, 12/18 and 1/1). A finished item is rewritten in the same section, not deleted: `a8a4645d` removes the voice-command sentences from 4 places (known).

`## Current Work` in `AGENTS.md` (`:1472-1486`) holds 5 rules that outrank the roadmap for the moment, led by a dated release freeze. It exists since `c691f749` (2026-07-23). `AGENTS.md` changes in 228 of the 1146 commits (known).

**Inferred:** the two files split by change rate. The roadmap moves with product progress. `## Current Work` moves with the week's constraint.

### Hex plans compared with wayfinder maps and `.scratch/` tickets

| | Hex plan | Hex `ROADMAP.md` | Chronicler map + tickets |
|---|---|---|---|
| Unit | one line of work | the whole product | one effort |
| Holds | whatever the work needs: facts, shape, order, limits, decisions | priorities and the proof each item needs | an index of tickets; the detail stays in a ticket (`.agents/skills/wayfinder/SKILL.md:27`) |
| Status | one Status line per plan | prose per section | `Status:` per ticket: open, claimed, resolved (`docs/agents/issue-tracker.md:26`) |
| Order | a numbered list inside the plan | section order | `Blocked by:` edges; the frontier is the set of open, unblocked, unclaimed tickets (`docs/agents/issue-tracker.md:27-28`) |
| Who may act | any agent; the private control repo sets the workflow | a person sets the priority | a session claims a ticket first (`docs/agents/issue-tracker.md:29`) |
| End | Status rewritten; deleted if the premise dies | item rewritten in place | `## Answer`, then `Status: resolved` (`docs/agents/issue-tracker.md:30`) |
| Self-contained? | yes: one file answers the reader | no: it links plans | no: the map is an index by design |

Two structural differences stand out (inferred):

- **Count.** Hex has 18 plans and 1 roadmap. Chronicler has 10 open maps, 97 open tickets, 44 assets across those maps, and 5 plans. A Chronicler reader pays one map read plus one ticket read to learn an area's state.
- **Decisions.** Hex writes the decision into the plan that holds it (`rune-dependency.md`), or into `ROADMAP.md`'s "Deliberate Deferrals" (`:276-284`), or into `AGENTS.md`. Chronicler writes a decision into a map's Notes, a ticket's `## Answer`, or a plan's Decisions table (`generic-chat-completions-connections.md:131-148`). Chronicler documents an ADR format (`.agents/skills/domain-modeling/ADR-FORMAT.md`) but has never created `docs/adr/` (known).

### Task ids

31 commit subjects carry a task id in parentheses, for example `a07f994e … (t694) (#408)` and `eadcd5eb … (t1153) (#705)` (known: `git log --format=%s | rg -c '\(t[0-9]+\)'` gives 31; this ticket's Context says 24). Ids range from `t405` to `t1153`. `AGENTS.md` never mentions the ids or their tracker (known). The findings asset infers the tracker is in the private control repo.

**Inferred:** the id joins several commits of one tracker item, across branches and PRs, and the PR number follows it in the same subject. Chronicler has no equivalent: a commit names no ticket, so the link from code to a map runs the other way (a plan or ticket names a commit).

## What I could not find or did not read

- **`~/code/control/AGENTS.md`.** It holds the Hex workflow rules that `AGENTS.md` defers to (findings asset). It is private and not in either clone. So I cannot say whether the plan rules are written there.
- **The task-id tracker.** I found the ids and no tracker. I did not call the GitHub API.
- **`docs/features/`.** I read only `README.md` and `capture.md` signatures, because candidate 3 belongs to another ticket. The link from a plan to a feature's proof is therefore unchecked.
- **`docs/research/`.** I did not open the 8 files. I read only the plans that synthesize them, so a research doc's own structure is unchecked.
- **Sigil.** I read the one file and searched for tooling. I did not find the format's specification, so the field meanings (`presence`, `dissolve`, `min_w`) are guesses.
- **`docs/specs/local-transcription-service.md`.** Not read. The ticket does not name it.
- **`docs/releases/` and `release-notes/`.** I listed 19 and 9 files and read none.
- **`ROADMAP.md` from line 98 to the end.** I read every heading, the last 2 sections in full, and the start of "Harden The Real-Time Boundary".
- **The other 7 tickets of this effort.** I read the map, this ticket and the findings asset only.

## Sampling

I did not read every plan line. The sample and its reason:

| Area | Read | Why that sample |
|---|---|---|
| `docs/plans/` | All 18 headings; 8 plans in full or nearly (`rune-ui`, `capture-in-hex`, `linux`, `engine-service`, `typescript-sdk`, `transcription-quality`, `rune-dependency`, `quickjs`); `granola-import`, `language-recommendations`, `shared-desktop-ui`, `swift-app-handoff`, `single-process`, `meeting-live-transcription`, `release-service`, `macos-signing`, `capture-handover-spec` by head and headings | The 8 cover the largest, the newest, the shortest and the 3 status kinds: done, active, not implemented |
| History | `git log --diff-filter=D` on `docs/`, `docs/plans/`, `.scratch/`; `--numstat` on 4 `ROADMAP.md` commits; `-S 'Current Work'`; `--stat` on `a247f2b3` and `3d7c581b` | To answer "how does a plan end" from evidence, not from a rule |
| Prototype | All 4 markdown files in `prototypes/app-shell/`; the 3 variants by name only | 252 words of docs answer the question |
| `.sigil` | All 161 lines | It is one small file |
| Chronicler | `docs/plans/` (5 files, one in full), `old-docs/archived-plans/`, `docs/agents/`, `.agents/skills/{wayfinder,prototype,research,domain-modeling}`, `AGENTS.md`, `docs/AGENTS.md`, `CONTEXT.md` head | These are the files the ticket names |

Counts come from `ls`, `wc` and `git log` on 2026-11-01 unless the text gives another date.
