# Test strategy

This file holds the rules for placing a test and for writing a spec scenario.
Each rule ends with its **Check**: the `build.py` step that fails when the rule
breaks, or **review-only** when no step checks it.

## The tiers

**What a tier fakes** defines it. Every directory under `tests/` has one row.
The bold rows are the tiers. The other rows are support code, or checks
outside the tier model.

| Directory | Tier | Faked | Real | Holds |
|---|---|---|---|---|
| `src/**/*_tests.rs` | **Unit** | LLM (`ScriptedModel`, or `MockBackend` in legacy files) | SQLite `:memory:` `Storage` | [the one rule](#the-one-rule): a seam contract's `CASE`s, a pure `domain/model/` function, or a file on the legacy list |
| `tests/http/` | **HTTP E2E (tier 1)** | LLM (`ScriptedModel`, one script entry per call) | axum router, SQLite `:memory:` storage | spec scenarios, asserted on [domain outcomes](#domain-outcome-the-tier-1-rule) |
| `tests/http/requires_migration/` | tier 1, quarantine | as `tests/http/` | as `tests/http/` | legacy tests that wait for a spec, untagged, with size pinned |
| `tests/http/support/` | support for tier 1 | — | — | request builders, seeding fixtures, HTML assertions, app wiring |
| `tests/browser/stub/` | **Stub browser (tier 2)** | the whole engine | stub server (real shell, real or canned fragments) + browser, a fresh page per test | client-side behaviour of the shipped JS |
| `tests/browser/` | **Full-stack browser (tier 3)** | LLM | browser + engine server | the full-stack hop: a client request whose effect is server state |
| `tests/storage/` | **Driven-adapter** | nothing | SQLite | the storage seam: CRUD, errors, referential integrity, queries |
| `tests/bootstrap/` | bootstrap smoke | nothing | `bootstrap::run()` | startup branches |
| `tests/llm/` | real provider | nothing | an LLM provider | provider flows, `#[ignore]`d and run with `--llm-only` |
| `tests/infrastructure/` | architecture | — | source tree | `arch-lint.toml` layer rules |
| `tests/infrastructure/guardrails/` | guardrails | — | source tree | source and test structure rules |
| `tests/helpers/` | support for driven-adapter | — | — | storage fixtures shared by the `storage` binary |
| `tests/test_utils/` | support for the `http`, `browser`, `storage`, `bootstrap` and `llm` binaries | — | — | server, browser, htmx-settle, convergence and wait helpers, and the stub server |
| `tests/test_utils/stub_fixtures/` | support for tier 2 | — | — | canned fragments the stub server serves |

The directory sets the tier, not the test attribute. A `#[tokio::test]` in
`src/` is a unit test.

## The one rule

A unit test lives in one of three places:

1. **A seam contract**: the test file of a box. The boxes behind a port
   boundary are the storage adapters, the LLM providers and the transport, the
   Harper text checker, each Agent, and the `Narrator`. The `PromptAssembler`
   is a box inside the `Narrator`. Its contract pins the prompt, and the
   `Narrator` contract pins only what the `Narrator` decides. The owner decides
   each new box. The storage contract lives at the driven-adapter tier, in
   `tests/storage/`.

   Each test in a seam contract carries one tag, on the comment line above it.
   The tag declares the case and holds its description:
   `// CASE <id>: <what it pins>`, or `// CASE <id> [<label>]: <what it pins>`.
   A test in a seam contract calls only that box's entry point. **Check:**
   `seam-cases` and the `guardrails` binary (the `unit test allowlist` rule).
   The entry-point clause is review-only.

   Each case has this shape:
   - **One clause**: one input shape, one result, or one failure.
   - **The entry point only**: the test builds the box as production does.
   - **The declared outcome**: assert what the clause promises a caller.
   - **A failure through the box's own seam**: for storage,
     `Storage::with_failure` with `TestOverride::internal`.
   - **The call record**: a box that records its model call through
     `LlmCallRecorder` has two cases. They pin the record of a good call and
     the record of a failed call.
2. **A pure function in `domain/model/`**, as a plain unit test. The function's
   own branches are its cases. **Check:** the `guardrails` binary checks the
   folder. Purity is review-only.
3. **A file on the legacy list**, in
   `tests/infrastructure/guardrails/unit_test_allowlist.rs`. The list holds
   the `src/**/*_tests.rs` files that are older than this rule. It only
   shrinks. A legacy test that breaks only by a mechanical change (a new parameter, a
   renamed type) changes in place. A legacy test whose behaviour breaks moves
   to its seam contract or its scenario. **Check:** the `guardrails` binary,
   which also fails a stale entry.

Test a behaviour that a player can see as a **scenario** in `docs/specs/`,
with one test at the tier that sees it. Every other module gets its coverage
from the scenarios that pass through it.

The coverage percentage is a diagnostic only.

### Why only a box writes cases

The scenarios that pass through a module between a box and its route already
observe its behaviour. A unit test there repeats the implementation, and it
breaks at every refactor. A box behind a port boundary is different, because
no route reaches all of its failure modes. So the box writes its clauses down
as cases, and one test pins each case.

## Domain outcome (the tier-1 rule)

A tier-1 test asserts **domain outcomes** only. A domain outcome is one of:

- the HTTP response: status, headers, body
- stored state read back through a **read seam**

A **read seam** is a read the application exposes to a driving adapter. It is an
HTTP GET, or a method on an application read service or port
(`GameViewQuery`, `GameCatalogue`, `SettingsService`,
`PromptPresetService`, `WorldCatalogue`). `Storage` is a driven adapter, not
a read seam.

A read seam that no driving adapter calls is allowed only when a test checks
a stored fact that no production read returns. The stored inputs of a Swipe
are one example. Name that fact in the doc comment of the read seam, so a
reviewer sees why the seam exists. The same holds for any production method
that only tests call.

Apply the rule to one line of test code. The line reads a domain outcome when
it reads the HTTP response or calls a read seam. It leaks when it
calls a `Storage` method to observe state.

- **Helpers.** The rule judges the read inside a helper's body. A helper may
  wrap a read seam.
- **Arrange and wait steps.** The rule governs observations. A test may seed
  state through `Storage` and fixtures. A synchronization wait such as
  `wait_idle` may read state, because it only decides when the test goes on.
  It observes no outcome.
- **What a model call carried.** A test may read the prompt and the call
  settings of a model call from the LLM port fake (`ScriptedModel::calls()`).
  The model is the boundary of tier 1, and no read seam carries the call
  settings.
- **A persistence fact** that has no read seam belongs in a driven-adapter or
  unit test, not at tier 1. A column of a Swipe row that no view shows is one
  example.

**Check:** review-only.

## Spec scenarios

Specs (`docs/specs/`) state what the engine does. An HTTP-observed scenario
maps to at least one tier-1 test that asserts a domain outcome. A DOM-observed
scenario (`browser_*.md`) maps to at least one tier-2 or tier-3 test.
`docs/AGENTS.md` "Where a fact lives" sets what a spec holds and what a
Diátaxis doc holds.

A scenario can be a single call ("POST /action with empty input → one
continuation narration"), or a sequence ("POST /action → POST /swipe/new →
POST /swipe/new → two swipes"). Both live at tier 1.

**Spec vocabulary.** A Given or Then names a domain thing with a `CONTEXT.md`
term, and a client-seen thing as a response value or a DOM `id` or `class`.
Plain English is fine when its nouns are glossary terms. When you edit a spec
file, reword its scenarios that name a Rust identifier, a field path, or a
module. **Check:** review-only.

**Markup.** A scenario names markup only as an element's `id` or `class` that
the frontend or a browser test selects on. A change to that `id` or `class`
breaks the client.
**Check:** review-only.

**Complete specs.** A spec covers each distinct response its route can
return: each success shape, each refusal, each error status. The rule is per
route. Fine-grained UI behaviour inside one response stays in the code and its
tests. Behaviour that a spec omits gets no end-to-end coverage. A spec
describes the system as it is. **Check:** review-only. `spec-coverage` checks
the other direction: every tagged test cites a declared scenario.

## Overlap

**Each tier asserts what it can see.** Overlap across tiers is expected: the
unit test covers the branch, tier 1 covers the domain outcome, and the
driven-adapter test covers persistence.

Within one tier, one behaviour has one test. When two tests at one tier assert
the same behaviour, keep the test whose failure names the regression more
precisely, and delete the other. **Check:** review-only.

## The UI placement rule

To place a new UI test, ask these questions in order.

### Ask three questions

1. **Is the outcome a domain outcome?** If yes, use tier 1. A rendered
   fragment, a response body, a status code, a header, and state read back
   through a read seam are domain outcomes. The delivered content is a domain
   outcome. The htmx swap that delivers it is not.
2. **Would a fake server change the behaviour?** If no, use tier 2,
   `tests/browser/stub/`. Pure client behaviour goes here: the slash palette,
   edit-mode activation, toast handling, and DOM layout.
3. Otherwise, use **tier 3**, `tests/browser/<surface>.rs`. The point of the
   test is the full-stack hop: a click or a `change` whose effect is real
   server-side state, read back through a reload.

**If two tiers fit, use the lower one.** A test that you can write a tier
lower belongs there.

**Stub-tier shape.** A tier-2 test is an `async fn check_*(page, stub)` that a
`run_*` runner test passes to `StubRunner::run`. The runner shares one Chromium.
Each check gets its own `StubServer` and page. The SCENARIO tag goes above the
`check_*` function, and `scripts/validate_feature_spec.py` exempts `run_*`.
Split a runner when it becomes the longest browser test by a wide margin.

**Layout-only checks.** A check that asserts only computed style, size or
position goes in `tests/browser/stub/layout.rs`. It has no spec scenario.
It builds its markup with client-side JS.

**Check:** review-only.

### Worked examples

| Question asked | Answer | Tier | Example |
|---|---|---|---|
| Is the outcome a domain outcome? | yes | 1 | A GET of `/screen?parts=games` asserts that the stored posture renders selected and that the auto-save routes are present (games.md 20.8). |
| Would a fake server change the behaviour? | no | 2 | The palette is a document-level `input` listener on the shipped shell. This test needs only the shell, `assets/index.html` (browser_slash_menu.md 31.1). |
| Would a fake server change the behaviour? | no | 2 | `showEditForm` and `cancelEdit` are client JS acting on the canned entry's structural elements (browser_story_log.md 30.2). |
| Would a fake server change the behaviour? | yes | 3 | The select's `change` must reach the server, and a reload must show the persisted tense (browser_worlds.md 29.2). |
| Would a fake server change the behaviour? | yes | 3 | The chain's result is real stored preset state (browser_prompt_presets.md 28.1). |

### The stub tier's upkeep

The stub server renders some fragments through the real Askama templates. It
serves the rest from `tests/test_utils/stub_fixtures/`. A canned fragment can
drift from its template, and regenerating it is the upkeep of this tier. The
fixture keeps the structural elements that the shipped client JS addresses.
So a change to only one side fails the stub-tier test on a missing element.

A stub-tier test asserts the behaviour of the client. A test that asserts the
content of a canned fragment belongs a tier higher. **Check:** review-only.

### Readiness gates

In a tier-3 test, interact through the htmx-settle helpers in
`tests/test_utils/htmx_settle.rs`, or their wrappers in
`tests/test_utils/browser.rs`. **Check:** `guardrails`.
`check_browser_interactions_use_htmx_settle` fails a raw interaction call in a
`tests/browser/` file outside `stub/`. A line that ends in
`// settle-guard-exempt: <why>` is exempt.

At every tier, wait on an htmx-settle helper or a DOM condition. The page
asks `GET /changes` every second, so `networkidle` never settles.
**Check:** review-only.

### Convergence check

Every stub check and every tier-3 check ends with the convergence check
(`assert_screen_converged`, `tests/test_utils/convergence.rs`).
`StubRunner::run` and `with_test_page` run it after the check, before the page
closes. The check waits until the page is quiet. Then it compares each visible
part with a fresh `GET /screen?parts=<name>`, and it fails with the part, the route
and a short diff. It also checks that each part of a closed tab is empty, and
that each part element has the attributes of the shell. The focus fallback
and the lock of the entry in edit are the exceptions to the attribute check.

The comparison drops what a load keeps or skips by design: the attributes that
`data-keep` names, the htmx classes, and the story-log
entry in edit (`data-editing`).

`StubRunner::run_without_convergence` is only for a check that leaves the page
unable to match a fresh render on purpose, for example with a stopped server.
That check gives the reason. **Check:** review-only.

## SCENARIO tags

`SCENARIO:` tags (format: `// [spec-path] SCENARIO: N.N`) go on tests in
`tests/http/` and `tests/browser/`. At the unit and driven-adapter tiers, a
test in a seam contract carries its `CASE` tag. Any other test names its
behaviour in its function name.

**Per-surface specs.** DOM-observed scenarios live in
`docs/specs/browser_<feature>.md`, and HTTP-observed scenarios stay in
`<feature>.md`. Each scenario has one observation surface, one spec file, and
one tag. Test files mirror the specs: `tests/browser/<feature>.rs` covers
`docs/specs/browser_<feature>.md`, and `tests/browser/stub/<feature>.rs` covers
that spec's stub-tier scenarios. Dashboard-chrome scenarios that no feature
panel owns live in `docs/specs/browser_dashboard.md`.

**Check:** `spec-coverage` (`scripts/validate_feature_spec.py`) fails when:

- a declared spec scenario has no covering test, or a tag cites an undeclared
  scenario.
- a `browser_*.md` spec is tagged from outside `tests/browser/`, or another
  spec is tagged from `tests/browser/`.
- a test under `tests/http/` or `tests/browser/` has no tag and no declared
  exemption. The script lists each exemption with its reason.
- the untagged test count in `tests/http/requires_migration/` differs from
  its pin. When a test moves out of the quarantine, lower the pin in the same
  change.

Two stub-tier files carry no tag. `TAG_EXEMPT_FILES` lists them:

- `layout.rs` holds the layout and computed-style checks, which have no HTTP
  surface.
- `htmx_events.rs` holds the probes of the vendored htmx event contract. They
  pin the library that the page relies on, not a scenario.
