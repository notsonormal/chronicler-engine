# Test strategy

The rules for placing a test and for writing a spec scenario. Each rule ends
with its **Check**: the `build.py` step that fails when the rule breaks, or
**review-only** when no step checks it.

## The tiers

Each tier is defined by **what is faked**. Every directory under `tests/` has
one row. The bold rows are the tiers. The other rows are support code, or
checks outside the tier model.

| Directory | Tier | Faked | Real | Holds |
|---|---|---|---|---|
| `src/**/*_tests.rs` | **Unit** | both driven ports (`MockBackend`, SQLite `:memory:` `Storage`) | nothing | [the one rule](#the-one-rule): a seam contract's `CASE`s, a pure `domain/model/` function, or a file on the legacy list |
| `tests/http/` | **HTTP E2E (tier 1)** | LLM (`ScriptedModel`, one script entry per call) | axum router, SQLite `:memory:` storage | spec scenarios, asserted on [domain outcomes](#domain-outcome-the-tier-1-rule) |
| `tests/http/requires_migration/` | tier 1, quarantine | as `tests/http/` | as `tests/http/` | legacy tests that wait for a spec; untagged, size pinned |
| `tests/http/support/` | support for tier 1 | — | — | request builders, seeding fixtures, HTML assertions, app wiring |
| `tests/browser/stub/` | **Stub browser (tier 2)** | the whole engine | stub server (real shell, real or canned fragments) + browser, a fresh page per test | client-side behaviour of the shipped JS |
| `tests/browser/` | **Full-stack browser (tier 3)** | LLM | browser + engine server | the full-stack hop: a client action whose effect is server state |
| `tests/storage/` | **Driven-adapter** | nothing | SQLite | the storage seam: CRUD, errors, referential integrity, queries |
| `tests/bootstrap/` | bootstrap smoke | nothing | `bootstrap::run()` | startup branches |
| `tests/llm/` | real provider | nothing | an LLM provider | provider flows; `#[ignore]`d, run with `--llm-only` |
| `tests/infrastructure/` | architecture | — | source tree | `arch-lint.toml` layer rules |
| `tests/infrastructure/guardrails/` | guardrails | — | source tree | source and test structure rules |
| `tests/helpers/` | support for driven-adapter | — | — | storage fixtures shared by the `storage` binary |
| `tests/test_utils/` | support for the `http`, `browser`, `storage`, `bootstrap` and `llm` binaries | — | — | server, browser, htmx-settle and wait helpers; the stub server |
| `tests/test_utils/stub_fixtures/` | support for tier 2 | — | — | canned fragments the stub server serves |

A `#[tokio::test]` with fakes is a unit test.

## The one rule

A unit test lives in one of three places, and nowhere else:

1. **A seam contract**: the test file of a box behind a port boundary. The
   boxes are the storage adapters, the LLM providers and the transport, the
   Harper text checker, each Agent, and the `Narrator`. The storage contract
   lives at the driven-adapter tier, in `tests/storage/`. The tag above each
   test declares its case and carries the description:
   `// CASE <id>: <what it pins>`, or `// CASE <id> [<label>]: <what it pins>`.
   Every test in a declaring file carries one tag. A unit test in a declaring
   file calls only that box's entry point. **Check:** `seam-cases` and the
   `guardrails` binary (the `unit test allowlist` rule); the entry-point clause
   is review-only.

   The shape of a case:
   - **One clause**: one input shape, one result, or one failure.
   - **The entry point only**: the test builds the box as production does. It
     does not call a helper that production never calls.
   - **The declared outcome**: assert what the clause promises a caller, not
     the internal steps.
   - **A failure through the box's own seam**: for storage,
     `Storage::with_failure` with `TestOverride::internal`. Do not mock an
     internal part.
   - **The call record**: a box that records its model call through
     `LlmCallRecorder` has a case for the record of a good call and of a
     failed call.
2. **A pure function in `domain/model/`**, as a plain unit test. The function's
   own branches are its cases. **Check:** the `guardrails` binary checks the
   folder. Purity is review-only.
3. **A file on the legacy list**, in
   `tests/infrastructure/guardrails/unit_test_allowlist.rs`. The list holds
   today's `src/**/*_tests.rs` files; it only shrinks, one module at a time.
   A legacy test that breaks only by a mechanical change (a new parameter, a
   renamed type) changes in place. A legacy test whose behaviour breaks moves
   to its seam contract or its scenario. **Check:** the `guardrails` binary,
   which also fails a stale entry.

A behaviour a player can see is not a unit test: it is a **scenario** in
`docs/specs/`, with one test at the tier that sees it. A module that is
neither a box nor a pure domain function has no unit test of its own. The
scenarios that pass through it are its coverage.

Why the rule exists. A module between a box and its route has its behaviour
observed by the scenarios that pass through it. A unit test there repeats the
implementation, and it breaks at every refactor. A box behind a port boundary
differs: no route reaches all of its failure modes. So the box writes its
clauses down as cases, and one test pins each case.

The coverage percentage stays a diagnostic: no step fails on it.

## Domain outcome (the tier-1 rule)

A tier-1 test asserts **domain outcomes** only. A domain outcome is one of:

- the HTTP response: status, headers, body;
- stored state read back through a **read seam**.

A **read seam** is a read the application exposes to a driving adapter: an
HTTP GET, or a method on an application read service or port
(`GameViewQuery`, `GameCatalogue`, `SettingsService`,
`PromptPresetService`, `WorldCatalogue`). `Storage` is a driven adapter, not
a read seam.

A read seam that no driving adapter calls is allowed only when a test checks
a stored fact that no production read returns, such as the stored inputs of a
Swipe. Its doc comment names that fact, so a reviewer can see why it exists.
The same holds for any production method that only tests call.

Apply the rule to one line of test code. The line reads a domain outcome when
it reads the HTTP response or calls a read seam. It leaks when it
calls a `Storage` method to observe state.

- **Helpers.** The rule judges the read inside a helper's body. A helper may
  wrap a read seam.
- **Arrange and wait steps.** The rule governs observations. A test may seed
  state through `Storage` and fixtures. A synchronization wait such as
  `wait_idle` may read state, because it gates the test and observes no
  outcome.
- **A persistence fact** that has no read seam (a column of a Swipe row that no view
  shows) is a driven-adapter or unit assertion, not a tier-1 one.

**Check:** review-only.

## Spec scenarios

Specs (`docs/specs/`) state what the engine does. An HTTP-observed scenario
maps to at least one tier-1 test that asserts a domain outcome. A DOM-observed
scenario (`browser_*.md`) maps to at least one tier-2 or tier-3 test.
`docs/AGENTS.md` "Where a fact lives" sets what a spec holds and what a
Diátaxis doc holds.

A scenario can be a single call ("POST /action with empty input → one
continuation narration"), or a sequence ("POST /action → POST /retry → POST
/retry → two swipes"). Both live at tier 1.

**Spec vocabulary.** A Given or Then names a domain thing with a `CONTEXT.md`
term, and a client-seen thing as a response value or a DOM `id` or `class`.
Plain English is fine when its nouns are glossary terms. When a spec file is
edited, reword its scenarios that name a Rust identifier, a field path, or a
module. **Check:** review-only.

**Markup.** A scenario names markup only as an element's `id` or `class` that
the frontend or a browser test selects on. A change there breaks the client.
**Check:** review-only.

**Complete specs.** A spec covers each distinct response its route can
return: each success shape, each refusal, each error status. The rule is per
route. Fine-grained UI behaviour inside one response stays in the code and its
tests. Behaviour that a spec omits gets no end-to-end coverage. Specs describe the system, not a
change to it. **Check:** review-only. `spec-coverage` checks the other
direction: every tagged test cites a declared scenario.

## Overlap

**Each tier asserts what it can see.** Overlap across tiers is expected: the
unit test covers the branch, tier 1 covers the domain outcome, and the
driven-adapter test covers persistence.

Within one tier, one behaviour has one test. When two tests at one tier assert
the same behaviour, keep the test whose failure names the regression more
precisely, and delete the other. **Check:** review-only.

## Placement rule: which of the three UI tiers

Place a new UI test by this rule.

### The rule, in order

1. **Is the outcome a domain outcome?** → tier 1. A rendered fragment, a
   response body, a status code, a header, or state read back through a read
   seam. The delivered content is a domain outcome. The htmx swap that
   delivers it is not.
2. **If the server behind this were fake, does the behaviour change?** No →
   tier 2, `tests/browser/stub/`. Pure client behaviour: the slash palette,
   edit-mode activation, toast handling, DOM layout.
3. Otherwise → **tier 3**, `tests/browser/<surface>.rs`. The test's point is
   the full-stack hop: a click or a `change` whose effect is real server-side
   state, read back through a reload.

**Tie-breaker: when in doubt, file down.** A test that can be expressed a tier
lower belongs there.

**Stub-tier shape.** A tier-2 test is an `async fn check_*(page, stub)` that a
`run_*` runner test passes to `StubRunner::run`. The runner shares one Chromium;
each check gets its own `StubServer` and page. The SCENARIO tag goes above the
`check_*` function, and `scripts/validate_feature_spec.py` exempts `run_*`.
Split a runner when it becomes the longest browser test by a wide margin.

**Layout-only checks.** A check that asserts only computed style, size or
position goes in `tests/browser/stub/layout.rs`. It has no spec scenario.
It puts its markup in place with client-side JS, not through a real failure.

**Check:** review-only.

### Worked examples

| Question asked | Answer | Tier | Example |
|---|---|---|---|
| Is the outcome a domain outcome? | yes | 1 | A GET of `/fragment/games` asserts that the stored posture renders selected and that the auto-save routes are present (games.md 20.8). |
| Would faking the server change it? | no | 2 | The palette is a document-level `input` listener on the shipped shell. This test needs only the shell, `assets/index.html` (browser_slash_menu.md 31.1). |
| Would faking the server change it? | no | 2 | `showEditForm` and `cancelEdit` are client JS acting on the canned entry's structural elements (browser_story_log.md 30.2). |
| Would faking the server change it? | yes | 3 | The select's `change` must reach the server, and a reload must show the persisted tense (browser_worlds.md 29.2). |
| Would faking the server change it? | yes | 3 | The chain's result is real stored preset state (browser_prompt_presets.md 28.1). |

### Stub tier's accepted tax

The stub server renders some fragments through the real Askama templates and
serves the rest from `tests/test_utils/stub_fixtures/`. A canned fragment
drifts from its template, and regenerating it is the upkeep this tier
demands. The failure is loud: the fixture keeps the structural elements the
shipped client JS addresses, so editing either side out of sync fails the
stub-tier test on a missing element. A stub-tier test asserts the client's
behaviour. A test that asserts a canned fragment's own content belongs a tier
higher. **Check:** review-only.

### Readiness gates

Full-stack interaction goes through the htmx-settle helpers in
`tests/test_utils/htmx_settle.rs`, with wrappers in
`tests/test_utils/browser.rs`. The `stub/` tests drive a stub with no htmx
swap lifecycle and need no gate. **Check:** `guardrails`
(`check_browser_interactions_use_htmx_settle` fails a raw interaction call in
a `tests/browser/` file outside `stub/`; a line that ends in
`// settle-guard-exempt: <why>` is exempt).

Wait on an htmx-settle helper or a DOM condition. `networkidle` never
settles: the dashboard pollers keep the network busy, so it is banned at
every tier. **Check:** review-only.

## SCENARIO tags

`SCENARIO:` tags (format: `// [spec-path] SCENARIO: N.N`) go on tests in
`tests/http/` and `tests/browser/`. At the unit and driven-adapter tiers, a
test's name describes the behaviour it covers instead.

**Per-surface specs.** DOM-observed scenarios live in
`docs/specs/browser_<feature>.md`, and HTTP-observed scenarios stay in
`<feature>.md`. Each scenario has one observation surface, one spec file, and
one tag. Test files mirror the specs: `tests/browser/<feature>.rs` covers
`docs/specs/browser_<feature>.md`, and `tests/browser/stub/<feature>.rs` covers
that spec's stub-tier scenarios. Dashboard-chrome scenarios that no feature
panel owns live in `docs/specs/browser_dashboard.md`.

**Check:** `spec-coverage` (`scripts/validate_feature_spec.py`) fails when:

- a declared spec scenario has no covering test, or a tag cites an undeclared
  scenario;
- a `browser_*.md` spec is tagged from outside `tests/browser/`, or another
  spec is tagged from `tests/browser/`;
- a test under `tests/http/` or `tests/browser/` has no tag and no declared
  exemption. The script lists each exemption with its reason;
- the untagged test count in `tests/http/requires_migration/` differs from
  its pin. Migrating a test out of the quarantine lowers the pin
  deliberately.

The validator scans `tests/http/` and `tests/browser/` only, so a tag belongs
in one of those two folders.
