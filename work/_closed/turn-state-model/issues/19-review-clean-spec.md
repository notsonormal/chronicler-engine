# Review the spec for leaks and sign off its requirements

Type: task
Status: resolved
Blocked by: 18

## Question

Does the spec of [Write the behaviour spec for the game-state lifecycle](18-write-clean-behaviour-spec.md), and the clean agents' task text, carry only observable behaviour, with no code shape? Does the owner accept each requirement? This task is step 2 of the clean-room route. It is HITL, because the owner signs off.

## Steps

1. **Mechanical check.** Run `python .scratch/_closed/turn-state-model/assets/research-15-leakcheck.py` on `../assets/clean-spec.md` and `../assets/clean-run-prompt.md`. The target is zero `src/` names, zero `src/` paths and zero `.rs` file names.
   - Known false positives: names that an outside system shares, and interface names that an LLM sees (for example `npcs_in_room`).
   - Known false negatives: mechanism written in domain words, such as "the anchor message has no snapshot" or "the generation gate is busy".
2. **Semantic review.** Delegate both files to a fresh-context `judge-glm` session. It flags:
   - each sentence that a client cannot observe through HTTP or the browser;
   - each sentence that names an internal step, an order of internal steps, or a storage shape.

   Fix each finding, or record why it stays.
3. **Owner sign-off.** The owner reads the requirements and the open questions. Each owner edit goes into `../assets/clean-spec-rulings.md` as a ruling, with the date, the requirement number, the text and the reason. The owner confirms the requirements. The owner does not have to confirm that no leaks remain.

## Context

- The rules for the review are in the [research 15 asset](../assets/research-15-clean-room-method.md) §5 Step 2.
- The owner treats pointer, anchor and phase-order rules as implementation (ticket 17, Q11 and Q12). A requirement of that kind is a leak.

## Done when

- The leak check reports zero hits, or each hit has a reason in the Answer.
- Each `judge-glm` finding is fixed or has a reason.
- The owner has signed off, and `../assets/clean-spec-rulings.md` exists.

## Answer

Resolved on 2026-10-12. The owner signed off requirements R-01 to R-92 (81 requirements after the review) and the 17 open questions. [`clean-spec-rulings.md`](../assets/clean-spec-rulings.md) holds rulings OR-01 to OR-03.

**Step 1, the leak check (tested).** `research-15-leakcheck.py` found 0 `src/` names, 0 `src/` paths and 0 `.rs` names in both files: before the review, after the `judge-glm` fixes, and after the document review. Final size: `clean-spec.md` 5,477 words, `clean-run-prompt.md` 671 words.

**Step 2, the semantic review.** A fresh `judge-glm` seat (session `01a1312d-d2a6-7119-b9fe-2a111f6d1b4a`) gave 15 findings and 2 side notes. Two of its code claims were checked against the code before they were accepted (known): `reconcile_post_trigger_npcs` runs the full Quantifier and keeps only the Characters, so no presence-only call exists (F1). `assert_state_consistency` does nothing without the `diagnostics` feature, so it is not a runtime check (F4).

| Finding | Result |
|---|---|
| F1 §3.2 presence-only call | Removed. R-14 states the effect that a player sees |
| F2 §3.2 "the call reports that it fell back" | Removed. R-12 states the result, and OQ-03 asks whether the player is told |
| F3 §3.4 "the rules report that the destination was unknown" | Removed. OQ-03 asks whether the player is told |
| F4 §3.4 a check of the game state against the World | Reworded as the R-20 failure |
| F5 §3.5 "does not filter entries" | Reworded as a contract: assembly removes nothing (R-82) |
| F6 §3.7 the banner's data source | Removed. The same sentence in §3.1 is also removed |
| F7 R-55 starting-point wording | Reworded. The first rewording was wrong against Retry 10.5, and the document review fixed it |
| F8 R-63 "rewinds nothing" | Reworded. The first rewording conflicted with R-14, and the document review fixed it |
| F9 R-58 and §3.4 "marked as fired" | Restated as "a one-shot Trigger cannot fire again" |
| F10 E-08 has no source | Owner ruling OR-01: saved Games may be discarded |
| F11 §3.3 "knows this before it makes a call" | Reworded |
| F12 §1 Notice "a step" | Changed to "work" |
| F13 §1 Opening Narration | Owner ruling OR-02: "No LLM call produces it." |
| F14 OQ-10 "at each process start" | Changed to "when the Game is created, or later" |
| F15 the "forced" or "free" mark in the run prompt | Kept by owner ruling OR-03 |
| Side note: the §2 `/retrigger` row had no `500` | Added, so it agrees with R-46 |
| Side note: R-30 conflicted with R-32 | Reworded: Options on demand, and after each Narration when the always-on toggle is on |

The judge cleared ticket 18's other suspected leaks. The four step names in R-71 are the literal answers of `/status/generating`. R-70, the controls on the last entry in §3.7, and OQ-06, OQ-12 and OQ-17 can all be observed.

**The document review (asked for by the owner before sign-off).** Every scenario citation exists in its source spec. Three errors were fixed: R-55 and R-63, which the F7 and F8 rewordings had made wrong, and R-14, whose rewording conflicted with §3.4. Repeated text was removed: R-43 (its sources moved into OQ-02), R-84 (R-17 says the same), E-06 (moved into the §2 intro), E-07 (the §1 World term says the same), the §3.4 bullet that repeated R-14, and the line about the spell-check routes, which are outside the scope. Repeated text was trimmed in the §1 Trigger term, E-05, §3.1 and §3.7. The semicolons inside the lists were left as they are on purpose.

**Lesson (inferred).** A reviewer's rewording can bring in an error that the reviewer did not find. Check each accepted rewording against its source scenario, not only against the leak rule. Ticket 21 changes the spec again, so it should use the same check.

**Owner decisions.**

- The requirements and the open questions are confirmed as they stand now.
- OQ-01 (the offered set after a swipe switch) and OQ-02 (a Retry that fires a Trigger) stay open. The two clean designs answer them, and ticket 21 closes them.
- The run prompt (`clean-run-prompt.md`) did not get a document review. Only the leak check and the `judge-glm` review covered it.

**For ticket 20.** Embed the spec and the run prompt as they are now. The rulings file stays out of the walled runs: the spec already cites each ruling where it applies.

