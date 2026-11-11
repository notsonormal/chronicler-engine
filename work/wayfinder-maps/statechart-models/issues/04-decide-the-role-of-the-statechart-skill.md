# Decide the role of the statechart skill in this repo

Type: grilling
Status: resolved
Blocked by: 02, 03, 05

## Question

Which role does the statechart skill get in this repo?

| Role | What stays in the repo | What enforces it |
|---|---|---|
| Drop it | nothing; remove `.agents/skills/statechart/` | — |
| Ad-hoc design aid | the skill only; TLA+ models stay in `tmp/` | nothing |
| Committed TLA+ models with paired rule ids | TLA+ models next to the docs they state, and rule ids in doc sentences | a validator that pairs each rule id with an invariant |
| Test generator only | the skill, and the tests each break yields | the existing `SCENARIO` and `CASE` validators |
| Trace validation (added by ticket 02) | the TLA+ model, a Trace spec, a transition log in `BusyRegistry::hold` behind the `testing` feature | TLC replays the logs of the tier-1 tests in a build step; needs a JDK in the build |
| One-time audit (added by ticket 03) | the skill and a short repo-local addendum to it; a new spec-only TLA+ model in `tmp/` for each audit; the findings go to the specs and the tests | nothing; a trigger starts an audit, for example a new flow or a change to the gate |
| Committed TLA+ models paired with spec scenario ids (added by ticket 03) | TLA+ models; each invariant names the spec scenario that states its rule | a validator that pairs each invariant with a scenario, as `validate_feature_spec.py` pairs tests with scenarios; no rule ids in doc sentences |

Decide with the yield of [Prototype: one TLA+ model of the gate, built from the spec only](03-prototype-spec-only-gate-model.md) in hand, and with the methods and costs of [Research: how other teams tie a TLA+ model to code](02-research-tying-a-tla-model-to-code.md). A role that ticket 02 shows as better may replace a row of this table.

## Context

- The owner's question (map Notes): can a TLA+ model find gaps between the spec and the code, or tie the docs to the code, or is it only a mechanical check of the code?
- The base rate so far (known): three TLA+ models written after a read of the code, no code defect, one break that `game_flow.md` already accepts.
- The cost so far (known): a JDK of about 210 MB and `tla2tools.jar`, not on this host by default; one session for the three charts.
- The decision graduates fog on the map: where a rule of a TLA+ model pairs, the toolchain, a repo-local addendum to the skill, the trigger for a new audit, and the provenance of the vendored skill. Write which of them the chosen role needs.
- Ticket 02 ranks the rows for this repo: paired rule ids with one tier-1 test per rule first, trace validation second as the upgrade path, with a named trigger for the move. Its open points belong here: the gate only, or the gate plus the Autoplay loop; whether a JDK in the build is acceptable at all; where the rule ids live.
- Ticket 03 answered (see its Answer): no code defect; three "own id" rules that the code holds and the spec holds in part; six candidate tests; 5 of 15 rules true by construction; every break in fewer than 4,000 states, and nothing more in the full check of 15,798,130 states.
- **The owner's leanings (2026-10-30), not a decision:**
  - `game_flow.md` need not cover every use case. The specs hold the event rules, and the docs give an understanding. So a role that adds rule ids or rules to `game_flow.md` goes against this leaning.
  - The TLA+ model is used to pick up gaps between the specs, the docs and the code.
  - A TLA+ model in `tmp/` is written again for each audit. A reused TLA+ model goes stale with no signal, and a new spec-only one stays independent of the code.
- Use `/grilling` and `/domain-modeling`, one question at a time. The owner answers for themselves.

## Comments

### 2026-11-05: how the creator of the skill uses it

The owner pointed at the practice of Lauren Tan (@poteto), the author of pstack, for her game tibo.dog. Its repo is private (the GitHub API answers 404), so the facts come from her posts, the pstack repo (commit `2776ddd2`, 2026-11-05) and one public chart. Local copies: `assets/local-only/poteto-tibo-dog-verify-skills.jpg` (her `.cursor/skills/` folder, many `verify-*` skills) and `assets/local-only/poteto-tla-autopick.jpg` (the chart).

