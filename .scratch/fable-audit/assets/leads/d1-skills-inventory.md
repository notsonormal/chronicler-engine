# D1 — Skills and harness inventory

Scope: 41 skill folders (`.agents/skills/*/` = 40, `.pi/skills/*/` = 1), `.pi/` harness config.
Method: `rg`, `wc`, `git log -1 --format=%cs -- <folder>`, `ls`.
Read-only audit. No file touched except this one.
Date of audit: 2026-11-10.

Legend: **known** = read directly from the file. **inferred** = reasoned from two quoted facts. **guessed** = my judgement, not provable from a quote.

---

## Part 1 — Skill inventory

Word counts are `wc -w` of `SKILL.md`. File counts are recursive, `__pycache__` excluded.
Git date is `git log -1 --format=%cs -- <folder>`.

| # | Skill | Words | Files | `disable-model-invocation` | Last commit |
|---|---|---|---|---|---|
| 1 | antipattern-checker | 309 | 1 | **absent** | 2026-10-23 |
| 2 | chronicler-after-plan-workflow-plus-review | 571 | 1 | `true` | 2026-10-25 |
| 3 | chronicler-after-plan-workflow | 430 | 1 | `true` | 2026-10-25 |
| 4 | chronicler-comment-fixer | 1249 | 2 | **absent** | 2026-10-25 |
| 5 | chronicler-docs-hygiene | 1086 | 1 | **absent** | 2026-10-25 |
| 6 | chronicler-ui-investigator | 1520 | 2 | **absent** | 2026-10-30 |
| 7 | code-review | 1229 | 1 | **absent** | 2026-10-23 |
| 8 | codebase-design | 865 | 3 | **absent** | 2026-10-03 |
| 9 | consider-the-bitter-lesson | 27 | 1 | `true` | 2026-10-23 |
| 10 | correct | 482 | 1 | `true` | 2026-10-04 |
| 11 | create-formal-verification-skill | 985 | 1 | `true` | 2026-11-08 |
| 12 | diataxis-doc-review | 874 | 1 | **absent** | 2026-10-23 |
| 13 | document-review | 1520 | 1 | **absent** | 2026-10-31 |
| 14 | domain-modeling | 515 | 3 | **absent** | 2026-06-29 |
| 15 | grill-me | 20 | 1 | `true` | 2026-06-29 |
| 16 | grill-with-docs | 34 | 1 | `true` | 2026-07-08 |
| 17 | grilling | 373 | 1 | **absent** | 2026-10-06 |
| 18 | handoff | 134 | 1 | `true` | 2026-07-08 |
| 19 | implement-spec | 426 | 1 | `true` | 2026-11-01 |
| 20 | improve-ai-plan | 271 | 1 | `true` | 2026-10-23 |
| 21 | improve-codebase-architecture | 797 | 2 | **absent** | 2026-10-03 |
| 22 | in-your-own-words | 48 | 1 | `true` | 2026-10-17 |
| 23 | lavish | 257 | 1 | **absent** | 2026-09-10 |
| 24 | maintain-formal-verification-skill | 1239 | 1 | `true` | 2026-11-08 |
| 25 | prototype | 504 | 3 | **absent** | 2026-06-29 |
| 26 | research | 133 | 1 | **absent** | 2026-07-08 |
| 27 | retro | 799 | 1 | `true` | 2026-10-03 |
| 28 | sharpen | 133 | 1 | `true` | 2026-10-17 |
| 29 | show-me | 458 | 1 | `true` | 2026-10-23 |
| 30 | simple-after-plan | 301 | 1 | `true` | 2026-10-21 |
| 31 | statechart | 2733 | 33 | `true` | 2026-11-09 |
| 32 | teach | 1490 | 5 | `true` | 2026-06-29 |
| 33 | technical-writing | 1910 | 1 | `true` | 2026-09-05 |
| 34 | test-police | 245 | 1 | `true` | 2026-11-01 |
| 35 | thermo-nuclear-code-quality-review | 2012 | 1 | `true` | 2026-10-23 |
| 36 | unslop | 915 | 1 | `true` | 2026-10-23 |
| 37 | verify-gate | 3088 | 26 | **absent** | 2026-11-09 |
| 38 | wait-what | 124 | 1 | `true` | 2026-08-20 |
| 39 | wayfinder | 2071 | 1 | `true` | 2026-10-17 |
| 40 | writing-for-agents | 1833 | 2 | **absent** | 2026-10-23 |
| 41 | impeccable (`.pi/skills/`) | 1595 | 59 | **absent** | 2026-11-01 |

### 1.1 Descriptions, quoted verbatim (known)

