# Re-synthesize the audit report from pattern-level evidence

Type: task
Status: resolved
Assignee: pi
Blocked by: 08 (resolved — unblocked)
Supersedes (for the report): [06](./06-synthesize-audit-report.md)

## Question

[Ticket 08](./08-re-extract-at-pattern-level.md) produced the pattern-level evidence asset
[`assets/pattern-trap-evidence.md`](../assets/pattern-trap-evidence.md) — 32 exact-verified quotes,
the accept/resist ratio as the pool's pattern, and all six negatives grounded to the verified-quote
standard. [Ticket 06](./06-synthesize-audit-report.md)'s report is **preliminary** (it rested on
03's instance-level extraction: one Progression instance + six "None detected" verdicts on
unverified citations). Re-synthesize the audit in the premise's three-part format **from 08's
pattern-level evidence**, so the report becomes the grounded audit the destination requires.

**Output.** The premise's three parts — (1) Summary Dashboard, (2) Detailed Audit Table (one row per
trap), (3) Single Actionable Rule — as a markdown asset under `assets/`. Replace the preliminary
banner on `assets/audit-report.md` with the final report (06's preliminary version is preserved in
its ticket answer and git history; one canonical final report is the destination), or write a new
final-report asset if a clean overwrite is worse — decide and note why.

**Grounding.** Use **only** the 32 exact-verbatim quotes 08 verified
([`tmp/verify_pattern_quotes.py`](../../tmp/verify_pattern_quotes.py), 32/32 PASS) — or ticket 04's
corrected-verbatim forms for the S11 cost-of-the-gap quotes (Q4/Q6). No unverified citation, no
as-written form that failed. The flattened transcripts remain the single source.

**What 08's data tells the synthesis to present.**

- **Progression: one verified instance (High), self-caught the same session.** Q1 (S11, line 1813)
  grounds it; the cost-of-the-gap quotes (Q3/Q5/Q7, + corrected Q4/Q6) ground the drift; the subsystem
  was retired in S11. The same-session self-correction (the user surfaces not-understanding in F2,
  drives a rethink, finds the drift, retires it) is a nuance to state — it does not negate the trap.
- **Mislead: Low, overlap on the same instance.** The user's dominant Mislead behavior is
  resistance (the 11 verified resistance quotes: Q8–Q12, R2, R6, R20a, R27, R30, R29). The INV-NNN
  is the exception, in the docs/named-concept layer.
- **Six negatives, each grounded** by a verified quote showing the opposite habit — not "None
  detected" on an unverified citation. Forming (surface uncertainty: F1–F3); Dislodging (pivot:
  D1–D4); Assumption (harden/generalize: A1–A2); Location (plan-first: L1–L3); Achievement
  (refactor/remove: AC3 + A1 + the Q1 retirement, with the instance-vs-pattern split from ticket 05
  — pattern-level not assessable without diffs, no positive pattern in evidence, no diffs ticket);
  Interruption (typo-corrections IT1–IT6 + the full 42-gap scan showing zero confusion-reprompts).
  Word each negative as "no candidate in this 10-day window; the user's habit here is the opposite —
  [verified quote]," not as proof the trap never occurs.
- **The accept/resist ratio is the pattern context.** Resistance dominates (11+ verified instances
  across 11 sessions); acceptance-without-understanding is n=1 (Q1, docs/concept layer);
  acceptance-after-investigation (R20b) is healthy and distinct from Progression.

**The primary-vulnerability question, reframed by 08.** 06 proposed "Progression (High)" as primary
on one instance. 08's pattern evidence **sharpens, not overturns**, this: the vulnerability is
**layer-specific** (docs/named-concept / architecture), not a general accept-without-understanding
habit — the user already applies anti-Progression rigor to code. The synthesis should present the
primary vulnerability as "Progression, layer-specific to AI-authored named concepts/docs," grounded
in Q1 and bounded by the accept/resist ratio. The rule proposal (restate any AI-authored named
concept in your own words and point to its enforcing code, or it doesn't go in) targets exactly
that layer — it stands, sharpened. Both remain proposals for [ticket 07](./07-confirm-vulnerability-and-derive-rule.md)
(HITL grilling) to confirm with the user.

**Do not re-open 06.** 06 stays resolved as the preliminary synthesis (auditable artifact). This
ticket produces the final report. [Ticket 07](./07-confirm-vulnerability-and-derive-rule.md) is
blocked by this ticket (rewired from 08): the user cannot be grilled on "which trap is primary /
does one instance warrant a rule" until the pattern-level synthesis is in.

## Answer

Resolved. The final report is [`assets/audit-report.md`](../assets/audit-report.md). It replaces the preliminary report from ticket 06 (preserved in 06's answer and git history). The synthesis uses only the 32 exact-verbatim quotes from ticket 08 (`tmp/verify_pattern_quotes.py`, 32/32 PASS) plus ticket 04's corrected verbatim forms for Q4/Q6.

**Key decisions recorded.**
- Progression remains the proposed primary vulnerability but is reframed as **layer-specific**: the user applies anti-Progression rigor to code/design (11 verified resistance instances across 11 sessions) but accepted the AI-authored INV-NNN named concept in docs/architecture without understanding it (Q1, S11, self-caught same-session).
- Mislead is Low, overlap on the same instance; the dominant Mislead pattern is resistance, not compliance.
- All six negatives are grounded by verified quotes showing the opposite habit, not by unverified citations.
- The single actionable rule targets the docs/named-concept layer: restate any AI-authored named concept in your own words and point to the code that enforces it, or it does not go in.
- 06 stays resolved as the preliminary artifact; it is not re-opened.

**Next step.** [Ticket 07](./07-confirm-vulnerability-and-derive-rule.md) is unblocked: HITL grilling to confirm the primary-vulnerability call and own the final rule wording.

## Notes for this ticket

- Inputs ready: 08 (pattern evidence, 32/32 verified), 04 (quote-verification, corrected forms),
  05 (assessability, Achievement split). No blocking.
- The destination (map) is a grounded one-time audit; the final report is the deliverable.
