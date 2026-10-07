# VARIANT 1 — AGENTS.md prune + co-locate (moderate)

Worktree: `/workspace/chronicler-engine/tmp/agents-restructure/v1/wt` (detached HEAD `5fa46615`).
Edited files (left in the worktree, not staged, not committed): `AGENTS.md`, `CODING_STANDARDS.md`.
No files created inside the worktree. Temp/helper material lives under `tmp/agents-restructure/v1/`.

Variant 1 scope honoured: no new files, generated Structure block untouched and unmoved, heading set
byte-identical, no `src/` or `tests/` edits, no cargo, no git write commands.

---

## 1. What moved where, and why

| Moved item | From | To | Lever / reason |
|---|---|---|---|
| Temporary-file rule ("Temporary files should be written into tmp folders e.g. `tmp`.") | `AGENTS.md § Development Loop` | `CODING_STANDARDS.md § Editing` (new, beside `## Implementation`) | Co-location: it is an editing rule, and the editing rules now have one home. Progressive disclosure behind the single existing `CODING_STANDARDS.md` pointer. |
| Per-edit discipline ("Re-read the exact target region immediately before every file edit … list the directory first.") | `AGENTS.md § Development Loop` | `CODING_STANDARDS.md § Editing` (same section, three short STE sentences) | Same section, same reason. "Development Loop" now contains only the build loop it names. |
| `logs/build_*.log` fact + pi-session stamp | `AGENTS.md § Commands → #### Iteration` | `AGENTS.md § Development Loop` | Duplication: "Development Loop" already held the log-reading step; one home per meaning. The rationale clause ("so the `mrn-context` extension attributes the log …") was dropped — it explains, it does not instruct. |
| Warm/cold build timing ("about 2 minutes … a cold one takes far longer") | `AGENTS.md § Development Loop` (standalone sentence) | `AGENTS.md § Commands → #### Final Validation` (gate command comment: `~2 min warm, far longer cold`) | Duplication: the same figure already sat in the gate command comment, and `ENVIRONMENT.md` carries the measured table. The cold-dir fact also survives in `§ Concurrent Builds` ("a new dir starts cold"). Relocated, not lost. |
| `ENVIRONMENT.md` pointer | its own paragraph in `§ Concurrent Builds` | merged into the first paragraph of `§ Concurrent Builds` | Co-location of the build-cost concept; the trigger wording ("slow, waits for the build slot, or runs out of memory") is preserved. |
| sccache paragraph | its own paragraph in `§ Concurrent Builds` | merged into the same `§ Concurrent Builds` paragraph | Co-location of "build cost / target dir" facts; content unchanged apart from joining. |
| `docs/AGENTS.md` / `tests/AGENTS.md` catalogue paragraph | its own paragraph in `§ Documentation Index` | merged with the generated-index paragraph in the same section | Both paragraphs describe the generated catalogues (what they are, when to read them, never hand-edit them); reading one should bring the other. |
| `python build.py <step>` code block (11 commands + descriptions) | `AGENTS.md § Commands → #### Iteration` | replaced by one prose line + `python build.py --help` pointer | Cache-vs-environment: every line restated `--help`. See §6 below for the per-command justification. |
| CODING_STANDARDS pointer wording | `AGENTS.md § Documentation Index` | same line, now "implementation, **editing**, code-comment, code-review, and testing rules" | Keeps exactly one pointer line (grep count `CODING_STANDARDS.md` in AGENTS.md = 1) while covering the moved editing rules. |

Nothing was moved into a new file; `CODING_STANDARDS.md` already existed and already had exactly one
pointer from `AGENTS.md`.

---

## 2. Deleted no-ops, and the default behaviour each restated