| Skill | `description:` from frontmatter |
|---|---|
| antipattern-checker | "Detect abstraction anti-patterns in Chronicler Engine Rust code." |
| chronicler-after-plan-workflow-plus-review | "The post-implementation pass, plus the four review subagents and the consolidated findings report" |
| chronicler-after-plan-workflow | "Post-implementation pass over landed changes — verify the plan, update docs and tests, clean up, run the full build" |
| chronicler-comment-fixer | "Remove AI-slop and restating comments from repo Rust, Python, JavaScript, HTML and CSS files." |
| chronicler-docs-hygiene | "Use when auditing docs/diataxis/ docs against AGENTS.md and src/ for what validate_docs.py can't catch: rule violations, mode drift, stale-source drift. Read-only; reports findings, never edits." |
| chronicler-ui-investigator | "Chronicler UI investigation — drive the running dashboard through Chrome DevTools and capture shot, DOM and endpoint evidence for ad-hoc checks, gameplay debugging, and post-plan review. Triggers on: /chronicler-ui-investigator, chronicler ui, dashboard review, ui screenshot" |
| code-review | "Review the changes since a fixed point (commit, branch, tag, or merge-base) along two axes — Standards ... and Spec ... Runs both reviews in parallel sub-agents and reports them side by side." |
| codebase-design | "Shared vocabulary for designing deep modules. Use when the user wants to design or improve a module's interface, find deepening opportunities, decide where a seam goes, make code more testable or AI-navigable, or when another skill needs the deep-module vocabulary." |
| consider-the-bitter-lesson | "Consider the bitter lesson in the current task." |
| correct | "Find the mistakes agents keep repeating in this repo and make each one impossible. ... Use for /correct." |
| create-formal-verification-skill | "Generate a project-local skill that formally verifies the code where a bug costs the most and tests reach the least ..." |
| diataxis-doc-review | "Break the LLM's status-quo bias when reviewing chronicler_engine docs. Judge whether each diataxis doc earns its place or is merely inherited from what's already there." |
| document-review | "Review markdown docs top-down: doc set, document, section, sentence. Use when reviewing any document or .md file." |
| domain-modeling | "Build and sharpen a project's domain model. Use when the user wants to pin down domain terminology ..." |
| grill-me | "A relentless interview to sharpen a plan or design." |
| grill-with-docs | "A relentless interview to sharpen a plan or design, which also creates docs (ADR's and glossary) as we go." |
| grilling | "Grill the user relentlessly about a plan, decision, or idea. Use when the user wants to stress-test their thinking, or uses any 'grill' trigger phrases." |
| handoff | "Compact the current conversation into a handoff document for another agent to pick up." |
| implement-spec | "Implement a spec and its tickets on one integration branch." |
| improve-ai-plan | "Review an implementation plan before code is written." |
| improve-codebase-architecture | "Scan a codebase for deepening opportunities, present them as a visual HTML report, then grill through whichever one you pick" |
| in-your-own-words | "Restate in your own words what you think the user's goals are and what problem they're trying to solve. Use for /in-your-own-words." |
| lavish | "Turn complex or visual agent responses into rich, reviewable HTML artifacts the user can annotate and send feedback on, using the lavish-axi CLI. ..." |
| maintain-formal-verification-skill | "Bring a project's formal verification back in line with the code after the code changes ..." |
| prototype | "Build a throwaway prototype to answer a design question. Use when the user wants to sanity-check whether a state model or logic feels right, or explore what a UI should look like." |
| research | "Investigate a question against high-trust primary sources and capture the findings as a Markdown file in the repo. ..." |
| retro | "Conduct a retrospective on a coding session." |
| sharpen | "Sharpen a task before starting: the expert approach, what the user is really after, and the right level of specification. Use for /sharpen." |
| show-me | "Help the user understand the current topic visually with concise diagrams, code-shape sketches, and focused HTML artifacts." |
| simple-after-plan | "Post-implementation pass — comment cleanup, doc review, code review, full build, then a retro" |
| statechart | "Check a TLA+ model of a system that moves between states ... and draw it as a Mermaid statechart. ..." |
| teach | "Teach the user a new skill or concept, within this workspace." |
| technical-writing | "Layered technical-writing standard: Diátaxis structure, Google developer style sentences, STE instruction rules, Global English syntax. ..." |
| test-police | "Reviewer for Chronicler Engine tests" |
| thermo-nuclear-code-quality-review | "Run an extremely strict maintainability review for abstraction quality, giant files, and spaghetti-condition growth. ..." |
| unslop | "Cut AI tells from any writing." |
| verify-gate | "Model-check the generation gate and generate its diagrams in docs/diataxis/reference/diagrams/. ..." |
| wait-what | "Stop. That last message did not land — re-pitch it." |
| wayfinder | "Plan a huge chunk of work — more than one agent session can hold — as a shared map of investigation tickets on your issue tracker ..." |
| writing-for-agents | "Writing documents for agents. Use when creating or editing skills, or modifying AGENTS.md or CLAUDE.md." |
| impeccable | "Use when the user wants to design, redesign, shape, critique, audit, polish, simplify, harden, animate, or otherwise improve a frontend interface ..." |

**Count of skills with `disable-model-invocation: true`: 26 of 41.** The other 15 are model-invocable, so their description text is the only routing signal (this is what `retro` calls out at `.agents/skills/retro/SKILL.md:24`).

Extra frontmatter fields, complete list (known):
- `metadata: {language: rust, workspace: chronicler_engine}` — `chronicler-after-plan-workflow/SKILL.md:5-7`
- `argument-hint: "[port] [world]"` — `chronicler-ui-investigator/SKILL.md:5`
- `argument-hint: "What will the next session be used for?"` — `handoff/SKILL.md:4`
- `argument-hint: "What would you like to learn about?"` — `teach/SKILL.md:5`
- `license`, `metadata.author`, `metadata.argument-hint`, `metadata.hermes-tags`, `metadata.hermes-category` — `lavish/SKILL.md:5-11`
- `version`, `license: Apache 2.0`, `allowed-tools: [Bash(npx impeccable *), Bash(.pi/skills/impeccable/scripts/impeccable *)]` — `impeccable/SKILL.md:5-9`

### 1.2 Cross-skill references (path:line, quoted)

