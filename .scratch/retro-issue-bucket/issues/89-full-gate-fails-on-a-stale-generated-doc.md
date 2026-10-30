# The full gate fails on a stale generated doc that it could regenerate

Type: grilling (HITL)
Status: open
Blocked by: —

## Question

The full gate fails when a generated doc is stale, for example `docs/diataxis/reference/coding_standards/guardrails.md` after a doc comment in a guardrail test changes. The pre-commit hook regenerates the same files, but only at commit time. The full gate already rewrites the source format with `fmt`, so it rewrites files in place on other steps.

Should the full gate regenerate the generated indexes and docs instead of failing on them?

## Context

- Source: `/retro` 2026-10-29, lead session `01a1743c-7303-77ca-953f-4e72116e9d9f` (the action-answers Final gate and its fix pass, branch `wf/change-driven-dashboard-refresh`).
- The comment pass of the Final gate changed one doc comment in a guardrail test. The next full gate failed at "Checking guardrails.md freshness..." (`logs/build_20261029_020413.log`). The lead ran `python scripts/generate_guardrails_doc.py` by hand and ran the full gate again (about 145 s).
- `build.py:792-796`: the `guardrails-doc-check` step runs `generate_guardrails_doc.py --check`; `http-routes-check` (`build.py:786`) works the same way. `scripts/precommit_regenerate.py:35` regenerates the guardrails doc in the hook.
- **Possible shape (a proposal).** The bare full gate runs the generators (as `fmt` rewrites); `--check` stays for the other tiers and for `--no-fmt`. Open: a full gate that edits tracked files may surprise a reviewer of the tree; `fmt` already sets that precedent.

## Further evidence (2026-10-30, design-review fix pass)

- Builder session `d0ca6bc4-7364-47ab-98b4-b354d422cbf8`, review fixes: "The gate first stopped at `guardrails.md freshness` because the L6 doc comment changed the generated doc. I regenerated it … and reran." (`tmp/dd-fixpass/report.md:106`). One full gate lost.
- Comment-pass session `db6713bb-c2f4-41df-855a-1919bd74f177`: deleting guardrail doc comments shifted the Source line numbers in `guardrails.md`, so the fixer regenerated it by hand before `guardrails` and `validate-docs` passed.
- The commit hook regenerated `guardrails.md` and both structure indexes again at commit `4a074808`.
