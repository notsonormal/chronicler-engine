# Research 07: what Hex's builders say in public

Status: reference. Asset of [ticket 07](../issues/07-research-public-sources.md) in the [map](../map.md).
Read on 2026-11-01. Every claim is labelled **known** (a source shows it) or **inferred** (I reason from sources).

## Summary

The Hex authors publish the product, the repo and their agent rules. They publish no write-up of the Hex build process.
The one private part is `~/code/control/AGENTS.md`, and it stays private. The repo names its tools, but the tools are not public.
Four names do not resolve: `hivemind`, `termctrl`, `abattoir` and `control` return 404 on GitHub (known).
So the process shows through three surface types: the Hex repo files, the public `anomalyco/rune` repo, and public statements by Kit Langton and Dax Raad.
Dax Raad states his own agent habits on X, and a daily newsletter quotes each post with a link and a date (known).
Kit Langton's site lists Hex as a product, but it has no post about the Hex build process (known).
The opencode docs and the public `anomalyco/opencode/AGENTS.md` give the clearest public picture of how that team briefs agents (known).
The public statements point at candidate 4, candidate 2 and candidate 3, and at seven habits that the [findings asset](./findings.md) does not list.

## Practices

This ticket names no Chronicler file, so the Chronicler state comes from the [findings asset](./findings.md) and from `AGENTS.md` in this repo. I did not re-read Chronicler source.

