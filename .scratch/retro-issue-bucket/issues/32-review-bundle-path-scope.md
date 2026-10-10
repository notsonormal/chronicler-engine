# Let the review bundle take a path list

Type: grilling (HITL)
Status: open
Blocked by: —

## Question

`prepare_review_bundle.py` has two modes, `--uncommitted` and `--ref`, and no path filter. In a shared checkout, `--uncommitted` takes in every other effort's in-flight work. The 2026-10-10 ticket-08 session had 14 unrelated modified files in the tree (`build.py`, `AGENTS.md`, `scripts/tests/*`, another effort's `.scratch` ticket). The lead built the bundle anyway, then wrote a separate `ticket08.patch` from a hand-kept `tmp/ticket08/scope.txt` and told both review axes to "IGNORE the other patches". The `/simple-after-plan` scope step and the comment pass (`comment_finder.py --files`) needed the same list again.

How should a change set be scoped in a shared tree?

- **Add `--paths` (or `--paths-from FILE`) to the bundle script.** The bundle holds only the listed paths, and its README records the list.
- **Make the scope file a convention.** `/simple-after-plan` step 1 writes `tmp/<effort>/scope.txt`, and every later step (comment pass, bundle, commit staging) reads it.
- **Both.**

## Context

- Source: `/retro` 2026-10-10, ticket-08 lead session (`/simple-after-plan` run). The reviewer sessions are `01a1267a-dd67` and `01a1267a-ebe3`.
- `scripts/prepare_review_bundle.py --help`: the only options are `--uncommitted`, `--ref` and `--out`.
- Related: [Never tell a review axis to skip `tooling.patch`](21-review-bundle-tooling-patch.md). A skip instruction was needed here as well. [Stop `commit-and-push` from staging the whole shared tree](13-commit-staging-shared-tree.md) needs the same list at commit time.
