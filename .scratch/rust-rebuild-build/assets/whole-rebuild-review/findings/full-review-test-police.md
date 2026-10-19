## Test-police review — Rust rebuild test suite

### Run results (my own target dir)

| Run | Command | Log | Result |
|---|---|---|---|
| Integration (1) | `python build.py --target-dir target/test-police integration` | `logs/build_20261019_223527.log` | **1337 passed, 0 failed, 2 skipped** (the 2 skips are the `#[ignore]`d `tests/llm/` tests); 26.1 s suite, 89 s step |
| Integration ×4 more | same, repeated | `tmp/test-police/repeat_1..4.log` | 1337 passed each |
| Integration under saturation | same, with 4 `yes` CPU burners on a 3-CPU quota | `tmp/test-police/loaded.log` | 1337 passed — the 1 s waits held |
| Guardrails | `python build.py --target-dir target/test-police guardrails` | `logs/build_20261019_223819.log` | **189 passed** |
| Full gate + coverage | `python build.py --coverage --target-dir target/test-police-cov` | `logs/build_20261019_224303.log` | clippy, data, docs, spec-coverage, seam-cases, `nextest: 1526 passed, 0 failed, 2 skipped` (integration w/ coverage), `29 passed` (browser) — **coverage 10229/11293 = 90.6 %, meets 80 %** |

**No flake observed** in 6 integration runs, including one under CPU saturation (known: the runs above). `.config/nextest.toml` keeps `Retries stay 0`, so nothing is masked by retries (known).

---

### Findings, by severity

#### 1. [Medium] The `options` step of `/status/generating` is asserted nowhere, and spec 39.4's row for it is skipped
- **Where:** `tests/http/races.rs::run_stop_row` (via `STOP_ROWS`, `tests/http/races.rs:691–699`)
- **Problem:** the Examples table of spec 39.4 declares the status while the Options call is held as `options`. The test row is the only one with no status assertion, and the comment contradicts the spec:

```rust
    StopRow {
        kind: JobKind::Narration {
            fires_trigger: false,
        },
        held: ModelRole::Options,
        // The design names the status of the generation's own steps only.
        status: None,
        after_stop: StoryLogAfterStop::InRoom { room: MOVED_ROOM },
    },
```