| Practice | Hex evidence (file:line, commit, URL, date) | Chronicler state | Candidate |
|---|---|---|---|
| 1. The workflow rules live in a private global guide, outside the repo | `AGENTS.md:9` points to `~/code/control/AGENTS.md`. `github.com/kitlangton/control` returns 404 (checked 2026-11-01). Known. | Gap. Chronicler keeps its rules in the repo: `AGENTS.md`, `CODING_STANDARDS.md`, `.pi/extensions/pi-permission-system/config.json`. Known. | new |
| 2. The agent merges its own PR when the checks pass | `AGENTS.md:5-9`: "merge it yourself once its checks pass; merge any open PR without asking". Known. | Gap, and the opposite rule. Chronicler `AGENTS.md`: "Don't commit without explicit approval". Known. | 4 |
| 3. A public team `AGENTS.md` sets autonomy, branch names and commit shape | `anomalyco/opencode/AGENTS.md` (raw, fetched 2026-11-01): "Prefer automation: execute requested actions without confirmation unless blocked by missing info or safety/irreversibility"; default branch `dev`; branch names of at most three words with no slashes; conventional-commit PR titles. Known. | Partly covered. Chronicler has no branch-name rule and no conventional-commit rule in the files I read. Inferred. | 4, plus a new naming habit |
| 4. A short ticket id in each commit subject (`tNNN`) from a private tracker | `git log`: `(t811)` 2026-10-29, `(t694)`, `(t965)`, `(t666)`, `(t691)`, `(t733)`, `(t405)`, `(t573)`. No public tracker holds these ids (inferred from the 404s below). | Partly covered. Chronicler `.scratch/` tickets carry numbers (`01` to `08`). I did not check whether commits name them. Unknown. | new |
| 5. A shared Rust lint block, synced into each repo by a private tool | `clippy.toml:1-10`, block marked `# abattoir-lints begin` / `end`; commit `99a1becd` "Roll in Abattoir's shared Rust lints (#557)", 2026-10-30. `anomalyco/rune` `clippy.toml` has the same ten lines under a `# shared-lints begin` / `end` marker (rune `e39f410d`, 2026-10-31). `github.com/anomalyco/abattoir` returns 404. Known, except the sync tool. | Gap for the sync. Chronicler `clippy.toml` "sets thresholds only" (findings asset). Known from findings. | 6, plus a new sync habit |
| 6. `termctrl` checks a TUI at two widths | `AGENTS.md:1448`: "Use `termctrl` to verify TUI changes at both wide and narrow dimensions." `github.com/kitlangton/termctrl` returns 404. Known. | Gap. Chronicler has no TUI. Its UI checks are Playwright (findings asset). Known from findings. | 5 |
| 7. A machine-wide land queue that builds, verifies and swaps the app | `.hivemind.toml:1-10`: section `[land]`, `branch = "dev"`, `batch_max = 4`, a three-step `verify` list, and `[land.app]` that swaps `/Applications/Hex.app` with `wait_idle = 30`. Commit `9aa79bc9` "feat(land): add declarative .hivemind.toml land queue + Hex.app swap (t811)", 2026-10-29. Known. | Partly covered. `scripts/build_slot.py` is a machine-wide slot for one cargo step at a time. It does not batch, verify or install. Known from the structure index. | 4, plus a new habit |
| 8. HiveMind also runs the build jobs on named workers | rune `scripts/rune-dev:728`: `hivemind run --worker … --kind cargo --no-sync`; `:952` sets `HIVEMIND_STRICT_PIN=1`; `.hivemind.toml` reads `HIVEMIND_LAND_HEAD`. rune `scripts/mac_app_bundle.py:1` says the lock "is also used by HiveMind land". `github.com/anomalyco/hivemind` returns 404. Known. | Gap. Chronicler builds on the local machine only. Known from `ENVIRONMENT.md` per `AGENTS.md`. | new (feeds 4) |
| 9. A named dev host holds the proof, and the PR body names it | "build passed in the apollo sandbox" (PR #714 body, via findings asset); `docs/features/README.md` names apollo with its OS and date over 20 times, for example line 246: "(Vulkan, 2026-10-31, LibriSpeech test-clean)". Known. | Partly covered. `scripts/validate_feature_spec.py` links a scenario to a test, not to an observation (findings asset). Known from findings. | 3 |
| 10. A UI change regenerates its goldens in the same PR, and a person looks at each image | `AGENTS.md:12-18`: run `HEX_UPDATE_GOLDENS=1 cargo test --locked --bin hex rune_ui`, "look at every changed image, and commit them with the change"; from a Mac, dispatch `check-linux.yml` and take its `golden-actual` artifact. `CONTRIBUTING.md:47` lists a `--snap` render of one scene. Known. | Gap. No screenshot comparison in Chronicler (findings asset). Known from findings. | 5 |
| 11. The "no seam" rule is stated in the public contribution guide | `CONTRIBUTING.md:63`: "Do not add platform, plugin, or provider abstractions without a second implemented adapter that needs the seam." `AGENTS.md:1468`: "Introduce a seam once there are two real adapters or a current test requires substitution." Known. | Partly covered. Chronicler has ports and the `antipattern-checker` skill (findings asset). Known from findings. | 7 |
| 12. No outside pull requests; issues are the surface, and a fix credits the reporter | `CONTRIBUTING.md:2-10`: "Hex's team writes every code change, so we don't accept pull requests from outside the team"; "When your analysis shapes a fix, we credit you in the pull request that lands it". Known. | Gap. Chronicler work goes through the owner, not a team, and it has no credit rule. Inferred from `AGENTS.md`. | new |
| 13. Labs: every uncertain feature ships off by default, with an audience | `AGENTS.md:453-456` (the `labs::Feature` registry, and the runtime gates); `AGENTS.md:1244-1249`: "New, unproven, user-requested, and externally contributed features (including ones the issue pipeline builds) land in Labs, off by default… Never add a master switch"; `docs/features/labs.md:1-18`. Known. | Gap. Chronicler has no feature flags (findings asset). Known from findings. | 8 |
| 14. A reviewer model approves a flagged patch, through a label | `.github/workflows/agent-policy.yml:3-9`: the job runs "the base branch's .github/agent-policy/check.ts on the diff, so a patch cannot change its own policy… flagged patches pass only after the pipeline's strict reviewer approved them (label agent-strict-review)". Head branches are `agent/*`. Known. | Gap. No risk gate in Chronicler (findings asset). Known from findings. | 1 |
| 15. The opencode GitHub agent is the public form of the pipeline's runner | [opencode.ai/docs/github](https://opencode.ai/docs/github/) (fetched 2026-11-01): it runs in the repo's GitHub Actions runner, takes `/oc` or `/opencode` mentions or issue and PR events, and takes `model`, `agent`, `prompt` and `mentions` inputs. The public app is [opencode-agent](https://github.com/apps/opencode-agent). Known. | Gap. Chronicler has no issue-to-PR pipeline (findings asset). Known from findings. | 4 |
| 16. Dax Raad: one long-running session for each project, and the work goes to subagents | X [2110898321976164393](https://x.com/thdxr/status/2110898321976164393), 2026-10-16: "i tell it do everything using subagents so i can work on many things at once if needed", quoted with its link by [AI Coding Daily](https://aicodingweekly.news/stories/dax-long-running-sessions-per-project-subagents). A 2026-10-20 story adds that the orchestrator session does no work itself. Known quote; the habit is his, not Hex's. | Nothing in the repo records a session shape. Gap, if the practice counts. Inferred. | new |
| 17. Dax Raad: ask where the source of truth for intended behavior is | X [2107543978719506907](https://x.com/thdxr/status/2107543978719506907), 2026-10-06, quoted by [AI Coding Daily](https://aicodingweekly.news/stories/dax-agents-undoing-functionality-decision-records-pocock): agents undo deliberate choices, because a test does not say why approaches one to four were rejected. Known quote. Hex answers it with `AGENTS.md` invariants (candidate 2). Inferred. | Partly covered. Chronicler has `CONTEXT.md`, the feature specs and the decisions in `docs/`. Inferred. | 2, 3 |
| 18. The author publishes agent skills as public artifacts | [skills.sh/kitlangton/skills/effect](https://www.skills.sh/kitlangton/skills/effect) (fetched 2026-11-01): an Effect v4 skill with a "Source Rule" that reads the nearest `AGENTS.md` first and names the upstream commit it was reviewed against. Known. | Covered. Chronicler has a skill set under `.agents/skills/`. Known. | new (as a publishing habit) |
| 19. The product exists to feed the agent loop (dogfooding) | [kitlangton.com](https://kitlangton.com/) (fetched 2026-11-01) lists Scrawl ("Show an agent a problem instead of describing it"), Skulk ("Wake an agent on a timer or event"), 2password ("1Password for coding agents") and Ghui ("Review GitHub PRs from the terminal") beside Hex. Known list; the motive is inferred. | Not applicable. Chronicler builds one product. Inferred. | new (inferred) |

Nineteen practices. Seven carry `new` alone: 1, 4, 8, 12, 16, 18, 19. Three more add a sub-habit to a listed candidate: 3, 5, 7.

## Detail: what stays private, and what that costs

- **Known.** `AGENTS.md:9` says "See `~/code/control/AGENTS.md`". That path is not in either clone, and `github.com/kitlangton/control` returns 404. So the merge order, the review steps and the approval rules that Hex's own file defers to are not readable.
- **Known.** `hivemind`, `termctrl` and `abattoir` also return 404 at the obvious repository paths. I did not search every alternative host for them. See "What I could not find".
- **Inferred.** The tools are private but their interfaces are public. Hex's `.hivemind.toml` shows the land queue's schema field by field. Rune's `scripts/rune-dev` shows two `hivemind` command lines and one environment variable. That is enough to describe the tool's contract, and not enough to describe its behaviour.

## Detail: the tools that the repo names

| Name | Where Hex or Rune names it | What the sources show | Public? |
|---|---|---|---|
| `hivemind` | `.hivemind.toml`; `AGENTS.md` (land path); rune `scripts/rune-dev` | A land queue (`[land]`, `batch_max`, `verify`, `[land.app]`), a remote worker runner (`hivemind run --worker <host> --kind cargo`), a strict-pin mode, and a shared lock file protocol | No. 404 |
| `termctrl` | `AGENTS.md:1448` | A way to drive a terminal UI, used to check one change at wide and narrow widths | No. 404 |
| `abattoir` | `clippy.toml:1,10`; commit `99a1becd` | A source of shared Rust lints, rolled into Hex on 2026-10-30. Rune carries the same ten lines under a different marker | No. 404 |
| `~/code/control` | `AGENTS.md:9` | The author's global agent guide | No. 404 |
| apollo | `docs/features/*`, `docs/plans/rune-ui.md` | A named Linux host. Examples: "Linux (apollo, niri Wayland, 2026-10-22)" (`docs/plans/rune-ui.md:96`); an X11 and Wayland check at `docs/features/capture.md:85` | Not a tool. It is a machine, and the sources name it as one. Known |
| Rune | `AGENTS.md:1021` calls it "`anomalyco/rune`, public MIT" | A public repo with a 53-line `AGENTS.md`, a 617-line `PLAN.md`, a `changelog/unreleased/` folder, and two scripts that enforce file size and folder shape | Yes |
| the "apollo sandbox" | PR bodies, for example PR #714 (findings asset) | A build and test environment on apollo. The repo has no workflow file with that name. Inferred: it is a sandbox on that host, not a service | Unclear |

## Detail: the public statements, with dates

### Kit Langton

- **Known.** [kitlangton.com](https://kitlangton.com/) (fetched 2026-11-01) lists Hex under both "APP" and "OPEN SOURCE", described as "A free voice-to-text app for macOS". It lists no Hex post. The post list holds "Effect and the Near Inexpressible Majesty of Layers" (2026-01-29), "The Ovoid Conglomerate" (2025-01-13) and "Spare Thoughts on App Design" (2025-01-13). The video library stops at 2024 for Scala and 2023 for Swift.
- **Known.** The site says "Currently working on opencode". So the author's public writing is about opencode and Effect, not about the Hex build process.
- **Known.** [kitlangton/skills](https://www.skills.sh/kitlangton/skills/effect) is a public skill. Its "Source Rule" tells an agent to read the nearest `AGENTS.md` and the project-pinned package before it guesses, and it names the upstream commit the skill was reviewed against.
- **Known.** [Kit Langton on Cause & Effect #8](https://open.spotify.com/episode/5QLZ3gOCAyOSS56w69vY82), "Effectifying OpenCode", June 2026: the episode covers opencode and the migration of a large TypeScript codebase to Effect. I did not transcribe it. I do not know whether it covers Hex or the agent workflow.
- **Secondary.** A personal note by Facundo Goñi, 2026-06-10, [Prompting agents by voice with Hex](https://www.facundogoni.com.ar/notes/prompting-agents-by-voice-with-hex/), describes using Hex to brief Claude Code and subagents. It is a user report, not an author statement.

### Dax Raad

- **Known, quoted with links by AI Coding Daily.** Each story quotes the X post and links it:
  - 2026-10-06: agents undo deliberate functionality, and the question "where is the source of truth for how your software is supposed to work?" ([story](https://aicodingweekly.news/stories/dax-agents-undoing-functionality-decision-records-pocock)). Matt Pocock answered with ADRs. The story states that nobody in the exchange showed a setup that keeps the record current.
  - 2026-10-16: one long-running session for each project, and all work through subagents ([story](https://aicodingweekly.news/stories/dax-long-running-sessions-per-project-subagents)).
  - 2026-10-20: one pinned orchestrator session farms work out to subagents in worktrees ([story](https://aicodingweekly.news/stories/dax-pinned-gooey-session-subagents-worktrees-screenshots), from the story index on [his page](https://aicodingweekly.news/people/dax-raad)).
  - 2026-10-07 area: "custom agent setups fix problems that no longer exist" ([story](https://aicodingweekly.news/stories/dax-custom-agent-setups-vanilla-codex-kent-c-dodds)).
- **Known.** [A conversation with Dax](https://www.baseten.co/blog/building-ai-agents-open-code-and-open-source-a-conversation-with-dax/), Baseten blog: he describes an agent as "just an LLM that has access to some tools", and says of inference providers "I've always been a big believer in anecdotes over data". The interview is about opencode and Zen. It does not cover the Hex process.
- **Known.** [opencode.ai/docs/github](https://opencode.ai/docs/github/): the GitHub agent runs in the caller's Actions runner and takes `model`, `agent`, `prompt` and `mentions`.
- **Known.** [anomalyco/opencode/AGENTS.md](https://github.com/anomalyco/opencode/blob/dev/AGENTS.md) is public and detailed. It covers: autonomy ("Prefer automation"), the `dev` base branch, branch names of at most three words with no slashes, conventional-commit types, an animated SVG in the PR body for user-facing changes, a live TUI testing loop, and style rules that forbid speculative helpers and unearned abstractions ("Do not extract single-use helpers preemptively").

The last item is the closest public substitute for the missing `~/code/control/AGENTS.md`. **Inferred:** the opencode file and the control file serve the same role for the same people, so the control file probably carries the same voice and the same autonomy rule.

## Detail: what the public sources do not confirm about the issue pipeline

- **Known.** The pipeline's visible parts are all in the repo or on GitHub: the issue templates (`.github/ISSUE_TEMPLATE/`), the policy checker (`.github/agent-policy/check.ts`), the workflow (`agent-policy.yml`, head branches `agent/*`), the `agent-strict-review` label, the PR bodies with run ids, and the `Co-authored-by: Hex issue pipeline` trailer (findings asset).
- **Known.** No public page describes the pipeline's prompts, its fixer and reviewer models, or its retry rules. The findings asset reached the same conclusion.
- **Inferred.** Because the policy job runs the base branch's checker and reads `github.head_ref`, the pipeline and the checker were built together. The commit "Add hex-agent-policy check for issue-pipeline pull requests (#371)", 2026-10-29, supports this.
- **Known.** PR #714 names the reviewer model `console-anthropic-cyber/claude-opus-5-5`, and the opencode docs example names `anthropic/claude-sonnet-5-5`. So the team runs at least two models in the loop.

## What I could not find or did not read

- **No public repository for** `hivemind`, `termctrl`, `abattoir` or `control` under `anomalyco/` or `kitlangton/`. I tried one obvious URL for each and got 404. I did not try package registries, forks or a web search for each name.
- **No blog post, thread or talk by Kit Langton about the Hex build process.** His site has no Hex post, and I did not transcribe his two long podcast appearances.
- **No public page for `~/code/control/AGENTS.md`.**
- **Episode content not read:** "Effectifying OpenCode" (Cause & Effect #8, June 2026), "Building OpenCode with Dax Raad" (Pragmatic Engineer, 2026), the Baseten interview after its first 9 000 characters, and four YouTube interviews. Titles and topics are known from search results only.
- **No X access.** Every Dax Raad quote comes through AI Coding Daily, which links the post and dates it. I did not open any X page.
- **The private tracker behind the `tNNN` ids.** I do not know its tool, so I cannot say whether the ids come from a public system that the repo hides.

## Sampling

- **Hex repo.** I read `AGENTS.md` in part: the head, the `## Future Direction` section, the `labs` bullets, the Labs rule, the `termctrl` line and the Rune bullets. I read `clippy.toml` and `.hivemind.toml` in full, `CONTRIBUTING.md` in full, `.github/workflows/agent-policy.yml`'s head, `.github/agent-policy/check.ts`'s head, and `docs/features/labs.md`'s head. I searched the whole clone for `apollo`, `hivemind`, `abattoir`, `tallow`, `termctrl`, `~/code/control` and `opencode`. I read `git log` for 400 commit subjects and for file histories.
- **Rune.** I fetched the repository page (commit `e39f410d`, 2026-10-31) and read `AGENTS.md`, `clippy.toml`, and the `hivemind` hits in `scripts/`.
- **Web.** About twelve searches and eight page fetches. I used no GitHub REST API call.
- **Chronicler.** I read `AGENTS.md` (the project instructions) and the structure index. I did not read Chronicler source, because the ticket names none. Every Chronicler state above comes from the findings asset, from `AGENTS.md`, or from the structure index, and it says which.
