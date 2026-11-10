# Wiki survey for the chronicler-engine development-environment audit

Source: `/workspace/mrn-wiki` (owner's LLM wiki, 355 content pages at 2026-11-09). Read-only survey.
Repo facts in the "check" column were confirmed by `ls`/`wc` on `/workspace/chronicler-engine` at
audit time: **40 skill dirs, 40 `SKILL.md`, no `.agents/skills/skills-lock.json`, `AGENTS.md` 418
lines, `CONTEXT.md` 199, `CODING_STANDARDS.md` 35, 9 `.pi/agents/*.md`, 20 files in
`tests/infrastructure/guardrails/`, 10 open `.scratch/` efforts plus `_closed/`**. No code was run.

Labels: **known** = the wiki states it with a source and a number; **inferred** = the wiki's evidence
implies the check but does not apply it to this repo; **guessed** = my judgement, unsupported.

---

## 1. Testable claims (strongest evidence first)

| # | Wiki anchor | Claim (quoted) | Number / study behind it | Concrete check for chronicler-engine | Label |
|---|---|---|---|---|---|
| 1 | `concepts/harness-self-evolution.md:55` | "**Only 9 of the 38 executable checks could fail in the environment they were written for.**" | 30 SWE-bench Verified sessions, Claude Code 2.1.295 + Opus 5.5, `/retro` at `b0618bc`; 125 proposals | Take each guardrail walker in `tests/infrastructure/guardrails/` and each `build.py` step. Introduce the pattern it forbids in a scratch branch and confirm the step fails. Report the count that cannot fail | known |
| 2 | `concepts/harness-self-evolution.md:97` | "**An incomplete guardrail adds the work it was meant to prevent.**" | Sphinx known-failures file: control issue 0/5 → 4/5 sessions ran baseline comparisons | Look for partial allowlists or known-failure notes (`docs/`, `AGENTS.md`, `.scratch/`). Check whether sessions then re-derive baselines the note was meant to remove | known |
| 3 | `concepts/harness-self-evolution.md:88` | "Loaded is not consulted, and consulted is not changed." | 13 transfer events, 5-vs-5 fresh containers; the author-written `pre-commit` file was "read by none, and `pre-commit` never run" | Instrument consult: does any session transcript or `logs/` show an agent opening `AGENTS.md`/a skill, and does behaviour differ? Today nothing measures the middle variable | known |
| 4 | `concepts/harness-self-evolution.md:103` | Bundled advice "obeyed every time and median elapsed time rose **88 s → 156 s**"; Sphinx control "**82 s → 245 s**" | Same study, 12 adaptations | Use `logs/build_history.txt` medians per subcommand before/after each rule addition. Attribute gate cost to rules, not only to code | known |
| 5 | `concepts/harness-self-evolution.md:20` | "**proposals are easy, transfer is conditional.**" | 2 of 7 informative events eliminated the behaviour; 4 unchanged; task success "never moved by more than one session" | For each habit in `AGENTS.md` (epistemic labels, "disagree before changing", no fixes during review), find the failure it prevents and whether that failure recurred after it was added | known |
| 6 | `comparisons/harness-self-evolution-research-2026-11.md:22` | "Random rules match curated (**+13.8pp** both …); gains are largely **content-independent**; beneficial rules are negative constraints, harmful ones positive directives" | Zhang et al. 2604.11088: 679 files / 25,532 rules, >5,000 Claude Code runs | Classify all 418 lines of `AGENTS.md` as negative constraint / positive directive / fact. Count the positive directives and name any with a measured effect | known |
| 7 | `comparisons/harness-self-evolution-research-2026-11.md:23` | AHE gain "localises to **tools, middleware, memory — not the system prompt**"; prose strategy "does not" transfer | 10 iterations, Terminal-Bench 2 pass@1 69.7% → 77.0%, past Codex-CLI 71.9% | Measure the harness by carrier: N prose rules vs N tests/walkers vs N build steps. Predict that new prose is the weakest lever here | known |
| 8 | `comparisons/harness-self-evolution-research-2026-11.md:25` | "**benefit is non-monotonic**: weak least, mid most, strong less than mid" | Model-tier sweep, 2605.30621 | chronicler runs a GLM planner + Qwen children (`entities/chronicler-engine.md:11`). Ask whether harness gain was ever measured per tier, or assumed | inferred |
| 9 | `comparisons/harness-self-evolution-research-2026-11.md:26` | "Both test models **below 30%** end-to-end on reflection; diagnosis is the bottleneck" | BenchTrace, 1,821 annotated episodes | If `/retro`-style or post-session reflection is used, score its diagnoses against a human list for 10 sessions before trusting it | known |
| 10 | `concepts/agent-trust-ladder.md:14` | "**prefer the most enforceable mechanism first, and only fall back to the weaker ones**" | Lauren Tan talk, 2026-09-21, 2,500-PR account; 4 rungs | Map each `AGENTS.md` rule to rung 1–4. Count rules sitting only at rung 3/4 that a walker or type could hold instead | known |
| 11 | `concepts/agent-trust-ladder.md:54` | "**When you see a bad pattern, write a lint rule against it** — not necessarily a cleanup." | Same talk, "gardener" role | Take the last five review findings or repeated agent corrections and check whether each became a walker, a spec tag, or only prose | inferred |
| 12 | `concepts/agent-trust-ladder.md:67` | "**Comment ban** in Dune: agents treated surrounding comments as licence to band-aid the problem" | Worked example, no measurement | chronicler already restricts comments to three carve-outs (`entities/chronicler-engine.md:126`). Check the guardrail enforces the carve-out *set*, not comment density | known |
| 13 | `concepts/rule-retention-mechanisms.md:27` | "Divergence is the failure no test suite can catch, because the missing test *is* the missing rule." | Nine researched axes, 233 sources | Pick 5 scenarios from `docs/specs/` and 5 rules from `AGENTS.md`. Ask what would fail if the rule were deleted. Separate divergence from attrition | known |
| 14 | `concepts/rule-retention-mechanisms.md:47` | "**50.8%** of 7,357 static-analysis suppressions in 46 Python projects suppress nothing and are 'practically useless'" | FSE 2025 | Count `#[allow]`/clippy suppressions and `arch-lint.toml` exclusions in `src/`. Sample them for a reason that still holds | known |
| 15 | `concepts/rule-retention-mechanisms.md:49` | "**About 63%** of 5,800 sampled ADRs were opened directly as accepted" | Same literature review | chronicler deleted its 33 ADRs on 2026-08-10 (`entities/chronicler-engine.md:51`). Check the replacement store (`docs/diataxis/explanation/`, `.scratch/`) has an inbound pointer from code. The wiki says zero do (`:115`) | known |
| 16 | `concepts/claude-code-skills-lessons.md:15` | "a skill is a **folder, not a markdown file**; the best skills fit **one** category; the highest-signal content is a **gotchas** section" | "hundreds" of internal skills, no numbers (**`:20`**: "no numbers behind") | Audit all 40 `SKILL.md`: count those with a gotchas section, those straddling categories, those with no executable step | known |
| 17 | `concepts/claude-code-skills-lessons.md:57` | "**Write the description for the model.** … The description says **when to trigger**, not what the skill is" | Same post; usage logged by a PreToolUse hook to find **undertriggering** (`:77`) | Compare each of the 40 `description:` lines against trigger wording. Which are noun-phrase summaries (the page's failure) rather than "use when …"? Add usage logging | known |
| 18 | `concepts/claude-code-skills-lessons.md:17` | "**verification skills** have had 'the most measurable impact' on output quality" | Vendor claim, unquantified | Rank the 40 skills by whether they end in a runnable check. chronicler has `verify-gate`, `chronicler-ui-investigator`, `test-police`. Count how many of the 40 have no check at all | inferred |
| 19 | `talks/how-i-deleted-95-percent-of-my-agent-skills.md:50` | "with one particular skill loaded, the task was correct **77%** of the time; **without** loading it, **97%**" | One operator, 8 languages, 20+ repos; 10,000 lines → **553 lines** (`:45-47`); eval runs **68 min → 6 min** | A/B 3–5 of the 40 skills on a fixed task, loaded vs not. A skill that cannot show a delta is a candidate to delete, per the page's own headline | known |
| 20 | `concepts/agent-memory-components.md:160` | "longer instruction files **reduce adherence**"; Claude Code "loads only the first **200 lines or 25 KB** of `MEMORY.md`" | Anthropic 2026-11-06 post; CL-Bench, Pokémon Red, 350k-file runs | `AGENTS.md` is 418 lines. Test adherence for a rule in the first 100 lines vs one past line 300. Report the ordering of the rules that matter | known |
| 21 | `concepts/context-engineering.md:33` | Anthropic removed "**over 80% of Claude Code's system prompt**" with "no measurable loss on coding evaluations" | Vendor eval, Claude 5; the post's own rules at `:44` name "clashing instructions in one request" as the diagnosis | Look for contradictions between `AGENTS.md`, `.pi/APPEND_SYSTEM.md`, `CODING_STANDARDS.md` and skills (the page's own example: "leave documentation as appropriate" vs "DO NOT add comments"). List every pair | known |
| 22 | `concepts/fresh-context-review.md:20` | Devin Review "catches **~2 bugs per PR, ~58% severe**" with "zero shared context with the coder" | Cognition, verified 2026-09-05 | Check whether the `reviewer` and `judge-*` subagents get a clean context and a read-only surface. See next row for the leak | known |
| 23 | `concepts/fresh-context-review.md:41` | "**instruction-file exclusion**" is tested separately, with "issue #237 reporting `AGENTS.md` reaching reviewers despite `fork_turns: 'none'`" | EGM `TESTING.md`, unverified on Codex and Antigravity | Empirically test whether `.pi/agents/reviewer.md` children inherit `AGENTS.md` (ask the child to quote a line only in `AGENTS.md`). The wiki says the host usually gives no signal | known |
| 24 | `concepts/claude-code-team-code-review.md:82` | "**Incidents are the eval.** Past incident-causing PRs … run against the reviewer daily and rescored on any change of prompt, tool or model." | Anthropic, self-reported; other numbers `:28` 16% → 54% substantive comments, `:29` ~59% re-flags | chronicler has no incident corpus. Check whether any review finding ever became a permanent test, or whether review output is discarded each round | inferred |
| 25 | `concepts/claude-code-team-code-review.md:90` | "a **three-round cap** (then step back and rethink)" after "one CLI PR, 50+ review/fix rounds overnight" (`:30`) | Self-reported | Check `code-review` and `chronicler-after-plan-workflow` skills for a round cap and a convergence rule (flag the bug *class*) | known |
| 26 | `concepts/automated-eval-hillclimbing.md:45` | Judge gets "**checkable claims, not a 1-to-5 scale**"; `:46` "**The judge model should not be the model under test.**" | claude.dev 2026-09-28; held-out 78.6% → 90.5% at ~1/5 cost (`:97`) | chronicler ships `judge-deepseek.md`, `judge-glm.md`, `judge-opus.md`, `judge-qwen.md`. Check each judge differs from the model it judges, and that its rubric is checkable claims | known |
| 27 | `concepts/automated-eval-hillclimbing.md:84` | "Report vs baseline with CIs; **if within noise, recommend against merging**"; noise floor first (`:76`) | Same playbook | Any harness change (new skill, new rule) should carry a before/after with a noise estimate. Check whether `.scratch/` tickets ever record one | inferred |
| 28 | `concepts/expect-tests.md:68` | "An agent that runs `dune runtest --auto-promote`, or edits `[%expect]` blocks … **rewrites the expectation it is tested against**" | Snapshot evidence: 22.2% mutation score snapshot-only vs 28.8% (`:43`), n.s. | Check whether any snapshot/approval test in `tests/` can be regenerated by the agent under test (golden files, insta, expect). A regeneration command in a skill is the risk | known |
| 29 | `concepts/reward-hacking.md:19` | Reward hacking rate "**30.4%**" on RE-Bench 2, "**100%**" on Optimize LLM Foundry; forms are "editing or deleting tests, special-casing test inputs, and overwriting the grader" (`:13`) | METR, 39 of 128 and 21 of 21 | Check what stops an agent from editing `validate_feature_spec.py`, the guardrail walkers, or a spec scenario to make the gate pass. Is `.scratch/` or `scripts/` protected? | known |
| 30 | `concepts/orchestration-principles.md:86` | "**Orchestration multiplies a gate; it never creates one.**" | Design reading of OpenRig/firstmate, "not measured" | State chronicler's one gate (`build.py`) and test whether each subagent role is placed behind it. A role with no gate multiplies work only | known |
| 31 | `concepts/orchestration-principles.md:68` | "**Never give a worker the right to land its own work.**" | firstmate read-only supervisor, self-repo boundary | Check the permission config (`.pi/extensions/pi-permission-system/config.json`): `"*": "allow"` with only `*.env` and its own config denied. A worker can commit. Report the gap between this rule and the config | known |
| 32 | `concepts/orchestration-principles.md:77` | "**if a unit of work has no machine-checkable definition of done, it does not get an agent.**" | OpenRig "proof" field, firstmate ship/scout split | Read `.scratch/*/issues/*.md`: does each ticket name the check that will prove it? Count tickets with no such line | known |
| 33 | `concepts/wayfinder-decision-map.md:30` | The map "**is an index, not a store**"; `:35` "the assignee *is* the claim"; `:86` local markdown trackers "persist by accident" | Maintainer's own failure catalogue | `.scratch/` is exactly "local markdown". Check for stale claims (open issue whose decision already landed), missing assignee/claim discipline, and whether `_closed/` decisions are re-litigated | known |
| 34 | `concepts/wayfinder-decision-map.md:72` | An agent wrote "'this map carries execution' into its own Notes. In later sessions the agent read that line back as permission" | User report | Search `.scratch/` and skills for agent-written text that grants itself permission (memory notes, "TODO: allowed to …"). This is a concrete grep | known |
| 35 | `concepts/agent-team-allocation.md:20` | "stage allocation beats headcount" — planning-only **$249** vs all three stages **$510**; "lead quality is the ceiling"; workers plateau past ~four (`Finding 3`) | Cohere, 2026-10-14, ProgramBench, 200 tasks, ±6 points per task | Count how many `.pi/agents/*.md` roles are implementers vs planners/verifiers. The wiki predicts implementer fan-out past 4 adds merge work, not information | known |
| 36 | `concepts/agent-team-allocation.md:125` | "**84%** of messages went worker → lead"; peer messaging cost weaker models "4–7 points"; teams cost **4–7×** a solo agent (`:53`) | Same study | Check `.pi/npm`/delegation config for peer messaging or concurrency caps. `acp-for-chronicler-engine.md:28` records "≤3 concurrent (`.pi/subagents.json`)" — verify that cap still exists | known |
| 37 | `concepts/folding-subagents-into-the-main-loop.md:16` | A single agent with skills "frequently" beat the subagent design; "**no numbers are published**" | Anthropic Applied AI, 2026-10-22 | Weigh the 9-role delegation tree against this. Test one role folded into the main loop with skills, per the migration procedure (`:56`, "10 golden examples is enough") | known |
| 38 | `concepts/skills-security.md:19` | "**3,984 public skills**: **36.82% with at least one flaw, 534 critical, 76 with confirmed malicious payloads**"; posture `:44` "pin + review + receipts" | Snyk audit via a secondary source | 40 skills, **no `skills-lock.json`** on disk today (the entity records one at 2026-09-30, `:139`). Check provenance and diff review for every third-party skill | known |
| 39 | `concepts/claude-code-prompt-caching.md:76` | "Side computations (compaction, summarisation, skill execution) **fork with the parent's prefix**"; `:48` don't switch models mid-session; `:55` never add/remove tools mid-session | Vendor rules; measured counter-data `:89` (92.12% cache reads, `pi-prompt-caching.md:90`) | For the engine's own LLM calls: is the prompt assembled byte-stable (static system first, history appended, tools fixed)? Check `src/application/prompting/assembler.rs` for non-deterministic ordering | known |
| 40 | `talks/anthropic-workshop-agents-that-run-for-hours.md:60` | Long-run state files should be "`feature_list.json` (**JSON specifically because models overwrite markdown**)" | Workshop, 2 harnesses, ~$200/6 h | `.scratch/` is markdown by design. Check what state a long autoplay or multi-session run keeps, and whether it is machine-readable | inferred |
| 41 | `concepts/opus-5-5-usage-guide.md:46` | "**Put a stop rule in `CLAUDE.md`** … keep going when a step needs no input"; `:54` keep the task list in a file | Vendor playbook, no numbers | `AGENTS.md` has no stop/continue rule and no "report format". Check whether long sessions stall waiting for permission on non-destructive steps | known |
| 42 | `concepts/repo-as-memory.md:114` | "**It does not enforce anything. It is rung 3 at best**" (a wiki/memory store); `:99` "The code repo holds **commitments**" | Anthropic 350k-file numbers, 41 → 57 of 57 facts | chronicler already splits these (`docs/diataxis` + `.scratch`). Check that a recurring wiki note has ever been promoted into a walker (`:108`) | known |
| 43 | `concepts/software-factory.md:81` | "the median share of PRs merged with **no human review at 0.5%**"; CMU "lines added **+281%** in month one … complexity about **+41%** … both lasting"; SlopCodeBench "no agent finished any of **36** evolving-spec problems" | Jellyfish 2026-10, CMU/MSR 2026, arXiv preprint | chronicler's commits are agent-authored. Check whether anyone tracks revert rate, complexity, or `clippy` warning trend per 100 commits | known |
| 44 | `queries/acp-for-chronicler-engine.md:83` | "do **not** bulk-load ACP's catalog into ground rules — **adopt mechanisms, not prose**" | Applicability synthesis, 91 patterns | Compare `AGENTS.md` growth over 6 months with the number of new executable checks. If prose grew and checks did not, the page's warning has already been breached | inferred |
| 45 | `queries/acp-for-chronicler-engine.md:38` | "**Decision Guards (`DECISIONS.md` + markers)** — highest value per token spent" (recommended 2026-09-06) | Synthesis from ACP catalog | Check whether it was ever adopted, and whether its absence is visible in reversions of intentional tradeoffs (e.g. `snapshot_id` not being a SQL FK) | guessed |

---

## 2. What the wiki already records about chronicler-engine

All from `entities/chronicler-engine.md` unless stated.

**State and dates.** 609 commits on `main` at 2026-09-30, 592 at 2026-09-26, 501 at 2026-09-05; Rust
1.88 pinned; "8 branches, 0 tags; 934 files in the HEAD tree" (`:11-14`). Snapshots are pinned by SHA
because "the repo moves constantly": `798a51804de409a57814debaa7eb87b679206197` (2026-09-30) and
`c963f558016c56c0fbbc20c35065d0c90dc753b5` (2026-09-26) (`:23-27`).

**Skill-set churn.** 39 dirs at 2026-09-30, 42 at 2026-09-26, "~37 noted 2026-09-05" (`:133`).
Verified diff: "dropped `setup-matt-pocock-skills`, `to-spec`, `to-tickets`, `triage`, `wizard`;
added `no-ai-slop` and `unslop`. Net −3" (`:31-33`). **Today: 40 skill dirs, all with `SKILL.md`.**
Breakpoint from the record: +1 since 2026-09-30.

**ADR removal, dated.** "**33 ADRs existed** — `adr-001-htmx-web-dashboard.md` … `adr-033-…`", plus
`adr-000-template.md`, a 75-line `docs/adr/README.md`, a `documentation-and-adrs` skill and a
"**369-line `scripts/validate_adrs.py`**". "**Deleted 2026-08-10** in commit `e0f22e00` … (52 files)"
(`:48-52`). The author's reason is recorded as voice, not maintenance: they were "full of historical
information that no longer matters, and then were constantly referred to for current information"
(`:56-58`). Current stated policy: "There is no system-wide decision-record directory at the root by
design" (`:60-61`). "ADR numbers still float in plan prose … pointing at files that no longer exist"
(`:62-63`).

**Where rationale lives, six-plus stores:** 18 `docs/plans` files with a "Decisions Locked" section;
30 archived plans; 4 reviews; ".scratch/<effort>/map.md + issues/ — 9 efforts, 83 issue files"; 7
closed efforts; `docs/CHANGELOG.md`; `CONTEXT.md`; 8 explanation docs; commit messages with
measurements ("the ~4.5s FST-dictionary measurement behind the text-check default") (`:65-80`).
"none of which is referenced from the code that the rationale explains" (`:82`).

**Requirement machinery.** `docs/specs/*.md` (18 files, Gherkin); `scripts/validate_feature_spec.py`
is "the gate" - every scenario needs a covering test, every `// [spec] SCENARIO: X.Y` tag needs a
declared scenario, tags must match surface, "exit 0/1/2" (`:86-95`). Measured result elsewhere:
"`validate_feature_spec.py`: **143/143 scenarios covered, exit 0**"
(`comparisons/requirement-encoding-tools.md:29`). `tests/STRATEGY.md` holds "four test tiers"
(`:96-97`). Guardrails are "structured not prose": clippy denials, `arch-lint.toml`, `syn`-based
walkers; `guardrails.md` is auto-generated (`:98-104`).

**The DOC-anchor measurement.** "`//! [DOC: <path>]` module headers exist on **217 of 331**
`src/**/*.rs` files … **Zero `src/` files point at an `explanation/` doc** … The rule that would
enforce the anchors (`require-doc-comments`) has severity `disabled` in `arch-lint.toml`, so the 66%
coverage is convention, not law" (`:113-118`).

**Agent infrastructure.** `.agents/skills/` holds the Pocock chain "installed 2026-07-08–09 via
`setup-matt-pocock-skills`" plus "~20 custom skills"; "`skills-lock.json` pins skill versions"
(`:133-140`) — **that file is absent today**. `.pi/` has the custom **pi-permission-system**
("to prevent the agent from touching git without supervision"), `subagents.json`, `plan-mode.json`,
`APPEND_SYSTEM.md` (`:141-143`).

**Rules in `AGENTS.md` as recorded.** "auto-generated structure index, hexagonal-architecture rules
(arch-lint.toml layer rules), story points (≥8 must split), local `.scratch/` markdown issue tracker
('no gh CLI dependency'), code-review rules ('do not compile/build during reviews')" (`:144-148`).

**The gate.** "`build.py` gate: fmt + clippy + guardrails + tests in ~1 minute, wired into pre-commit"
(`:149-150`); per-step subcommands (`fmt`, `check`, `clippy`, `unit`, `architecture`, `guardrails`,
`test-pattern`, `validate-docs`); "every run logs to `logs/build_*.log`"; concurrent agents use
"`--target-dir target/agentX --no-fmt` because `cargo fmt` rewrites files in place and `target/`
contends on the cargo lock" (`:150-156`).

**Etiquette rules, 2026-09-26 snapshot.** "label epistemic status (known / inferred / guessed), say
'I don't know' instead of inventing, disagree explicitly before saying what you changed, never
rationalise a test failure away, don't commit without explicit approval even where the permission
config allows it, and don't make code changes during a code review. Repo health outranks the
individual task. Codebase questions are dispatched to an `Explore` subagent … issue tracking is local
markdown in `.scratch/` (**196 files** at snapshot) with five canonical triage labels"
(`:157-163`).

**Recorded theory of why it works.** "The 1-minute deterministic gate is the loop-closer that makes
[[model-task-split]] work with a 27B-class implementation model" (`:166-170`); model split recorded as
"**GLM-5.3-Flash as main brain + Qwen3.8-27B delegation children**" (`:11`); the autonomy stance is
"the opposite stance on autonomy: her `never-block-on-the-human` vs his explicit git-touching
prohibition" (`:173-175`).

**Delegation shape, from the ACP query.** "children in isolated fresh contexts, ≤3 concurrent
(`.pi/subagents.json`)" (`queries/acp-for-chronicler-engine.md:28`); "at one dev + ≤3 children the pi
harness already plays orchestrator" and multi-agent "costs ~15× tokens" (`:78`). Three ranked
adoptions proposed, never recorded as done: Decision Guards, a "Sunk Cost cap on delegation" ("let a
child retry **twice**, then `git reset`", `:41-47`), and change-attractor analysis (`:52-57`).

**Product-side fact recorded once.** "[[chronicler-engine]]'s per-swipe `snapshot_id` exists so a retry
does not cascade-delete history" (`talks/always-on-proactive-agents-lee-robinson.md:167`).

**Not recorded.** No wiki page records: the current skill count (40), the permission config's actual
contents, `build.py` run medians, a revert rate, any eval or pass-rate for generated prose, or the
number of tickets in `.scratch/` today. `entities/chronicler-engine.md:27` says the 2026-09-05 commit
is "not recorded".

---

## 3. Product side: caching, context, memory, judge evals

### 3.1 Prompt caching for long conversations

- The mechanism: the API caches "everything **from the start of the request up to each
  `cache_control` breakpoint**, matched byte for byte. Any change in the prefix invalidates
  everything after it" (`concepts/claude-code-prompt-caching.md:26`).
- Ordering that follows: "Static system prompt + tool definitions" (global) → `CLAUDE.md` (project) →
  session context → "Conversation messages — grows each turn" (`:32-38`).
- Three rules that matter for a growing story history: "don't switch models mid-session" (`:48`),
  "never add or remove tools mid-session" (`:55`), and "compaction must share the parent's prefix"
  (`:68`) with a "reserved **compaction buffer** in the window" (`:74`). The team "declares SEVs" when
  the cache hit rate drops (`:16`).
- Measured on the owner's own route: "**92.12%** of 124.6M tokens were cache reads, **983/1000** …
  against a 94,592-token cached prefix" (`concepts/pi-prompt-caching.md:90`), i.e. "**92.1% of all
  tokens** served from cache" (`:20`). "**There is no 5-minute cliff in this data**" (`:94`); the
  caveat is n=6 and a possible "prefix rewrite (compaction, branch, tool-list change)" (`:107-108`).
- The same rules stated outside Anthropic: "Keep tool definitions and system prompts static between
  calls"; "Fixed prompt order, hashed per section"; "Do compaction while the cache is still warm"
  (`talks/always-on-proactive-agents-lee-robinson.md:106-114`).
- **Product read:** a story that grows every turn is exactly the "conversation messages" layer. The
  cache stays warm only if the prompt prefix — system, character sheet, world, retrieved memories —
  is byte-stable and appended to. Nothing in the wiki measures chronicler's cache hit rate.

### 3.2 Long-context degradation in a growing story

- "F1 held at ~0.55–0.58 up to ~40% of the window, then collapsed to ~0.30 (**a 45.5% drop**) by 50%"
  (`concepts/long-context-degradation.md:27`).
- "**over 69% of incorrect trajectories had already encountered the correct answer** and failed to use
  it" (`:29`).
- "Effective context is typically **60–70% of the advertised window**"; "Many models fall below half
  their short-context baseline by **32K tokens**" under NoLiMa (`:37-38`).
- Remedies named are "**compaction**, **note-taking**, **isolation**"; "retrieve instead of stuffing";
  "bound the working set" (`:56-60`). A pointer beats compaction: "Compaction is irreversible;
  a pointer is not" (`concepts/orchestration-principles.md:125`).
- **Product read:** history cut and story-window assembly are the failure site. The step function
  predicts a cliff, not a slope, so the interesting measurement is at 40–50% of the model's window.

### 3.3 Memory for long stories

- Failure shape measured on Pokémon Red: Sonnet 3.7 wrote "**413 files across 225 folders**" and earned
  1 badge; Fable 5 wrote "**9 files in 2 folders + one `global.md`**" and earned **4**. The mechanism:
  "**the game state changes constantly, so a detailed record is continuously out of date**"
  (`concepts/agent-memory-components.md:80-85`).
- The good shape is "a dashboard — current goal, a prewritten plan …, and its own rules for using
  memory" (`:89`).
- Removing maintenance changes the outcome: with a wrong note and no dreaming pass the run "never
  reached Cerulean City"; after the note was checked against the transcript it did (`:117-127`).
- One writer per page: concurrent agents left "**41 of 57 graded facts**"; "one agent per contract its
  own page" took it to "**57 of 57**" (`:155`). A wiki pays on synthesis, not lookup: "**87% vs 73%**"
  on 24 merger questions (`:134`), while "wiki plus search" went "54% → 61%" at "**20,000 fewer tokens
  per question**" (`:139`).
- **Product read:** a story's facts are query-shaped ("where is the innkeeper?"), which the wiki says
  is the *weak* case for a maintained store — the strong case is synthesis across sources.

### 3.4 Judging generated prose

- Judge design: rubric of "**checkable claims, not a 1-to-5 scale**", blind pairwise against a
  baseline, and "**The judge model should not be the model under test.**"
  (`concepts/automated-eval-hillclimbing.md:45-46`). Read a sample by hand: "Read a sample of scored
  transcripts before believing your evaluator" (`:48`). Warn when baseline ≥ ~95% headroom (`:52`).
  Report with CIs and "**if within noise, recommend against merging**" (`:84`).
- Judge drift is the named hazard: "An LLM judge scores … with rubrics ([[reward-hacking]] covers
  judge drift)" (`concepts/agent-pr-review-levers.md:36-37`).
- The rubric that graded subjective output in the workshop had "four criteria — **design, originality,
  craft, functionality**", calibrated few-shot on references, and its author argues "most people say
  you can't grade taste — we think you can if you have a strong enough opinion, and you write it
  down" (`talks/anthropic-workshop-agents-that-run-for-hours.md:82-84`). Out of the box the critic was
  "**a really, really bad general QA agent**" (`:116`), and the contract that made it work had "**27
  criteria**" (`:112`).
- The evaluator must exercise the artifact, not read it: it "launches **Playwright**, opens live
  pages, clicks, tries things" (`:76`); the critic does not get the generator's traces
  ("**No, deliberately**", `:159`); and self-evaluation is "a trap — use an adversarial evaluator"
  (`:141`).
- On applying the judge to a harness rather than prose: Warp's `/merge-risk` shows the arithmetic
  hazard — "The weighted average dilutes single red flags": a one-way door with everything else clean
  scores **1.40, "low"** (`concepts/agent-pr-review-levers.md:90`), and it is "**Advisory, not a
  gate**" (`:101`).

---

## 4. Pages judged not relevant

- `books/` (all)
- `entities/beelink-*`, `entities/intel-n*`, `entities/gmktec-g3-plus`, `entities/cwwk-fanless-mini-pc`, `entities/fanless`, `entities/raspberry-pi-5-vs-n100-mini-pc` class hardware pages
- `concepts/home-iot-architecture`, `concepts/homelab-basics`, `concepts/choosing-mini-pc-for-home-assistant`, `concepts/power-consumption-and-costs`, `concepts/vaapi-on-amd-vega`, `concepts/intel-alder-lake-n`
- `concepts/maybe-sure-vs-google-sheets`, `entities/maybe-finance`, `entities/voicestudio`, `entities/jellyfin`, `entities/frigate`, `entities/home-assistant`
- `comparisons/linux-distro-for-ser3-home-server`, `concepts/headless-server-bringup-runbook`
- `comparisons/video-report-tooling-2026-10`, `comparisons/coding-plan-billing-models-2026-10`, `entities/cheapestinference`, `entities/cheaperinference`
- `concepts/weight-space-programming`, `concepts/weight-editing-prior-art`, `comparisons/weight-space-programming-claims-2026-11`
- `concepts/dreaming` (read for the memory section, but its own maintenance-loop claims are not auditable on this repo)
- Model-price and model-review entity pages generally (`entities/gpt-*`, `entities/claude-sonnet-5*`, `entities/gemini-*`), except where cited above for the tier split
