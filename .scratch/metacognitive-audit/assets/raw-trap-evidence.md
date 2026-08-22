# Raw trap evidence — metacognitive audit

Built by [ticket 03](../../.scratch/metacognitive-audit/issues/03-extract-trap-evidence.md).
Source: the flattened transcripts in `tmp/flattened_pool/` — the audit's single source (per the
[map](../../.scratch/metacognitive-audit/map.md) and [ticket 02](../../.scratch/metacognitive-audit/issues/02-flatten-sessions.md)).
Every quote below is a verbatim substring of the cited flattened file; ticket 04 mechanically
re-verifies each by substring match.

**Locator convention.** The flattener (`tmp/flatten_sessions.py`) preserves the ISO timestamp on
every entry but does not preserve a separate message id. So the **timestamp + flattened filename +
line number** is the message locator. "line N" is the 1-indexed line of the user text in the
flattened file; the `## @ <ts> — user` header sits on line N−1.

**Trap reference (from the premise, verbatim):**
1. Forming — right goal, wrong mental model
2. Dislodging — repeating a failing approach instead of pivoting
3. Assumption — solving a hardcoded/narrow case, not generalized
4. Location — skipping core architecture, realizing too late
5. Achievement — endless band-aid fixes instead of refactoring
6. Progression (AI) — accepting generated code beyond understanding
7. Interruption (AI) — prompting within seconds of confusion, not reasoning through the pause
8. Mislead (AI) — blindly following off-target/over-complicated/incorrect AI suggestions

**Pool.** 30 qualifying sessions, 2026-08-11 → 2026-08-20 (per
[ticket 01](../../.scratch/metacognitive-audit/issues/01-select-audit-pool.md)). All 30 read in
full from the flattened transcripts (user turns verbatim + the preceding assistant stimulus); this
file records only sessions with a candidate.

**Headline.** The pool shows a highly disciplined user: plan-then-implement, code-review after
nearly every change, frequent pushback, investigation before accepting, scope-aware. Traps are rare.
One clear trap is detected — **Progression**, in session 11 — with a **Mislead** overlap on the same
instance. All other traps have no candidate and are not forced.

---

## Trap 6 — Progression (AI-induced)

### Session 11 — `01a0017d` (guardrails.md auto-gen + INV-NNN invariants rethink)

- **Timestamp:** 2026-08-14T18:36:11.778Z · **line 1813**
- **Flattened file:** `tmp/flattened_pool/2026-08-14T18-16-25-811Z_01a0017d-53d3-7a69-b98f-468e0953ed12.md`
- **Verbatim quote:** `I kind of just left the AI write it as is but I'm not really sure about it`
- **Full user turn:** `Right so maybe we need to rethink invarients as part of this? What does invarient even mean? I kind of just left the AI write it as is but I'm not really sure about it`
- **Context (one line).** Re-examining the repo's INV-NNN "runtime invariants" category (§5 of guardrails.md + `invariant_contract.rs` + labels INV-001..INV-007 + cross-refs from architecture.md), the user admits they let the AI author the whole subsystem and do not understand it.
- **Cost of the gap (confirmed in the same session, from the flattened transcript).** The AI's investigation finds the AI-generated system had drifted / was broken:
  - `INV-005 is double-defined` — "Two different guarantees, same number." (line 1990)
  - `INV-006 ("All Actions Are Async") has no fn test_inv006_* anywhere. It's a guarantee with no enforcement.` (line 1991)
  - §5 points to `tests/poison_recovery.rs`, which does not exist — a ghost file (line 1790).
  - The AI's summary: `So the table you "let the AI write" has drifted from the code it claims to describe.` (line 1993)
  The subsystem was retired this session (§5 and the INV-NNN category removed; the underlying unit tests kept).
- **Proposed severity (for ticket 06 to finalize): High.** A whole AI-generated architectural concept — 7 named invariants, a contract-test file, a doc section, and cross-references — was accepted without understanding and lived in the repo until the user happened to revisit it.

---

## Trap 8 — Mislead (AI-induced)

### Session 11 — `01a0017d` (same instance as the Progression candidate above)

- **Timestamp / line / file:** same as the Progression candidate (2026-08-14T18:36:11.778Z · line 1813).
- **Verbatim quote:** `I kind of just left the AI write it as is but I'm not really sure about it`
- **Context (one line).** The INV-NNN invariants subsystem was an over-complicated and partly incorrect AI suggestion (tautological relabels INV-003/INV-005; phantom INV-006; colliding INV-005; mismatched INV-007; ghost-file reference), and the user followed it — let it stand.
- **Cross-reference, not double-counted.** This is the *same event* as the Progression candidate. It evidences both traps: accepted without understanding (Progression) **and** an off-target / over-complicated AI suggestion that was followed (Mislead). Recorded under both traps because one event can ground either; ticket 06 decides how to present.
- **The user's dominant Mislead behavior is resistance, not compliance** — see "Observed resistance" below. The invariants instance is the exception, not the pattern: it sat in the documentation / named-concept layer, which the user reviewed far less rigorously than code.

