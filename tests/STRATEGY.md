# Test Strategy

The normative tier-placement rules for the engine.

## The tiers

Each tier is defined by **what is faked**.

| Tier | Driven ports | Real | Location | Purpose |
|---|---|---|---|---|
| **Unit** | both (`MockBackend` + in-memory `Storage`) | nothing | `src/`, `*_tests.rs` | branch coverage — every branch in the code gets a test |
| **HTTP E2E (tier 1)** | LLM (`MockBackend` via pipeline override) | real axum router, real or in-memory storage | `tests/http/` | spec validation — every spec scenario, end-to-end through the real driving adapter, asserted on the response `curl` sees |
| **Stub browser (tier 2)** | the whole engine | stub server (real shell + canned fragments) + real browser, one process with a fresh page per test | `tests/browser/stub/` | client-side behaviour — DOM, JS wiring, exercising the shipped client JS with no engine process behind it |
| **Full-stack browser (tier 3)** | LLM | real browser, real server | `tests/browser/<surface>.rs` | presentation and full-stack wiring — DOM, CSS, JS interaction that reaches real server state |
| **Driven-adapter** | nothing | real SQLite | `tests/storage/` | the storage seam — CRUD, error handling, referential integrity, query correctness |

`#[tokio::test]` with fakes is a unit test. The unit tier includes async
scenarios (cancellation, mid-flight timing) that need in-process seams.

## Spec scenarios and HTTP E2E

Specs (`docs/specs/`) are the behavioural authority. Every spec scenario maps
to at least one HTTP E2E test that validates it end-to-end through the real
driving adapter, so a scenario states behaviour a client can observe.
Single-call scenarios ("POST /action with empty input → one continuation
narration") and multi-call sequence scenarios ("POST /action → POST /retry →
POST /retry → assert swipe=2") both live at the HTTP tier.

**Spec completeness is mandatory.** Specs only prevent drift when they
are complete: every failure mode, every edge case. Behaviour a spec omits
gets no end-to-end coverage.

**Quote markup only as an element's `id` or `class`.** A scenario describes
behaviour in words. It names markup only for an `id` or `class` that the
frontend or a browser test selects on. A change there breaks the client.

## Overlap rule

**Each tier asserts what it can see.** Cross-tier overlap is expected and
correct: unit tests the branch (internal state), HTTP E2E validates the spec
scenario (client-observable behaviour), driven-adapter tests the storage seam
(persistence integrity). They cover the same behaviour from different angles.

The violation is **same-tier duplication**: two tests at the same tier
asserting the same behaviour. Delete the weaker one.

## Placement rule: which of the three UI tiers

The UI suite splits by **what is faked** into the three tiers in the table
above: HTTP E2E (tier 1), stub browser (tier 2), and full-stack browser
(tier 3). **This section is the decision of record**: place a new UI test by
applying the rule below.

### The rule, in order

1. **Could `curl` observe this?** → tier 1. A rendered fragment, a response
   body, a status code, a header. The delivered content is observable. The
   htmx swap that delivers it is not.
2. **If the server behind this were fake, does the behaviour change?** No →
   tier 2, `tests/browser/stub/`. Pure client behaviour: the slash palette,
   edit-mode activation, toast handling, DOM rendering invariants.
3. Otherwise → **tier 3**, `tests/browser/<surface>.rs`. The test's point is
   the full-stack hop: a click or a `change` whose effect is real server-side
   state, read back through a reload.

**Tie-breaker: when in doubt, file down.** A test that can be expressed a tier
lower belongs there.

### Worked examples

| Question asked | Answer | Tier | Example |
|---|---|---|---|
| Could `curl` observe this? | yes | 1 | A GET of `/fragment/games` asserts that the stored posture renders selected and that the auto-save routes are present (games.md 20.8). |
| Would faking the server change it? | no | 2 | The palette is a document-level `input` listener on the shipped shell. The stub only serves `assets/index.html` (browser_slash_menu.md 31.1). |
| Would faking the server change it? | no | 2 | `showEditForm` and `cancelEdit` are client JS acting on the canned entry's structural elements (browser_story_log.md 30.2). |
| Would faking the server change it? | yes | 3 | The select's `change` must reach the server, and a reload must show the persisted tense (browser_worlds.md 29.2). |
| Would faking the server change it? | yes | 3 | The chain's result is real stored preset state (browser_prompt_presets.md 28.1). |

### Stub tier's accepted tax

The canned fragments drift from the real Askama templates, and regenerating the
affected fixture is the upkeep this tier demands. The failure is loud: the
fixture keeps the structural elements the shipped client JS addresses, so
editing either side out of sync fails the stub-tier test on a missing element.
A test that has drifted into asserting the canned fragment's own content
belongs a tier higher.

### Readiness gates

Full-stack interaction goes through the htmx-settle helpers in
`tests/test_utils/htmx_settle.rs`, with wrappers in `tests/test_utils/browser.rs`.
A raw `.click(` or `.select_option(` in a `with_test_page` file fails the build
(`check_browser_interactions_use_htmx_settle`). The `stub/` tests drive a stub
with no htmx swap lifecycle and need no gate.

**`networkidle` is banned as a wait strategy at every tier.** Five dashboard
pollers keep the network busy, so a quiet window never arrives. Review is the
only enforcement.

## SCENARIO tags

`SCENARIO:` tags (format: `// [spec-path] SCENARIO: N.N`) go on HTTP E2E tests
in `tests/http/` and browser tests in `tests/browser/`. At the unit and
driven-adapter tiers, a test's name describes the behaviour it covers instead.

**Per-surface specs.** DOM-observed scenarios live in
`docs/specs/browser_<feature>.md`, and HTTP-observed scenarios stay in
`<feature>.md`. Each scenario has one observation surface, one spec file, and
one tag. Test files mirror the specs: `tests/browser/<feature>.rs` covers
`docs/specs/browser_<feature>.md`, and `tests/browser/stub/<feature>.rs` covers
that spec's stub-tier scenarios. Dashboard-chrome scenarios no feature panel
owns live in `docs/specs/browser_dashboard.md` and `tests/browser/dashboard.rs` (static command form,
status display, error toast), with the stub-tier ones in
`tests/browser/stub/dashboard.rs`.

`scripts/validate_feature_spec.py` enforces the rules in the `spec-coverage`
gate step of `build.py`:

- Every declared spec scenario has at least one covering test, and every tag
  references a declared scenario.
- Surface consistency: `browser_*.md` specs are tagged only from
  `tests/browser/`, and non-`browser_*` specs are never tagged from
  `tests/browser/`.
- Every test under `tests/http/` and `tests/browser/` carries a tag.
  The script declares each exemption with its reason, and it is the source of
  truth for which files and tests are exempt.
- `tests/http/requires_migration/` is the legacy quarantine, untagged by
  design with its size pinned in the script. Migrating a test off the
  quarantine lowers the pin deliberately. A new untagged test in the folder
  fails the gate.

The validator scans `tests/http/` and `tests/browser/` only, so a tag belongs
in one of those two folders.