| Deleted sentence (source) | Default behaviour it restated |
|---|---|
| "If you don't know something, say 'I don't know' instead of inventing an answer." (`§ Communication`) | Instruction-tuned models are already trained against fabrication and do emit uncertainty. It also duplicated the meaning of the retained "Label epistemic status when it matters: known, inferred, or guessed", which is the *stronger* instruction (it names the vocabulary and the trigger). Two homes, one meaning → kept the better one. |
| "These guidelines bias toward caution over speed. For trivial tasks, use judgment." (`§ Decision Making`) | Default effort calibration: a competent agent already scales deliberation to task size. It states a policy for the whole list without changing any action, and mildly contradicts the imperatives above it. |
| "Avoid **analysis paralysis**: when reasoning stops converging, act instead — read, run, or write a test, check the UI directly in the browser, or add logging and diagnostics to the production code." (`§ The Test-First Philosophy`) | The loop already terminates reasoning and acts; "act instead" restates the default. The parenthetical list (read/run/test/add logging) is generic agent behaviour, not a project rule. |
| "A comprehensive suite of unit and integration tests is the ultimate source of truth for behavior." (`§ The Test-First Philosophy`) | A value statement, not an action. The two behaviour-changing rules that follow (read `tests/` before source; create a failing test case) are retained and are what actually move a run. |
| "**Don't assume. Don't hide confusion. Surface tradeoffs.**" (`§ Decision Making`) | Restated its own five bullets (duplication — one home per meaning), and in negation form ("Don't assume") it is weaker than the positive bullets ("State your assumptions explicitly", "say so before acting", "present them"). Deleted the summary, kept the bullets. |
| "Do not bury it." (`§ Decision Making`, last bullet) | Restated "name the trade-off in the response" in the same sentence: same meaning twice, and the negation adds no behaviour. |
| "This is a preference, …" (`§ Subagents`) | Restated "Prefer one long-running subagent over several short ones"; the branch list that follows it was kept. |
| "These restrictions exist to prevent the agent from touching git without supervision." (`§ Permissions System`) | Rationale for the adjacent commit guardrail, not an instruction; the guardrails themselves are retained. |
| "Almost every full-gate step is also a subcommand." (`§ Commands`) | Duplicated the exception list in the next clause ("Packaging, the test suite, and coverage stay gate-internal; every other full-gate step is also a subcommand"). Kept the one statement that carries the same meaning plus the exception. |
| 11 trailing `#` comments in the command block (`# Format sources (rewrites files in place)`, `# Run the unit tests`, …) | Each one restated a line of `python build.py --help` verbatim (environment cache). |
| "…, stamped with the pi session id so the `mrn-context` extension attributes the log to the session that ran it." (clause) | Explanation of a mechanism; the observable fact (stamp + path) was retained. |

Also removed without losing meaning: the trailing two-space hard break after the log-tail fence, and the
stale example count `nextest: 1482 passed, 0 failed, 2 skipped`, replaced by the pattern
`nextest: N passed, N failed, N skipped` inline (compression, not deletion).

---

## 3. Rules that look like no-ops but are not (kept, with reason)

| Rule | Why it is not a no-op |
|---|---|
| "When you answer user feedback or an analysis, say whether you agree or disagree before you say what you changed." | Tested against the `### Decision Making` heading as instructed: the bullets cover assumptions, contradictions, interpretations, positions and trade-offs — none covers *responding to feedback*. The default is to silently implement the feedback; this forces a stated verdict first. Kept. |
| "Label epistemic status when it matters: known, inferred, or guessed." | Non-default output shape. Models produce unlabelled confident prose by default. |
| "Never delete or revert an unknown or unexpected file, especially an untracked one …" | Guardrail demanded inline by the task. Default agents tidy "stray" files out of the way; this repository has concurrent agents. |
| "During code reviews, make no code changes. Report the problems instead." | Default is to fix what the review finds. Guardrail demanded inline. |
| "Never commit without explicit approval, even when the config allows commits." | Non-default: the permission config *does* allow commits; the rule overrides it. Guardrail demanded inline. |
| "Never circumvent them, and never edit that config without explicit user approval …" | Guardrail demanded inline; also states the config file is read-only by policy, which the default does not know. |
| "Never hand-edit a generated block." | Default is to edit the file in front of you; the pre-commit hook would silently overwrite it. |
| "Do not silently work around the mismatch or proceed as if the instruction were accurate." | Default is to satisfy the literal instruction; this forbids the silent workaround. |
| "Run one targeted step instead of the full suite, or run the gate directly." | Looks like generic efficiency advice, but the default failure mode is over-verification: run everything, then run the gate. It redirects an actual habit. |
| "Only create `scout` subagents when the current model is Anthropic …" | Non-default, self-referential (the agent must know its own model family), and names the reason (`scout` cost/behaviour). |
| "Prefer one long-running subagent over several short ones …" | Default delegation style is fan-out; this inverts it with the cost reason attached. |
| "Use the Pi bash tool with a timeout of 1200 seconds when you call `build.py`." | Default timeout kills the build; the number is the behaviour. |
| "In a git worktree, run `python build.py` with no extra flags." | Looks like a restatement of the target-dir rule but is the worktree branch — the isolated default target dir needs no flag. |
| "`--target-dir` works on either side of the subcommand; all other top-level flags are full-gate only." | CLI trivia absent from `--help`'s text (the usage line does not say "either side"). |
| "Packaging, the test suite, and coverage stay gate-internal …" | Prevents an agent hunting for a subcommand that does not exist. |
| The whole `### Progress updates` paragraph | Communication protocol the model does not default to (one sentence before the first call; update on material change only; outcome first). |

