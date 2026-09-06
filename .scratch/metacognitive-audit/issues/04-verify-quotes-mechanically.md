# Mechanically verify every evidence quote

Type: task
Status: resolved
Blocked by: 03

## Question

For every quote in the raw evidence file (ticket 03), confirm the exact text occurs verbatim (substring match) in the cited flattened transcript (ticket 02 output).

The flattened transcripts are the audit's single source: extraction (ticket 03) reads only them, so any quote not found in the flattened file is a hallucination. Verification runs against the flattened files, not the raw jsonl. Safe because the flattener preserves user and assistant text byte-identical (confirmed by the ticket 02 demo integrity check) — the quoted content is faithful to the raw session, just without the noise.

Produce a verification report: each quote marked verified or failed. For any failed quote, record the actual text found in the source (the closest match), so the failure is diagnosable.

A failed quote is itself a finding: the auditor hallucinated or paraphrased, which breaks the premise's Zero-Hallucination guardrail (Q2=B). Flag failed quotes back to the extraction step (ticket 03) for correction or removal.

A script that opens each cited flattened transcript and checks the substring is the cleanest implementation.

## Answer

Verification done with [`tmp/verify_quotes.py`](../../../tmp/verify_quotes.py) — opens each cited flattened transcript, tests exact substring match (the gate), and on failure runs normalization variants to classify the failure and print the closest in-source text. Report at [`assets/quote-verification.md`](../assets/quote-verification.md).

**Result: 9 of 12 quotes pass exact verbatim substring match as written.** The other 3 fail as written but each has a corrected verbatim form that passes exact — the failures are formatting omissions, not invented text. No quote is hallucinated.

- **Q1 (the headline Progression/Mislead trap quote)** — **PASS exact**, line 1813: `I kind of just left the AI write it as is but I'm not really sure about it`. The audit's core finding is grounded.
- **Q8–Q12 (all five resistance quotes, S5/17/19/23/24)** — **PASS exact**, each at its cited line. The "disciplined user / resistance is the pattern" context is grounded.
- **Q3, Q5, Q7 (cost-of-the-gap quotes)** — **PASS exact**.
- **Q2 ("full user turn")** — FAIL: dropped a literal `>` the transcript contains (`mean?> I kind of`). Corrected form passes exact. Recommendation: ticket 06 cites Q1 (the verbatim slice) and drops the "full turn" wrapper.
- **Q4, Q6 (INV-005 / INV-006 cost quotes)** — FAIL-cosmetic: stripped markdown backticks the transcript contains (`` `INV-005` ``, `` `INV-006` ``, `` `fn test_inv006_*` ``). Corrected verbatim forms pass exact.

**Zero-Hallucination guardrail (Q2=B): holds in substance, with 3 formatting corrections.** Every quote corresponds to real text in its cited transcript; the three as-written failures are formatting omissions (backticks, one `>`), not fabricated content. Strict verbatim compliance is restored by the corrected forms in the report. Every exact-pass quote was found at its expected line — no line drift.

**For ticket 06:** cite only exact-passing quotes or the corrected verbatim forms in the report; do not reproduce the as-written forms of Q2, Q4, Q6. The Progression proposal rests on Q1 (exact-verified) and is not weakened by the corrections. The `raw-trap-evidence.md` asset is left intact as ticket 03's output; corrected forms live in the verification report so each ticket's artifact stays auditable.

**No fog graduated; no new tickets; nothing ruled out of scope** by this verification. The map's Achievement/diffs fog remains for ticket 05.
