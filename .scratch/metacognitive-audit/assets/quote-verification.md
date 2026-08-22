# Quote verification report — metacognitive audit

Built by [ticket 04](../../.scratch/metacognitive-audit/issues/04-verify-quotes-mechanically.md).
Verifier: [`tmp/verify_quotes.py`](../../tmp/verify_quotes.py) — opens each cited flattened
transcript and tests exact substring match (the gate), then runs normalization variants to classify
any failure as cosmetic or substantive and prints the closest in-source text.

**Verification source.** The flattened transcripts in `tmp/flattened_pool/` — the audit's single
source, per the [map](../map.md) ticket-02 decision and ticket 04's body. The flattener's docstring
("verify against the RAW originals") is stale (flagged by ticket 03); the map decision is
authoritative. The flattener preserves user/assistant text byte-identical (demo-verified by ticket
02), so flattened-only verification is safe.

**Subject.** Every quote in [`assets/raw-trap-evidence.md`](./raw-trap-evidence.md) (ticket 03's
output). 12 distinct quoted strings were tested: the headline Progression/Mislead trap quote, the
"full user turn" it sits in, four cost-of-the-gap snippets from the same session, and five
resistance quotes across sessions 5, 17, 19, 23, 24.

## Result

**9 of 12 quotes pass exact verbatim substring match as written.** The other 3 fail as written but
each has a corrected verbatim form that passes exact — the failures are formatting omissions
(stripped markdown backticks; one dropped literal `>`), not invented text. No quote is
hallucinated: every quote corresponds to real text in its cited transcript.

| ID | Trap / role | Gate (as written) | Line (exp/found) | Diagnostic / correction |
|---|---|---|---|---|
| Q1 | Progression + Mislead (headline) | **PASS exact** | 1813 / 1813 | `I kind of just left the AI write it as is but I'm not really sure about it` |
| Q2 | Progression (full user turn) | FAIL — paraphrase | 1813 / — | dropped a literal `>`; corrected form below **PASS exact** |
| Q3 | Progression (ghost file) | **PASS exact** | 1790 / 1790 | `tests/poison_recovery.rs` |
| Q4 | Progression (INV-005) | FAIL — cosmetic | 1990 / — | stripped markdown backticks; corrected form below **PASS exact** |
| Q5 | Progression (same number) | **PASS exact** | 1990 / 1990 | `Two different guarantees, same number.` |
| Q6 | Progression (INV-006) | FAIL — cosmetic | 1991 / — | stripped markdown backticks; corrected form below **PASS exact** |
| Q7 | Progression (drift summary) | **PASS exact** | 1993 / 1993 | `So the table you "let the AI write" has drifted from the code it claims to describe.` |
| Q8 | Resistance (S5) | **PASS exact** | 2024 / 2024 | `I'm looking for an investigation, I'm not sure the review comments are right` |
| Q9 | Resistance (S17) | **PASS exact** | 1174 / 1174 | `It doesn't seem like you are giving me options, you think there is one of real choice for each of those questions?` |
| Q10 | Resistance (S19) | **PASS exact** | 3101 / 3101 | `I don't understand why a research ticket is changing code` |
| Q11 | Resistance (S23) | **PASS exact** | 3261 / 3261 | `If we have tests for BOTH the in memory test and the sqllite, we aren't making anything faster by having in memory test. We are still have the sqllite tests!` |
| Q12 | Resistance (S24) | **PASS exact** | 5315 / 5315 | `I don't understand where that came from. If that behaviour wasn't in the silly tavern or Marinara Engine prompts then it probably isn't a good idea.` |

Cited flattened files (session → basename, confirmed against the [audit-pool manifest](./audit-pool-manifest.md)):

| Session | Flattened file |
|---|---|
| 11 (`01a0017d`) | `2026-08-14T18-16-25-811Z_01a0017d-53d3-7a69-b98f-468e0953ed12.md` |
| 5 (`019ff7e2`) | `2026-08-12T21-30-18-396Z_019ff7e2-1b9c-7bc7-9a93-4c4c40186831.md` |
| 17 (`01a00688`) | `2026-08-15T17-46-25-393Z_01a00688-36f1-7af7-a852-cf50dcf2dc15.md` |
| 19 (`01a006e8`) | `2026-08-15T19-31-30-859Z_01a006e8-6dab-7862-8ea4-6c301ae293a5.md` |
| 23 (`01a00c8f`) | `2026-08-16T21-51-51-270Z_01a00c8f-45de-7366-9d09-5fec325e29b6.md` |
| 24 (`01a01bda`) | `2026-08-19T21-08-17-353Z_01a01bda-7748-7313-b0e5-efe5950c09e9.md` |

