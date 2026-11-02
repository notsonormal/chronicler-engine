# Research the Hex issue pipeline from its public PRs and issues

Type: research (AFK)
Status: resolved
Blocked by: —

## Question

How does the Hex issue pipeline take an issue to a merged change? What do its gates and its reviewer catch, and how often does a pipeline change fail or get reverted? Which parts does Chronicler have?

## Context

- **Facts so far** ([findings asset](../assets/findings.md), "What is public and what is private"):
  - Each pipeline PR body describes its run. For example, [PR #714](https://github.com/anomalyco/hex/pull/714) says that the fixer and the reviewer saw only the validated issue form, never the issue text. Its gates were a build in the "apollo sandbox", the policy verdict and a reviewer model's approval. It also gives a run id.
  - 59 commits carry the trailer `Co-authored-by: Hex issue pipeline`, dated from 2026-10-29 to 2026-10-31. 8 commits come from `opencode-agent[bot]`.
  - On 2026-11-01 the repo had 497 PRs and 219 issues, and 17 PRs had the label `agent-strict-review`.
- **Hex sources.**
  - `.github/ISSUE_TEMPLATE/`: `bug_report.yml`, `feature_request.yml`, `config.yml`
  - `.github/agent-policy/check.ts` and `.github/workflows/agent-policy.yml`
  - PR bodies, review comments and labels
  - closed pipeline PRs that were not merged
  - reverts: 5 commits start with "Revert"
- **What to find.**
  - how an issue form is validated
  - what the strict reviewer flags
  - what a failed run looks like
  - how a human steps in
  - how fast an issue goes from opened to merged
- **API limits** (see the map's Notes). `GET /repos/anomalyco/hex/pulls?state=all&per_page=100` returns PR bodies in 5 pages. Comments and reviews cost one request for each PR, so sample them: the 17 strict-review PRs and about 10 closed PRs that were not merged. Keep the raw JSON in `tmp/hex-compare/github/`.
- **Chronicler side.**
  - `docs/agents/issue-tracker.md` and `docs/agents/triage-labels.md`
  - the `.scratch/` tickets
  - the review skills: `code-review` and the `chronicler-after-plan-workflow*` skills
  - `.pi/extensions/pi-permission-system/config.json`
- **Overlap.** The ticket "Research how a Hex change goes from branch to installed app" covers where the agent policy sits in the path. This ticket covers how the pipeline uses the policy.
- **Candidates this touches:** 1 (risk gate) and 4 (small units that land automatically).

## Answer

The pipeline runs on the issue, not on a branch. One issue makes one run with an id, one PR on branch `agent/issue-N`, and one squash commit on `dev`. 56 of the 64 agent PRs were issue fixes; 8 were `agent/ci-` branch repairs of a failed platform check. 60 of 64 merged, with a median of 3 seconds from PR to merge and 6.7 minutes from issue to merge.

The only gates are a deterministic diff sort, a second model's approval (a label), and an "apollo sandbox" build. No person holds the merge. The land queue verifies the library tests and an app build on one host, so a platform check cannot hold it: 8 of the 59 pipeline commits are `fix(ci)` repairs that a later run wrote. No pipeline commit was reverted.

Chronicler has none of the loop: no `.github/`, no PR, no risk sort of a diff, no automatic land. It does have a ticket lifecycle, five triage roles and a two-axis review skill, and all of those act inside a session.

12 practices are recorded, 6 of them new. Candidates 1 and 4 both grow, and neither shrinks.

Asset: [../assets/research-02-issue-pipeline.md](../assets/research-02-issue-pipeline.md)
