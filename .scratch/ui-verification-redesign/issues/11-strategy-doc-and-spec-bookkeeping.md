# Record the placement rule and reconcile spec/test bookkeeping

Type: task
Status: resolved
Blocked by: 07, 08, 08b, 09

## Question

Two documentation-tier tasks the rollout leaves behind (per ticket 04's
ratified placement rule, and the map's Not-yet-specified on
`validate_feature_spec.py`):

1. **Write the placement rule into `tests/STRATEGY.md`** with a worked-example
   table — the tier's question, the answer, and two real examples per tier
   drawn from the rollout (e.g. ticket 21's new HTTP fragment test; a slash-
   menu tier-2 test; a wiring smoke guard tier-3) — plus the tie-breaker
   ("when in doubt, file down; tier 3 is the exception list"). Also fold in
   ticket 10's `networkidle` ban (one convention, one place). The rule is what
   the user ratified as stopping future tests being adjudicated one-by-one;
   write it as that — the decision of record, not an invitation to re-derive
   placement per test.
2. **Spec/test bookkeeping sweep.** Each rollout ticket (07, 08, 08b, 09) now
   reconciles its own scenario↔test mapping and exits with
   `validate_feature_spec.py` at `0 gap(s), 0 orphan(s), 0 untagged,
   0 surface mismatch(es)` — the validator rejects a `browser_*.md` tag from
   `tests/http/` *and* flags an uncovered declared scenario, so reconciliation
   cannot be deferred to this ticket without leaving the gate red in between.
   This ticket's job is therefore the **sweep**, not the per-test fixes: run the
   validator against the finished tree, fix anything the rollout tickets missed,
   and confirm no scenario lost coverage and no orphan tag remains. Check
   **before editing anything**: run the validator, let it name the drift. Do not
   weaken the validator to make a move pass — if a scenario genuinely belongs to
   two tiers now, the spec says so; if it can't be asserted, the scenario
   changes (surface that in `## Answer`).

Do this after 07–09 so the rule and the examples describe the suite as it
actually stands, not as ticket 04 imagined it — and so a test's
scenario-coverage only changes once. If 07–09 surface a fourth tier edge case
the rule doesn't cover, record it and ask before widening the rule.

Record under `## Answer`: the STRATEGY.md section's final wording, the list of
tests whose scenario mapping changed and how, and validator output before and
after (`python build.py validate-docs` does not cover this — the feature-spec
validator is a separate script).

## Answer

Resolved 2026-09-20. The placement rule is written into `tests/STRATEGY.md` as
the decision of record, and the bookkeeping sweep found **zero scenario drift
to fix** — the validator was already green on the finished tree, so this ticket
changed no test and no spec scenario.

### 1. The placement rule — `tests/STRATEGY.md`

The old `## Browser placement test` section (4 lines) was replaced by
`## Placement rule: which of the three UI tiers`, and the tier table's
four rows became five (a new **Quick browser (tier 2)** row; the old
**Browser** row is now **Browser (tier 3)**). Final wording:

- **The three UI tiers** table — tier → what's faked → location → the question
  it answers. Tier 2's location is `tests/browser/tier2.rs`; tier 3's is
  `tests/browser/<surface>.rs`.
- **The rule, in order**: (1) could `curl` observe this? → tier 1; (2) if the
  server behind this were fake, does the behaviour change? No → tier 2;
  (3) otherwise → tier 3.
- **Tie-breaker: when in doubt, file down.** Tier 3 is the exception list, not
  the default.
- **Worked examples** — a 5-row table, each row quoting the question asked,
  the answer, the tier, and a *real* test path from the finished suite:
  `games_fragment.rs::test_games_fragment_renders_posture_controls_http` (1),
  `tier2.rs::test_slash_menu_opens_on_slash` (2),
  `tier2.rs::test_edit_cancel_restores_original` (2),
  `worlds.rs::test_world_posture_change_autosaves_server_state` (3),
  `prompt_presets.rs::test_preset_duplicate_edit_save_click_chain` (3).
- **Tier 2's accepted tax** — the loud-failure property (a template rename
  breaks the fixture test on a missing element, it cannot silently pass) and
  the rule that an assertion which has become the fixture's own content belongs
  a tier higher. This carries ticket 07's exit-check finding into the doc.
- **Readiness gates** — tier 3 goes through the settle-gate helpers, enforced
  by `check_browser_interactions_use_settle_gate` so no test can opt out;
  tier 2 needs no gate.

The rule is written as the decision of record, not as an invitation to
test-by-test adjudication — its opening line says so explicitly.

### 2. `networkidle` ban — folded in

Ticket 10 declined the ban write and passed over its evidence; this ticket owns
it. Written into the same new section under **Readiness gates**: "banned as a
wait strategy at every tier", with the reason (five dashboard pollers make a
quiet window unsatisfiable) and ticket 10's re-verified grep result (zero uses
in `tests/`/`src/`/`assets/`).

### 3. Two doc-drift corrections the validator cannot see

Both are places where the finished suite no longer matches the prose. Neither
affects the validator (which reads tags, not mirror sentences):

| Location | Was | Now | Why |
|---|---|---|---|
| `tests/STRATEGY.md` "Per-surface specs" | named `browser_slash_menu.md` ↔ `slash_menu.rs` as the mirror example | states the general mirror rule, then names **tier-2 consolidation** as the one exception: `tier2.rs` hosts class-1 scenarios from four specs (`browser_slash_menu`, `browser_story_log`, `browser_options`, `browser_dashboard`) because they share the stub fixture; the mirror is per-*test* there | `slash_menu.rs` no longer exists — ticket 07 moved its tests to `tier2.rs` and deleted the file |
| `tests/AGENTS.md` preamble | "the normative tier-placement rules (unit / HTTP E2E / browser / driven-adapter)" | "the normative placement rule (which of the three UI tiers a test belongs to)" | the preamble listed four tiers while the UI question ticket 04 settled is the three-tier split; `AGENTS.md`'s own pointer is the first thing a session reads |

