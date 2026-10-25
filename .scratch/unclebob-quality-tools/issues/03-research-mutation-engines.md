# Research: which mutation engine works on this codebase, and what it shows

Type: research (AFK)
Status: open
Blocked by:

## Question

Which mutation engine can run on Chronicler Engine inside the build slot and the shared tree, at what cost, and what do its surviving mutants show?

## Context

- **mutator** (see the [research asset](../assets/research-crapper-mutator-dryer.md), section *mutator*): operator-only sites (`&&`/`||`, `!`, comparisons, `+`/`-`, `*`/`/`, `true`/`false`, `0`/`1`); skips lines coverage does not hit; per-function text hash for differential reruns; timeout = 10 × baseline. `--scan` found 2 sites in `quantifier/utils/parser.rs` and 21 in `generation/gate.rs`. As is, it is unusable here: each parallel worker overlay has no `target/` (cold build per worker), and the default command is bare `cargo test` (runs the browser tier, bypasses the build slot).
- **cargo-mutants**: v27.1.0 was piloted in `.scratch/_closed/wayfinder-uncle-bob/issues/10-research-crap-mutation-testing.md`. It made 20 mutants on the parser (function bodies replaced with defaults, `match` arms deleted, operators), and a copied-tree run with `--jobs 2` did not finish in 5 min. Not tried: `--in-place` (reuses the warm `target/`), `--in-diff`, `--iterate`, `--test-tool nextest`, a test filter. Not installed now.
- **A third option:** a driver of our own that applies one mutant in place, runs `python build.py test-pattern "<pattern>" --test-target <bin>` (about 6 s after a one-line edit, through the build slot), and restores.
- **Safety requirements** (from retro-issue-bucket ticket 83, deleted 2026-10-25 when it moved into this map). Two hand-run mutation checks went wrong during the Autoplay review fixes (2026-10-22, branch `autoplay-engine`):
  1. *Stale result.* Session `4e7d7150` reported that a no-op `*job = None` in `JobWriter::drop` stays green. It ran before its own guard in `GameWriter::accept_for_autoplay` landed; on the final tree the same mutant turned 12 Autoplay tests red.
  2. *Clobbered code.* Session `ccfcc177` let a scratch edit replace `Tx::set_game_config` with a stub, then rewrote the function from memory (it left `tmp/transaction.rs.bak`). The rewrite was right, but nothing checked it.
  
  Ticket 83's options were: A, `scripts/mutate.py` that hashes the file, applies the mutant, asserts the change, runs `build.py test-pattern`, restores, asserts the hash, and prints the tree's `git diff --stat` hash; B, a rule only; C, `git stash` (rejected-leaning in a shared tree).
  Sessions: `~/.pi/agent/sessions/--workspace-chronicler-engine--/2026-10-22T19-22-56-056Z_4e7d7150-25db-4f80-bca8-9646432b30e8.jsonl`, `…/2026-10-22T20-37-45-914Z_ccfcc177-8deb-4e58-b80c-8a189a82cc61.jsonl`.

## Work

1. Install `cargo-mutants` (latest). On `src/application/agents/quantifier/utils/parser.rs` and `src/application/generation/gate.rs`: list mutants, then run with `--in-place` and a nextest filter for the matching tests, through the build slot. Record wall time, mutants caught / missed / unviable / timeout, and what `--in-place` leaves behind after an interrupt.
2. Build a throwaway in-place driver in `tmp/` (not `scripts/`) over `build.py test-pattern`, with hash check and restore. Run it on the same two files with mutator's site set. Record the same numbers.
3. Check each engine against the safety requirements above: restore after a crash, proof of the tree it ran on, no `git stash`, no collision with another agent's build.
4. For every surviving mutant, read it and annotate: a real test gap, an equivalent mutant, or noise.

## Expected output

- An asset under `assets/` with both engines side by side (cost, mutant kinds, survivors annotated, safety check), written so the user can judge value without reading the tools' source.
- On resolution, graduate grilling tickets: "Does mutation testing have value here, with which engine?" and the shared-tree safety question (see *Not yet specified* on the map).
