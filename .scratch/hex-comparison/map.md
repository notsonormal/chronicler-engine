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
- **AFK first** (owner, 2026-11-01). Each kept practice gets an AFK ticket that measures it. The ticket ends with a recommendation and at most 2 questions. [Confirm the recommendations of tickets 10 to 15](./issues/16-grilling-confirm-the-recommendations.md) confirms them together, with at most 3 questions in each round. Only [Decide whether anything lands without the owner](./issues/17-grilling-decide-whether-anything-lands-without-the-owner.md) is a grilling of its own. The reason: after a grilling of about 33 questions, the owner had low confidence in his answers. New tickets number from 19.
- **Terms of this map** (owner, 2026-11-01). "Spec" means only a Chronicler `docs/specs/` file. Hex's `AGENTS.md` is "the Hex instruction file". The **work report** is the durable record that one agent session leaves when it finishes a change: which checks ran, which did not, and the review verdict. "Run" alone is not used, because `CONTEXT.md` reserves it for an Autoplay run.

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
- [Agree the practices to decide](./issues/08-grilling-agree-the-practices-to-decide.md) — of 52 practices: 15 kept, in 9 new tickets. 6 AFK tickets measure them, one grilling confirms their recommendations, one grilling decides landing without the owner, and the outside check comes last. 7 doc practices wait for research on how agents use the docs, 11 are covered and 19 are dropped. Terms: "spec" means only a `docs/specs/` file, and the "work report" is the record of one agent session. The part 7 corrections are now in the assets.

## Not yet specified

- **Docs organized as feature maps, and the doc practices.** This entry waits for [Research how agents use the current docs and plans](./issues/10-research-agent-use-of-docs-and-plans.md). The questions in its answer become tickets here.
  - **The waiting practices** of [candidates.md](./assets/candidates.md):
    - C-01: one always-loaded instruction file
    - C-02: invariants with exact values
    - C-03: docs change with the behaviour, in both directions
    - C-09: docs by user capability
    - C-10: a stable id for each behaviour
    - C-11: a mechanical check on the doc map
    - C-32: test names that carry the number of the invariant

    The answer of [Agree the practices to decide](./issues/08-grilling-agree-the-practices-to-decide.md) holds the owner's first view of each one.
  - **The owner's position** (2026-11-01). An AI writes the docs, and an AI reads them. A better structure for AI would help, but he wants no rewrite that rests on a hypothetical. Feature maps appealed to him for the same reason that Diátaxis did: a focused, logical structure.
  - **What Chronicler has (known).** `docs/specs/` is already most of a feature map, and `chronicler-ui-investigator` uses the browser specs as its selector vocabulary. Four parts are missing:
    - one page for each capability (Autoplay is split over two specs, by test surface)
    - proof records (ticket 12)
    - known gaps (ticket 12)
    - a rule for which docs exist
  - **Other maps.** C-02 and C-10 may join `statechart-models`, which plans TLA+ invariants paired with spec scenario ids, or `code-concepts`.
  - **Sources.**
    - research 04
    - Lauren Tan's talk, [excerpt](./assets/excerpt-poteto-feature-maps.md) ([08:55]–[11:23]; the full transcript is `tmp/poteto-2500-prs-talk-2026-09-21.md`)
    - the `pstack` skills `create-verification-skill` and `maintain-verification-skill` (`tmp/hex-compare/cursor-plugins/`, commit `ae5650f`)

    The owner wants no more research on `pstack`.

## Out of scope

- Working a large adopted practice to its end inside this map. It leaves as a brief, or as a question for a new or existing map.
- Changes to Hex. This map only reads it.
