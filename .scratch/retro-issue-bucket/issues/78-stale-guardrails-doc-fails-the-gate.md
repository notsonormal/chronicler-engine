# A stale `guardrails.md` fails `build.py` before the hook can regenerate it

Type: grilling (HITL)
Status: open
Blocked by: —

## Question

`docs/diataxis/reference/coding_standards/guardrails.md` is generated, and the pre-commit hook regenerates it. But `build.py` checks it before any commit, so a slice that edits a guardrail fails its first gate run on a stale doc and needs a manual `python scripts/generate_guardrails_doc.py`. What should change?

- **`build.py` regenerates** the generated docs before it validates them (as `fmt` rewrites files).
- **The failure message names the fix**: the check prints the generator command.
- **Leave it.**

## Context

- Source: lead session `01a15b99` (2026-10-21).
- R7 (implementer `2b97dd7c`): "The first gate run failed on a stale `guardrails.md`. I regenerated it … and the rerun passed." One extra `--no-browser` run (~50 s plus the compile).
- The lead added the generator command to the R8 and R9 briefs by hand.