| Referrer | Line | Target skill | Quote |
|---|---|---|---|
| simple-after-plan | `:10` | chronicler-comment-fixer, document-review | "run `/chronicler-comment-fixer` (`.agents/skills/chronicler-comment-fixer/SKILL.md`) in an `implementer` subagent and `/document-review` (`.agents/skills/document-review/SKILL.md`) in a `reviewer` subagent" |
| simple-after-plan | `:12` | code-review | "Run `/code-review` (`.agents/skills/code-review/SKILL.md`)" |
| simple-after-plan | `:14` | retro, wayfinder | "Run `/retro` (`.agents/skills/retro/SKILL.md`) ... Read `/wayfinder` (`.agents/skills/wayfinder/SKILL.md`) first for the issue format." |
| chronicler-after-plan-workflow | `:17` | wayfinder | "If the plan was created through the skill `/wayfinder` (`.agents/skills/wayfinder/SKILL.md`)" |
| chronicler-after-plan-workflow | `:24` | chronicler-docs-hygiene | "Standards: `/chronicler-docs-hygiene` (`.agents/skills/chronicler-docs-hygiene/SKILL.md`)" |
| chronicler-after-plan-workflow | `:23` | chronicler-comment-fixer | "Run the `/chronicler-comment-fixer` skill against the (usually uncommitted) changes" |
| chronicler-after-plan-workflow-plus-review | `:14` | wayfinder | same sentence as above |
| chronicler-after-plan-workflow-plus-review | `:20` | chronicler-docs-hygiene | same sentence as above |
| chronicler-after-plan-workflow-plus-review | `:26` | chronicler-after-plan-workflow | "This is intentionally a copy of `.agents/skills/chronicler-after-plan-workflow/SKILL.md`." |
| chronicler-after-plan-workflow-plus-review | `:24` | thermo-nuclear, code-review, antipattern-checker, test-police | "Run the 4 review subagents — `/thermo-nuclear-code-quality-review`, `/code-review` and `/antipattern-checker` under the read-only `reviewer` definition, `/test-police` under `implementer`" |
| code-review | `:95` | antipattern-checker | "- `/antipattern-checker` (`.agents/skills/antipattern-checker/SKILL.md`)" |
| code-review | `:96` | chronicler-docs-hygiene | "- `/chronicler-docs-hygiene` (`.agents/skills/chronicler-docs-hygiene/SKILL.md`)" |
| thermo-nuclear-code-quality-review | `:145-147` | antipattern-checker, chronicler-docs-hygiene, improve-codebase-architecture | three bullets, same path form |
| thermo-nuclear-code-quality-review | `:139` | codebase-design (implicit) | "keeps a helper whose name carries a concept, keeps a seam that tests use" |
| document-review | `:26` | chronicler-docs-hygiene, diataxis-doc-review | "`.agents/skills/chronicler-docs-hygiene/SKILL.md`; background in `.agents/skills/diataxis-doc-review/SKILL.md`" |
| document-review | `:28` | writing-for-agents | "`.agents/skills/writing-for-agents/SKILL.md`" |
| document-review | `:29` | domain-modeling | "`.agents/skills/domain-modeling`" |
| document-review | `:87` | unslop, technical-writing | "apply `.agents/skills/unslop/SKILL.md` and this checklist from `.agents/skills/technical-writing/SKILL.md`" |
| diataxis-doc-review | `:45` | chronicler-docs-hygiene | "`.agents/skills/chronicler-docs-hygiene/SKILL.md` — rule-compliance findings" |
| diataxis-doc-review | `:46` | domain-modeling | "`.agents/skills/domain-modeling` — for the project's ubiquitous language" |
| grill-me | `:7` | grilling | "Run a `/grilling` session." |
| grill-with-docs | `:7` | grilling, domain-modeling | "Run a `/grilling` session, using the `/domain-modeling` skill." |
| wayfinder | `:83,115,127` | grilling, domain-modeling | "Conversation via the /grilling and /domain-modeling skills"; "If in doubt, use `/grilling` and `/domain-modeling`." |
| improve-codebase-architecture | `:58` | grilling | "run the `/grilling` skill to walk the design tree with them" |
| verify-gate | `:73` | maintain-formal-verification-skill | "use the `/maintain-formal-verification-skill` skill" |
| verify-gate | `:164` | statechart | "`check.py`, and the scripts of `/statechart`" |
| create-formal-verification-skill | `:24` (approx.) | maintain-formal-verification-skill | names the upkeep skill as its maintenance counterpart |
| retro | `:7` | writing-for-agents | "Use the `/writing-for-agents` skill for the writing style guide." |
| technical-writing | `:24` | unslop | "Add new offenders to `unslop`'s abstract-metaphor rule with their replacement." |
| writing-for-agents | `:8` | own `SKILL-MECHANICS.md` | "read [`SKILL-MECHANICS.md`](SKILL-MECHANICS.md)" |
| impeccable | `:20,83` | own scripts | "`.pi/skills/impeccable/scripts/impeccable <verb>`" |

Orphans: skills that name no other skill and that no other skill names — see Part 3.

### 1.3 Stale-reference check

Every repository path, script, doc and command named by a skill, checked with `ls` / `rg`.

| Skill:line | Named path | Exists? | Verdict |
|---|---|---|---|
| `code-review/SKILL.md:34` | `CONTRIBUTING.md` | **no** | **stale**. Quote: "Anything in the repo that documents how code should be written, such as `CODING_STANDARDS.md` or `CONTRIBUTING.md`." |
| `writing-for-agents/SKILL.md:3` | `CLAUDE.md` | **no** | **stale** (soft). Quote: "modifying AGENTS.md or CLAUDE.md". |
| `writing-for-agents/SKILL.md:6` | "`AGENTS.md` / `CLAUDE.md`" | **no** | **stale** (soft). |
| `document-review/SKILL.md:28` | `CLAUDE.md` | **no** | **stale** (soft). Table row "A skill, `AGENTS.md`, `CLAUDE.md`". |
| `improve-codebase-architecture/SKILL.md:13` | `docs/adr/` | **no** | **stale**. Quote: "ADRs in `docs/adr/` record decisions this command should not re-litigate." |
| `domain-modeling/SKILL.md:34,37,40` | `docs/adr/` | **no** | **not stale, prescriptive**. Quote: "If no `docs/adr/` exists, create it when the first ADR is needed." |
| `simple-after-plan/SKILL.md:14` | `session_list` tool | **no tool of that name** in this harness | **stale**. Quote: "(find them with `session_list`)". `TaskCreate` at `:7` also names no tool in this harness. |
| `chronicler-after-plan-workflow/SKILL.md:14` | `old-docs/archived-plans` | **yes** | fine |
| `chronicler-after-plan-workflow/SKILL.md:14` | `docs/plans` | **yes** | fine |
| `chronicler-after-plan-workflow/SKILL.md:19` | `scripts/healthcheck.py duplicates` | **yes** (`healthcheck.py:221` `@register("duplicates")`) | fine |
| `chronicler-docs-hygiene/SKILL.md:20` | `scripts/validate_docs.py --strict` | **yes** (`:1229`) | fine |
| `code-review/SKILL.md:58` | `scripts/prepare_review_bundle.py --ref/--uncommitted` | **yes** (`:273,275`) | fine |
| `test-police/SKILL.md:20` | `build.py --coverage`, `--target-dir` | **yes** (`build.py:472,496`) | fine |
| `chronicler-comment-fixer/SKILL.md:14-27` | `scripts/comment_finder.py --uncommitted/--branch` | **yes** (`:388,390`) | fine |
| `verify-gate/SKILL.md:19` | `diagrams.toml` | **yes**, at `diagrams/diagrams.toml` | fine (bare name in a warning string) |
| `verify-gate/SKILL.md:69,129` | `model/mutants.toml` | **yes** | fine |
| `verify-gate/SKILL.md:159` | `diagrams/diagrams.toml` | **yes** | fine |
| `verify-gate/SKILL.md:33` | `mise.toml` | **yes** | fine |
| `statechart/SKILL.md:22` | `scripts/statechart.ts` | **yes** | fine |
| `chronicler-ui-investigator/SKILL.md` | `scripts/cdp.mjs`, `scripts/target_seed.py`, `tests/browser/`, `docs/diataxis/reference/frontend/http_routes.md` | **all yes** | fine |
| `chronicler-ui-investigator/SKILL.md:14` | `/home/node/.pi/start-browser.sh` | **yes** | fine |
| `lavish/SKILL.md:21` | `npx -y lavish-axi` | `npx` yes, `lavish-axi` not on PATH | fine: the command fetches it |
| `verify-gate/SKILL.md:33` | `tlc` (TLC binary) | **not on PATH** | provisioned by `mise.toml` (known: the skill says so) |
| `impeccable/SKILL.md:8,20` | `.pi/skills/impeccable/scripts/impeccable`, `impeccable.cmd` | **both yes** | fine |
| `retro/SKILL.md` (`package.json`) | `package.json` | **no** at root | **stale** (soft): "its `package.json`/build-tool `lint`/`check` scripts". Repo is Rust + `build.py`. |

