# Test Strategy

The normative tier-placement rules for the engine. Referenced from
`tests/AGENTS.md`. Decisions settled via the wayfinder maps
`.scratch/test-strategy/` (the tiers) and `.scratch/ui-verification-redesign/`
(the placement rule and the quick-browser tier); this doc is the codified output.

## The tiers

The tier is defined by **what's faked**, not by sync vs async.

| Tier | Driven ports | Real | Location | Purpose |
|---|---|---|---|---|
| **Unit** | both (`MockBackend` + in-memory `Storage`) | nothing | `src/`, `*_tests.rs` | branch coverage — every branch in the code gets a test |
| **HTTP E2E** | LLM (`MockBackend` via pipeline override) | real axum router, real or in-memory storage | `tests/http/` | spec validation — all spec scenarios validated end-to-end through the real driving adapter |
| **Quick browser (tier 2)** | the whole engine | stub server (real shell + canned fragments) + real browser | `tests/browser/tier2.rs` | client-side behaviour — DOM, JS wiring, exercising the shipped client JS with no engine process behind it |
| **Browser (tier 3)** | LLM | real browser, real server | `tests/browser/` | presentation and full-stack wiring — DOM, CSS, JS interaction that reaches real server state |
| **Driven-adapter** | nothing | real SQLite | `tests/storage/` | the storage seam — CRUD, error handling, referential integrity, query correctness |

`#[tokio::test]` with fakes is a unit test. The unit tier includes async
scenarios (cancellation, mid-flight timing) that need in-process seams — these
are unit tests doing their job, not exceptions. There is no component tier;
former pipeline-level tests moved to the unit tier or to HTTP E2E.

## Spec scenarios and HTTP E2E

