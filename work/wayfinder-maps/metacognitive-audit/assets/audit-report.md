# Metacognitive audit report — chronicler-engine sessions

Built by [ticket 09](../issues/09-re-synthesize-audit-report.md). Final synthesis from the
pattern-level evidence asset ([ticket 08](./pattern-trap-evidence.md)). Replaces the preliminary
report from [ticket 06](../issues/06-synthesize-audit-report.md); the preliminary version is
preserved in ticket 06's answer and in git history. The pool, flattening, and mechanical
verification decisions are in [ticket 01](./audit-pool-manifest.md), [ticket
02](../issues/02-flatten-sessions.md), and [ticket 04](./quote-verification.md).

**Status: final report — pending user confirmation of primary vulnerability and rule in [ticket 07](../issues/07-confirm-vulnerability-and-derive-rule.md).**

## Grounding & scope

- **Pool.** 30 qualifying sessions, ~161 substantive user turns, 2026-08-11 → 2026-08-20. The
  30-day window is only 10 days of data: the chronicler-engine project's earliest session is
  2026-08-11 (repo moved out of `mrn-general` that day); pre-08-11 work is out of scope (Q5=B).
  This is a data-availability constraint, not a scoping choice.
- **Single source.** The flattened transcripts in `tmp/flattened_pool/` (built by
  [`tmp/flatten_sessions.py`](../../tmp/flatten_sessions.py), per
  [ticket 02](../issues/02-flatten-sessions.md)). The flattener preserves user/assistant text
  byte-identical (demo-verified), so flattened-only verification is safe.
- **Zero-Hallucination guardrail (Q2=B).** Every quote below is an exact verbatim substring of
  its cited flattened transcript, verified mechanically by
  [`tmp/verify_pattern_quotes.py`](../../tmp/verify_pattern_quotes.py) (ticket 08: **32/32 PASS
  exact**). Cost-of-the-gap quotes Q4 and Q6 use the corrected verbatim forms from
  [ticket 04](./quote-verification.md) (markdown backticks restored); the as-written forms that
  failed are not cited. Locator = manifest session number (`S<n>`) + flattened filename + 1-indexed
  line + ISO timestamp.