Missing skills named by other files (not by skills): `.scratch/retro-issue-bucket/` names five skill folders that no longer exist — `commit-and-push`, `tdd`, `to-spec`, `to-tickets`, `setup-matt-pocock-skills`. Proof: `docs/plans/parallel-ticket-runs-without-commits.md:186` — "Remove from the skill: ... the references to `/to-spec`, `/to-tickets` and `/setup-matt-pocock-skills`." Those `.scratch` files are historical records, so this is stale-but-expected. **inferred**.

**Summary: 5 hard stale references in skills** (`CONTRIBUTING.md`, two `docs/adr/` claims, `session_list`, `package.json`), plus **4 soft** (`CLAUDE.md` x3, `TaskCreate`).

---

## Part 2 — Overlapping skill groups

### Group A — Post-implementation workflow (3 skills)

| Skill | Description | Words |
|---|---|---|
| chronicler-after-plan-workflow | "Post-implementation pass over landed changes — verify the plan, update docs and tests, clean up, run the full build" | 430 |
| chronicler-after-plan-workflow-plus-review | "The post-implementation pass, plus the four review subagents and the consolidated findings report" | 571 |
| simple-after-plan | "Post-implementation pass — comment cleanup, doc review, code review, full build, then a retro" | 301 |

Differing lines (quoted):
- Only `plus-review` has the review step: `:24` "Run the 4 review subagents — `/thermo-nuclear-code-quality-review`, `/code-review` and `/antipattern-checker` under the read-only `reviewer` definition, `/test-police` under `implementer`".
- Only `chronicler-after-plan-workflow` has coverage: `:18` "Check the code coverage and try to keep it at 80% minimum for all files (run `build.py --coverage`)."
- Only `simple-after-plan` has the retro: `:14` "Run `/retro` ... Add new issues to `.scratch/retro-issue-bucket/`".
- `simple-after-plan` has no plan verification and no plan archiving. `chronicler-after-plan-workflow` `:14` archives: "The archive folder is `old-docs/archived-plans`."

Self-declared duplication: `chronicler-after-plan-workflow-plus-review/SKILL.md:26` — "This is intentionally a copy of `.agents/skills/chronicler-after-plan-workflow/SKILL.md`. Everything is the same except for the additional review step and removal of the code coverage step (handled by `test-police`)."

Which one a new agent picks: **all three have `disable-model-invocation: true`**, so an agent cannot pick — a human must type the name. That is clear for routing. **Unclear** is which of the three the human should type: the descriptions overlap on "post-implementation pass" and only `simple-after-plan` mentions the retro, so a reader cannot tell that `simple-after-plan` is the only one that closes the issue file. **inferred**.

### Group B — Grilling (3 skills, plus 2 inline copies)

| Skill | Words | Body |
|---|---|---|
| grilling | 373 | the full method |
| grill-me | 20 | `:7` "Run a `/grilling` session." |
| grill-with-docs | 34 | `:7` "Run a `/grilling` session, using the `/domain-modeling` skill." |

Differing lines: the whole body, above. `grilling` is model-invocable (no disable field), the two wrappers are not.

Which one a new agent picks: an agent that needs to grill will match `grilling` on its description ("uses any 'grill' trigger phrases"), which is the right one. The two wrappers are user-only aliases. **Clear, but only because of the disable field.** A user reading the skill list sees three near-identical names. **inferred**.

Duplication outside the group:
- `sharpen/SKILL.md:7` and `wayfinder/SKILL.md:13` contain the **identical** paragraph, word for word: "Before starting, ask how a top expert would approach this task (their raw sources, their real methods, an outside check that the result is right) ... Don't hand back a rewritten prompt." `wayfinder` copies `sharpen` inline instead of calling it. **known**.

### Group C — Review (7 skills)

| Skill | Axes it covers | Model-invocable |
|---|---|---|
| code-review | Standards + Spec, 2 parallel subagents | yes |
| thermo-nuclear-code-quality-review | "abstraction quality, giant files, and spaghetti-condition growth" | no |
| antipattern-checker | "abstraction anti-patterns in Rust code that clippy and arch-lint cannot see" | yes |
| test-police | "Reviewer for Chronicler Engine tests" | no |
| chronicler-docs-hygiene | "auditing docs/diataxis/ docs", read-only | yes |
| diataxis-doc-review | "whether each diataxis doc earns its place" | yes |
| document-review | "markdown docs top-down" | yes |