---

## Traps with no candidate (not forced)

Per the premise's Zero-Hallucination guardrail, a trap with no candidate is omitted, not invented.

- **Trap 1 (Forming): no candidate.** The user's mental models are correct or self-corrected quickly; no instance of working from a wrong model toward a right goal. (The S22 tavily-config path error is tooling/environment, excluded per ticket 01.)
- **Trap 2 (Dislodging): no candidate.** The user pivots readily when an approach fails — condenses over-granular ticket sets (S6), flips a wrong blocking direction (S15), corrects the map (S22 "the map is wrong").
- **Trap 3 (Assumption): no candidate.** No hardcoded / narrow-case problem-solving by the user. (The `is_cfg_test` substring match was AI-written guardrail code the user asked to harden — S20 — not a user assumption.)
- **Trap 4 (Location): no candidate.** The user is architecture-aware; guardrail-collision tensions (S13, S15) are caught during planning, not realized too late.
- **Trap 5 (Achievement): no candidate.** The user favors refactoring over band-aids (preset-store removal, pipeline split, invariants retirement) and *removes* existing band-aids (the `is_cfg_test` `ponytail:` shortcut — S20 "Yeah, fix it now").
- **Trap 7 (Interruption): no clear candidate.** Substantive user prompts have deliberate gaps (minutes to hours). Sub-minute re-sends are typo corrections (e.g. "Oh, not implement" → "Oh, now implement"; "Of associated function or whatever" → "Or associated function or whatever"), not confusion → prompting. Timestamps are millisecond-precise (ticket 01 established fact), so Interruption *is* assessable; the assessment is that the trap did not occur.

---

## Observed resistance to AI-induced traps (context, not trap candidates)

These verbatim user prompts show the user's dominant habit — catching AI over-reach and refusing
unverified AI output. Recorded as context for the correction strategy (ticket 06) and the rule
(ticket 07): the user already applies anti-Progression / anti-Mislead discipline in code-review and
design-grilling contexts; the gap (the invariants instance) is in the documentation / named-concept /
architecture layer, where the user let the AI author freely. Each quote is a verbatim substring of
the cited flattened file.

- **S5** (`019ff7e2`) — 2026-08-12T22:11:08.431Z · line 2024:
  `I'm looking for an investigation, I'm not sure the review comments are right`
  — demands investigation before accepting AI review findings (the AI confirmed them, then the user accepted).
- **S17** (`01a00688`) — 2026-08-15T17:51:45.954Z · line 1174:
  `It doesn't seem like you are giving me options, you think there is one of real choice for each of those questions?`
  — refuses to rubber-stamp foregone-conclusion framing in a design grilling.
- **S19** (`01a006e8`) — 2026-08-15T19:38:55.917Z · line 3101:
  `I don't understand why a research ticket is changing code`
  — catches AI scope-creep; the AI agrees it over-interpreted the ticket.
- **S23** (`01a00c8f`) — 2026-08-16T22:43:42.370Z · line 3261:
  `If we have tests for BOTH the in memory test and the sqllite, we aren't making anything faster by having in memory test. We are still have the sqllite tests!`
  — corrects an AI overclaim about test-pairing speed; the AI concedes.
- **S24** (`01a01bda`) — 2026-08-19T22:24:43.689Z · line 5315:
  `I don't understand where that came from. If that behaviour wasn't in the silly tavern or Marinara Engine prompts then it probably isn't a good idea.`
  — catches the AI adding template macros beyond the spec; provenance-checked against the reference implementations.

---

## Notes for downstream tickets

- **Ticket 04 (verification).** Every quote above was located by `grep` against the cited flattened
  file and is an exact substring. The flattener's docstring says "Verification runs against the RAW
  originals" — that docstring is **stale**; the map's ticket-02 decision and ticket 04's body both
  specify the flattened files as the verification source. Follow the map (flattened files), not the
  docstring. The `line N` values are 1-indexed lines in the flattened file for diagnosis.
- **Ticket 05 (assessability).** From this extraction: Progression and Mislead are assessable from
  transcript (the user's own admission + the AI's drift findings are both in the transcript).
  Interruption is assessable (millisecond timestamps) and assessed as not-occurring. Achievement is
  assessable from transcript here because the user's refactor-vs-band-aid choices are stated — but
  the map's fog (diffs may be needed for some Achievement calls) still applies to impl sessions not
  in this design-heavy pool.
- **Ticket 06 (synthesis).** Proposed primary vulnerability: **Progression** (one High instance,
  the invariants). Mislead overlaps on the same instance; the user's dominant Mislead behavior is
  resistance. This is a proposal for ticket 07 (HITL grilling) to confirm with the user.
