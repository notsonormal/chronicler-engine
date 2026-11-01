# Map: What Chronicler can learn from Hex

Labels: wayfinder:map

## Destination

Each Hex practice that the research finds has a decision: adopt, already covered, or drop with a reason. An adopted practice ends as a brief that someone can build from. If the practice is too large for that, it ends as a question for a new map or for an existing map.

## Notes

- **Domain.** How an AI-coded Rust project works with its agents: agent instructions, planning, specs, code and test style, gates, landing and review. The subject is [anomalyco/hex](https://github.com/anomalyco/hex) at commit `7609574ca92c` (2026-10-31), the Rust rewrite of the archived Swift app [kitlangton/Hex](https://github.com/kitlangton/Hex). Chronicler terms are in `CONTEXT.md`.
- **The first look was shallow** (owner, 2026-11-01). The [findings asset](./assets/findings.md) holds 8 candidate practices from a short read. It is not a complete list, and a candidate is not ready for a decision until research covers it.
- **Research before decisions.** Seven research tickets each read one area of Hex. Next, the task [Prepare the candidate list for the grilling](./issues/09-task-prepare-the-candidate-list.md) merges the practices, checks the claims they rest on, and proposes a decision for each one (owner, 2026-11-01). Then the grilling ticket [Agree the practices to decide](./issues/08-grilling-agree-the-practices-to-decide.md) checks the proposals with the owner, one theme at a time, and picks the type of each decision ticket. A research ticket does not create decision tickets.
- **Each research ticket reads both sides.** It reads its Hex area and the Chronicler files that its ticket names. For each practice it records: the Hex evidence (file, commit, PR or URL), and the Chronicler state (covered, partly covered, or a gap, with the Chronicler file). Each claim is labelled known or inferred.
- **Research output.** One asset for each ticket: `assets/research-NN-<slug>.md`. Follow the `/research` skill: primary sources, and a source for each claim. The ticket's `## Answer` gives the gist and links the asset.
- **Planning, with one exception** (owner, 2026-11-01). The map produces decisions, not builds. One exception: if a decision is a change of one session or less, the map gets a follow-on task ticket that builds it. A larger adopted practice leaves the map as a brief, or as a question for a new or existing map (`statechart-models`, `code-concepts` or another).
- **Outside check** (owner, 2026-11-01). The map ends with a check by a fresh `reviewer` agent. It reads the map, the assets and both repos. It confirms that each practice has a decision with evidence.
- **Sources.** Clones are in `tmp/hex-compare/anomalyco-hex/` and `tmp/hex-compare/kitlangton-hex/`. `tmp/` is git-ignored. If the clones are gone, clone the repos again at the commits in the findings asset. The GitHub REST API is public, but without a token it allows 60 requests an hour, and the search API allows 10 requests a minute. Keep raw API answers in `tmp/hex-compare/github/` so that no request runs twice. This repo has no `gh` CLI dependency.
- **Skills.** `/research` for the research tickets. `/grilling` and `/domain-modeling` for the grilling tickets. `/codebase-design` when a decision concerns module or seam design.
- **Writing.** Simplified Technical English, as in `AGENTS.md`.

## Decisions so far

<!-- one line per closed ticket: [title](link) — gist -->

- [Research what Hex's builders say in public](./issues/07-research-public-sources.md) — no public write-up of the Hex build process; `~/code/control`, hivemind, `termctrl` and `abattoir` are not public, but `.hivemind.toml`, `clippy.toml` and `anomalyco/opencode/AGENTS.md` show their interfaces and team rules; "apollo" is a Linux dev host. 19 practices, 7 new (a private global guide, ticket ids in commits, HiveMind build workers, no outside PRs, one long session per project with subagents, published skills, dogfooding).
- [Research how a Hex change goes from branch to installed app](./issues/06-research-branch-to-installed-app.md) — one squashed commit per PR on `dev`, merged by the agent when 4 workflows pass (agent policy, Linux with Nix, macOS, Windows); a push to `dev` repeats the checks, publishes a dev channel and test builds, and `.hivemind.toml` installs the app. 10 `fix(ci)` commits on 2026-10-31 show post-merge checks catching cross-OS breakage. Chronicler has no remote gate: `build.py` and owner approval are the whole path. 9 practices, 6 new.
- [Research how Hex writes and evolves its agent instructions](./issues/01-research-agent-instructions.md) — one always-loaded `AGENTS.md`: a 97-bullet module map and 73 invariants (31 with numbers); workflow rules point to a private file. The spec changes in the commit that makes it true (7 of 225 `AGENTS.md` commits touch nothing else), rules enter with their incident and leave with their feature. Chronicler's seam rule is only in the `codebase-design` skill, and `docs/AGENTS.md` ("Values stay in the code", "No negative explaining") opposes the Hex style. 10 practices, 5 new.
- [Research Hex code and test style](./issues/05-research-code-and-test-style.md) — Hex gets its strictness from prose, not tools: 6 traits in about 190k lines, `color_eyre` string errors, files up to 6120 lines, inline test modules, test names that state the invariant with its number. Its only code-level tool is a shared lint block with a debt ledger (remaining errors for each lint turned off); `disallowed_methods` is one of them, set to `allow` with 85 errors left. Two Chronicler guardrails (long comment runs, no inline test modules) conflict with Hex habits. 16 practices, 10 new.
- [Research how Hex plans work](./issues/03-research-planning.md) — one long-lived file per line of work in `docs/plans/`, kept as current state: a Status line with evidence, a numbered landing order with done marks, a limits section, a checklist for a person, and decisions with rejected alternatives. A finished plan stays, with its Status rewritten. `ROADMAP.md` is the one ranked work list. Chronicler covers the Status line and decision tables, archives finished plans in `old-docs/archived-plans/`, and has no roadmap, `## Current Work`, limits section, landing order or `docs/adr/`. Task ids: 24 unique ids (the commit count depends on the pattern: 31 or 51). 14 practices, 8 new.
- [Research the Hex issue pipeline from its public PRs and issues](./issues/02-research-issue-pipeline.md) — one issue makes one run, one PR on `agent/issue-N` and one squash commit; the run also acknowledges, labels and closes the issue. 60 of 64 agent PRs merged; median issue to merge 6.7 min. Gates: `check.ts`, a second model's approval, a build on one host; no person and no platform CI holds the merge. 8 of 59 pipeline commits are `fix(ci)` repairs, 0 reverts. The pipeline code is not public, so the strict reviewer's criteria are unknown. Chronicler has no part of this loop. 12 practices, 6 new.
- [Research the Hex feature map and its proof records](./issues/04-research-feature-map.md) — `docs/features/` holds 142 behaviors with stable dotted ids and keeps three states apart: built, a check exists, and someone saw it work (a dated paragraph naming the machine and build, with "Proof limit" and "not yet observed" lines). No code, test or gate reads the ids or the map, so drift survives (11 of 611 named checks no longer resolve). Chronicler enforces the spec-to-test link (`spec-coverage`, 554 scenarios) but has no record of what someone saw work and keeps known gaps in closed tickets. 15 practices, 3 new.
- [Prepare the candidate list for the grilling](./issues/09-task-prepare-the-candidate-list.md) — 103 rows merge into 52 practices in 7 themes: keep 19, covered 13, drop 20. Proposed: 6 decision tickets (G1 landing and gates, G2 spec and docs as feature maps with an optional proof-records split, G4 golden images, G5 seams with env and clock, G6 gate scope, G7 planning). 47 checks: 5 claims wrong, none changes a keep or a drop. Three tests depend on the clock, not one. The two agent hook traits each have one impl. Corrections for this map are in part 7 of [candidates.md](./assets/candidates.md).

## Not yet specified

- **The 8 candidate practices** from the [findings asset](./assets/findings.md). The research turns each one into a decision ticket, merges it with another, or removes it. Each ticket gets its type from what the research finds:
  1. A deterministic risk gate on agent diffs (`.github/agent-policy/check.ts`).
  2. Invariants in prose with exact values (`AGENTS.md`, `## Invariants`).
  3. A feature map that states its proof (`docs/features/`).
  4. Small units of work that land automatically (the issue pipeline, `.hivemind.toml`).
  5. Golden-image UI tests (`tests/golden/`). Read the decision in the closed `ui-verification-redesign` map first.
  6. Clippy `disallowed-methods` for env and clock reads. Hex writes this ban but turns it off (research 05). The findings asset lists the Chronicler call sites, and one test that needs a substitute clock. Options seen so far: ban env reads only, and make the clock a separate decision; ban both and mark each existing site; ban both and add an injected clock.
  7. "No seam until there are two real adapters or a test needs one."
  8. Labs: new features ship off by default behind a toggle.
- **Linked candidates.** The owner agreed on 2026-11-01 that candidates 1 and 4 belong together, and that candidates 2 and 3 belong together. Candidates 2 and 3 may join `statechart-models` (it proposes paired rule ids) or `code-concepts`.
- **Docs organized as feature maps** (owner, 2026-11-01). The owner finds it hard to decide how to organize the docs and which docs to create. Hex `docs/features/` and `pstack` both organize docs by user capability. Hex `AGENTS.md` says: "Organize by user capability, not Rust module or UI pane." Chronicler's reference docs follow the code: each `src/` file has a DOC anchor to `docs/diataxis/reference/`. Diátaxis decides the kind of a doc, not which docs exist (inferred). Sources:
  - research 04
  - Lauren Tan's talk, [excerpt](./assets/excerpt-poteto-feature-maps.md) ([08:55]–[11:23]; the full transcript is `tmp/poteto-2500-prs-talk-2026-09-21.md`): a feature map is "a form of materialized memory", like a site map, kept in the skill directory, maintained by an automation, and paired with a control CLI
  - the `pstack` skills `create-verification-skill` and `maintain-verification-skill` (`tmp/hex-compare/cursor-plugins/`, commit `ae5650f`)
  
  The owner wants no more research on `pstack`. It may have reached the same idea as Hex on its own. This belongs with candidates 2 and 3.
- **Smaller habits seen in the first look:** a `## Current Work` section in `AGENTS.md`; `ROADMAP.md` as the one work list; capability folders for modules; short commit bodies (median: 3 lines).
- **The outside check ticket.** It is created when the decision tickets exist, and it is blocked by all of them.

## Out of scope

- Working a large adopted practice to its end inside this map. It leaves as a brief, or as a question for a new or existing map.
- Changes to Hex. This map only reads it.