Differing lines:
- `code-review/SKILL.md:9` restricts itself: "does the code conform to this repo's documented coding standards?" vs `thermo-nuclear/SKILL.md:24` "Be ambitious, if there is a clear path to improving the implementation that involves restructuring some of the codebase, go for it."
- `chronicler-docs-hygiene/SKILL.md:3` "Read-only; reports findings, never edits." vs `document-review` which has no such sentence.
- `antipattern-checker` and `thermo-nuclear` overlap directly: `code-review:95` and `thermo-nuclear:145` load the *same* two extra lenses.
**Boundary unclear:** `chronicler-docs-hygiene` says "what validate_docs.py can't catch", and `validate_docs.py` also owns rule compliance; `diataxis-doc-review` says "Judge whether each diataxis doc earns its place", which `document-review:26` also claims as its own test ("It carries purpose, invariants or a frame, in one Diátaxis mode"); then `document-review:26` defers to `diataxis-doc-review` "background in ...". Three skills claim the same existence test.

Which one a new agent picks: for "review my code" the agent sees `code-review` and `thermo-nuclear-code-quality-review` both model-visible; only `code-review` is model-invocable, so the routing is clear. For "review my docs" **both** `document-review`, `diataxis-doc-review` and `chronicler-docs-hygiene` are model-invocable, and all three descriptions contain "docs". That is **unclear**: nothing in the descriptions says document-review is the umbrella and the other two are lenses. **inferred**.

### Group D — Formal verification (4 skills)

