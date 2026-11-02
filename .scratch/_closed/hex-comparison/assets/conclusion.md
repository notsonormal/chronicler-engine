# Conclusion: what Chronicler learned from Hex

Status: final. The owner closed the map on 2026-11-01.

Subject: [anomalyco/hex](https://github.com/anomalyco/hex) at `7609574ca92c` (2026-10-31), compared with this repo on 2026-11-01. Labels: **known** means a file, a command or a commit shows it. **Inferred** means a conclusion from that evidence.

## Summary

- Hex does not measurably keep higher code quality than Chronicler. Its fix rate is about the same, and by Chronicler's rules its code is looser (inferred, parts 1 and 3).
- Hex keeps the app working because it finds a fault fast, limits the damage, and repairs it fast. It does not prevent more faults (inferred, part 2).
- Change size does not explain quality. Larger commits do not need more fixes per changed line, in either repo (known, part 3).
- Of 52 Hex practices, the map adopted none. Each one is covered in Chronicler, measured and found small, or dropped (part 4).
- The difference that remains is real use soon after each change. Its value depends on how often the owner uses Chronicler (inferred, part 2).

## 1. Is Hex high quality?

The public data does not show that it is.

| Sign | Hex | Chronicler | Source |
|---|---|---|---|
| Repaired lines per 1000 changed lines (SZZ) | 22.0 | 17.8 | [ticket 21](./research-21-change-size-and-fixes.md) (known, wide limits) |
| Test attributes per 1000 Rust lines | about 8 (2216 in 263k lines) | about 20 (2000 in 98k lines) | count of `#[test]` and `#[tokio::test]` on 2026-11-01 (known, rough) |
| Largest file | 6120 lines | capped at 2000 by a guardrail | [research 05](./research-05-code-and-test-style.md) |
| Error type | `color_eyre` strings | `thiserror` enums | research 05 |
| Lints | env and clock ban turned off, 85 errors left | 55 guardrail rules enforced | research 05 |
| Doc drift | 11 of 611 named checks no longer resolve | spec-to-test link enforced by the gate | [research 04](./research-04-feature-map.md) |

## 2. How Hex keeps the app working with fast changes

Four layers. Most of the work happens after the merge.

1. **A narrow task and the rules every time (known).** The issue form requires the behaviour and the steps to reproduce it ([research 02](./research-02-issue-pipeline.md)). `AGENTS.md` loads into every session, with 73 invariants, 31 of them with exact values ([research 01](./research-01-agent-instructions.md)). A rule enters with its incident and leaves with its feature. Tests name the invariant with its number and check the values around it (research 05).
2. **Automatic checks (known).** `check.ts` sorts each diff by path and size with no AI, from the base branch. A flagged diff needs a second model's approval. CI builds and tests on Linux, macOS and Windows ([research 06](./research-06-branch-to-installed-app.md)). 122 golden images run only in `check-linux.yml` (`AGENTS.md:13-17` of the clone).
3. **New features are off by default (known).** New, unproven and pipeline-built features land in Labs ([research 07](./research-07-public-sources.md), row 13).
4. **Fast detection and repair after the merge.** A push to `dev` publishes a dev-channel build (known). `.hivemind.toml` verifies, builds and installs the app on the developer's machine (known, config only). The developers probably use Hex every day (inferred, research 07, row 19). A repair is a small commit, and a rollback promotes an earlier build (known, research 06).

**The gates by path (known, from the clone):**

| Path | What runs | Golden images? |
|---|---|---|
| `scripts/check` | fmt, clippy, every test through nextest | only on Linux |
| Land queue, `.hivemind.toml` | `cargo test --locked --workspace --lib`, app build, install | no (inferred: the golden tests are in `--bin hex`) |
| CI `check-linux.yml` | nextest on `--bin hex`, clippy, Nix, install and service smoke tests | yes, only here |
| CI `check-macos.yml`, `check-windows.yml` | fmt, clippy, nextest, build | no |

Agent PRs merged a median of 3 s after they opened (known), so CI and the golden images checked the agent pipeline after the merge. 8 of 59 pipeline commits are `fix(ci)` repairs (known). Before the merge, the gates were `check.ts`, the second-model review and the lib tests of the land queue.

**Not researched.** `scripts/smoke-macos-app.sh`, `scripts/smoke-embedded-service.sh`, `scripts/dictation-corpus.sh` and `scripts/capture-dictation-fixtures.sh` suggest smoke tests and a corpus from real audio (inferred from the names). The private part, `~/code/control` and the criteria of the second-model review, is not public. If Hex has more quality controls, they can be there.

**Hex and Chronicler side by side (inferred):**

| | Hex | Chronicler |
|---|---|---|
| Main defence | find the fault fast, limit it, repair it | prevent the fault |
| Where the strictness is | prose in `AGENTS.md`, read every session | tools: guardrails, specs, gate |
| Who meets a fault | the developers, in daily use, within hours | the gate, or the owner in a later session |

## 3. Change size and fixes

From [research 21](./research-21-change-size-and-fixes.md) (known, SZZ on `fix` and `Revert` commits):

- Hex pipeline commits have a median of 80 changed lines (p90 253). Chronicler commits have 377.5 (p90 3049).
- Repaired lines per 1000 changed lines do not grow with size. Commits of 1000 lines or more have 0.67 times the rate of smaller ones in Chronicler (95% limits 0.38 to 1.13), and 0.64 in Hex (0.41 to 1.00).
- A large commit needs more fix commits only in proportion to its lines.
- The largest Chronicler commits are plan stages: batches of tickets, refactors, migrations and generated files.
- Limits: only faults that someone fixed count. 70% of Chronicler's repaired lines are in tests. After the 14-day cut-off, 196 Hex commits are left.

## 4. The practices and their end state

[candidates.md](./candidates.md) holds the 52 practices. Ticket 08 found 11 covered and dropped 19. The other 22 ended as follows. None is adopted.

| Practices | End state | Reason | Ticket |
|---|---|---|---|
| C-01, C-03, C-09, C-10, C-11 | covered | specs, scenario numbers, `spec-coverage` and the `docs/AGENTS.md` rules already do the work | [20](../issues/20-task-recommend-the-doc-practices.md), [19](./research-19-docs-mislead.md) |
| C-02, C-32 | dropped | a spec scenario and its `SCENARIO` tag already pin a value | 20 |
| C-12 | dropped by the owner | a fault that a manual check finds becomes a test, so a record of a pass adds nothing | 16 |
| C-13, C-14 | not adopted | no case is known where an agent was misled by a missing gap or limit | [12](./research-12-evidence.md) |
| C-15 | not adopted | the test works at threshold 0.05, but no case is known of a CSS regression that the checks missed | [15](./task-15-golden-image.md) |
| C-20 | not adopted | no breakage is known from commits whose relevant checks ran on an earlier tree | [13](./task-13-work-report.md) |
| C-21 | not adopted | a scoped gate saves 3.7% of gate time | [14](./research-14-gate-scope.md) |
| C-30, C-31 | not adopted | the hook traits are a design preference; the env and clock ban does not catch the date-test defect | [11](./task-11-env-clock-ban.md) |
| C-42, C-44 | dropped | agents work inside one effort folder and meet decisions in tickets | [10](./research-10-agent-use-of-docs.md) |
| C-43 | not adopted | no problem is known with the current plan form | 10 |
| C-17, C-18, C-19, C-50 | no change | change size does not affect the fix rate | 21 |

## 5. Small facts found on the way

None is a practice. Each one can be fixed at any time, if the owner wants it.

- Three tests fail if the UTC date changes during the test: `src/domain/model/game_tests.rs:15` and `:23`, and `src/application/games/catalogue_tests.rs:162`. The cause is that `generate_game_name` reads the date itself (known, ticket 11).
- `AGENTS.md` gives the full gate as about 145 s and the browser step as 92 s. The medians are 330 s and 234 s (known, ticket 14).
- About 750 s went to gate runs that repeated a passing run on the same tree (known, ticket 14).
- Tracked notes cite 207 `tmp/` paths, and 73 of them no longer exist (known, ticket 12).
- `scripts/trial.py` does not record the engine commit in its reading pack (known, ticket 12).
- The convergence check fails on the empty `style` attribute that Playwright leaves after a screenshot (known, ticket 15).

## 6. Owner positions recorded

- **Real providers** (2026-11-01). A run with a real provider costs money and is slower, and no evidence shows that it finds more bugs. When a trial or a manual check finds a fault, find why the tests did not catch it, and add the test. Autoplay and `chronicler-ui-investigator` stay available for a manual check.
- **Docs** (2026-11-01). An AI writes and reads the docs. No rewrite that rests on a hypothetical.
- **The aim of the map** (2026-11-01). Find what keeps Hex's code good while several developers build it fast with agents.

## 7. Lessons for later comparison maps

- Measure the other project's quality first. This map assumed that Hex had higher quality, and it measured that only in ticket 21, near the end.
- Look for the mechanism of the whole system, and test it with one cheap measure, before the map splits the other project into single practices. Ticket 21 took one AFK session and settled the main question. The 20 tickets before it each judged one practice alone.
- Judge each practice by a problem that someone has seen, not by how much it could help.
