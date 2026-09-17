# Record the placement rule and reconcile spec/test bookkeeping

Type: task
Status:
Blocked by: 07, 08, 09

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
2. **Spec/test bookkeeping.** For every test that moved tiers in tickets
   07–09, `validate_feature_spec.py` gates the scenario↔test mapping. Update
   test annotations so each moved test still maps to its declared scenario, and
   each scenario still has a covering test in *some* tier. Check **before
   editing tests**: run the validator, let it name the drift, fix the
   annotations it names. Do not weaken the validator to make the move pass —
   if a scenario genuinely belongs to two tiers now, the spec says so; if it
   can't be asserted, the scenario changes (surface that in `## Answer`).

Do this after 07–09 so the rule and the examples describe the suite as it
actually stands, not as ticket 04 imagined it — and so a test's
scenario-coverage only changes once. If 07–09 surface a fourth tier edge case
the rule doesn't cover, record it and ask before widening the rule.

Record under `## Answer`: the STRATEGY.md section's final wording, the list of
tests whose scenario mapping changed and how, and validator output before and
after (`python build.py validate-docs` does not cover this — the feature-spec
validator is a separate script).