---

## 4. Line counts (prose below the Structure block)

Measured with `AGENTS.md` split on `\n`, counting every element after the 308-line
`AUTO-STRUCTURE` head.

| | Lines |
|---|---|
| Before (`HEAD`) | 431 total; **123** prose below the block |
| After | 396 total; **88** prose below the block |
| Change | **−35 lines, −28.5 %** (target: ≥ 25 %) |

Integrity checks:

- Structure block byte-identical before and after regeneration: `True` (verified by comparing the first 308 newline-joined lines of `HEAD:AGENTS.md` against the edited file, before *and* after running the generator).
- Heading set byte-identical: `diff <(git show HEAD:AGENTS.md | grep "^#") <(grep "^#" AGENTS.md)` → `HEADINGS IDENTICAL` (same headings, same levels, same order; no heading added, removed or renamed).
- Guardrails required inline are all present: lines 312 (repo health + never delete unknown/untracked), 314 (no code changes during review), 388 (permission system + config), 390 (no commit without approval).
- No residual copy of the moved editing rules in `AGENTS.md`: `grep -n "Temporary files\|tmp folders\|Re-read the exact\|glob or wildcard" AGENTS.md` → no match (exit 1).
- `grep -c "CODING_STANDARDS.md" AGENTS.md` → `1`.
- File keeps its no-trailing-newline ending (unchanged from `HEAD`).

---

## 5. Exact validation output

Run from inside `/workspace/chronicler-engine/tmp/agents-restructure/v1/wt`.

```
$ git diff --stat
 AGENTS.md           | 77 +++++++++++++++--------------------------------------
 CODING_STANDARDS.md |  8 ++++++
 2 files changed, 29 insertions(+), 56 deletions(-)
```

```
$ python scripts/generate_structure_index.py
  Updated STRUCTURE section in AGENTS.md
  Structure index regenerated successfully
exit=0
```

```
$ python scripts/validate_docs.py

============================================================
Scanned 25 file(s)
Summary: 0 error(s), 0 warning(s) across 0 file(s)

============================================================
Scanned 420 file(s)
Summary: 0 error(s), 0 warning(s) across 0 file(s)

PASS
exit=0
```

`git status --short` → ` M AGENTS.md`, ` M CODING_STANDARDS.md` only (no untracked files in the worktree).

---

## 6. Requirement D — the command list, drop by drop

The whole block below `#### Iteration` was a cache of `python build.py --help`. Each dropped item is
justified against the `--help` text (quoted from the run in this session):

