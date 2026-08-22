# Re-extract trap evidence at pattern level

Type: task
Status: resolved
Blocked by: (none — inputs 01, 02, 05 are resolved)
Supersedes (for synthesis): [03](./03-extract-trap-evidence.md)

## Question

Ticket 03 extracted trap evidence at the **instance** level — it scanned the 30 flattened sessions for textbook trap moments, found one (Progression, session 11), and filed the rest as "context" or "None detected." That method could find a clean hit; it could not find a **pattern**. The map's destination is a grounded audit of the user's problem-solving habits across the pool, not one verified instance. n=1 is a property of the method, not the data — 10 days, 30 sessions, ~161 substantive turns is sufficient (see map Notes).

Re-read the same 30 flattened transcripts in `tmp/flattened_pool/` (no new data, no scope change) at **pattern level**, and produce a new evidence asset `assets/pattern-trap-evidence.md` (leave `assets/raw-trap-evidence.md` intact as 03's auditable artifact). Pattern level means:

- **Recurring tendencies, not just textbook hits.** Read for the gradient: milder forms of each trap that never reach a clean instance (skimming a generated block, rubber-stamping a rename, accepting a framing without testing it), and how often each surfaces across the 30 sessions.
- **The accept/resist ratio.** The five "resistance" quotes 03 filed as context (S5/17/19/23/24) are the closest thing to a pattern. Read them *as* the pattern: when does the user accept AI output vs resist it, and what conditions trigger each? The INV-NNN instance is one data point on the accept side; trace whether there are others.
- **Ground the negatives to the same standard as the positive.** 06's six "None detected" verdicts currently rest on unverified session citations (S6, S15, S22…), not the verified-quote standard Q1 meets. Either ground each negative with a verified quote showing the trap's absence (the user pivoting, the user restating a correct model), or mark it honestly as "no candidate found in this window" — an absence over 10 days, not proof the trap doesn't occur. Do not present absence and the one positive at equal confidence.

**Zero-Hallucination guardrail (Q2=B).** Every quote cited must be an exact verbatim substring of its cited flattened transcript, verified mechanically (reuse [`tmp/verify_quotes.py`](../../tmp/verify_quotes.py) or equivalent substring match). 03's inline `grep` spot-verified only 9/12 (ticket 04 caught the gap); inline grep is insufficient. Verify all.

**Do not pre-decide the outcome.** The re-extraction may (a) confirm n=1 — the INV-NNN instance is genuinely the only Progression moment and the negatives hold; (b) find a richer pattern — milder Progression moments, a real accept/resist shape, or additional instances; or (c) find the negatives are weaker than 06 claimed. Report whichever the data shows. The point is the data has not yet been read for a pattern, so n=1 is premature, not final.

This ticket supersedes 03's instance-level evidence for synthesis. [Ticket 06](./06-synthesize-audit-report.md)'s report (`assets/audit-report.md`) is **preliminary** until re-synthesized from this ticket's output; a re-synthesis ticket graduates from this ticket's result. [Ticket 07](./07-confirm-vulnerability-and-derive-rule.md) (HITL confirmation) waits on the re-synthesis.

## Notes for this ticket

- Inputs ready: the pool ([ticket 01](./01-select-audit-pool.md)), the flattener output ([ticket 02](./02-flatten-sessions.md)), the assessability table ([ticket 05](./05-declare-per-trap-assessability.md) — all 8 traps assessable; Achievement split). No blocking.
- Reading aids from 03 (`tmp/extract_user_turns.py`, `tmp/extract_signal.py`) may be reused; note 03's known locator-misattribution bug and take final locators by `grep` from the flattened files.
- Output asset: `assets/pattern-trap-evidence.md`, organized per-trap — each candidate with verbatim quote + flattened file + line + timestamp + one-line context, and each negative with its grounding (verified quote or honest absence marker).

## Answer

Asset: [`assets/pattern-trap-evidence.md`](../assets/pattern-trap-evidence.md). Verifiers:
[`tmp/verify_pattern_quotes.py`](../../tmp/verify_pattern_quotes.py) (32/32 PASS exact verbatim),
[`tmp/scan_interruption.py`](../../tmp/scan_interruption.py) (all 42 sub-60s gaps classified),
[`tmp/locators.py`](../../tmp/locators.py) (authoritative line+timestamp per quote). Reading aid:
[`tmp/extract_signal.py`](../../tmp/extract_signal.py) (regenerated, 283 user turns kept verbatim
where reasoning; skill blocks and pastes collapsed to markers). `assets/raw-trap-evidence.md` left
intact as 03's auditable artifact.

**What was done.** Re-read the same 30 flattened transcripts at pattern level: recurring tendencies
and milder forms (the gradient), the accept/resist ratio, and every negative grounded to the
verified-quote standard. Read for the milder Progression forms the ticket named (skimming a
generated block, rubber-stamping a rename, accepting a framing without testing it) — none met the
bar. The outcome was not pre-decided; the data shows a mix the ticket anticipated: (a) the one clear
Progression instance (S11) holds, with no additional clear instance; (b) the accept/resist shape is
real and rich — resistance is the dominant pattern; (c) the negatives are stronger than 06 claimed,
each now grounded by a verified quote.

**Findings.**

- **Progression: one verified instance (High), self-caught the same session.** Q1 (S11, line 1813,
  2026-08-14T18:36:11.778Z) — the user accepted the AI-authored INV-NNN subsystem without
  understanding it; same-session they surfaced the not-understanding (F2), drove a rethink, found the
  drift, and retired it. No additional clear instance; milder forms read for and not found.
- **The accept/resist ratio is the pattern.** 11 verified resistance instances across 11 sessions
  (Q8–Q12, R2, R6, R20a, R27, R30, R29) — probing, source-verifying, catching over-reach, demanding
  options; the AI concedes in several. Acceptance-without-understanding is n=1 (Q1, docs/named-concept
  layer). Acceptance-after-investigation (R20b) is healthy and distinct from Progression. The gap is
  layer-specific: code/design = high rigor; docs/named-concept = where Q1 slipped through.
- **Six negatives, each grounded** (not unverified citations): Forming → surface uncertainty (F1–F3);
  Dislodging → pivot (D1–D4); Assumption → harden/generalize (A1–A2); Location → plan-first (L1–L3);
  Achievement → refactor/remove (AC3 + A1 + the Q1 retirement; pattern-level split from ticket 05
  holds, no positive pattern, no diffs ticket); Interruption → typo-corrections (IT1–IT6) plus a
  full scan: 42 sub-60s gaps across 30 sessions, zero confusion-reprompts (the pause is reasoning).
- **Mislead: Low, overlap on Q1.** Resistance is the dominant Mislead behavior; the INV-NNN is the
  exception in the docs/concept layer.

**Consequence for synthesis.** 06's "primary vulnerability = Progression (High)" is supportable on
the instance but reframed by the pattern: the vulnerability is **layer-specific** (docs/named-concept
/ architecture), not a general accept-without-understanding habit. This sharpens, not overturns, the
06 proposal — the rule (restate any AI-authored named concept in your own words and point to its
enforcing code, or it doesn't go in) targets exactly that layer.

**Fog graduated.** The map's "Re-synthesis of the audit report" fog is now specifiable → new ticket
[09](./09-re-synthesize-audit-report.md) (re-synthesize from this asset; do not re-open 06).
[Ticket 07](./07-confirm-vulnerability-and-derive-rule.md) rewired to block on 09. The map's
"Mechanical re-verification of 08's quotes" fog is **resolved without a ticket**: 08 verified
mechanically inline (32/32 PASS), so a separate verification pass is not needed — that fog patch
clears. No work ruled out of scope.
