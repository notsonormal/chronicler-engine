# The strong-implementer re-ran narrow builds and the gate many times in one task

Type: grilling (HITL)
Status: open
Blocked by: —

## Question

[03](03-run-the-full-gate-once.md) is resolved ("run the full gate once per change set"), but one `strong-implementer` run still made 39 `build.py` invocations in 47 minutes (some batched two per bash call): `clippy` ×15, `test-pattern autoplay` (with or without `--test-target http`) ×7, `guardrails` ×3, `integration` ×2, `browser` ×1, `validate-docs` ×2, the full gate ×2, plus a `--coverage` run on its own target dir. The run peaked at 375k context; the build logs it read back are part of that.

What steering makes the loop cheaper?

- **A. Brief template:** the lead's implementer brief names the loop: one `--test-target` pattern while editing, `clippy` once per edit batch, `--no-browser` once before the report, no full gate (the lead runs it).
- **B. `build.py` refuses a repeat:** a second identical subcommand on an unchanged tree prints the earlier log path and exits (the history file already records the tree hash: "same as …, tree b717670b5d+…").
- **C. Reopen 03** with this evidence and widen it from the full gate to every subcommand.

## Context

- Proof: `~/.pi/agent/sessions/--workspace-chronicler-engine--/2026-10-22T18-09-34-124Z_ef3a228d-e9e8-4b2e-9146-d78e30887fdc.jsonl` (delegate `autoplay-review-fixes`, 132 turns, 2026-10-22 18:09–18:57). Extract the calls with: `jq -r 'select(.message.role=="assistant") | .message.content[]? | select(.type=="toolCall" and .name=="bash") | .arguments.command' <file> | grep -o 'build\.py[^;&|]*'`.
- Timeline highlights: `clippy` at 18:17:19, 18:17:37, 18:19:20, 18:21:40 (four runs in four minutes); `test-pattern autoplay --test-target http` at 18:21:50 and 18:22:29; full gate at 18:33:18 and 18:52:52, with a coverage run in between.
- Related: [03](03-run-the-full-gate-once.md), [70 — build wait helper](70-build-wait-helper.md), [75 — lead delegate brief quality](75-lead-delegate-brief-quality.md).