| Dropped from AGENTS.md | `python build.py --help` already carries it |
|---|---|
| `python build.py fmt # Format sources (rewrites files in place)` | `fmt  Format sources in place (same as the full gate's fmt step).` |
| `python build.py check # Fast compile check across all targets (no lint)` | `check  Fast compile check across all targets; no lint (lighter than clippy).` — command name dropped entirely (see note) |
| `python build.py clippy # ~10s — fix warnings here` | `clippy  Lint Rust sources; warnings are errors.` (the name is kept, because *choosing* it first is a decision the agent makes) |
| `python build.py unit # Run the unit tests` | `unit  Run the Rust unit tests (lib target only).` |
| `python build.py architecture # Run the architecture tests` | `architecture  Run the architecture integration tests.` |
| `python build.py guardrails # Run the guardrails tests` | `guardrails  Run the guardrail integration tests.` |
| `python build.py test-pattern "action_pipeline::options_tests"` | `test-pattern  Run the tests whose name matches a pattern, across all test binaries.` The literal example was dropped; the invocation form was kept as `test-pattern "<substring>"`. |
| `python build.py integration # … (~20s)` | `integration  Run every test binary except browser, architecture and guardrails (~20s warm).` — even the duration was in `--help`. |
| `python build.py browser # Only the browser/Playwright binary (~1 min)` | `browser  Run only the browser (Playwright) test binary (~1 min warm).` |
| `python build.py validate-docs # Validate markdown docs` | `validate-docs  Validate markdown docs and DOC anchors under docs/.` |
| `python build.py run # Run the dev server (see ENVIRONMENT.md)` | `run  Build the dev server and replace this process with the binary.` |
| `#### Iteration` heading intro ("All build actions go through `build.py`. Every run writes `logs/build_*.log`, stamped with the pi session id …") | Moved to `§ Development Loop` (the log fact) and restated once as "`build.py` is the only build entry point"; the mrn-context rationale was deleted. |

Names dropped from AGENTS.md entirely, each justified by `--help`:

- **`check`** — `--help` gives its cost/benefit ("lighter than clippy"); an agent that wants a compile-only pass reads it there. Keeping the name in always-loaded context duplicated `clippy` with a weaker action.
- **`fmt`** — the full gate runs fmt (`--help`: "same as the full gate's fmt step"), and its caveat "rewrites files in place" is stated in its own `--help` line. In the shared-checkout case the guardrail that matters (`--no-fmt`) is preserved in `§ Concurrent Builds`.

Names kept (clippy, unit, architecture, guardrails, integration, browser, test-pattern, validate-docs,
run): the *choice* of tier is behaviour, not lookup — `--help` lists the steps but does not tell the
agent which ones to run after a change.

Not dropped: `python build.py` (the gate) — it is the step in `#### Final Validation`, not a lookup.

---

## 7. Where this variant is still too conservative

- **The dominant context load is untouched.** The 300-line generated Structure block is ~78 % of the
  file. Constraint 1 forbids moving it; the real win for a Variant 2/3 is disclosing it behind a
  pointer (`scripts/generate_structure_index.py --help` / a generated file) instead of inlining it.
- **Heading set frozen.** `§ Agent Skills` holds three near-identical one-line pointers
  (`### Issue tracker`, `### Triage labels`, `### Domain docs`) that want to be one block or one table;
  constraint "do not change AGENTS.md's set of headings" forces them apart.
- **Two paragraphs in `§ Permissions System`** could be one (saves a line); they were kept split to
  keep the required guardrails visually separate.
- **`§ Your Responsibility`** still spends two sentences on repo health where one would do; the second
  line exists because the task forces the code-review rule to stay inline rather than joining
  `CODING_STANDARDS.md § Code Reviews`.
- **Command names kept.** A harsher variant deletes the prosaic `#### Iteration` paragraph entirely and
  leaves only the `--help` pointer.
- **No verification of the "no-op" calls.** The skill says the no-op test is model-relative and settled
  by running the document; this variant's deletions (and its *keeps*) are argued, not measured.
- **STE lite.** Short sentences were applied to rewritten text only; the untouched bullets keep their
  original syntax. Note the metric fights STE: splitting a long line into three short ones *increases*
  the measured line count, so the −28.5 % was achieved by removing meaning-duplicates, not by
  re-wrapping.
- **`docs/AGENTS.md` / `tests/AGENTS.md` pointer sentence kept** — a Variant 3 could trust discovery of
  the catalogue files.
- **Not validated by the project's own gates**: per instruction, no `build.py` run; `validate-docs` and
  the structure generator were used instead. `generate_guardrails_doc.py` does not read
  `CODING_STANDARDS.md`, so the new `## Editing` section cannot make the guardrails doc stale — but
  `python build.py guardrails-doc-check` was not run (cargo-free, but outside the stated validation
  list).
