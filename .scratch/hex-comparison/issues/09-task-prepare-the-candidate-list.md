# Prepare the candidate list for the grilling

Type: task (AFK)
Status: resolved
Blocked by: 01, 02, 03, 04, 05, 06, 07

## Question

Which unique practices did the research find? Which claims does each one rest on, and are those claims correct? What does the agent propose for each practice, so that [Agree the practices to decide](./08-grilling-agree-the-practices-to-decide.md) can check proposals theme by theme, and does not start from about 95 raw rows?

## Context

- **Why this ticket exists** (owner, 2026-11-01). The seven research assets and the findings asset hold about 21k words and about 95 practice rows, and many rows repeat across assets. The grilling is a session with the owner. It should spend that session on decisions, not on reading and merging.
- **Inputs.**
  - the [findings asset](../assets/findings.md)
  - `assets/research-01-*.md` to `assets/research-07-*.md`
  - the map's **Not yet specified**, which includes the entry **Docs organized as feature maps**
- **Output.** One asset: `assets/candidates.md`, with these parts:
  1. **Themes.** Group the practices into themes. A first proposal: agent instructions and spec; docs organization; proof and verification; landing and gates; code and test style; planning and tracking; other. Change the themes if the practices fit other groups better.
  2. **One table for each theme.** Each row is one unique practice, with these columns:
     - id (`C-NN`) and short name
     - one-line description
     - sources (asset and row)
     - Hex evidence, checked (see part 4)
     - Chronicler state (covered, partly covered or a gap), with the Chronicler file
     - any conflict with a Chronicler rule, for example "Values stay in the code" (`docs/AGENTS.md`), the guardrail on long comment runs, or `check_test_structure.py`
     - the linked candidates (1–8, a smaller habit, or docs as feature maps)
     - a proposal: keep for a decision, already covered, or drop, each with a reason
     - for a kept practice: a proposed ticket type (research, prototype, grilling or task)
  3. **Groups.** List the kept practices that belong in one decision ticket. The owner already joined these: candidates 1 and 4; candidates 2 and 3, with docs as feature maps.
  4. **Claim checks.** List each claim that a proposal depends on. For each one, give: the claim, its source, the check (a command, or a file and line), and the result: confirmed, wrong, or not checkable. Give a correction for each wrong claim.
     - Check against the clones in `tmp/hex-compare/` and against this repo.
     - For GitHub data, use the cached JSON in `tmp/hex-compare/github/`. Make a new API request only if a proposal depends on it. Without a token, the limit is 60 requests an hour.
     - Do not check these again. They are already corrected in the map or the findings asset:
       - Hex sets `disallowed_methods = "allow"`.
       - `sdk/commands/workspace-template/AGENTS.md` exists only in old history.
       - The commits hold 24 unique task ids.
       - The pipeline is partly public through its PR bodies.
  5. **Depth.** Collect the section "What I could not find or did not read" from each asset, and group it by theme. For each gap, say whether it could change a proposal. The owner judges the depth in the grilling with this list.
- **Limits.**
  - The proposals are proposals only. The owner decides in the grilling.
  - Do not create decision tickets.
  - Do not edit the research assets, the findings asset or `map.md`. Record each correction in `candidates.md`. The coordinator moves corrections into the map.
  - Use one session. If the checks do not all fit, check first the claims that the most proposals depend on, and list the claims that you did not check.
- **Writing.** Simplified Technical English. Label each claim as known or inferred.
- **When done.** Add `## Answer` with a gist and a link to the asset, and set `Status: resolved`.

## Answer

The 103 practice rows merge into 52 unique practices in 7 themes. Proposals: keep 19, already covered 13, drop 20, each with a reason. The 19 kept practices fit in 6 proposed decision tickets: landing and gates (candidates 1 and 4, grilling), spec and docs as feature maps (candidates 2 and 3, prototype, with an optional split for proof records), golden-image UI tests (candidate 5, prototype), seams, env and clock (candidates 6 and 7, grilling), gate scope (grilling), and planning and tracking (grilling).

47 checks ran against the clones, the cached GitHub JSON and this repo. 41 confirm a claim, 5 find a claim wrong, and 1 is new. No wrong claim changes a keep or a drop. Two results change the evidence of a kept practice:

- Three tests read the clock apart from the code under test, not one.
- The agent hook traits `AfterNarrationAgent` and `EndOfJobAgent` each have one impl and no test impl, so Chronicler does not fully follow the seam rule.

The closed `ui-verification-redesign` map did not weigh image comparison, and `playwright-rs` 0.9 is already a dependency. Three depth gaps can change the design or the type of a kept practice. None blocks a keep or a drop.

Asset: [candidates.md](../assets/candidates.md). Part 7 lists the corrections for the coordinator.
