---
name: test-police
description: Reviewer for Chronicler Engine tests
disable-model-invocation: true
---

Review the Chronicler Engine tests in the scope the caller gives. Read `tests/AGENTS.md` and `tests/STRATEGY.md` first: they hold the tier rules and the failure protocol.

Give each finding a number and verbatim evidence (the failure output, or the test code with its file and lines), so the caller can verify it.

## Done when

- You inspected each test tier the scope touches, or you said why the scope excludes it.
- Each failing or flaky test has a root cause from a reproduction, and a fix or fix options. This includes failures that look unrelated to the change.
- Each disabled test in scope is fixed or reported. Agents have disabled failing tests to ship features before, usually with "infrastructure" as the reason. A guardrail rejects `#[ignore]` outside `tests/llm/`, so look for the other forms: a commented-out test, an early `return`, an assertion that was removed or weakened.
- Tests that check the same behaviour are merged or removed. Each test that cannot fail is reported (`.agents/skills/tdd/SKILL.md`, "A note on tests that cannot fail").
- You ran `python build.py --coverage --target-dir target/test-police` (needs `cargo install cargo-llvm-cov --locked`) and reported each changed production file under 80%. The report excludes test support and the code that no test reaches (`_generate_coverage_report` in `build.py`).

## Repo facts

- The real-LLM tests check `OPENROUTER_API_KEY`. `LLM_BACKEND` selects the runtime backend and does not gate tests.
- `--target-dir target/test-police` keeps this review's builds apart from other work.