Specs (`docs/specs/`) are the behavioural authority. Every spec scenario maps
to at least one HTTP E2E test that validates it end-to-end through the real
driving adapter. Single-call scenarios ("POST /action with empty input → one
continuation narration") and multi-call sequence scenarios ("POST /action →
POST /retry → POST /retry → assert swipe=2") both live at the HTTP tier. The
`flow/` tests are not a separate tier — they are multi-call spec scenarios at
HTTP E2E.

Scenarios that can't be expressed through HTTP surfaces — because their Givens
or Thens touch seams that only exist in-process — live at the unit or
driven-adapter tier instead:

- **Cancellation** — needs `CancellationToken` → unit (`#[tokio::test]` in `src/` with fakes)
- **Internal state** (e.g. `last_trigger`, phase transitions) — assert on `GameState` fields → unit
- **Mid-flight observation** — needs sync flags → unit
- **Call sequencing** — direct call-count assertion → unit

These aren't exceptions to the model — they're the unit and driven-adapter
tiers doing their job. A scenario that can't be expressed through HTTP simply
doesn't get an HTTP E2E test.

**Spec completeness is mandatory.** The model only prevents drift if specs
are complete — every failure mode, every edge case. A half-written spec +
HTTP E2E + comprehensive unit tests is less safe than the old component tier,
because the component tier was catching unspecified behaviour the new model
doesn't cover end-to-end.

## Overlap rule

**Each tier asserts what it can see.** Cross-tier overlap is expected and
correct — unit tests the branch (internal state), HTTP E2E validates the spec
scenario (client-observable behaviour), driven-adapter tests the storage seam
(persistence integrity). They cover the same behaviour from different angles
for different reasons.

The violation is **same-tier duplication**: two tests at the same tier
asserting the same thing about the same behaviour. If a test's assertions are
fully covered by another test at the same tier, delete the weaker one.

## Placement rule: which of the three UI tiers

The UI suite splits by what is **faked**, not by how fast it runs. Three tiers
cover the UI, and one ordered rule places a test among them. **This section is
the decision of record** — a new UI test is placed by applying the rule, not by
re-adjudicating the question test by test.

### The three UI tiers

| Tier | What's faked | Location | Answers |
|---|---|---|---|
| **1 — HTTP contract** | nothing; a real request through the real router | `tests/http/` | what the server sends |
| **2 — quick browser** | the engine: a stub server serves the real shell + canned fragments; one shared browser process, fresh page per test | `tests/browser/tier2.rs` | what the shipped client JS and DOM do |
| **3 — full-stack browser** | nothing: real server boot, real browser | `tests/browser/<surface>.rs` | whether client wiring reaches real server state |

### The rule, in order

1. **Could `curl` observe this?** → tier 1. A rendered fragment, a response
   body, a status code, a header. The swap that *delivers* it is not
   curl-observable, but the delivered content is.
2. **If the server behind this were fake, does the behaviour change?** No →
   tier 2. Pure client behaviour: the slash palette, edit-mode activation,
   toast handling, DOM rendering invariants.
3. Otherwise → **tier 3**. The test's point is the full-stack hop: a click or a
   `change` whose effect is real server-side state, read back through a reload.

**Tie-breaker: when in doubt, file down.** Tier 3 is the exception list, not
the default. A test that can be expressed a tier lower belongs there.

### Worked examples from the rollout

| Question asked | Answer | Tier | Real example |
|---|---|---|---|
| Could `curl` observe this? | yes | 1 | `tests/http/games_fragment.rs::test_games_fragment_renders_posture_controls_http` — a GET of `/fragment/games` asserts the stored posture renders selected and pins the auto-save routes, with no browser at all (games.md 20.8). |
| Would faking the server change it? | no | 2 | `tests/browser/tier2.rs::test_slash_menu_opens_on_slash` — the palette is a document-level `input` listener on the shipped shell; the stub's only job is to serve `assets/index.html` (browser_slash_menu.md 31.1). |
| Would faking the server change it? | no | 2 | `tests/browser/tier2.rs::test_edit_cancel_restores_original` — `showEditForm`/`cancelEdit` are client JS acting on the canned entry's structural hooks (browser_story_log.md 30.2). |
| Would faking the server change it? | yes | 3 | `tests/browser/worlds.rs::test_world_posture_change_autosaves_server_state` — the select's `change` must reach the server, and a reload must show the persisted tense (browser_worlds.md 29.2). |
| Would faking the server change it? | yes | 3 | `tests/browser/prompt_presets.rs::test_preset_duplicate_edit_save_click_chain` — the chain's result is real stored preset state (browser_prompt_presets.md 28.1). |

### Tier 2's accepted tax

The canned fragments can drift from the real Askama templates. The failure
mode is **loud, not silent**: the fixture must keep the structural hooks the
client JS addresses, so a rename in the real template makes the tier-2 test
fail on a missing element. Updating the fixture in the same change as the
template it mirrors is the cost of the tier. A test whose assertion has quietly
become the canned fragment's own content has stopped exercising behaviour and
belongs a tier higher.

### Readiness gates

Tier 3 interaction goes through the settle-gate helpers
(`tests/test_utils/settle_gate.rs`; wrappers in `tests/test_utils/browser.rs`).
A raw `.click(`/`.select_option(` in a `with_test_page` file fails the build
(`check_browser_interactions_use_settle_gate`), so no test can opt out. Tier 2
drives a stub with no htmx swap lifecycle and needs no gate.

**`networkidle` is banned as a wait strategy at every tier.** Five dashboard
pollers make a quiet network window unsatisfiable, so it can only ever time
out. Zero uses exist today, and this ban keeps it that way.

## SCENARIO tags

`SCENARIO:` tags (format: `// [spec-path] SCENARIO: N.N`) go on HTTP E2E tests
in `tests/http/` and browser tests in `tests/browser/`. Scenarios at the unit
or driven-adapter tier get no tags; tests whose names describe the behaviour
cover them there.

**Per-surface specs.** DOM-observed scenarios live in
`docs/specs/browser_<feature>.md`; HTTP-observed scenarios stay in
`<feature>.md` — one scenario = one observation surface = one spec file = one
tag. Test files mirror the specs: `tests/browser/<feature>.rs` covers
`docs/specs/browser_<feature>.md`. Dashboard-chrome scenarios no feature panel
owns live in `browser_dashboard.md` / `dashboard.rs` (static command form,
status display, error toast).

The one exception is the **tier-2 consolidation**: `tests/browser/tier2.rs` is
one file hosting the class-1 scenarios of several specs (`browser_slash_menu`,
`browser_story_log`, `browser_options`, `browser_dashboard`) because they share
the stub fixture. The mirror is per-*test* there, not per-file; tags stay
scoped to their own spec.

`scripts/validate_feature_spec.py` enforces the rules in the `spec-coverage`
gate step of `build.py`:

- Every declared spec scenario has at least one covering test, and every tag
  references a declared scenario.
- Surface consistency: `browser_*.md` specs are tagged only from
  `tests/browser/`, and non-`browser_*` specs are never tagged from
  `tests/browser/`.
- Every test under `tests/http/` and `tests/browser/` carries a tag, unless
  the script declares an exemption with its reason (`TAG_EXEMPT_DIRS`,
  `TAG_EXEMPT_FILES`, `TAG_EXEMPT_TESTS`). Exempt today:
  `tests/browser/invariants.rs` (no spec link; test code is the definition).
  The stdout-tee health check (`test_engine_output_teed_to_file`) is not an
  exemption: it moved to `tests/bootstrap/run_branches.rs`, outside this
  validator's scan scope, because it needs a real server but no browser.
- `tests/http/requires_migration/` is the legacy quarantine: untagged by
  design. `REQUIRES_MIGRATION_TEST_COUNT` in the script pins its size, and
  the count may only go down. Migrating a test off the quarantine lowers
  the constant deliberately; a new untagged test in the folder fails the
  gate.

The validator scans only `tests/http/` and `tests/browser/`. Put no tags in
`tests/storage/` or any other tier.
