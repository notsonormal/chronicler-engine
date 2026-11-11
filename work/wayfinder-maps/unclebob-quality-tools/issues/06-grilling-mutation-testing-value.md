# Grilling: does mutation testing have value here, with which engine?

Type: grilling (HITL)
Status: open
Blocked by: 03

## Question

Does a mutation run earn its cost in Chronicler Engine, and if it does, with which engine and which mutant kinds?

## Context

Evaluation: [Research: which mutation engine works on this codebase, and what it shows](./03-research-mutation-engines.md), asset [03-mutation-engines.md](../assets/03-mutation-engines.md). Both engines ran on `quantifier/utils/parser.rs` and `generation/gate.rs`, in a linked worktree.

- cargo-mutants 27.1.0: 106 mutants (body replacements, `match` arm deletions, operators). On `gate.rs`, 43 % (37/87) do not compile, because the types have no `Default`. About 11 s per mutant on `parser.rs` and about 24 s on `gate.rs`. All of `gate.rs` took 2,131 s.
- Throwaway driver over `build.py test-pattern` (mutator's operator-only sites): 26 sites, about 9–12.5 s per mutant. It found 24 sites on 911 lines of `gate.rs`.
- 10 surviving mutant instances, all equivalent (the agent's reading, inferred). The one actionable result: the 4 `parser.rs` survivors indicate that `extract_json_from_code_fence` is redundant, because the brace-scan fallback parses every fenced input. The action is to delete the branch, not to add a test.
- No real test gap was found in these two files.

## Decisions wanted

1. Value: is a run that found no test gap in two well-tested files, and found one piece of redundant code, worth its cost? Would other files (lower coverage, ticket 05's CRAP list) give a different answer?
2. Engine: cargo-mutants (richer mutants, needs a slot wrapper, no tree proof, unsafe on `SIGKILL`) or a driver of our own (operator-only, slot-aware, restores itself)?
3. Mutant kinds: operator-only, or body replacements too?
4. Place: on-demand only (for example per file, at review time), or any gate step?

## Out of scope

Shared-tree safety rules: see [Grilling: how does a mutation run keep its result and its tree in step?](./07-grilling-mutation-shared-tree-safety.md). The build of the tool.