- **She pairs the skill with two more pstack skills (known).** "best combined with /create-formal-verification-skill" (her post, 2026-10-29). `create-formal-verification-skill` writes a project-local verify skill for one area. `maintain-formal-verification-skill` brings it back in line after code changes. This repo vendored `statechart` only. Apart from `scripts/test/`, the vendored copy is the same as pstack `main` on 2026-11-05.
- **Her method is not ours (known, from the two SKILL.md files).** The TLA+ model is "true to the code": each part points to code that a reviewer can open. It stays in the repo in a project-local skill, with a coverage record (files and functions) and a sync commit. Each property must fail once on a broken variant of the code. The maintain skill diffs the covered code from the sync commit and sorts each failure as drift or a bug.
- **Her TLA+ models found real code bugs (known, her chart).** One TLA+ model holds the level-up offer and pause and focus as parts that run at the same time. With the fix of her PR #1191 off, TLC found three bugs: the 8 s auto-pick cut a lesson, it picked while the player looked away, and after a reroll it pointed at a card of the old hand.
- **A correction to ticket 03's Context (inferred).** "A TLA+ model written from the code can only agree with the code" is too strong. A TLA+ model true to the code can break a stated rule, and hers did. What it cannot find is a gap between the docs and the code.
- **The upkeep here (known, `git log`).** Since 2026-10-05 (244 commits in all): `gate.rs` 28 commits, `job.rs` 27, `driver.rs` 5. So a committed TLA+ model of the gate needs a maintain pass about once a day at the present rate.

### 2026-11-05: grilling round 1 (owner)

- **Q1, the role: Option I, for the gate only, as a trial.** A spec-only audit finds the gaps against the spec first, as ticket 03 did. Then the same TLA+ model is brought in line with the code and kept as a project-local verify skill of the gate, by the method of pstack's `create-formal-verification-skill`: true to the code, a coverage record, a sync commit, and `maintain-formal-verification-skill` for drift. The audit half finds gaps between the spec, the docs and the code. The kept half catches regressions on each change to the gate. Open cost: a reproducible tool version, and this host has no JDK by default.
- **Q2, the `tmp/` folders: Option A, yes.** Ticket 04 decides what happens to `tmp/statechart/` and `tmp/tla-ts-run/`.

### 2026-11-05: grilling round 2 (owner)

- **Q3, the `tmp/` folders: Option A.** Copy the three TLA+ models from the code (`job-life/`, `autoplay/`, `swipe-state/`, about 440 KB) into `assets/00-code-models/`, tracked in git. Delete the rest of `tmp/statechart/` and all of `tmp/tla-ts-run/`.
- **Q4, the toolchain: Option A.** Install `mise` on this host. The skill's own pins (`tla2tools.jar` 1.7.4 with its SHA-256, Java `temurin-25`) do the install. Keep the bounds small, because memory (8 GiB) is the limit of this host.
- **Q5, the drift signal: Option C, a warning only.** A Python check in `build.py` warns when covered code has a change after the sync commit. It needs no JDK. The recommendation was a check that fails; the owner chose a warning.
- **Q6, the coverage scope: Option B.** `gate.rs` and the functions that take a hold of the gate in `job.rs`, `driver.rs` and `catalogue.rs`. The coverage record names functions. Inferred: the check of Q5 must read the diff of each function, not only the commit list of each file.
- **Q7: challenged by the owner.** "If the model is created from the src, then the source of the model is the src, not the spec, so what's the point in doing a spec-only audit?" Answered in round 3.

### 2026-11-05: grilling round 3 (owner)

- **Q7 (revised), where the rules come from: challenged again.** The owner: make the TLA+ model from the code, then compare it with the spec and the docs to find the gaps, because the code, the TLA+ model, the spec and the docs overlap. Answered in round 4.
- **Q8, where the trial work lives: Option A.** New task tickets from 06 in this map. The Destination becomes wider, and the "Plan, don't do" note ends. The recommendation was a new map; the owner chose this map.
- **Q9, the provenance: Option A.** Copy `create-formal-verification-skill` and `maintain-formal-verification-skill` by hand from pstack commit `2776ddd2` (2026-11-05), with no `skills-lock.json` entry, as the six other pstack copies.

### 2026-11-05: grilling round 4 (owner)

- **Q7 (third version), the comparison: Option A.** The TLA+ model is made from the code. It is the hub, and the verify skill compares it with each other source: (1) TLA+ model and code: each action points to a function, and each rule fails once on a broken copy of the code; (2) spec to TLA+ model: each gate scenario sentence becomes a rule, quoted, not written after a read of the code; (3) TLA+ model to spec: each action and each branch has a spec scenario that covers it; (4) TLA+ model and tests: a tier-1 test forces the order of each break path; (5) TLA+ model and docs: no sentence of `game_flow.md` contradicts the chart. Two lists in the verify skill: the actions (code function, spec scenario) and the rules (quoted spec sentence). Each run reports its gaps by kind: code bug, spec gap, test gap, doc error. **This replaces the spec-only audit half of Option I** (owner, round 3): no new spec-only audit of the gate; ticket 03 was the first and the last.
- **Q10, who runs the maintain pass: Option A.** The agent that sees the `build.py` warning runs `/maintain-formal-verification-skill` before it ends its task. The warning text names the skill.
- **Q11, the end of the trial: Option C, withdrawn in round 5.** No end date, and each weekly retro reviews the trial. The premise was wrong: the weekly retro of 2026-11-04 was one event with that name, not a meeting every week (owner, round 5).

### 2026-11-05: grilling round 5 (owner)

