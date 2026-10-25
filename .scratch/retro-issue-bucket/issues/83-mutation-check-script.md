# Give the mutation check a script that restores by hash and runs on the final tree

Type: grilling (HITL)
Status: open
Blocked by: —

## Question

Two subagents of the Autoplay review-fix work did their mutation checks by hand: `cp` to a backup, a `sed` edit, a test run, a copy back. Both ways it went wrong:

1. **Stale result.** Implementer `code-review-fixes-apply` (session `4e7d7150`) reported "a no-op `*job = None` in `JobWriter::drop` stays green", and called it a test gap. It ran that mutation before it added its own S1 guard in `GameWriter::accept_for_autoplay`. The lead re-ran the same mutation on the final tree: 12 Autoplay tests went red. The reported gap did not exist.
2. **Clobbered code.** Implementer `s3-config-columns` (session `ccfcc177`) let a scratch edit replace `Tx::set_game_config` with a stub, and rewrote the function from the code it had read earlier. The rewrite was correct (the lead read it), but no check would have caught a wrong one; the gate only proves the tests that exist.

Should the repo give the mutation check one tool?

- **A. `scripts/mutate.py <file> <python-or-sed-expr> <test pattern> [--test-target X]`.** Hashes the file, applies the mutation, asserts the file changed, runs `build.py test-pattern`, restores, asserts the hash matches the original, and prints the tree's `git diff --stat` hash so the report names the tree the check ran on. Reports red/green per test.
- **B. Rule only:** the tdd skill and implementer briefs say "mutation checks run after the last code change, restored with `git diff` empty for the file". No tool.
- **C. `git stash`-based:** mutate in a stash-protected state. Rejected-leaning: a stash in a tree shared with other agents is risky (see the concurrent commit `b717670b` in the same session).

## Context

- Source: lead session of the Autoplay review fixes (2026-10-22), branch `autoplay-engine`.
- Sessions: `~/.pi/agent/sessions/--workspace-chronicler-engine--/2026-10-22T19-22-56-056Z_4e7d7150-25db-4f80-bca8-9646432b30e8.jsonl` (stale result), `~/.pi/agent/sessions/--workspace-chronicler-engine--/2026-10-22T20-37-45-914Z_ccfcc177-8deb-4e58-b80c-8a189a82cc61.jsonl` (clobbered `set_game_config`; it left `tmp/transaction.rs.bak`).
- Related: [23 — unconditional mutation check](23-unconditional-mutation-check.md) decides *when* a mutation check is required; this ticket is about *how* it runs.

## Comments

- 2026-10-25, audit: the user deferred this ticket. Do not implement it yet; it stays open.
