# Research what Hex's builders say in public

Type: research (AFK)
Status: resolved
Blocked by: —

## Question

What do Kit Langton, Dax Raad and the opencode project say in public about how they build Hex and how they work with agents?

## Context

- **What to find.**
  - talks, blog posts, threads and podcasts
  - the opencode docs on its GitHub agent
  - the tools named in the repo:
    - hivemind (`.hivemind.toml`)
    - `abattoir-lints` (`clippy.toml`)
    - `termctrl` (`AGENTS.md`)
    - the "apollo sandbox" (pipeline PR bodies)
    - Rune (`anomalyco/rune`)
    - `~/code/control` (`AGENTS.md`)
- **Sources.** Use primary sources first: the authors' own posts and repos. A secondary write-up counts only if it links to its source.
- **Dates.** Date each source. The Rust rewrite began on 2026-07-14, and the issue pipeline on 2026-10-29. An older source may describe an older process.
- **Tools.** This ticket needs web access: `web_enable`, or the `researcher` role.
- **Chronicler side.** None. Link each finding to the candidate it touches in the [findings asset](../assets/findings.md), or mark it as new.

## Answer

The authors publish the product, the repo and their agent rules, but no write-up of the Hex build process. `~/code/control/AGENTS.md` stays private, and `hivemind`, `termctrl`, `abattoir` and `control` return 404 on GitHub.
Their interfaces are public anyway: `.hivemind.toml` shows the land queue field by field, `clippy.toml` shows the shared lint block, and `anomalyco/opencode/AGENTS.md` shows the team's autonomy, branch and commit rules.
Kit Langton's site has no Hex post. Dax Raad's statements come through AI Coding Daily, which links each X post and dates it.
Nineteen practices, of which seven are new: a private global guide, ticket ids in commit subjects, HiveMind build workers, the no-outside-PR contribution model, the session shape, published skills, and dogfooding. The rest sharpen candidates 1, 2, 3, 4, 5, 6, 7 and 8.
Asset: [research-07-public-sources.md](../assets/research-07-public-sources.md)
