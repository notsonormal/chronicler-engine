# Research whether the docs mislead the agents that read them

Type: research (AFK)
Status: resolved
Blocked by: —

## Question

When an agent reads a Chronicler doc and then edits it in the same session, how often does the edit fix a statement that was already wrong before the session? Did the wrong statement change what the agent did?

## Context

- **Source.** Question 3 of [Research how agents use the current docs and plans](./10-research-agent-use-of-docs-and-plans.md), part 7 of `assets/research-10-agent-use-of-docs.md`. In 230 of 260 (session, doc) pairs, a code-work session read a spec or a Diátaxis doc and later edited it (part 3.4 of that asset). The scripts that found the pairs are in `tmp/hex-compare/r10/`.
- **Covers** C-03 (docs change with the behaviour) and C-11 (a mechanical check on the doc map). See `assets/candidates.md`, part 2. The answer gives evidence for the doc-practice ticket [Recommend a decision for each doc practice](./20-task-recommend-the-doc-practices.md).
- **Measure.**
  1. Take a random sample of at least 40 of the 230 pairs, with a fixed seed. State the seed. If the r10 scripts are gone, rebuild the pair list with the rule in part 1 of the research-10 asset.
  2. For each pair, sort the doc edit into one kind: (a) it records the session's own new behaviour; (b) it fixes a statement that was wrong before the session started (stale); (c) a rename or an index regeneration; (d) wording only; (e) `unknown`. Use the diff of the edit, and `git log` on the doc and on the source files that its `[DOC: …]` anchors name, to date when a statement went stale.
  3. For each (b) edit, record whether the session acted on the wrong statement before it found the error: a wrong edit, a failed build, or a question to the user. Give the evidence by tool-call index, not by quote.
  4. Count how many (b) cases `scripts/validate_docs.py` or `scripts/validate_feature_spec.py` could have caught.
- **Failure modes.**
  - A pair where the session id or the doc path no longer resolves: skip it and count it.
  - An edit that holds two kinds: record the main kind and note the second one.
  - The kind is not clear: `unknown`. Do not guess.
- **Not in scope.** Do not judge the doc layout, and do not propose a new structure. Edit no doc.
- **Rules.** Quote no raw transcript text in the asset. Give counts, file paths, commits and session ids only. Label each claim known or inferred. Use the `/research` skill.
- **Output.** Write `assets/research-19-docs-mislead.md`. The `## Answer` gives the gist, the rate of stale statements with its confidence limit, and what the rate means for C-03 and C-11.
- **When done.** Add `## Answer` and set `Status: resolved`.

## Answer

**Gist.** Docs rarely mislead through a statement that is already wrong. About 8% of the doc edits by sessions that read the doc first fix such a statement. No agent acted on one before it found the error.

- **Sample.** I rebuilt the pair list from `tmp/hex-compare/r10/` (260 pairs, 230 read before the edit, as in research 10). I read 70 pairs in 51 sessions: 45 random pairs (seed 19) and 25 more from the 86 pairs that a screen flags as "no related code edit before the doc edit" (seed 1919). No pair was skipped.
- **Kinds (known for the 70, inferred for the 230).** Records the change of the session or its unit: about 79% (66% own change, 13% planned doc step of a rebuild). Rename or move: about 8%. Wording only: about 6%. Stale before the session: about **7.7%** (8 of 70 pairs read, about 18 of 230). `unknown`: 0.
- **Confidence limit.** 95% limits for the stale share are **3.5% to 20.7%** (conservative, exact limits for each stratum). The normal approximation, with 0 stale pairs in the unflagged stratum, gives 4.1% to 11.2%. The random part alone gives 2 of 45 (4.4%, Wilson 1.2% to 14.8%).
- **Did the wrong statement change what the agent did?** No, in 0 of 8 cases. The agent checked the claim in the code, or a ticket or review had already named it. 7 of 8 were found because the task was an audit, a review item or a doc ticket. 1 was found in passing.
- **Where.** All 8 are in Diátaxis docs (6 Reference, 2 how-to). 0 are in `docs/specs/`, though 33 of the 70 pairs are specs. 2 were born wrong less than 4 days before. 6 are drift, from 22 hours to at most 82 days.
- **Scripts.** `scripts/validate_docs.py` (`LINE_CITATION`, added on the same day as the fix) flags the old text of 2 of 8. `scripts/validate_feature_spec.py` flags 0 of 8. 6 of 8 are wrong names, places or behaviour, and need a fact check against the code.

**Meaning for C-03 (docs change with the behaviour).** The rule is followed by the sessions that act, and it covers specs well. It cannot reach a doc that no change names, and that is where the 8 cases are: prose docs that copy line numbers, helper names and counts. The cost seen is audit work, not wrong action. Recommendation for ticket 20: keep C-03 as covered. If a new rule is wanted, make it narrow: fact copies in prose docs.

**Meaning for C-11 (a mechanical check on the doc map).** A check on ids, links and test tags would not have caught any of the 8. The class that scripts catch today is 2 of 8. A check that also catches the other 6 must compare a statement with the code, and neither repo has one. Recommendation for ticket 20: keep C-11 as covered, and carry the constraint "an adopted map gets a mechanical check" with this limit.

**Limits.** One reader decided the kinds. Only edited docs are in the data, so stale statements that agents read and did not fix are not counted. If the owner counts the planned doc steps as stale, the share is about 21%.

**Asset.** [research-19-docs-mislead.md](../assets/research-19-docs-mislead.md). Scripts: `tmp/hex-compare/r19/` (git-ignored).