## Corrected verbatim forms for the 3 failed quotes

Each corrected form below is itself an **exact substring** of the cited flattened file (re-verified
by the script). Ticket 06 must cite these forms (or the passing-equivalent quotes noted), not the
as-written forms in `raw-trap-evidence.md`.

### Q2 — the full user turn (dropped a literal `>`)

- As written in the evidence: `Right so maybe we need to rethink invarients as part of this? What does invarient even mean? I kind of just left the AI write it as is but I'm not really sure about it`
- **Corrected verbatim** (line 1813, exact PASS): `Right so maybe we need to rethink invarients as part of this? What does invarient even mean?> I kind of just left the AI write it as is but I'm not really sure about it`
- The transcript has `mean?> I kind of` — a literal `>` between the two sentences. The extraction dropped it.
- **Recommendation to ticket 06:** cite the headline slice **Q1** (the `I kind of just left the AI write it as is...` portion), which is exact-verbatim on its own; the "full turn" wrapper is not needed for the finding.

### Q4 — INV-005 (stripped markdown backticks)

- As written: `INV-005 is double-defined`
- **Corrected verbatim** (line 1990, exact PASS): `` `INV-005` is double-defined `` (markdown backticks around `INV-005`)
- **Recommendation to ticket 06:** cite the exact-passing **Q5** (`Two different guarantees, same number.`) for the same point, or this corrected form.

### Q6 — INV-006 (stripped markdown backticks)

- As written: `INV-006 ("All Actions Are Async") has no fn test_inv006_* anywhere. It's a guarantee with no enforcement.`
- **Corrected verbatim** (line 1991, exact PASS): `` `INV-006` ("All Actions Are Async") has no `fn test_inv006_*` anywhere. It's a guarantee with no enforcement. `` (markdown backticks around `INV-006` and `fn test_inv006_*`)
- **Recommendation to ticket 06:** cite this corrected form.

## Integrity conclusion

- **The audit's core finding is grounded.** The headline Progression/Mislead trap evidence (Q1) is
  an exact verbatim substring of the session-11 flattened transcript at line 1813. The user did say
  it.
- **The "disciplined user / resistance is the pattern" context is grounded.** All five resistance
  quotes (Q8–Q12) pass exact verbatim, each at its cited line.
- **The cost-of-the-gap narrative is grounded.** Q3, Q5, Q7 pass exact; Q4 and Q6 have corrected
  verbatim forms that pass exact. No cost quote is invented.
- **Zero-Hallucination guardrail (Q2=B): holds in substance, with 3 formatting corrections.**
  Every quote corresponds to real text in its cited transcript; the three as-written failures are
  formatting omissions (stripped backticks, one dropped `>`), not fabricated content. Strict
  verbatim compliance is restored by the corrected forms above.
- **Line attribution.** Every exact-pass quote was found at its expected line; no line-number drift
  was detected. (Q4/Q5 were both attributed by the evidence to line 1990; Q5 sits on 1990, Q4's
  corrected form also on 1990 — consistent.)

## Notes for downstream tickets

- **Ticket 06 (synthesis).** Use only exact-passing quotes, or the corrected verbatim forms above.
  Do not reproduce the as-written forms of Q2, Q4, Q6. The primary-vulnerability proposal
  (Progression) rests on Q1, which is exact-verified — the proposal is not weakened by the three
  formatting corrections.
- **Ticket 05 (assessability).** Verification does not change assessability. Progression and Mislead
  remain assessable (Q1 grounds them); the cost quotes confirm the same-session drift the user
  themselves reacted to. No fog graduated by this verification; the map's Achievement/diffs fog
  remains for ticket 05.
- **Process note (no action required).** Ticket 03's answer claimed "9/9 spot-verified by grep"; it
  spot-verified 9, not all 12, so the three as-written failures (Q2, Q4, Q6) were not caught at
  extraction. This full mechanical pass (ticket 04) is the authoritative check. The
  `raw-trap-evidence.md` asset is left intact as ticket 03's output; the corrected forms live here so
  each ticket's artifact stays auditable. No re-open of ticket 03 is needed — the corrections are
  mechanical and recorded.
