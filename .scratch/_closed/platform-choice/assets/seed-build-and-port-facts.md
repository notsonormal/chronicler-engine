# Seed: build, test and port facts from the ticket-17 grilling

Gathered on 2026-10-12, during [Decide whether and how to run a clean-room design of the generation model](../../turn-state-model/issues/17-decide-clean-room-run.md). Tree `9cbf3112`.

Labels: **known** = read in a file or measured from a log in that session. **inferred** = follows from known facts, not run. **guessed** = not supported by a source.

## 1. Build and test timing

Source: `logs/build_history.txt`, 1,000 rows from 2026-10-03 to 2026-10-11, only the runs that passed. The medians come from a short script. Host load varies, so treat gaps under 10 % as noise (`ENVIRONMENT.md`).

| Step | Runs | Median | p90 | Label |
|---|---|---|---|---|
| `test-pattern` (targeted tests) | 238 | 3.3 s | 30.6 s | known |
| `clippy` (compile and lint) | 82 | 13.0 s | 26.0 s | known |
| `check` | 53 | 7.0 s | 16.4 s | known |
| `integration` | 13 | 22.5 s | 46.9 s | known |
| `guardrails` | 32 | 8.5 s | 15.3 s | known |
| `browser` | 68 | 131.4 s | 164.4 s | known |
| Full gate | 75 | 161.3 s | 252.9 s | known |

- The slow `test-pattern` runs come after a Rust edit. One example: 53.2 s for the first run after an edit, then 2.6 s and 2.4 s (known for the example; inferred for the pattern).
- The browser tier is about 80 % of the median gate. It drives a real Chromium, so a new language keeps most of that time (inferred).

## 2. The host

Source: `ENVIRONMENT.md` (known).

- A 3-CPU quota on a shared 4-thread host. Other work often saturates it.
- 8 GiB of memory. This limit bites first. Three overlapping builds ran 2.1 times slower than the same builds one after another.
- One heavy cargo step runs at a time across all checkouts (`scripts/build_slot.py`), so parallel agents wait for each other.
- Mitigations already in place: the lld linker, sccache, seeding a cold target dir from a warm sibling, and nextest. A seed miss in a new worktree cost 728 s, and a seeded one cost 344 s.

## 3. What a port must carry

| Fact | Value | Label |
|---|---|---|
| Production code | 49,000 lines of Rust in 327 files under `src/` | known |
| Test code | 27,000 lines of Rust under `tests/` | known |
| Test counts | About 1,640 integration, 163 guardrails, 29 browser (`logs/build_history.txt`, the tests column) | known |
| HTTP tests | They call the router in-process: 174 `oneshot` calls, 9 `reqwest::Client` uses and 4 `TcpListener::bind` | known |
| Browser tests | `playwright-rs`, a Rust crate | known |
| Behaviour record that does not depend on the language | 26 specs in `docs/specs/`, about 286 scenarios | known |
| Subsystems with no spec | The Quantifier, Triggers, prompt assembly and storage (turn-state-model research 16 §3.1) | known |
| Dependencies tied to Rust | `harper-core`, `askama`, `rusqlite` (bundled SQLite), `axum`, `arch-lint`, `syn` (`Cargo.toml`) | known |
| A JavaScript build of Harper | It may exist. Not checked. | guessed |
| Tooling tied to Rust | `build.py`, `scripts/build_slot.py`, `scripts/target_seed.py`, `scripts/check_test_structure.py`, `scripts/generate_structure_index.py`, `CODING_STANDARDS.md` and the Rust skills (for example `antipattern-checker`) | known |

**Inferred:** today no test can check a server written in another language. A rewrite that keeps the specs as its oracle needs black-box tests over HTTP and the browser first.

## 4. Rust options not tried yet

Neither option is measured. Both are guesses about where time could go.

- The Cranelift codegen backend for dev builds. It needs nightly Rust.
- A split of the crate into several crates, so that an edit recompiles less code.

## 5. The owner's answers

- Q4 (2026-10-12): a language move must solve the edit-to-test loop, parallel agents that wait for the build slot or run out of memory, and design churn and defects.
- "Performance isn't really that important for this kind of application."
- "I'm not sure how much a clean room design would really fix, compared to starting fresh with a clean plan, stronger models and a stronger understand on how to build an ai native project."
- The answer given in the grilling (inferred): a design gets its fixes from the spec, not from the wall. The wall adds a design with no anchor on the code, and a test of the spec. A rewrite needs a complete spec, because the spec becomes its test oracle.
