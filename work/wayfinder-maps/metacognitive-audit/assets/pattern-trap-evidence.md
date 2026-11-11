# Pattern trap evidence — metacognitive audit

Built by [ticket 08](../issues/08-re-extract-at-pattern-level.md). Re-reads the same 30 flattened
transcripts as [ticket 03](./raw-trap-evidence.md) at **pattern level** (not instance level),
per the map's course correction. Supersedes 03's instance-level evidence for synthesis:
[ticket 06](../issues/06-synthesize-audit-report.md)'s report is preliminary until re-synthesized
from this asset. `assets/raw-trap-evidence.md` is left intact as 03's auditable artifact.

**Source.** The flattened transcripts in `tmp/flattened_pool/` — the audit's single source (per the
map's [ticket-02](../issues/02-flatten-sessions.md) decision). Read via `tmp/extract_signal.py`
(a reading aid that keeps user reasoning verbatim, collapses skill blocks and pasted plans to
one-line markers, and truncates long turns to a verbatim head — quotes are lifted only from the
verbatim portions). 283 user turns across 30 sessions were read; this file records the pattern.

**Zero-Hallucination guardrail (Q2=B).** Every quote cited below is an exact verbatim substring of
its cited flattened transcript, verified mechanically by [`tmp/verify_pattern_quotes.py`](../../tmp/verify_pattern_quotes.py)
(reuses ticket 04's exact-substring gate + normalization classifier). **Result: 32/32 quotes pass
exact verbatim.** No quote is hallucinated; four were corrected to the user's actual typo'd / double-
spaced text (`inlinng`, `direclty`, `potentionally`, `pipline`, double spaces) before passing — the
Zero-Hallucination standard requires the user's text as written, typos included. The Interruption
negative is grounded by a full scan ([`tmp/scan_interruption.py`](../../tmp/scan_interruption.py)):
all 42 sub-60s gaps between consecutive user prompts across the 30 sessions, each classified.

**Locator convention.** `S<n>` = manifest session number (1–30). `line N` = 1-indexed line of the
quote's first character in the flattened file (the body line, not the `## @ <ts> — user` header).
Timestamp = the ISO millisecond stamp on that user turn's header. Session → flattened-file basename:

| S | Flattened file | | S | Flattened file |
|---|---|---|---|---|
| 2 | `2026-08-11T23-36-33-239Z_019ff32f-…3e32.md` | | 19 | `2026-08-15T19-31-30-859Z_01a006e8-…293a5.md` |
| 4 | `2026-08-12T20-26-55-057Z_019ff7a8-…6051.md` | | 20 | `2026-08-15T21-05-05-528Z_01a0073e-…13561.md` |
| 5 | `2026-08-12T21-30-18-396Z_019ff7e2-…6831.md` | | 22 | `2026-08-16T08-48-32-912Z_01a009c2-…68f9.md` |
| 6 | `2026-08-12T21-58-44-358Z_019ff7fc-…3f614.md` | | 23 | `2026-08-16T21-51-51-270Z_01a00c8f-…29b6.md` |
| 8 | `2026-08-13T22-16-35-411Z_019ffd32-…223b.md` | | 24 | `2026-08-19T21-08-17-353Z_01a01bda-…09e9.md` |
| 11 | `2026-08-14T18-16-25-811Z_01a0017d-…3ed12.md` | | 25 | `2026-08-19T21-21-30-157Z_01a01be6-…21c0.md` |
| 13 | `2026-08-15T12-37-23-994Z_01a0056d-…617c.md` | | 27 | `2026-08-19T21-56-39-243Z_01a01c06-…6c57.md` |
| 14 | `2026-08-15T14-20-24-881Z_01a005cb-…efa1.md` | | 28 | `2026-08-19T22-08-04-984Z_01a01c11-…250a.md` |
| 15 | `2026-08-15T16-25-47-176Z_01a0063e-…0c5f.md` | | 29 | `2026-08-19T22-46-23-532Z_01a01c34-…6c9e.md` |
| 17 | `2026-08-15T17-46-25-393Z_01a00688-…dc15.md` | | 30 | `2026-08-20T19-25-11-367Z_01a020a2-…338a.md` |

## Method — what "pattern level" changed

Ticket 03 scanned for textbook trap moments (instance level) and found one (Progression, S11).
This pass reads the same transcripts for the **gradient**: recurring tendencies, milder forms, and
the **accept/resist ratio** — when the user accepts AI output versus resists it, and what triggers
each. It also grounds every negative to the same verified-quote standard the positive meets (Q2=B),
rather than the unverified session citations ticket 06's six "None detected" verdicts rested on.

**The outcome was not pre-decided** (per the ticket). The data shows a mix: (a) the one clear
Progression instance (S11) holds — no additional clear instance was found, though the transcripts
were read for milder forms; (b) the accept/resist shape is real and rich — resistance is the
dominant pattern across the pool, with the INV-NNN as a layer-specific exception; (c) the negatives
are **stronger** than 06 claimed — each is now grounded by a verified quote showing the opposite
habit, not an unverified citation.

## The accept/resist ratio — the pattern across the pool

This is the headline pattern-level finding. The user's **dominant behavior is resistance**: probing
AI suggestions, demanding options, source-verifying against reference implementations, catching
over-reach, and refusing unverified output. It appears in nearly every session with substantive
turns. Eleven verified instances across eleven sessions (Q8–Q12 plus R2, R6, R20a, R27, R30, R29):

- **S5 · line 2024 · 2026-08-12T22:11:08.431Z (Q8):**
  `I'm looking for an investigation, I'm not sure the review comments are right`
  — demands investigation before accepting AI review findings (the AI confirmed them, then the user accepted).
- **S17 · line 1174 · 2026-08-15T17:51:45.954Z (Q9):**
  `It doesn't seem like you are giving me options, you think there is one of real choice for each of those questions?`
  — refuses foregone-conclusion framing in a design grilling.
- **S19 · line 3101 · 2026-08-15T19:38:55.917Z (Q10):**
  `I don't understand why a research ticket is changing code`
  — catches AI scope-creep (the AI agreed it over-interpreted the ticket).
- **S23 · line 3261 · 2026-08-16T22:43:42.370Z (Q11):**
  `If we have tests for BOTH the in memory test and the sqllite, we aren't making anything faster by having in memory test. We are still have the sqllite tests!`
  — corrects an AI overclaim about test-pairing speed (the AI conceded).
- **S24 · line 5315 · 2026-08-19T22:24:43.689Z (Q12):**
  `I don't understand where that came from. If that behaviour wasn't in the silly tavern or Marinara Engine prompts then it probably isn't a good idea.`
  — catches the AI adding template macros beyond the spec, provenance-checked.
- **S2 · line 135 · 2026-08-11T23:38:09.381Z (R2):**
  `What was this added? It seems pointless to do it for one specific class and not for the others`
  — questions the purpose of an AI-authored `PresetStore` newtype (later removed).
- **S6 · line 3960 · 2026-08-12T22:35:15.885Z (R6):**
  `I think having a separate ticket for each issue is a bit excessive?`
  — pushes back on AI's ticket granularity (map condensed 11→6 tickets).
- **S20 · line 3137 · 2026-08-15T21:46:46.492Z (R20a):**
  `It's only called by PipelineRun, so doesn't it logically make sense as a private method in the Impl instead?`
  — corrects an AI recommendation (a free function); the AI conceded ("You're right, and I overcorrected").
- **S27 · line 293 · 2026-08-19T21:59:45.348Z (R27):**
  `What do you mean by 'pattern to adopt'. 'pattern' suggests something non-deterministic, like a claude rule or skill, which isn't what I want`
  — catches ambiguous terminology; refuses to accept a vague term.
- **S30 · line 436 · 2026-08-20T19:36:24.906Z (R30):**
  `Oh rewrite is likely because of this extension https://github.com/Vistyy/nopus. Not because of the output rules.`
  — catches a confound the AI missed (a nopus rewrite, not the model restating).
- **S29 · line 2389 · 2026-08-19T23:05:55.663Z (R29):**
  `For ticket 12, I don't see what the difference between preflight.py and the existing build.py`
  — questions a redundant proposed script (the AI revised the ticket twice, then dropped the idea).

**The accept side.** Routine acceptances (`Yes`, `confirm`, `implement the plan`, `do it now`,
option letters) are frequent — but these are acceptances of plans the user drove through grilling or
of code matching a stated intent, **not** acceptance-without-understanding. The distinction matters:
Progression is accepting generated concepts *beyond understanding*. The one such instance is Q1
(below). A second, instructive contrast — acceptance **after** investigation — is:

- **S20 · line 3397 · 2026-08-15T21:52:10.408Z (R20b):**
  `Maybe you are right that you should be just inlinng it, calling state.execute_freeaction_impl direclty instead.`
  — the user accepts the AI's held position (to inline), *after* the AI pushed back with evidence. This is
  the opposite of Progression: the user heard the AI's reasoning, weighed it, and conceded. (Typos
  `inlinng`, `direclty` are the user's, preserved verbatim.)

**The layer-specific gap.** The resistance pattern holds in code review and design grilling. The one
acceptance-without-understanding (Q1) sat in the **documentation / named-concept / architecture
layer** (the INV-NNN "runtime invariants" subsystem), which the user reviewed far less rigorously than
code. This is the actionable shape of the vulnerability: not a general accept-without-understanding
habit, but a layer where the user's existing code-review rigor did not extend.

## Per-trap findings

### Trap 1 — Forming (right goal, wrong mental model)

**Candidate: none in this 10-day window.** No instance of a wrong mental model driving a right goal.

**Negative grounding.** The user's habit is to **surface not-understanding** rather than drive a wrong
model — the opposite of Forming. Three verified instances:

- **S25 · line 308 · 2026-08-19T21:28:32.981Z (F1):**
  `Right now I don't really know what I want from it. Because I don't know where is there exactly.`
  — refuses to fake a destination; asks to ground the model first.
- **S11 · line 1813 · 2026-08-14T18:36:11.778Z (F2):**
  `What does invarient even mean?`
  — surfaces not-understanding of an AI-authored concept (same turn as Q1).
- **S20 · line 3107 · 2026-08-15T21:45:56.791Z (F3):**
  `What do you mean by free function? Like, outside of the class?`
  — surfaces not-understanding of a term rather than acting on a wrong model.

**Honest marker.** No Forming candidate in this 10-day window. This is an absence over the window,
not proof the trap never occurs (a *silent* wrong model — never stated, never manifesting — is a
general limit of transcript audit, per ticket 05). The user's observable habit is the opposite.

### Trap 2 — Dislodging (repeating a failing approach instead of pivoting)

**Candidate: none.** The user pivots readily when an approach fails.

**Negative grounding.** Four verified pivots:

- **S4 · line 3621 · 2026-08-12T21:06:22.714Z (D1):**
  `Updating them in place would surely end up with a completely messed up plan so it's better to write clean with whatever remains`
  — chose `-revised` rewrites over in-place edits of stale plans.
- **S15 · line 565 · 2026-08-15T16:32:22.235Z (D2):**
  `It also seems like ticket 11 shouldn't be blocked by ticket 1, rather ticket 1 should be blocked by ticket 11.`
  — flips a wrong blocking direction the AI had set.
- **S22 · line 1414 · 2026-08-16T21:34:21.309Z (D3):**
  `the map is wrong, I guess it wasn't updated`
  — catches and corrects a stale map gating the work.
- **S28 · line 1373 · 2026-08-19T22:41:26.319Z (D4):**
  `It makes more sense to do research tickets here and then move any spikes or prototypes or implemention into a new map. Because always the next map will be half-baked`
  — corrects the AI's process suggestion; the AI conceded ("You're right").

**Honest marker.** No Dislodging candidate in this 10-day window; pivot is the user's default
response to a failing approach.

### Trap 3 — Assumption (solving a hardcoded/narrow case, not generalized)

**Candidate: none.** No hardcoded/narrow-case problem-solving by the user.

**Negative grounding.** The user tends to **harden narrow shortcuts and generalize rules** — the
opposite of Assumption:

- **S20 · line 3629 · 2026-08-15T21:56:11.494Z (A1):**
  `Yeah, fix it now`
  — decides to harden the `is_cfg_test` substring-match shortcut (a known "ponytail:" shortcut)
  rather than leave it narrow.
- **S14 · line 793 · 2026-08-15T14:32:19.627Z (A2):**
  `I do want to split out  the multiple impls into separate files`
  — generalizes the inherent-impl-locality rule's application (split multiple impls per type, not
  just one). (Double space after "out" is the user's, preserved verbatim.)

**Honest marker.** No Assumption candidate in this 10-day window. (The `is_cfg_test` substring match
03 noted was AI-written guardrail code; the user asked to harden it — A1 — not a user assumption.)

### Trap 4 — Location (skipping core architecture, realizing too late)

**Candidate: none.** The user is architecture-aware and raises concerns during planning.

**Negative grounding.** Three verified instances of architecture concerns raised **before** acting:

- **S15 · line 382 · 2026-08-15T16:27:33.863Z (L1):**
  `I am concerned at ticket 11 will affect the other tickets`
  — foresight about a layout change's blast radius, raised during planning.
- **S13 · line 1355 · 2026-08-15T12:54:24.406Z (L2):**
  `So I'm not really sure about putting it in action_pipline/pipline because up until now  we don't go that deep into the file structure.`
  — questions the proposed folder depth before the refactor. (Typos `action_pipline`, `pipline`, and
  the double space are the user's, preserved verbatim.)
- **S24 · line 5776 · 2026-08-19T22:43:28.592Z (L3):**
  `Then perhaps the structure of the narration and the idea of swapping itself is a problem?`
  — steps back to architecture mid-discussion, rather than realizing too late.

**Honest marker.** No Location candidate in this 10-day window; architecture tensions are caught
during planning (S13, S15) or re-examined mid-design (S24), not realized too late.

### Trap 5 — Achievement (endless band-aid fixes instead of refactoring)

**Split verdict (per [ticket 05](./assessability-table.md)).** Assessable at instance level; **not
assessable at pattern level without diffs** — the "endless/repeated" qualifier is a frequency claim
over code changes that a transcript alone cannot ground.

**Instance-level negative grounding.** The user favors refactoring and **removes** existing
band-aids — the opposite of Achievement:

- **S23 · line 3358 · 2026-08-16T22:51:27.449Z (AC3):**
  `Can you create a plan in docs/plans to investigate this issue and potentionally remove the in memory storage.`
  — considers removing a whole storage layer rather than band-aiding. (Typo `potentionally` is the
  user's, preserved verbatim.)
- **S20 · A1 (above):** `Yeah, fix it now` — removes the `is_cfg_test` ponytail shortcut.
- **S11 · Q1 turn (below):** the INV-NNN invariants subsystem was **retired** (the whole concept
  removed), not band-aided.
- **S2/S3 (R2, and the preset_store removal):** `PresetStore` was removed as pointless, not patched.

**Honest marker.** At instance level, no Achievement candidate in this 10-day window. Pattern level
("endless/repeated") is not assessable from transcript without diffs, but **no positive Achievement
pattern is in evidence** to confirm — so no diffs-research ticket graduates (the conditional
resolution from ticket 05 holds: diffs are needed only for a positive pattern finding, which the
pool does not contain).

### Trap 6 — Progression (AI-induced) — accepting generated code beyond understanding

**Candidate: one.** Session 11, the INV-NNN invariants subsystem.

- **S11 · line 1813 · 2026-08-14T18:36:11.778Z (Q1, PASS exact):**
  `I kind of just left the AI write it as is but I'm not really sure about it`
  — the user admits they let the AI author the whole INV-NNN "runtime invariants" subsystem (7 named
  invariants, a contract-test file, a guardrails.md §5, cross-refs) and do not understand it. The
  same-session investigation found it had drifted/broken — a phantom INV-006, a double-defined
  INV-005, a ghost-file reference (`tests/poison_recovery.rs`) — and the subsystem was retired that
  session (cost-of-the-gap quotes Q3/Q5/Q7, verified in ticket 04, ground the drift).

**The gradient question (read for, not found).** The ticket asked for milder Progression forms —
skimming a generated block, rubber-stamping a rename, accepting a framing without testing it. The
data shows **none that meet the bar**:

- **No skimming of generated blocks.** The auto-generated `guardrails.md` (S11) was driven by the
  user, who asked what prose was valuable, cut redundant framing, and caught (via the AI) a parser
  bug in the first generation. Not skimming.
- **No rubber-stamped renames.** The user caught the `phases.rs` → `pipeline_run.rs` rename
  themselves (S13) and deliberately declined `character.rs` → `db_character.rs` after weighing it (S14).
- **No untested framing accepted.** The user caught foregone-conclusion framing (S17, Q9) and
  provenance-checked against reference implementations (S24, Q12).

**Same-session self-correction (nuance for synthesis).** The acceptance and the catch are in the
**same session**: the user surfaces the not-understanding (F2, `What does invarient even mean?`),
drives a rethink, finds the drift, and retires the subsystem — all in S11. The trap was real (the
concept lived in the repo un-understood until revisited), but the user caught it themselves without
external prompting. This does not negate the trap; it sharpens its shape.

**Severity.** High for the instance (a whole AI-authored architectural concept lived un-understood).
As a **pattern**, the accept/resist ratio shows this is the exception, not the habit: the user
applies anti-Progression rigor to code and design (11+ verified resistance instances); the gap is
the **docs/named-concept layer** where this one slipped through.

### Trap 7 — Interruption (AI-induced) — prompting within seconds of confusion, not reasoning through the pause

**Candidate: none.** Assessable (millisecond timestamps on every entry). The trap did not occur.

**Negative grounding — full scan.** [`tmp/scan_interruption.py`](../../tmp/scan_interruption.py)
found **42 sub-60s gaps between consecutive user prompts across the 30 sessions**. Every one
classifies as something other than confusion→re-prompt:

- **Typo / correction re-sends (same intent, fixed wording)** — the clearest counter-evidence. The
  user re-sends within seconds to fix a typo, not because of confusion:
  - **S8 · line 3191 → 3194 · 2026-08-14T18:19:06.129Z → 18:19:16.000Z (9.9s gap, IT1→IT2):**
    `Oh, not implement` → `Oh, now implement` (typo `not`→`now`).
  - **S20 · line 3489 → 3497 · 2026-08-15T21:53:43.288Z → 21:53:50.430Z (7.1s gap, IT3→IT4):**
    `Of associated function or whatever` → `Or associated function or whatever` (typo `Of`→`Or`).
  - **S5 · line 2559 → 2568 · 2026-08-12T22:15:41.010Z → 22:15:50.986Z (10.0s gap, IT5→IT6):**
    `…make a plan to fix the result.` → `…make a plan to fix the rest.` (typo `result`→`rest`).
- **Amendments / additions** to the user's own just-sent prompt (adding a clause or a follow-up):
  e.g. S11 `Implement the plan` → `Implement the plan while creating a task list` (42.1s); S20
  `…make the method private…` → `Of associated function or whatever` (19.9s).
- **Answers / decisions / confirmations after reading** the AI's response: the majority (e.g. S15
  `Option A` → `confirm` 42.8s; S14 `So does option b include…` → `I'm fine with not renaming…`
  53.6s; S20 `What do you mean by free function?…` → `It's only called by PipelineRun…` 49.7s).
  Here the pause **is** the reasoning — the opposite of the trap.
- **Clarifications answering the AI's own question back** (e.g. S21 `Was there any other open
  issues?` → `That you raised here` 5.6s; S22 `…the wsl equivalent of that path` 23.2s).
- One **impatience re-send** (S2 `continue` → `continue`, 32s) — a nudge, not a confusion-reprompt.

**Honest marker.** Of 42 sub-60s gaps, **zero** are confusion→re-prompt. The trap did not occur in
this 10-day window. The user's sub-minute re-sends are typo corrections and amendments; substantive
prompts have deliberate (minutes-to-hours) gaps, and the sub-minute ones that follow an AI response
are the user reasoning through the pause, then answering.

### Trap 8 — Mislead (AI-induced) — blindly following off-target/over-complicated/incorrect AI suggestions

**Candidate: overlaps Progression on Q1.** The INV-NNN subsystem was an over-complicated and partly
incorrect AI suggestion (tautological relabels INV-003/INV-005; phantom INV-006; colliding INV-005;
ghost-file ref), and the user followed it — let it stand. Same event as the Progression candidate;
recorded under both because one event grounds either (ticket 06 decides presentation).

**The pattern is resistance, not compliance.** The user's dominant Mislead behavior is the 11
verified resistance instances above (Q8–Q12, R2, R6, R20a, R27, R30, R29) — catching AI over-reach
and refusing unverified output. In several, the AI concedes (R20a: "You're right, and I
overcorrected"; Q11: "You're right, and I overstated the defense"; D4: "You're right"). The INV-NNN
instance is the **exception**, not the pattern, and it sat in the docs/named-concept layer the user
reviewed less rigorously than code.

**Honest marker.** Mislead as a *vulnerability pattern* is Low: one overlap instance (Q1) on the
docs/concept layer, against a dominant resistance pattern across the pool.

## Outcome — what the data shows (not pre-decided)

- **Progression: n=1 confirmed, self-caught same-session.** No additional clear instance found;
  milder forms (skimming, rubber-stamping, untested framing) were read for and not found.
- **The accept/resist ratio is the pattern.** Resistance dominates (11+ verified instances across
  11 sessions; observable in nearly every substantive session). Acceptance-without-understanding is
  n=1 (Q1, docs/named-concept layer). Acceptance-after-investigation (R20b) is distinct from
  Progression and is healthy.
- **All six negatives grounded to the verified-quote standard.** Each has a verified quote showing
  the opposite habit (Forming: surface uncertainty F1–F3; Dislodging: pivot D1–D4; Assumption: harden
  A1–A2; Location: plan-first L1–L3; Achievement: refactor/remove AC3+A1+Q1-retirement; Interruption:
  typo-corrections IT1–IT6 + full 42-gap scan). None rests on an unverified session citation.
- **Achievement split holds.** Instance-level negative grounded; pattern-level not assessable
  without diffs, and no positive pattern in evidence — no diffs ticket graduates (ticket 05's
  conditional resolution).

**Consequence for re-synthesis.** Ticket 06's "primary vulnerability = Progression (High)" call is
**supportable on the instance but reframed by the pattern**: the vulnerability is **layer-specific**
(docs/named-concept / architecture), not a general accept-without-understanding habit. The user
already applies anti-Progression rigor to code; the actionable gap is extending that rigor to the
docs/concept layer. This **sharpens, not overturns**, the ticket-06 proposal — the rule proposal
(restate any AI-authored named concept in your own words and point to its enforcing code) targets
exactly the layer the pattern evidence identifies. Re-synthesis should present: one verified
Progression instance (High, self-caught same-session), Mislead Low-overlap on the same instance, six
grounded negatives, and the accept/resist ratio as the pattern context that makes Mislead-Low and
Progression-High-but-layer-specific coherent. Ticket 07 (HITL confirmation) waits on re-synthesis.

## Verification artifact

[`tmp/verify_pattern_quotes.py`](../../tmp/verify_pattern_quotes.py) — 32/32 quotes PASS exact
verbatim substring match against the cited flattened transcripts. [`tmp/scan_interruption.py`](../../tmp/scan_interruption.py)
— the full sub-60s-gap scan grounding the Interruption negative. [`tmp/locators.py`](../../tmp/locators.py)
— extracted the authoritative line + timestamp for each quote by walking back to its user header.
