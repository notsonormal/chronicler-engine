# Research 05: tla-ts in place of TLC

Written 2026-10-31 for [Research: tla-ts instead of TLC](../issues/05-research-tla-ts-instead-of-tlc.md).

Labels: **known** (read in the cited source, or in a run on this host), **inferred** (follows from known facts), **guessed**. Run outputs, the probe specs and the two scripts are in [05-tla-ts/](05-tla-ts/).

## 0. Answer in brief

tla-ts is a real TLA+ checker. It reads the `.tla` file and the `.cfg` file as they are, and it needs no rewrite in TypeScript (**known**). On the ticket 03 TLA+ model it gives the same answers as TLC on every config that I ran: the same number of distinct states, the same first broken rule and the same exit code, and the same trace on the three configs where I diffed it (**known**, section 3).

It cannot replace TLC in this repo today. The `statechart` skill reads three things that the published release of tla-ts does not write: TLC's `-tool` message framing, the `-dump dot,actionlabels` state graph, and SANY's XML tree (**known**, section 5). The maintainer has an open ticket for exactly this need (#228) and an open pull request that adds it (#232), and no release has it.

| | Keep TLC | Use tla-ts beside TLC | Use tla-ts in place of TLC |
|---|---|---|---|
| The `statechart` skill works | yes | yes, for the chart | **no**, it needs a new frontend |
| Needs a JDK | yes | yes | no |
| Same answers on the ticket 03 TLA+ model | — | yes | yes |
| Wall time, this host | — | about equal (section 4) | about equal |
| Peak memory, this host | — | tla-ts lower on small configs | tla-ts lower on small configs |
| Release history | TLC 2.19 of 2024-08-08, the current build | — | 5 days, one maintainer, `0.0.3` |

**Recommendation: keep TLC** (section 9).

## 1. What it is

| Question | Answer | Status |
|---|---|---|
| Is it a TLA+ checker? | Yes. It parses a TLA+ module, parses a TLC config, and searches the state space. | known (README, `src/check.ts`, `src/parallel.ts`) |
| Does it read `.tla` files? | Yes. `npx tla-ts Spec.tla` reads `Spec.tla` and the `.cfg` beside it, or the file named by `-config`. | known (README, run) |
| Must a TLA+ model be written again in TypeScript? | No. The ticket 03 TLA+ model ran unchanged. | known (section 3) |
| Is it a TypeScript DSL that is like TLA+? | No. A separate package, `tsprover`, compiles TypeScript models to TLA+ (`0.0.2` changelog). That package is not what the README of `tla-ts` describes, and this asset did not test it. | known (CHANGELOG) |
| What is its own description? | "A TLA+ interpreter and model checker in TypeScript, 2× faster than TLC". The README calls it "a drop-in replacement for TLC". | known (`package.json`, README) |
| Does it also print proof obligations? | Yes, `tla-ts prove` prints what `tlapm -N` prints, and runs `tlapm` from `PATH` otherwise. Not relevant to this map. | known (README, CHANGELOG `0.0.3`) |

Version tested: `tla-ts@0.0.3`, the only GitHub release (2026-10-28) and the npm `latest` tag. The npm registry also holds 67 `0.0.3-dev.*` builds, the newest at 2026-10-31T12:59Z (**known**, registry `time` field).

## 2. Coverage of the language

The ticket 03 TLA+ model uses: `EXTENDS Naturals, FiniteSets`; `CONSTANTS` with a different value in each config; `SPECIFICATION`; 15 `INVARIANTS`; functions; records; `EXCEPT` with `!`; sequences; sets; `\E` and `\A`; `CASE` and `IF`; string values; `BOOLEAN`; and the range `1..n`. It uses no temporal property, no fairness, no `VIEW`, no `SYMMETRY`, no `CONSTRAINT` and no PlusCal.

| Feature | In the ticket 03 TLA+ model | Supported by tla-ts `0.0.3` | Status |
|---|---|---|---|
| `CONSTANTS` per config | yes: the 16 configs of `configs/` hold different constant values | yes: the 11 configs that I ran each used their own values | known (run) |
| Invariants | yes, 15 | yes, each broken rule reported with its trace | known (run) |
| Temporal properties | no | yes, on a probe spec: `<>(x = 3)` with `WF_vars` gave the same 4 distinct states and the same verdict as TLC | known (`probes/Temp.*`) |
| Fairness (`WF_`, `SF_`) | no | yes, on the same probe | known (`probes/Temp.*`) |
| `VIEW` | no | yes, on a probe spec: 3 states generated and 2 distinct, as TLC | known (`probes/Cv.*`) |
| `SYMMETRY` | no | yes, on a probe spec: the same deadlock and the same trace as TLC | known (`probes/Sym.*`) |
| State constraints (`CONSTRAINT`) | no | yes, on a probe spec, and by the config parser | known (`probes/Cv.*`) |
| `ACTION_CONSTRAINT`, `POSTCONDITION`, `ALIAS`, `_POSSIBLE` | no | the config parser reads all four; this asset did not run them | known for the parser, untested for behaviour |
| PlusCal | no | no translator. `pcal.trans` lives in `tla2tools.jar`, so a PlusCal source still needs a JDK. The only PlusCal text in the package is a comment and a copied TLC error message. | known |

The standard modules, `TLCExt`, `Json` and `Randomization`, `INSTANCE` and `LAMBDA` are claimed by the README (**known** as a claim, not run here).

## 3. Equal results

Method: TLC 2.19 from `tla2tools.jar` 1.7.4 on Temurin 21, and `tla-ts@0.0.3` on Node 24.22.1, both with `-workers 1 -fp 0`, on the ticket 03 TLA+ model and its own `configs/*.cfg`. The scripts are [05-tla-ts/ab.sh](05-tla-ts/ab.sh) and [05-tla-ts/measure.sh](05-tla-ts/measure.sh). The raw outputs are in [05-tla-ts/runs/](05-tla-ts/runs/).

| Config | Distinct states (TLC) | Distinct states (tla-ts) | States generated | Depth | First broken rule | Exit |
|---|---|---|---|---|---|---|
| `run1-plain-jobs` | 116 | 116 | 285 | 6 | `R02` | 12 |
| `gap-g2-only` | 55 | 55 | 94 | 4 | `R04` | 12 |
| `run2-guard-off` | 38 | 38 | 64 | 4 | `R01` | 12 |
| `GateSpecOnly` (run 1) | 55 | 55 | 94 | 4 | `R04` | 12 |
| `gap-g1-only` | 137 | 137 | 276 | 5 | `R02` | 12 |
| `gap-g3-only` | 3,608 | 3,608 | 9,195 | 9 | `R10` | 12 |
| `run1b-c-restart` | 95,286 | 95,286 | 511,478 | 32 | none | 0 |
| `run1-c-rest` | 152,135 | 152,135 | 810,277 | 32 | none | 0 |
| `run1b-b-two-jobs` | 196,199 | 196,199 | 1,002,599 | 35 | none | 0 |
| `run1-b-rest` | 494,842 | 494,842 | 2,456,098 | 34 | none | 0 |
| `run2-b-rest` | 2,728,171 | 2,728,171 | 12,696,691 | 37 | none | 0 |

Both checkers gave the same value in every column. The stored TLC reports in [03-gate-spec-only/runs/](../03-gate-spec-only/runs/) give the same distinct-state numbers for the runs that they name, so the 2026-10-29 runs and these runs agree (**known**).

**The traces.** A line-by-line diff of the `gap-g3-only` output shows one difference in the eight-step trace, and it is not in the states:

| Line | TLC | tla-ts |
|---|---|---|
| The action name of a step | `State 3: <TheInputIsSaved line 261, col 3 ...>` | `State 3: <TheInputIsSaved(1) line 261, col 3 ...>` |

tla-ts writes the arguments of the operator in the action name; TLC writes the name alone. Every state, every variable, their order and their indentation are equal. The remaining lines of the two files differ only in TLC's startup line and in the summary lines above. The `gap-g1-only` and `run2-guard-off` traces differ in the same way and in no other way (**known**, `runs/`).

Two smaller differences: tla-ts leaves out TLC's "average outdegree" line and TLC's `Starting... (timestamp)` line (**known**).

**Rules that a single run does not report.** Both checkers stop at the first broken rule, so one run of `GateSpecOnly.cfg` reports `R04` only. The five breaks in the ticket 03 report come from the skill's method: remove the broken rule and run again (**known**, `SKILL.md`). Both checkers need that method. tla-ts also ignores `-continue`, so it cannot report every broken rule in one run (**known**, issue #230).

## 4. Speed and memory

Hardware: AMD Ryzen 3 3200U, 4 threads. The container has a 3-CPU quota and an 8 GiB memory limit, and other work shares the host (`ENVIRONMENT.md`). Wall time includes JVM and Node startup. Peak RSS is the high-water mark of the process tree, polled every 200 ms.

| Config | Distinct states | TLC wall | tla-ts wall | TLC peak RSS | tla-ts peak RSS |
|---|---|---|---|---|---|
| `run1-plain-jobs` | 116 | 4.4 s | 1.0 s | 108 MB | 81 MB |
| `gap-g3-only` | 3,608 | 8.8 s | 1.7 s | 132 MB | 91 MB |
| `run1b-c-restart` | 95,286 | 18.9 s | 15.1 s | 661 MB | 239 MB |
| `run1-c-rest` | 152,135 | 24.9 s | 24.6 s | 802 MB | 271 MB |
| `run1b-b-two-jobs` | 196,199 | 32.2 s | 28.3 s | 809 MB | 286 MB |
| `run1-b-rest` | 494,842 | 40.1 / 51.8 / 51.9 s | 51.0 / 58.4 / 101.9 s | 846 MB | 411 MB |
| `run2-b-rest` | 2,728,171 | 356.0 s | 281.5 s | 1,053 MB | 1,166 MB |

**Read this table with care.** The host is shared, and `run1-b-rest` gave 40 s, 52 s and 52 s for TLC and 51 s, 58 s and 102 s for tla-ts in three passes of the same config (**known**). So the wall time of the two checkers is about equal on this host, and the README's claim of "2× faster" was not reproduced. The README measures an Apple M4 Pro with 8 workers; this host has 3 usable cores.

Memory is the clear difference. tla-ts used 34 % to 75 % of TLC's peak RSS on every config below 500,000 states. On the 2.7-million-state config the two are equal, and tla-ts was higher (**known**).

## 5. The `statechart` skill

The skill does not work with tla-ts `0.0.3`. It reads TLC's output at the message level, and tla-ts writes plain text.

| What the skill reads | Where | tla-ts `0.0.3` |
|---|---|---|
| TLC's coded messages, `@!@!@STARTMSG <code>:<level> @!@!@` | `scripts/frontends/messages.ts`, `readMessages` | writes no message codes at all. A `-tool` flag is accepted and ignored. |
| The name of the broken invariant, by message code | `scripts/frontends/verdict.ts`, `NAMED` map on `EC.TLC_INVARIANT_VIOLATED_BEHAVIOR` | writes `Error: Invariant R04_... is violated.` as plain text, with no code |
| The distinct-state count, by message code | `scripts/frontends/verdict.ts`, `DISTINCT` map on `EC.TLC_STATS` | writes `94 states generated, 55 distinct states found` as plain text |
| The state graph, `-dump dot,actionlabels <file>` | `scripts/run.ts`, the chart run | **no dump at all** in `0.0.3`. The flag is accepted and no file is written. |
| SANY's XML tree, `tla2sany.xml.XMLExporter` | `scripts/run.ts`, for the variable list and the rule kinds | no equivalent |
| The action name of a trace step | `scripts/frontends/trace.ts`, `actionOf` | writes `TheInputIsSaved(1)`, so the regex `^\d+: (?:Back to state: )?<([^\s>]+)` returns `TheInputIsSaved(1)`, and a view's `step()` switch on `"TheInputIsSaved"` misses |
| `CONSTRAINT ChartBound` bound runs | `scripts/run.ts`, `boundRun` | works: tla-ts supports `CONSTRAINT` |
| `-workers 1`, `-lncheck final` | `scripts/run.ts`, `PINNED_FLAGS` | both accepted |
| `-fp 0` | `scripts/run.ts`, `PINNED_FLAGS` | accepted and ignored; tla-ts has no switch for the fingerprint set |

**The changes that the skill needs** are therefore: a second output frontend that reads tla-ts's plain-text verdict and statistics, a graph source other than `-dump`, a replacement for the SANY XML read, and a trace-action-name normaliser. The `SKILL.md` prose, the view contract (`state()`, `step()`, `group()`, `colors`, `title`) and the Mermaid and report backends need no change, because they read the skill's own intermediate form and not TLC's text (**known**, `scripts/ir.ts` and `scripts/backends/`).

**The dump test.** I ran `tla-ts GateSpecOnly.tla -config probe-small.cfg -dump dot,actionlabels <file>`, and the same command through the `check({ dump })` API with a `writeFile` that prints. Neither wrote a file, and neither reported an error (**known**, `runs/tlats-probe.txt` against `runs/tlc-probe.txt`, where TLC wrote a 1.6 MB `.dot`). [`probes/dump-probe.mjs`](05-tla-ts/probes/dump-probe.mjs) is the API call. `probe-small.cfg` is the ticket 03 config with `MaxJobs = 1` and `MaxRestarts = 0`, and both checkers gave 1,339 distinct states on it.

**The maintainer knows.** Issue [#228](https://github.com/aidenybai/tla-ts/issues/228), "Check command: add TLC's `-tool` output so the statechart skill can run on tla-ts", names this skill and its flags. Pull request [#232](https://github.com/aidenybai/tla-ts/pull/232) implements it: 116 files, 30,536 additions, 49 commits, open since 2026-10-30 (**known**, GitHub API).

## 6. Toolchain fit

| Question | Answer | Status |
|---|---|---|
| Runtime | Node. The CI matrix uses `actions/setup-node` with `node-version: 22`. It also runs in Bun, Deno and a cross-origin-isolated page. | known (`.github/workflows/ci.yml`, README) |
| Declared engine | none: `package.json` has no `engines` field. | known |
| npm packages | one. `npm install tla-ts@0.0.3` added 1 package and no dependency. | known (run) |
| Install command | `npm install tla-ts`, or `npx tla-ts Spec.tla` | known (README, run) |
| Does this repo have Node? | yes: Node 24.22.1 and npm 11.19.1 on this host. The repo root has a `package.json` with `pnpm@9.15.4` for browser tooling, and the vendored skill's own `scripts/package.json`. | known |
| Does `build.py` use Node? | no. No line of `build.py` names `node`, `npm` or `pnpm`. | known (grep) |
| Can `mise` install it? | yes. `mise` can install a Node toolchain, and the skill's runner already installs Java through `mise` (`scripts/run.ts`, `JAVA_TOOL = "java@temurin-25"`). | inferred |
| Can `build.py` install it? | yes, as one more step: `npm ci` or `npx --yes tla-ts@<pinned>`. It would be the first Node step in `build.py`. | inferred |

The JDK cost that [the tying research](02-tying-a-tla-model-to-code.md) put on its places 2 and 3 is the cost of a JDK **in the build**. A check of the TLA+ models alone needs no JDK with tla-ts (**known**). A **chart** from the `statechart` skill still needs TLC and therefore a JDK (**known**, section 5).

## 7. Maturity

| Item | Value | Status |
|---|---|---|
| First commit | 2026-10-26, five days before this research | known (GitHub API) |
| Commits | 414 | known (GitHub API) |
| Contributors | 1 (`aidenybai`, 414 commits). No `git shortlog` history in the clone, because it is shallow. | known (GitHub API) |
| Stars, forks | 114, 6 | known (GitHub API) |
| Releases | 1 GitHub release (`tla-ts@0.0.3`, 2026-10-28). npm: `0.0.1`, `0.0.2`, `0.0.3`, plus 67 `0.0.3-dev.*` builds in five days. | known (GitHub API, npm registry) |
| Open issues | 13, of which 8 are features of its own proof tooling | known (GitHub API) |
| Licence | MIT | known (`LICENSE`, `package.json`) |
| Test suite | 39 unit test files, 44 fixture folders, a conformance suite with **823 recorded TLC results** over 17 upstream repositories | known (`tests/`, `tests/conformance/expected.json`) |
| Evidence that results were checked against TLC | yes, and it is the design: `scripts/record-tlc.ts` runs TLC and records `status`, `generatedStates`, `distinctStates`, `depth` and `tlcMilliseconds` per corpus TLA+ model; `AGENTS.md` says "Match TLC's semantics. When behavior is unclear, run the spec through TLC and compare." | known |
| Known gaps against TLC | `-dump` and `-tool` are missing from the release (#228); `-continue` is accepted and ignored (#230) | known |

**The 823 recorded TLC results are the strongest maturity signal.** They are the reason the equal state counts of section 3 are not a surprise.

**The weakest signal is the release.** `0.0.3` is three days older than the code that this asset read from the repository head, and the difference matters: the published `0.0.3` tarball has no `state-graph.ts` and no `-dump` code at all, although the README at the repository head documents `-dump` (**known**, `node_modules/tla-ts/` against the clone).

## 8. Other checkers

| Checker | Runtime | Changes the answer? | Status |
|---|---|---|---|
| Apalache | JVM. Its README offers "the jar from the most recent release" and a Docker image, and its installation manual has a JVM page. | No. It needs a JDK too, and it is a symbolic checker with a bounded search, so it would give different state counts. | known for the JVM |
| TLAPS / `tlapm` | OCaml and external provers; `tla-ts prove` calls it from `PATH`. | No. It proves, it does not enumerate states. | known (README) |
| SANY | JVM, inside `tla2tools.jar`. | No, and the skill needs it today. | known (skill runner) |
| `tla-ts` | Node, no JDK. | Yes, it is the only checker here without a JDK. | known |

So tla-ts is the only checker without a JDK that this map can use. That is why the ticket is worth its answer.

## 9. Recommendation

**Keep TLC.**

The reason is section 5. The method that [the tying research](02-tying-a-tla-model-to-code.md) puts first is the skill's method plus a rule-id pairing, and the skill needs TLC's `-tool` messages, its dot dump and SANY's XML. tla-ts gives none of the three in a release. The equal results of section 3 show that tla-ts *could* take TLC's place, not that it can today.

Three facts keep this recommendation open to change:

- **A release that ships #232 changes the answer.** It adds `-tool` and SANY's XML export. The skill would then need only a graph source and a trace-name normaliser.
- **The check alone needs no JDK.** A repo-local `build.py` step that checks the TLA+ models, without a chart, can use `npx --yes tla-ts@0.0.3` today. The wall time is about equal to TLC on this host, and the memory is lower (section 4).
- **The equality is measured, not claimed.** Eleven configs of the ticket 03 TLA+ model agree on every count, on the broken rule and on the exit code, and three of them agree on every line of the trace.

**Which "Not yet specified" items this answer changes.** One: **The toolchain.** It now has two parts, and the answer separates them.

| Part | Answer after this research |
|---|---|
| The check of a TLA+ model | tla-ts `0.0.3`, Node only, no JDK. Same answers as TLC. |
| The chart from the `statechart` skill | TLC 1.7.4 and a JDK 11+, as today, until a tla-ts release ships #232. |

No other "Not yet specified" item changes: the pairing site, the repo-local addendum, the trigger for a new audit and the provenance of the vendored skill do not depend on which checker runs.

**One correction to the map's Notes.** The **Toolchain** note says "This host has neither" a JDK nor `tla2tools.jar`. On 2026-10-31 both are still in `/tmp/statechart-tools/` (Temurin 21.0.13 and `tla2tools.jar` 1.7.4), so this research ran TLC again instead of reading the stored reports (**known**).

## 10. Make the runs again

```sh
mkdir -p tmp/tla-ts-run && cd tmp/tla-ts-run
npm init -y && npm install tla-ts@0.0.3
cp ../../.scratch/statechart-models/assets/03-gate-spec-only/GateSpecOnly.tla .
cp ../../.scratch/statechart-models/assets/03-gate-spec-only/configs/*.cfg .
cp ../../.scratch/statechart-models/assets/05-tla-ts/ab.sh .
cp ../../.scratch/statechart-models/assets/05-tla-ts/measure.sh .
export JAVA=/tmp/statechart-tools/jdk/bin/java TLA2TOOLS=/tmp/statechart-tools/tla2tools.jar
./ab.sh run1b-c-restart
```

`ab.sh` runs both checkers on one config and prints the wall time, the peak RSS and the state counts. `measure.sh <out> <command>...` is the timer that `ab.sh` uses.