- **Q12, what the trial does with each gap: Option A, report only.** "We just find it and then we figure it out" (owner). Each gap goes to the trial log. The trial lands no test and edits no spec, doc or product code.
- **The end of the trial (replaces Q11).** "We do a trial, see the results, and then decide what to do in the next ticket" (owner). A grilling ticket reads the results and decides each gap and the future of the verify skill. There is no preset limit to keep the skill.
- **Q13, the trial log.** `assets/trial-log.md` in this map: one row for each run and each maintain pass. No line in the `retro` skill, because no retro happens every week.
- **Q14, the assumptions.** No objection to 1 (the warning in the end summary of the build log), 3 (`.agents/skills/verify-gate/`, loadable by agents) and 4 (each rule quotes a gate scenario of `docs/specs/`; a rule in `game_flow.md` only is a spec gap). Item 2 (a measure for a retro) is replaced by the grilling ticket above. Item 5 (the tickets) changes: no ticket to land tests, and a grilling ticket at the end.

### 2026-11-05: grilling round 6 (owner)

- **Q15, what the results include: Option A, the first run only.** Ticket 09 is blocked by 07. It decides from the gaps of the first run, with no data about the upkeep. The recommendation was the first run and three maintain passes; the owner chose the first run only.

## Answer

**The `statechart` skill stays, and a trial makes one verify skill of the gate** (owner, 2026-11-05, rounds 1 to 6 above).

- **Method.** pstack's `create-formal-verification-skill` writes `.agents/skills/verify-gate/`. Agents can load it by themselves. Its TLA+ model is made from the code: each action points to a function that a reviewer can open. It has a coverage record by function and a sync commit. `maintain-formal-verification-skill` brings it back in line after a change to covered code.
- **The comparison.** The TLA+ model is the hub. The verify skill compares it with five sources: the code (each rule fails once on a broken copy of the code), the spec to the TLA+ model (each gate scenario sentence of `docs/specs/` becomes a quoted rule, not written after a read of the code), the TLA+ model to the spec (each action and each branch has a covering scenario), the tests (a tier-1 test forces the order of each break path), and the docs (no sentence of `game_flow.md` contradicts the chart). A rule that only `game_flow.md` states is a spec gap. The verify skill keeps two lists: the actions (code function, spec scenario) and the rules (quoted spec sentence).
- **No new spec-only audit.** Ticket 03 was the first and the last spec-only audit of the gate. The comparison with the spec replaces it.
- **Coverage.** `gate.rs`, and the functions that take a hold of the gate in `job.rs`, `driver.rs` and `catalogue.rs`.
- **Toolchain.** `mise` on this host. The skill's own pins do the install: `tla2tools.jar` 1.7.4 with its SHA-256, and Java `temurin-25`. Small bounds, because memory (8 GiB) is the limit of this host. tla-ts is not used.
- **Drift.** A check in `build.py` warns, in the end summary of the build log, when covered code changed after the sync commit. The warning names `/maintain-formal-verification-skill`, and the agent that sees it runs the pass. This waits for ticket 09.
- **Gaps.** Report only, by kind (code bug, spec gap, test gap, doc error), in `assets/trial-log.md`. The trial lands no test and edits no spec, doc or product code.
- **Provenance.** The two pstack skills are copied by hand from `cursor/plugins` commit `2776ddd2` (2026-11-05), with no `skills-lock.json` entry, as the six other pstack copies.
- **The end.** Grilling ticket 09 reads the gaps of the first run, decides each gap, and decides whether `verify-gate` stays. With no data about the upkeep (owner, Q15).
- **`tmp/`.** Done on 2026-11-05: the three TLA+ models from the code are in [`assets/00-code-models/`](../assets/00-code-models/). `tmp/statechart/` and `tmp/tla-ts-run/` are deleted. The weekly retro map keeps its own copy of both in its `assets/local-only/tmp-evidence/`.
- **Rejected.** Drop the skill (A). An ad-hoc aid with nothing kept (B). A spec-only audit for each trigger (C). Rule ids in doc sentences (D), and a validator that pairs invariants with scenario ids (E). A test generator only (F). Trace validation with TLC in the build (G).

**The fog of the map:**

| Item | Resolution |
|---|---|
| Where a rule pairs | A gate scenario of `docs/specs/`, quoted in the rule. No validator. |
| The toolchain | `mise` on this host, with the skill's pins. |
| A repo-local addendum | None to `statechart`. The discipline lives in `verify-gate`. |
| The trigger for a new audit | None. The drift warning and the maintain pass replace it, if ticket 09 keeps the skill. |
| Provenance | A copy by hand from `2776ddd2`, with no lock entry. |

**New tickets.** [06](06-task-set-up-the-toolchain-and-the-pstack-skills.md), [07](07-task-write-the-verify-skill-of-the-gate.md), [09](09-grilling-decide-the-gaps-and-the-verify-skill.md), and [08](08-task-add-the-staleness-warning.md), which waits for 09.