**Flagged, not widened:** the tier-2 multi-spec file is documented as an
exception rather than by widening the placement rule. If the user would rather
the mirror convention itself change (e.g. split `tier2.rs` per spec), that is a
rule change and is being raised here rather than assumed.

### 4. Spec/test bookkeeping sweep — no drift

The rollout tickets each reconciled their own scenario↔test mapping, and that
held: the validator on the finished tree reports **zero gaps, orphans, untagged
tests, and surface mismatches**, so the sweep found nothing to fix. Per the
ticket's instruction, the validator was run **before editing anything** to let
it name the drift — it named none.

| Checkpoint | Validator output |
|---|---|
| Before (ticket 10's landing, at claim) | `141 declared, 141 covered, 0 gap(s), 0 orphan(s), 0 untagged, 0 surface mismatch(es), quarantine 85/85` |
| After (this ticket) | `141 declared, 141 covered, 0 gap(s), 0 orphan(s), 0 untagged, 0 surface mismatch(es), quarantine 85/85` |

By ticket 10's close the count was 141 declared and the quarantine 85; this
ticket is docs-only and changes neither. **The validator was not weakened** —
no scenario was added, retired, or retagged.

**Tests whose scenario mapping changed: none.** The mapping ticket 09 recorded
(`browser_games.md` 27.1, `browser_worlds.md` 29.2, `browser_options.md` 26.4,
`browser_prompt_presets.md` 28.1, and the retirements of 27.1/29.1's
predecessors) is exactly what the validator now sees; no residual scenario is
uncovered and no orphan tag remains.

### 5. Ticket 07's recorded drift-tax gap — corrected here, closed by ticket 14

**Correction appended 2026-09-20, after this ticket's original Answer.** The
paragraph below carried ticket 07's note forward unverified, and the note is
half wrong. What checking established:

- **Wrong:** the claim that the real template's `.edit-btn`/`.delete-btn`
  assertions "live only in the quarantine". The unit tier pins both —
  `test_story_log_template_has_message_actions` renders the real
  `NarrativeLogTemplate` and asserts `edit-btn` and `delete-btn`
  (`src/adapters/driving/http/templates_tests.rs`), and the gate runs it.
- **The real gap, narrower and different:** the hooks the edit JavaScript reads
  — `data-raw-text` (`assets/index.html:242`, seeds the edit textarea),
  `data-id`, and `class="text"` — were pinned by **no gated test at all**.
  Mutation proof: renaming `data-raw-text` in the real template left the full
  gate green (1604 integration / 137 guardrails / 20 browser / 1 architecture,
  0 failed) while the product broke — Edit opens an empty textarea and a save
  writes an empty value. Ticket 07's "fails loudly" reasoning does not hold:
  the tier-2 fixture is a separate file, so a template change cannot fail a
  fixture-reading test.
- **Closed** as ticket 14 (`issues/14-pin-message-edit-template-hooks.md`),
  resolved in this session with the user's approval: one unit test on the real
  template pins all three hooks plus the `showEditForm(1)` wiring and the
  `| escape` wire form, each assert mutation-proven to fire. Full gate green
  (1605 integration — the +1 is the new test).

The original paragraph, kept for the record of what was believed at the time:

> Not a fix, a fact for the record. Ticket 07 flagged that the tier-2 fixture is
> the sole *gated* structural check on the story-log edit path, and that the only
> remaining assertions that pin the **real** template's `.edit-btn`/`.delete-btn`
> DOM live in `tests/http/requires_migration/fragment.rs:86-97` — the quarantine.
> Verified on the finished tree: `invariants.rs:321` now reads `.edit-btn`
> through the tier-2 stub (`SharedBrowser` + `Tier2StubServer`), so it checks the
> canned fixture, not the template. The gap is therefore real and unchanged.
>
> It is drifted past this ticket's scope (the ticket asked for the sweep, not new
> tests), and it is **not** validator-visible because the quarantine is untagged
> by design. Recording it here so it does not vanish: closing it means either a
> new HTTP test on the real `/fragment/story-log` render, or migrating the
> quarantine test off the quarantine and re-tagging it. That is a coverage
> decision, not bookkeeping.

### 6. Verification

| Check | Result |
|---|---|
| Full gate (`python build.py`) | **green, exit 0** |
| integration tests | 1604 passed, 0 failed, 2 skipped |
| guardrail tests | 137 passed, 0 failed |
| browser tests | 20 passed, 0 failed |
| architecture tests | 1 passed, 0 failed |
| `validate_feature_spec.py` | 141 declared, 141 covered, 0 gap(s), 0 orphan(s), 0 untagged, 0 surface mismatch(es), quarantine 85/85 |
| `cargo fmt --check` | clean (gate step) |

Files touched: `tests/STRATEGY.md` (+89/−16), `tests/AGENTS.md` (1 line). No
source, test, or spec file changed.

### 7. Map bookkeeping

Nothing to graduate or rule out — the sweep confirmed the map's Not-yet-specified
note that ticket 11 "shrinks to the STRATEGY.md placement rule, the `networkidle`
ban, and a final sweep". Ticket 12 (acceptance gate) is now unblocked on its
remaining blockers.