| Skill | Role |
|---|---|
| create-formal-verification-skill | build a project-local verify skill |
| maintain-formal-verification-skill | upkeep after drift |
| verify-gate | *the* project-local verify skill (Chronicler's instance) |
| statechart | TLA+ model + Mermaid rendering |

Differing lines: `verify-gate/SKILL.md:18` "The agent that changed a listed function runs this skill, or `/maintain-formal-verification-skill`, before it ends its task." The `or` is the only routing rule, and it gives no test. `create-formal-verification-skill` generates what `verify-gate` already is, so a fresh agent can read it as a request to regenerate the repo's existing gate. **guessed** that this is the failure mode.

### Group E — Writing and prose rules (3 skills + 1 harness file)

| Skill | Scope |
|---|---|
| unslop | "Cut AI tells from any writing." 33 numbered rules |
| technical-writing | "Layered technical-writing standard: Diátaxis structure, Google developer style sentences, STE instruction rules, Global English syntax." |
| writing-for-agents | "Writing documents for agents." |
| `.pi/APPEND_SYSTEM.md` | "Prose: ASD-STE100 Simplified Technical English." — always loaded |

`wait-what/SKILL.md:9-16` **restates the whole `APPEND_SYSTEM.md` STE block** verbatim inside the skill. Quote from `wait-what`: "Prose: ASD-STE100 Simplified Technical English. - Short sentences. One main action or statement per sentence." And `APPEND_SYSTEM.md` has the identical lines. That is 8 duplicated lines in an always-loaded harness file and a skill.

`technical-writing/SKILL.md:24` "Add new offenders to `unslop`'s abstract-metaphor rule" — a skill that edits another skill. `unslop` rules 3-33 keep stable ids ("Rule numbers are stable ids that other skills cite"), so the two are coupled but not redundant.

Which one a new agent picks: for "clean up prose" — `unslop` and `technical-writing` both model-invocable? No: both have `disable-model-invocation: true`. So user-invoked. `writing-for-agents` is model-invocable, and its description mentions `AGENTS.md`, so an agent asked to "update AGENTS.md" will reach it correctly. **Clear.**

### Group F — Visual artefacts (4 skills)

| Skill | Medium | Model-invocable |
|---|---|---|
| show-me | "concise diagrams, code-shape sketches, and focused HTML artifacts" | no |
| lavish | "rich, reviewable HTML artifacts ... using the lavish-axi CLI" | yes |
| impeccable | frontend design/redesign, `.pi/skills/` | yes |
| improve-codebase-architecture | "present them as a visual HTML report" | yes |

Differing lines: `show-me/SKILL.md:7` "Pick the smallest view that makes the key point clear." vs `lavish:3` "rich, reviewable HTML artifacts the user can annotate". The axis is small-and-fast vs rich-and-annotatable, and neither description says so. `improve-codebase-architecture/SKILL.md:1` says it "present[s] them as a visual HTML report" — a third way to produce HTML. **Unclear for a model**: three model-invocable skills produce HTML. **inferred**.

### Group G — Preamble / clarification micro-skills (3 skills)

`sharpen` (133 w), `in-your-own-words` (48 w), `wait-what` (124 w). All user-invoked. Differing lines: `in-your-own-words` is pure restatement; `sharpen` also changes the approach; `wait-what` re-pitches the *previous assistant message* in STE. No overlap in output, but `sharpen` and `wait-what` both inherit the STE and CONTEXT.md framing. **Clear.**

---

## Part 3 — Orphan skills and mention counts

Counts are `rg -c` summed. Buckets: **other-skills** = mentions inside the other 40 skill folders; **root+docs** = `AGENTS.md`, `CODING_STANDARDS.md`, `docs/`; **.scratch** = the issue tracker, which shows real use.

| Skill | other-skills | root+docs | .scratch | Orphan? |
|---|---|---|---|---|
| antipattern-checker | 3 | 0 | 90 | no |
| chronicler-after-plan-workflow-plus-review | 0 | 0 | 48 | **orphan** |
| chronicler-after-plan-workflow | 1 | 0 | 69 | no |
| chronicler-comment-fixer | 3 | 0 | 299 | no |
| chronicler-docs-hygiene | 7 | 1 | 136 | no |
| chronicler-ui-investigator | 0 | 0 | 253 | **orphan** (but heavy real use) |
| code-review | 4 | 3 | 507 | no |
| codebase-design | 6 | 0 | 90 | no |
| consider-the-bitter-lesson | 0 | 0 | 8 | **orphan** |
| correct | 0 | 7 | 3633 | no (see noise note) |
| create-formal-verification-skill | 2 | 0 | 39 | no |
| diataxis-doc-review | 1 | 0 | 46 | no |
| document-review | 1 | 0 | 168 | no |
| domain-modeling | 7 | 3 | 214 | no |
| grill-me | 0 | 0 | 5 | **orphan** |
| grill-with-docs | 0 | 0 | 32 | **orphan** |
| grilling | 7 | 2 | 1218 | no |
| handoff | 0 | 0 | 943 | **orphan** (see noise note) |
| implement-spec | 0 | 6 | 32 | **orphan in skills, named in docs** |
| improve-ai-plan | 0 | 0 | 64 | **orphan** |
| improve-codebase-architecture | 1 | 0 | 35 | no |
| in-your-own-words | 0 | 0 | 15 | **orphan** |
| lavish | 0 | 0 | 817 | **orphan** (see noise note) |
| maintain-formal-verification-skill | 4 | 0 | 34 | no |
| prototype | 2 | 1 | 908 | no (see noise note) |
| research | 1 | 1 | 2552 | no (see noise note) |
| retro | 1 | 9 | 828 | no |
| sharpen | 0 | 0 | 790 | **orphan** (see noise note) |
| show-me | 0 | 0 | 65 | **orphan** |
| simple-after-plan | 0 | 4 | 132 | **orphan in skills, named in docs** |
| statechart | 17 | 0 | 1773 | no |
| teach | 1 | 1 | 209 | no |
| technical-writing | 1 | 0 | 68 | no |
| test-police | 2 | 1 | 169 | no |
| thermo-nuclear-code-quality-review | 1 | 0 | 32 | no |
| unslop | 2 | 0 | 108 | no |
| verify-gate | 0 | 27 | 116 | no (named heavily in docs) |
| wait-what | 0 | 0 | 7 | **orphan** |
| wayfinder | 3 | 6 | 1028 | no |
| writing-for-agents | 3 | 1 | 197 | no |
| impeccable | 0 | 0 | 924 | **orphan** |

### Noise warning (known)

These names are ordinary English words, so the count is inflated and the skill may be effectively unreferenced:

| Skill name | Problem |
|---|---|
| `correct` | 3633 `.scratch` hits are the word "correct" in prose, not the skill. `docs+root=7` are also prose. |
| `research` | 2552 hits are the noun/verb. |
| `statechart` | 1773 hits are mostly the `.scratch/statechart-models/` folder and `verify-gate`'s model, not the skill. |
| `grilling` | 1218 hits include `.scratch` prose about grilling as an activity. |
| `handoff`, `sharpen`, `lavish`, `impeccable`, `retro`, `wayfinder` | common words; counts are upper bounds. |
| `antipattern-checker` | 90 `.scratch` hits are the real thing. |

**True orphans (zero other-skill mentions, weak or zero real use):** `consider-the-bitter-lesson` (8), `wait-what` (7), `grill-me` (5), `in-your-own-words` (15), `grill-with-docs` (32), `thermo-nuclear-code-quality-review` is not an orphan (1 skill mention), `improve-ai-plan` (64 but zero skill mentions), `show-me` (65, zero skill mentions), `create-formal-verification-skill` is referenced by `maintain-formal-verification-skill`.

**Orphans that are used only by a human typing the name:** `chronicler-after-plan-workflow-plus-review`, `chronicler-ui-investigator`, `lavish`, `impeccable`. All four have `disable-model-invocation: true` (except `chronicler-ui-investigator` and `impeccable`, which do not) and none is named by another skill. **inferred**: that is the intended design for user-invoked commands, but it also means nothing tells an agent they exist.

**Skills named only in `docs/` and not by any other skill:** `implement-spec` (6 root/doc hits), `simple-after-plan` (4), `verify-gate` (27). Their discovery depends on the doc, not a skill.

---

## Part 4 — The harness

### 4.1 `.pi/settings.json` (known)

Quote: `{"packages": ["extensions/pi-plan-mode", "../../pi-emote", "npm:@agnishc/edb-todo"]}`.
Only one of the three packages is in this repo (`extensions/pi-plan-mode`). `../../pi-emote` resolves outside the repo, and `npm:@agnishc/edb-todo` is installed from npm. **inferred**.

### 4.2 Extensions

| Extension | What it does (source quote) | Loaded? |
|---|---|---|
| `delegate/` | "delegate: run a task in a fresh Pi agent in its own herdr pane, and send the agent's last message back to the parent session as a message that starts a turn." (`index.ts:1-3`) | not in `settings.json` — loaded by another path. **not recorded** |
| `mrn-context/` | "mrn-context: inject a compact context block (Model, Build Log, Anthropic usage) into each pi turn." (`index.ts:1-2`) | not in `settings.json` |
| `pi-permission-system/` | The rules at 4.4. No `index.ts`, only `config.json` — the code is elsewhere. | **not recorded** |
| `pi-plan-mode/` | "`@notsonormal/pi-plan-mode` adds a Codex-like `/plan` collaboration mode to Pi." (`README.md:6`) | **yes** (`settings.json`) |
| `rate-limit-wait/` | "hold a run that hit the Anthropic 5-hour limit, then continue it." (`index.ts:2`) | not in `settings.json` |
| `smoke-break/` | "Smoke Break: nudges an agent that has gone down a rabbit hole." (`index.ts:1`) | not in `settings.json` |
| `neuralwatt.json` | Provider quota config: `"quotaWarnings": {"enabled": false}` | config only |

**inferred**: the five extensions absent from `settings.json` are loaded by a user-level pi config outside this repo, since the repo has no other loader.

### 4.3 Delegate roles (`.pi/agents/*.md`)

| File | `name:` | Model | Thinking | Tools |
|---|---|---|---|---|
| `implementer.md` | implementer | `anthropic/claude-sonnet-5-5` (+ `spare_model: anthropic/claude-sonnet-5-5`) | high | not recorded (no `tools:` field → inherits) |
| `judge-deepseek.md` | **`judge-glm`** | `opencode-go/deepseek-v4.1-flash` | high | read, ls, find, grep, bash |
| `judge-glm.md` | judge-glm | `opencode-go/glm-5.3-flash` | high | read, ls, find, grep, bash |
| `judge-opus.md` | **`judge-glm`** | `anthropic/claude-opus-5-5` | high | read, ls, find, grep, bash |
| `judge-qwen.md` | judge-qwen | `opencode-go/qwen3.7-plus` | high | read, ls, find, grep, bash |
| `researcher.md` | researcher | `anthropic/claude-sonnet-5-5` | high | read, ls, find, grep, bash, web_search, fetch_content, get_search_content, source_check |
| `reviewer.md` | reviewer | `anthropic/claude-opus-5-5` | high | read, ls, find, grep, bash |
| `scout.md` | scout | `opencode-go/deepseek-v4.1-flash` | high | read, ls, find, grep, bash |
| `strong-implementer.md` | strong-implementer | `anthropic/claude-opus-5-5` | medium | not recorded (no `tools:` field) |

**Defect (known):** three files declare `name: judge-glm` — `judge-deepseek.md:2`, `judge-glm.md:2`, `judge-opus.md:2`. Only `judge-qwen.md` has a unique name among the judges. The `delegate` extension reads only "name, model, thinking and tools" (`delegate/index.ts:14`), so a delegate role named `judge-deepseek` or `judge-opus` does not exist. **inferred**: two of the three files are dead or shadow each other.

**Tool gap (known → inferred):** `scout`, `reviewer`, `implementer` and the three judges have **no** `write` or `edit` tool. Both review skills and `simple-after-plan:10` expect them to write findings files: `chronicler-after-plan-workflow-plus-review/SKILL.md:24` — "Reviews answer inline — write all four findings files under `tmp/` yourself." The skill already accounts for this. **Consistent.**

**Also inconsistent:** `AGENTS.md:404` says "`implementer` is the default for implementation work"; `implementer` has no `tools:` field, so its grant is not recorded here.

### 4.4 Permission rules (`.pi/extensions/pi-permission-system/config.json`)

| Rule | Effect |
|---|---|
| `"*": "allow"` | everything else allowed |
| `path: "*.env": "deny"` | reading/writing `.env` |
| `path: "*.env.*": "deny"` | reading/writing `.env.local`, `.env.production` |
| `path: "*.env.example": "allow"` | example file is readable |
| `write: ".pi/extensions/pi-permission-system/config.json": "deny"` | the agent cannot widen its own permissions. Quote: `"*.env"` rules and this deny are the whole set. |
| `edit: same path: "deny"` | same, via edit |
| `bash: "timeout"` and `"timeout *"` | deny the `timeout` command. Quote: "Do not use the timeout command. Set the timeout parameter of the bash tool instead" |
| `bash: "grep * . *"`, `"grep * ./ *"`, `"grep * /workspace/chronicler-engine *"`, `"grep * /workspace/chronicler-engine/ *"` | deny repo-root `grep`. Quote: "Repo-root grep walks target/ (~10 GB) and hangs. Use rg (skips target/ via .gitignore) or grep a subdirectory such as src/ or tests/" |
| `bash: "find / *"` | deny `find /`. Quote: "find / walks target/ (~10 GB) and hangs." |

Not denied, though `AGENTS.md:381` implies it: `cargo` and `build.py` are allowed by `"*": "allow"`. **known**. There is no rule protecting git.

### 4.5 Other `.pi` files

- `.pi/APPEND_SYSTEM.md` (727 B): the 9-line STE output-style block, always appended. It duplicates the constraints block that the project prompt already carries.
- `.pi/plan-mode.json`: `"defaultTools"` lists 17 tools including `subagent`, `recall`, `rag_*`, `visual_explainer`, `graphify` — **none of these are registered in this session**, so plan mode grants names this harness cannot resolve. **inferred**. `"planFolder": "docs/plans"` and `"scratchFolders": ["tmp", "docs/plans", ".scratch"]` match the tree. **known**.
- `.pi/lsp.json`: `rust` → `rust-analyzer`, extensions `[".rs"]`. Matches the repo language. **known**.
- `.pi/extensions/neuralwatt.json` + `neuralwatt.v0.10.2-flat-config.json`: two copies of the same two settings (`includeLegacyModelIds: false`, `quotaWarnings` off). The `v0.10.2-flat-config` file is a pinned legacy copy. **inferred**.

---

## Part 5 — Contradictions with `AGENTS.md` and `CODING_STANDARDS.md`

### 5.1 A review must not change code — but one review skill tells the reviewer to restructure

- `AGENTS.md:335` (known): "However, during code reviews you should not make code changes to fix problems."
- `thermo-nuclear-code-quality-review/SKILL.md:24` (known): "> Be ambitious, if there is a clear path to improving the implementation that involves restructuring some of the codebase, go for it."

Also `code-review/SKILL.md:4-6` frames itself as reviewing and then aggregating, with no edit step; `code-review/SKILL.md:78` "Do **not** merge or rerank findings". So `code-review` agrees with `AGENTS.md`. `thermo-nuclear` does not. **Known contradiction.** Severity: it is a prompt to the reviewer subagent, not a licence to write, so the practical damage depends on the subagent's tool grant — and `reviewer` has no `write`/`edit` tool (4.3), which silently blocks it. **inferred**.

### 5.2 A review must not build — but the review skills run the gate

- `CODING_STANDARDS.md:24` (known): "Do not try to compile, build or test the code when doing code reviews. Unless the review explicitly calls for it (e.g. the `test-police` review)."
- `chronicler-after-plan-workflow-plus-review/SKILL.md:24` (known): "Run the 4 review subagents — ... `/test-police` under `implementer`, which re-runs the tests in its own target dir".
- `code-review/SKILL.md:34` (known): "Anything in the repo that documents how code should be written, such as `CODING_STANDARDS.md`".

The `test-police` carve-out is explicit in `CODING_STANDARDS.md`, so `plus-review` obeys the letter. **Not a contradiction** once the carve-out is read. **known**. But `simple-after-plan/SKILL.md:8-10` runs `/code-review` and then `:12` "Run the full gate once, here: `python build.py`" — the gate runs *after* the review, so that is consistent too.

The real gap: `chronicler-docs-hygiene` and `code-review` are read-only by description, while `simple-after-plan:10` runs `chronicler-comment-fixer` under `implementer`, which edits. `chronicler-comment-fixer/SKILL.md` has no read-only claim, so no contradiction.

### 5.3 `docs/diataxis/` must change only for parts, invariants, names — but one skill audits all of it

- `AGENTS.md:349` (known): "`docs/AGENTS.md` is an autogenerated catalogue of `docs/diataxis/`."
- `chronicler-after-plan-workflow/SKILL.md:24` (known): "A doc in `docs/diataxis/` changes only when the changes add, remove or rename a part, an invariant or a name that the doc states".
- `chronicler-docs-hygiene/SKILL.md:3` (known): "Read-only; reports findings, never edits." — consistent.

**No contradiction.** The skill restates the rule correctly.

### 5.4 `AGENTS.md` forbids committing without approval — `implement-spec` builds a branch and PR

- `AGENTS.md:400` (known): "Don't commit without explicit approval, even if committing is allowed in the permissions config."
- `implement-spec/SKILL.md:23` (known): "Create the integration branch. If the issue tracker closes work through PRs, or the user asks for one, open a draft PR after the first merge in step 5".
- `implement-spec/SKILL.md:31` (known): "Once an **implementer subagent** completes, merge its work to the integration branch with a **merger subagent**."

**Known contradiction in effect.** `implement-spec` requires commits and merges. `AGENTS.md:400` requires explicit approval for a commit. The skill says nothing about asking. There is also no merge role in `.pi/agents/` (4.3): no `merger` agent exists, so step 5 names a subagent type the harness does not define. **known**.

### 5.5 Delegate only substantial tasks — but two skills delegate freely

- `AGENTS.md:404` (known): "Delegate a task only when it is substantial, and do a short task yourself: each new subagent pays again to load the context."
- `grilling/SKILL.md:27` (known): "Finding _facts_ is your job, never the user's. When a frontier question needs a fact from the environment (filesystem, tools, etc.), dispatch a sub-agent to find it".
- `research/SKILL.md:6` (known): "Spin up a **background agent** to do the research, so you keep working while it reads."

**Mild tension, not a contradiction.** Each skill's delegation is inside its own workflow, and `AGENTS.md:404` exempts that: "A skill that runs several subagents, such as code review or parallel wayfinder tickets, keeps its own split." **inferred**.

### 5.6 `AGENTS.md` says read `CODING_STANDARDS.md` before reviewing — one review skill does not

- `AGENTS.md:351` (known): "`CODING_STANDARDS.md` holds the implementation, code-comment, code-review and testing rules. Read it before writing or reviewing code."
- `test-police/SKILL.md:7` (known): "Read `tests/AGENTS.md` and `tests/STRATEGY.md` first" — names only the test docs.
- `antipattern-checker/SKILL.md:8` (known): "Read `CODING_STANDARDS.md` first." — obeys.
- `code-review/SKILL.md:34` (known): treats `CODING_STANDARDS.md` as *one candidate* standard source — "such as" — and lets the Standards sub-agent receive only the files the parent found.

**Inferred**: a `test-police` run can skip `CODING_STANDARDS.md` without breaking any skill's own rule, which weakens `AGENTS.md:351` for one review.

### 5.7 The gate is the only build path — `verify-gate` runs its own toolchain

- `AGENTS.md:357` (known): "Every build action goes through `build.py`."
- `verify-gate/SKILL.md:28` (known): "`python3 .agents/skills/verify-gate/check.py`", and `:33` "`check.py` gets TLC and the JDK through `mise`".

**Not a contradiction.** TLC is a model check, not a build, and `build.py` calls the model's coverage warning (`verify-gate/SKILL.md:18`). But `AGENTS.md:357` is worded as "every build action", and `check.py` compiles nothing. **inferred**.

---

## Findings, ranked

| # | Finding | Evidence | Severity |
|---|---|---|---|
| 1 | Three agent files declare `name: judge-glm`. Two judge roles cannot be addressed. | `.pi/agents/judge-deepseek.md:2`, `judge-glm.md:2`, `judge-opus.md:2` | High |
| 2 | `implement-spec` requires commits and merges, and names a `merger` subagent type that is not defined. | `implement-spec/SKILL.md:23,31`; no `.pi/agents/merger.md`; vs `AGENTS.md:400` | High |
| 3 | `thermo-nuclear-code-quality-review` tells a reviewer to restructure code, against `AGENTS.md:335`. | quoted both sides in 5.1 | Medium |
| 4 | `simple-after-plan` names `session_list` and `TaskCreate` tools that this harness does not register, so step 6 cannot run as written. | `simple-after-plan/SKILL.md:7,14` | Medium |
| 5 | 4 stale paths named by model-invocable skills: `CONTRIBUTING.md`, `docs/adr/` (as existing), `CLAUDE.md` x3, root `package.json`. | 1.3 | Medium |
| 6 | 26-line STE block duplicated in `wait-what` and in `.pi/APPEND_SYSTEM.md`, which is always loaded. | `wait-what/SKILL.md:9-16` vs `APPEND_SYSTEM.md:3-14` | Low |
| 7 | `wayfinder:13` inlines `sharpen:7` verbatim instead of calling `/sharpen`, so the two drift apart on any edit. | quoted in Group B | Low |
| 8 | Three skills (`document-review`, `diataxis-doc-review`, `chronicler-docs-hygiene`) are model-invocable and all claim the docs-review space, with no description-level routing rule. | Part 2 Group C | Low |
| 9 | 8 skills have zero mentions in any other skill, any doc and have near-zero real use: `consider-the-bitter-lesson`, `wait-what`, `grill-me`, `in-your-own-words`, `grill-with-docs`, `improve-ai-plan`, `show-me`, plus `antipattern-checker` has zero docs mentions. | Part 3 | Low |
| 10 | `chronicler-after-plan-workflow` and `-plus-review` are hand-maintained copies; the file says so itself, so any edit to one silently diverges. | `plus-review/SKILL.md:26` | Low |