- **Evidence (known):** `docs/specs/dashboard.md:71` — `| Narration | the Options call | options | the Input of the Job, the Narration, and the room the Narration arrived in |`. Production answers exactly that string: `src/application/pipeline/job.rs:520` `self.run_step(Step::Options, kind, || {` and `src/domain/model/state/generation_status.rs:177` `Self::Options => "options",`. All 6 other rows carry `status: Some(...)` (`narrating`, `quantifying`, `generating-event`). `"options"` appears in no `/status/generating` assertion in `tests/`, and `as_endpoint_str` has no unit test.
- **Fix:** `status: Some("options")` on that row (known to be the declared value; inferred that the assertion passes, since the row's other assertions already prove the Options step ran).

#### 2. [Medium] A panicking Job is invisible to tier-1 tests; "nothing was written" rows can pass on a crash
- **Where:** `src/application/generation/job_tasks.rs` (the harness every tier-1 wait uses) and the rows of `tests/http/races.rs::assert_story_log_after_stop` with `StoryLogAfterStop::NoMessage`.
- **Problem:** a Job runs in `spawn_blocking` (`job.rs:169`). Its panic arrives as a `JoinError`, which is logged and dropped:

```rust
                // A Job that panicked failed its own test already; this only orders
                // the test after the task.
                if let Err(join_error) = handle.await {
                    tracing::debug!("a Job task ended without a result: {join_error}");
                }
```

`watch` also drops finished handles without awaiting them (`handles.retain(|handle| !handle.is_finished());`), so a Job that panicked before the next `watch` call is never reported. `wait_settled` only fails a test on an *unscripted model call*, so nothing observes the panic.
- **Evidence (known):** the coverage report shows `job_tasks.rs` 24/27 lines with uncovered lines `[38, 39]` — the panic arm is executed by no test. The affected assertion is:

```rust
        StoryLogAfterStop::NoMessage => {
            assert_eq!(
                after_texts, before_texts,
                "{label}: a stopped Job saves no Message of its own"
            );
```
A Job that crashed before writing satisfies this exactly as a refused one does.
- **Fix:** make `wait_for_each` fail on `Err(join_error)` (panic with the message, or return a report that `wait_settled`/`wait_for_jobs` asserts), and make `watch` keep and check finished handles. Add a case proving it.

#### 3. [Medium] `newest_game_of_world_falls_back_to_updated_at` cannot pin `updated_at`
- **Where:** `tests/storage/transaction_storage.rs:220::newest_game_of_world_falls_back_to_updated_at` (CASE `storage-newest-game-updated-at`)
- **Problem:** the expected winner has both the higher id and the later `updated_at`, so ordering by id alone satisfies the assertion. The CASE says "without a Message, `updated_at` answers".

```rust
    let with_old_message = create_game(&storage, "With old message");
    insert_message_at(&storage, with_old_message, PAST_MESSAGE_TIME);
    let without_message = create_game(&storage, "Without message");

    assert_eq!(
        storage.newest_game_of_world(WORLD_KEY).unwrap(),
        Some(without_message),
        "a Game with no Message is keyed by `updated_at`"
    );
```
It does distinguish "has a Message" from "no Message", but not `updated_at` from `id`.
- **Evidence (known):** `src/adapters/driven/storage/games.rs:121` orders `COALESCE(m.last_message, g.updated_at) DESC, g.id DESC`. `src/adapters/driven/storage/core.rs:213` `pub fn touch_game_updated_at(&self, game_id: GameId)` sets `updated_at` to now, so the fixture can flip the winner.
- **Fix:** after creating both, call `storage.touch_game_updated_at(with_old_message)` and assert `with_old_message` wins — the lower id then wins only if `updated_at` decides (inferred that this passes).

#### 4. [Low] Three seam cases named "recorded with its failure text" assert only that the text exists
- **Where:** `src/application/agents/options/agent_tests.rs:234::test_a_failed_call_is_recorded_with_its_failure_text`, `src/application/agents/quantifier/agent_tests.rs:281::test_a_failed_call_is_recorded_with_its_failure_text`, `src/application/narration/narrator_tests.rs:1077::test_a_failed_call_is_recorded_with_its_failure_text`
- **Problem:** the declared case promises the failure *text*; the assertion checks presence only, and the options case discards the outcome:

```rust
    let _ = agent.run(&scene.input(&world), &AgentSettings::default());
    ...
    let failure = recorded
        .iter()
        .find(|message| message.error_message.is_some())
        .expect("the failed attempt is recorded");
    assert_eq!(failure.agent_name, "options");
```
- **Evidence (known):** the recorder stores `error_message: Some(error.llm_error_string())` (`src/application/llm_recorder.rs:111`); tier 1 pins the rendered text (`tests/http/llm_messages.rs` asserts `.contains("configured_failure")`), so the seam case is the weaker one of the pair.
- **Fix:** assert `failure.error_message.as_deref().unwrap_or_default().contains("configured_failure")` and `assert!(agent.run(...).is_err())`.

#### 5. [Low] Spec 3.5's "opening Narration and exactly one Narration" clause has no direct assertion
- **Where:** `tests/http/races.rs::test_two_actions_at_the_same_moment_save_one_input_http`
- **Evidence (known):** `docs/specs/actions.md:214` — `And the story log holds the opening Narration and exactly one Narration of that Action`. The test asserts the refusal count, `texts_of(..., Input) == ["look"]` and `llm_call_count(&state, "narrator") == 1`, but never the Narration count. The sibling scenario 3.4 test does assert it (`assert_eq!(narrations, 2, ...)`).
- **Fix:** add the same narration-count assertion (`vec!["<scripted>"]` after the opening).

#### 6. [Low] The storage seam has no case for a read failure; `docks.rs` error arms are dead in tests
- **Where:** `tests/storage/story_storage.rs` (26 cases, all happy-path or write-failure), `tests/storage/transaction_storage.rs` (write failures only)
- **Evidence (known):** every injected failure in both files is a write (`handle.set_write(TxWrite::…, TestOverride::internal(…))`); grep finds no `with_failure("…")` read injection. The coverage JSON gives `src/adapters/driven/storage/docks.rs` 22/30 with uncovered lines `[29, 33, 34, 40, 41, 42, 44]` — the `QueryReturnedNoRows`/`other` arms, the `Failed to parse the offered set` arm and `no_in_memory("docks_get")`. Read failures are reachable through the seam: `src/adapters/driven/storage/core.rs:194` looks up `overrides.get(method)` for any method name.
- **Fix:** add one case per read box's `Err` arm, e.g. `// CASE story-dock-read-failure [failure]: a dock read that fails surfaces the error`, with `Storage::with_failure("docks_get", TestOverride::internal("the dock read fails"))`.

#### 7. [Low] The CASE grammar has two independent implementations; only the Rust copy has detection tests
- **Where:** `tests/infrastructure/guardrails/unit_test_allowlist.rs::is_case_declaration` vs `scripts/validate_seam_cases.py` (`CASE_TAG_RE`, `TEST_ATTR_RE`, `TAG_LOOKAHEAD`)
- **Problem:** the guardrail asserts "a file carries `CASE` tags" anywhere in the file, while the Python validator requires the tag to sit within 5 lines above a test attribute. The two can drift: only `unit_test_allowlist_tests.rs` pins the Rust copy's behaviour, and nothing compares the two.
- **Evidence (known):** the guardrail's `declares_cases(content)` scans every line; the Python side anchors each tag to a test.
- **Fix:** note the shared contract in both files, or have the Rust guardrail assert the tag is anchored (reuse the same lookahead constant), so one rule has one meaning.

#### 8. [Low] `FAILURE_KINDS` can silently fall behind `GenerationFailureKind`
- **Where:** `tests/browser/stub/failure_display.rs` (`const FAILURE_KINDS: [GenerationFailureKind; 12]`, `assert_kind_is_listed`)
- **Problem:** `assert_kind_is_listed` forces a compile error when a variant is added to the `match`, but that does not force the array. A variant added to the array's exclusion arm leaves `check_every_failure_sentence_reads_in_full` covering one sentence fewer with no signal.
- **Evidence (known):** `src/domain/model/state/generation_status.rs:49` declares 12 variants with no `ALL` const; the array is a hand-written literal and the test iterates it.
- **Fix:** add `GenerationFailureKind::ALL` to the domain type and assert `FAILURE_KINDS.len() == GenerationFailureKind::ALL.len()` (or iterate `ALL` directly).

#### 9. [Low] Dead test-only constructors in the two Agent boxes
- **Where:** `src/application/agents/options/agent.rs`, `src/application/agents/quantifier/agent.rs`
- **Evidence (known):** `with_provider` (and both `Debug` impls) are called by no test — grep finds no `OptionsGeneratorAgent::with_provider` / `QuantifierAgent::with_provider` caller. Coverage: options 13/27 = 48.1 %, quantifier 21/35 = 60.0 %, with the uncovered lines exactly the `Debug::fmt` body, `with_provider` and `name()`.
- **Why it matters:** the one rule says the test builds the box as production does — the seam tests already use `from_config`, so the provider-taking constructor is unused surface that inflates the box.
- **Fix:** delete `with_provider` and the `Debug` impls, or use `name()` in a case.

#### 10. [Note] The test-only Job watch ships in production builds
- **Where:** `src/application/generation/job_tasks.rs`
- **Evidence (known):** the module doc says *"It is built into production because the `testing` feature is a default one."*, and `Cargo.toml` has `default = ["testing"]`. So `JobTasks` — a structure whose only reader is `wait_for_jobs` in `tests/http` — is compiled into every normal build. This is a deliberate trade-off, not a test defect, but it is the reason finding 2 exists in production code rather than under `src/test_support`.

#### 11. [Low] Diagnostic weak spot: 21 new `unwrap()` calls where the surrounding style is `expect("…")`
- **Where:** e.g. `tests/storage/transaction_storage.rs` (`storage.stories().tail(game_id).unwrap().is_none()`, `storage.list_games().unwrap().is_empty()`), `tests/http/failure_display.rs:39`
- **Evidence (known):** `git diff bc7d6cd4 -- tests src/application/agents src/application/narration | grep -c "^+.*\.unwrap()"` = 21; neighbouring lines in the same files use `expect("the story reads back")`.
- **Fix:** replace with `expect("<what the read proves>")`, so a broken fixture names itself.

---

### Checked and found sound (no finding)

- `#[ignore]`/disabled tests: only `tests/llm/flow_llm_tests.rs` (×2, the allowed case).
- Tier-1 tier discipline: the only `Storage` read in `tests/http/` is `races.rs:530` `storage.require_game(game_a.0)` for the persona key (arrange), plus seeding helpers; all outcome reads go through `GameViewQuery`.
- Spec coverage: `validate_feature_spec.py` → `297 declared, 297 covered, 0 gap(s), 0 orphan(s), 0 untagged`, quarantine `74/74`. Deleted test `games_switch::test_switch_game_heals_stale_generating_status` (18.3) is legitimate: the spec scenario is gone and no heal path remains in `src/` (grep for `heal` finds none).
- `ScriptedModel` strictness: an unscripted call is an error, recorded as a stray, and fails the test both through `wait_settled` and through the fake's `Drop`; the double-`hold` guard and `ModelHold::drop` release prevent stranded held calls.
- Guardrail rows cannot silently go stale: `scan_row` returns `dead_path_violation(...)` when a row's scope matches no file.
- Browser stub diffs (`Phase` → `Step`, failure-kind list growth) are mechanical renames consistent with the renamed domain types.

Artifacts: `tmp/test-police/` (`run1.log`, `repeat_1..4.log`, `loaded.log`, `guardrails.log`, `coverage.log`); coverage JSON at `target/test-police-cov/llvm-cov/coverage.json`. No source or test file was edited, and nothing was committed.