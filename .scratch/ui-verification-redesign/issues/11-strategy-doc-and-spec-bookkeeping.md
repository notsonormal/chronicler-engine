# Record the placement rule and reconcile spec/test bookkeeping

Type: task
Status:
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
