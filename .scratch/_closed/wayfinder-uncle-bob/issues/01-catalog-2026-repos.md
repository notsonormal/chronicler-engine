# Catalog Uncle Bob's 2026-updated GitHub repositories

Type: research
Status: resolved
Blocked by:

## Question

What are all the repositories under https://github.com/unclebob that received commits, releases, or README updates in 2026, and what is each one about at a high level?

## Answer

Investigated `https://api.github.com/users/unclebob/repos` (100 per page, all pages).

### Summary

- Total public repositories: **94**
- Repositories with `updated_at` in 2026: **59**
- Repositories with actual push activity (`pushed_at`) in 2026: **34**
- Repositories with only non-push metadata updates in 2026: **25**

For applicability to Chronicler Engine, **push activity is the meaningful signal** — it means the code changed. The 34 pushed repos are listed below. The 25 “updated only” repos are listed at the end for completeness; most are old course/example repos whose metadata changed (issues, stars, settings).

### Repositories with push activity in 2026 (34)

| Repository | Language | Last push (2026) | Description |
|---|---|---|---|
| [swarm-forge](https://github.com/unclebob/swarm-forge) | Clojure | Aug 19 | A simple tool for coordinating several AI agents. |
| [negative-test-experiment](https://github.com/unclebob/negative-test-experiment) | Clojure | Aug 17 | Eight independent Hunt the Wumpus runs: testing discipline × CRAP, then mutation. |
| [Acceptance-Pipeline-Specification](https://github.com/unclebob/Acceptance-Pipeline-Specification) | Go | Aug 15 | Portable acceptance pipeline specification. |
| [crap4clj](https://github.com/unclebob/crap4clj) | Clojure | Aug 4 | CRAP formula skill for Clojure with speclj. |
| [missile-command](https://github.com/unclebob/missile-command) | Clojure | Jul 30 | Dual-platform Missile Command remake (Clojure/ClojureScript). |
| [ubc-website](https://github.com/unclebob/ubc-website) | JavaScript | Jul 29 | Uncle Bob Consulting website. Example for functional programming series. |
| [deintroverter4clj](https://github.com/unclebob/deintroverter4clj) | Clojure | Jun 22 | (no description) |
| [dry4clj](https://github.com/unclebob/dry4clj) | Clojure | Jun 20 | (no description) |
| [empire-2025](https://github.com/unclebob/empire-2025) | Clojure | Jun 19 | (no description) |
| [springslim](https://github.com/unclebob/springslim) | Java | Jun 17 | A Java Slim Service that manages Spring Transactions. |
| [clj-mutate](https://github.com/unclebob/clj-mutate) | Clojure | Jun 17 | A mutation tester for clojure, specific to speclj but easy to modify. |
| [dependency-checker](https://github.com/unclebob/dependency-checker) | Clojure | Jun 17 | A namespace dependency checker for clojure apps. |
| [htw-6-clj-vid](https://github.com/unclebob/htw-6-clj-vid) | Clojure | Jun 6 | (no description) |
| [htw-clj-six-pack](https://github.com/unclebob/htw-clj-six-pack) | Clojure | Jun 4 | (no description) |
| [speclj-structure-check](https://github.com/unclebob/speclj-structure-check) | Clojure | Jun 3 | A Claude skill for clojure and speclj that keeps the parentheses and the spec structure sane. |
| [experiment-htw-go-swarm](https://github.com/unclebob/experiment-htw-go-swarm) | Go | Jun 2 | Hunt the Wumpus in Go built with SwarmForge. |
| [experiment-htw-clj-swarm](https://github.com/unclebob/experiment-htw-clj-swarm) | Clojure | Jun 2 | Hunt the Wumpus in Clojure built with SwarmForge. |
| [gospringies](https://github.com/unclebob/gospringies) | Go | May 23 | (no description) |
| [mutate4go](https://github.com/unclebob/mutate4go) | Go | May 23 | (no description) |
| [crap4go](https://github.com/unclebob/crap4go) | Go | May 21 | (no description) |
| [sf-orbit-simulator](https://github.com/unclebob/sf-orbit-simulator) | Java | May 16 | (no description) |
| [dry4go](https://github.com/unclebob/dry4go) | Go | May 11 | (no description) |
| [dry4java](https://github.com/unclebob/dry4java) | Java | May 11 | (no description) |
| [skillBoard](https://github.com/unclebob/skillBoard) | Clojure | May 9 | Skill Aviation flight schedule board. |
| [spacewar](https://github.com/unclebob/spacewar) | Clojure | May 5 | Space War starting in Episode 55 of cleancoders.com. |
| [fitnesse](https://github.com/unclebob/fitnesse) | Java | Apr 20 | FitNesse — The Acceptance Test Wiki. |
| [craftsman-series](https://github.com/unclebob/craftsman-series) | None | Apr 13 | Unclebob's “Craftsman” series of episodes from 2002–2010. |
| [arch-view](https://github.com/unclebob/arch-view) | Clojure | Mar 20 | A clojure tool that puts up an interactive viewer of the structure of a clojure project. |
| [scrap](https://github.com/unclebob/scrap) | Clojure | Mar 17 | Tool for assessing whether, where, and how speclj specs should be refactored. |
| [AIR-J](https://github.com/unclebob/AIR-J) | Clojure | Mar 16 | A simple language for AIs to use and humans to ignore. JVM based. |
| [mutate4java](https://github.com/unclebob/mutate4java) | Java | Mar 14 | Mutation testing tool for my java projects. |
| [crap4java](https://github.com/unclebob/crap4java) | Java | Mar 13 | Crap analyzer for my java projects. |
| [Pharaoh-js](https://github.com/unclebob/Pharaoh-js) | JavaScript | Feb 22 | Old Pharaoh game from the 80s generated from gherkin by Claude in Javascript. |
| [Pharaoh](https://github.com/unclebob/Pharaoh) | Clojure | Feb 21 | My old Mac game from the '80s. Rewritten in Clojure by Claude. |

### Categories that emerge

- **AI / agent coordination**: `swarm-forge`, `experiment-htw-*-swarm`, `AIR-J`, `speclj-structure-check`
- **Test quality metrics**: `crap4java`, `crap4go`, `crap4clj`, `mutate4java`, `mutate4go`, `clj-mutate`, `negative-test-experiment`
- **Architecture / dependency tooling**: `arch-view`, `dependency-checker`
- **Acceptance / specification**: `Acceptance-Pipeline-Specification`, `fitnesse`
- **Games / graphics / examples**: `missile-command`, `spacewar`, `Pharaoh`, `Pharaoh-js`, `empire-2025`
- **Refactoring / code health helpers**: `scrap`, `dry4java`, `dry4go`, `dry4clj`, `deintroverter4clj`
- **Course / website / legacy**: `ubc-website`, `craftsman-series`, `springslim`, `skillBoard`, `sf-orbit-simulator`, `htw-6-clj-vid`, `htw-clj-six-pack`

### Repositories updated in 2026 but not pushed (25)

These changed metadata (issues, settings, stars, etc.) but received no code commits in 2026. They are mostly courseware, example code, or archived experiments and are unlikely to contain applicable new patterns unless a deeper historical review is requested.

`AdventOfCode2021`, `AdventOfCode2022`, `BoboliaTaxes`, `CC_SMC`, `CMuratori-discussion`, `Episode-10-ExpenseReport`, `Euler`, `FunctionalDesign`, `GoVideoStore`, `HTW`, `HTWCleanCoders`, `WTFisaMonad`, `Welc`, `clojureOrbit`, `empty`, `fitnesseextras`, `fitnessedotorg`, `iot-synapse`, `javaargs`, `more-speech`, `PPP`, `Sudoku`, `Unclebob.github.io`, `Videostore`, `Wator`.

### Methodology

- Data source: GitHub REST API `GET /users/unclebob/repos`.
- `updated_at` is GitHub’s “last metadata change” timestamp and is noisy; `pushed_at` is the last git push.
- Used `pushed_at >= 2026-01-01T00:00:00Z` as the applicability filter.