- **Assessability (Q7=B).** All 8 traps are assessable from transcript
  ([ticket 05](./assessability-table.md)). None is written as "not assessable from transcript."
  Achievement takes a split verdict: instance-level assessable; pattern-level ("endless/repeated")
  needs git diffs. The pool contains no positive Achievement pattern, so no diffs-research ticket
  was needed (ticket 05's conditional resolution).

## 1. Summary Dashboard

- **Total traps detected: 2** — Progression (AI-induced) and Mislead (AI-induced), both from a
  **single instance** in session 11 (2026-08-14). Not double-counted: one event evidences both.
- **Traps with no candidate in this window: 6** — Forming, Dislodging, Assumption, Location,
  Achievement (instance level), Interruption. Each is assessable from transcript and assessed as
  not occurring in this pool; each negative is grounded by a verified quote showing the opposite
  habit, not by an unverified citation.
- **Distinct instances: 1.** The pool shows a highly disciplined user (plan-then-implement,
  code-review after nearly every change, frequent pushback, investigation before accepting,
  scope-aware). The one trap is the exception, not the pattern.
- **Pattern context: the accept/resist ratio.** Resistance is the dominant behavior: **11
  verified resistance instances across 11 sessions** (Q8–Q12, R2, R6, R20a, R27, R30, R29). The
  one acceptance-without-understanding instance (Q1, session 11) sits in the
  **documentation / named-concept / architecture layer**. Acceptance-after-investigation (R20b) is
  healthy and distinct from Progression.
- **Proposed primary cognitive vulnerability: Progression (AI-induced), layer-specific to
  docs/named-concepts, severity High.** One High-impact instance: the user accepted an entire
  AI-authored architectural concept (the INV-NNN "runtime invariants" subsystem — 7 named
  invariants, a contract-test file, a doc section, cross-references) without understanding it; it
  had drifted/broken by the time the user revisited it and was retired that session. The user's own
  admission grounds it. The pattern evidence reframes this from a general habit to a **layer-specific
  gap**: the user already applies anti-Progression rigor to code and design, but not to AI-authored
  named concepts that live in docs and architecture.
- **Why Progression, not Mislead.** Both rest on the same instance. Progression is primary because
  the actionable gap is the *acceptance-without-understanding* (the user's side). Mislead's
  "blindly following" is the same act viewed from the AI-suggestion side, and the user does **not**
  generally do it — their dominant Mislead behavior is *resistance*. So Mislead is detected but Low
  as a vulnerability pattern.

## 2. Detailed Audit Table

One row per trap. Verdicts in this window only; an absence is not proof the trap never occurs.

| # | Trap | Severity | Exact quote(s) (verified) | What happened | Correction strategy (proposed) |
|---|------|----------|---------------------------|---------------|--------------------------------|
| 1 | Forming | — | `Right now I don't really know what I want from it. Because I don't know where is there exactly.` (S25, line 308, 2026-08-19T21:28:32.981Z, F1) | No instance of a wrong mental model driving a right goal in this window. The user's habit is to **surface not-understanding** rather than fake a model. | — |
| 2 | Dislodging | — | `Updating them in place would surely end up with a completely messed up plan so it's better to write clean with whatever remains` (S4, line 3621, 2026-08-12T21:06:22.714Z, D1) | No instance of repeating a failing approach. The user pivots readily: rewrites stale plans cleanly, flips wrong blocking directions, corrects stale maps, and redirects research-vs-implementation scope. | — |
| 3 | Assumption | — | `Yeah, fix it now` (S20, line 3629, 2026-08-15T21:56:11.494Z, A1) | No hardcoded/narrow-case problem-solving by the user. The user tends to **harden and generalize** shortcuts rather than leave them narrow. | — |
| 4 | Location | — | `I am concerned at ticket 11 will affect the other tickets` (S15, line 382, 2026-08-15T16:27:33.863Z, L1) | No "skip architecture, realize too late" instance. Architecture tensions are raised **during** planning (S13, S15) or re-examined mid-design (S24), not realized too late. | — |
| 5 | Achievement | — | `Can you create a plan in docs/plans to investigate this issue and potentionally remove the in memory storage.` (S23, line 3358, 2026-08-16T22:51:27.449Z, AC3) | At instance level, the user favors removing whole layers over band-aiding. Pattern level ("endless/repeated") is not assessable from transcript without diffs; no positive pattern is in evidence. | — |
| 6 | Progression (AI) | **High** | `I kind of just left the AI write it as is but I'm not really sure about it` (S11, line 1813, 2026-08-14T18:36:11.778Z, Q1) | The user accepted the AI-authored INV-NNN "runtime invariants" subsystem without understanding it. It had drifted/broken by the same session — phantom INV-006, double-defined INV-005, ghost-file reference — and was retired that session. The trap was self-caught same-session, but a whole named architectural concept lived un-understood until revisited. | Extend existing code-review discipline to the **docs/named-concept / architecture layer**: before any AI-authored named concept is accepted, restate it in your own words and point to the code that enforces it, or it does not go in. |
| 7 | Interruption (AI) | — | `Oh, not implement` → `Oh, now implement` (S8, lines 3191 → 3194, 9.9s gap, IT1→IT2) | Of 42 sub-60s gaps across 30 sessions, **zero** were confusion→re-prompt. Sub-minute re-sends are typo corrections and amendments; substantive prompts have deliberate (minutes-to-hours) gaps; post-response sub-minute prompts are reasoning-then-answering. The trap did not occur. | — |
| 8 | Mislead (AI) | **Low** (overlap; pattern = resistance) | `I kind of just left the AI write it as is but I'm not really sure about it` (same instance as Progression, S11 line 1813, Q1) | Same event as Progression: the user followed an over-complicated and partly incorrect AI suggestion. This is the **exception**. The dominant pattern is resistance (11 verified instances). The exception sits in the docs/named-concept layer where code-review rigor was not applied. | Same as Progression: verify AI suggestions against the code/spec before accepting; apply this discipline to AI-authored named concepts in docs and architecture. |

### Supporting evidence for the Progression / Mislead instance (session 11)

All verbatim and verified (ticket 08, 32/32 exact). `01a0017d` = flattened file
`2026-08-14T18-16-25-811Z_01a0017d-53d3-7a69-b98f-468e0953ed12.md`.

- **The admission (Q1, PASS exact, line 1813, 2026-08-14T18:36:11.778Z):**

  ```
  I kind of just left the AI write it as is but I'm not really sure about it
  ```

- **Same-session surfacing of not-understanding (F2, PASS exact, line 1813, same turn as Q1):**

  ```
  What does invarient even mean?
  ```

- **Ghost-file reference (Q3, PASS exact, line 1790):**

  ```
  tests/poison_recovery.rs
  ```

- **Double-defined invariant (Q5, PASS exact, line 1990):**

  ```
  Two different guarantees, same number.
  ```

- **Double-defined invariant (Q4, corrected verbatim form per ticket 04, line 1990):**

  ```
  `INV-005` is double-defined
  ```

- **Phantom invariant (Q6, corrected verbatim form per ticket 04, line 1991):**

  ```
  `INV-006` ("All Actions Are Async") has no `fn test_inv006_*` anywhere. It's a guarantee with no enforcement.
  ```

- **The drift summary (Q7, PASS exact, line 1993):**

  ```
  So the table you "let the AI write" has drifted from the code it claims to describe.
  ```

## 3. Single Actionable Rule (proposed)

> Before any AI-authored named concept — an invariant, contract, category, or architecture section —
> enters the repo, restate what it means in your own words and point to the code that enforces it; if
> you cannot, it does not go in.

**Why this rule.** The pattern evidence shows the user's anti-Progression rigor is already strong
in code and design grilling (11 verified resistance instances). The one Progression instance
occurred where that rigor did not extend: the **docs/named-concept / architecture layer**. A named
concept, once accepted, propagates — contract tests, doc cross-refs, later work that assumes it —
so catching it at the door is high-leverage. The rule is a *transfer* of an existing habit to the
layer that currently lacks it, not a new behavior to build from scratch.

**How to apply it.** When the AI proposes a named concept (e.g. "runtime invariants", "guardrails
subsystem", "action pipeline phase model"):

1. **Restate** the concept in one sentence, in your own words, without copying the AI's phrasing.
2. **Point** to the function, test, type, or file that enforces it.
3. If either step fails, reject or refactor the concept until both are possible before accepting it.

**This is a proposal.** [Ticket 07](../issues/07-confirm-vulnerability-and-derive-rule.md) (HITL
grilling) confirms the primary-vulnerability call and owns the final rule. The user may sharpen,
replace, or reject it.

## 4. Pattern context — the accept/resist ratio

This section is not part of the three-part premise output, but it is required to interpret the
verdicts honestly. The 11 verified resistance instances below are **counter-evidence**: they show
why Progression is an exception and why Mislead is Low as a pattern. They also define the
layer-specific gap — the same user who resists code and design over-reach accepted an AI-authored
named concept in docs/architecture without understanding it.

### Verified resistance instances (11 across 11 sessions)

- **S5 · line 2024 · 2026-08-12T22:11:08.431Z (Q8):**
  ```
  I'm looking for an investigation, I'm not sure the review comments are right
  ```
  — demands investigation before accepting AI review findings.

- **S17 · line 1174 · 2026-08-15T17:51:45.954Z (Q9):**
  ```
  It doesn't seem like you are giving me options, you think there is one of real choice for each of those questions?
  ```
  — refuses foregone-conclusion framing in a design grilling.

- **S19 · line 3101 · 2026-08-15T19:38:55.917Z (Q10):**
  ```
  I don't understand why a research ticket is changing code
  ```
  — catches AI scope-creep.

- **S23 · line 3261 · 2026-08-16T22:43:42.370Z (Q11):**
  ```
  If we have tests for BOTH the in memory test and the sqllite, we aren't making anything faster by having in memory test. We are still have the sqllite tests!
  ```
  — corrects an AI overclaim about test-pairing speed.

- **S24 · line 5315 · 2026-08-19T22:24:43.689Z (Q12):**
  ```
  I don't understand where that came from. If that behaviour wasn't in the silly tavern or Marinara Engine prompts then it probably isn't a good idea.
  ```
  — catches the AI adding template macros beyond the spec, provenance-checked.

- **S2 · line 135 · 2026-08-11T23:38:09.381Z (R2):**
  ```
  What was this added? It seems pointless to do it for one specific class and not for the others
  ```
  — questions the purpose of an AI-authored `PresetStore` newtype (later removed).

- **S6 · line 3960 · 2026-08-12T22:35:15.885Z (R6):**
  ```
  I think having a separate ticket for each issue is a bit excessive?
  ```
  — pushes back on AI ticket granularity.

- **S20 · line 3137 · 2026-08-15T21:46:46.492Z (R20a):**
  ```
  It's only called by PipelineRun, so doesn't it logically make sense as a private method in the Impl instead?
  ```
  — corrects an AI recommendation; the AI conceded.

- **S27 · line 293 · 2026-08-19T21:59:45.348Z (R27):**
  ```
  What do you mean by 'pattern to adopt'. 'pattern' suggests something non-deterministic, like a claude rule or skill, which isn't what I want
  ```
  — catches ambiguous terminology; refuses to accept a vague term.

- **S30 · line 436 · 2026-08-20T19:36:24.906Z (R30):**
  ```
  Oh rewrite is likely because of this extension https://github.com/Vistyy/nopus. Not because of the output rules.
  ```
  — catches a confound the AI missed.

- **S29 · line 2389 · 2026-08-19T23:05:55.663Z (R29):**
  ```
  For ticket 12, I don't see what the difference between preflight.py and the existing build.py
  ```
  — questions a redundant proposed script.

### Acceptance-after-investigation contrast (not Progression)

- **S20 · line 3397 · 2026-08-15T21:52:10.408Z (R20b):**
  ```
  Maybe you are right that you should be just inlinng it, calling state.execute_freeaction_impl direclty instead.
  ```
  — the user accepts the AI's held position *after* the AI pushed back with evidence. The typos
  (`inlinng`, `direclty`) are the user's, preserved verbatim. This is healthy, not Progression.

## 5. Grounded negatives — opposite-habit evidence

For each of the six traps with no candidate, one representative verified quote showing the user's
habit runs the other way.

### Forming — opposite habit: surface not-understanding

- **F2 (same turn as Q1):** `What does invarient even mean?`
- **F3 (S20):** `What do you mean by free function? Like, outside of the class?`

### Dislodging — opposite habit: pivot

- **D2 (S15):** `It also seems like ticket 11 shouldn't be blocked by ticket 1, rather ticket 1 should be blocked by ticket 11.`
- **D3 (S22):** `the map is wrong, I guess it wasn't updated`
- **D4 (S28):** `It makes more sense to do research tickets here and then move any spikes or prototypes or implemention into a new map. Because always the next map will be half-baked`

### Assumption — opposite habit: harden and generalize

- **A2 (S14):** `I do want to split out  the multiple impls into separate files`

### Location — opposite habit: raise architecture concerns before acting

- **L2 (S13):** `So I'm not really sure about putting it in action_pipline/pipline because up until now  we don't go that deep into the file structure.`
- **L3 (S24):** `Then perhaps the structure of the narration and the idea of swapping itself is a problem?`

### Achievement — opposite habit: remove rather than band-aid

- **A1 (S20):** `Yeah, fix it now` — removes the `is_cfg_test` substring shortcut.
- **Q1 retirement:** the INV-NNN subsystem was retired, not patched.
- **R2 removal:** `PresetStore` was removed as pointless, not patched.

### Interruption — opposite habit: deliberate pauses; sub-minute re-sends are typo corrections

- **IT1→IT2 (S8, 9.9s):** `Oh, not implement` → `Oh, now implement`
- **IT3→IT4 (S20, 7.1s):** `Of associated function or whatever` → `Or associated function or whatever`
- **IT5→IT6 (S5, 10.0s):** `…make a plan to fix the result.` → `…make a plan to fix the rest.`

Full scan: [`tmp/scan_interruption.py`](../../tmp/scan_interruption.py) found 42 sub-60s gaps; zero
confusion-reprompts.

## Notes for ticket 07

- The **primary-vulnerability proposal** is Progression, layer-specific to AI-authored named
  concepts / docs / architecture, severity High, grounded in the user's own admission (Q1) and
  the same-session drift findings (Q3–Q7). Mislead overlaps but is Low as a pattern because
  resistance dominates.
- The **rule proposal** targets that exact layer: restate + point-to-enforcing-code before any
  AI-authored named concept enters the repo.
- Nothing in this synthesis graduated new fog or ruled work out of scope. The Achievement/diffs fog
  was already resolved conditionally by ticket 05 (no positive pattern → no diffs ticket).
