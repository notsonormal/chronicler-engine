# Write the behaviour spec for the game-state lifecycle

Type: task
Status: resolved
Blocked by: —

## Question

What does the game-state lifecycle do, stated only as behaviour that a player or an HTTP client can observe, so that an agent with no access to the code can design it? This task writes that spec. It is step 1 of the clean-room route that [Decide whether and how to run a clean-room design of the generation model](17-decide-clean-room-run.md) chose.

## Context

- **The route.** Spec (this ticket) → leak review and sign-off → two walled clean designs → close the spec gaps → open target design → comparison with the code. Read the Answer of ticket 17 first.
- **Method and rules:** the [research 15 asset](../assets/research-15-clean-room-method.md) §5 Step 1 (the rules for the spec writer) and §3 (the class of each input).
- **Scope B:** the [research 16 asset](../assets/research-16-clean-room-collapse.md):
  - §3.1 row B gives the inputs. They are `actions.md`, `swipe_new.md`, `retrigger.md`, `swipe_switch.md`, and the `options.md` scenarios that a generation or an on-demand Options run reaches (research 16 counts 10 of 15). They also include `story_log.md` 8.1–8.4, `reset.md` 7.1–7.2, `games.md` 17.1, 18.1 and 18.3, `dashboard.md` 39.1 and 39.3, and `failure_display.md` 38.3, 38.5 and 38.7.
  - §3.2 gives the HTTP writers that B includes: edit, delete, swipe switch, `POST /reset`, `/status/reset-generating`, and the game switch and create.
  - §3.4 gives the storage requirement and the black-box contracts.
  - §3.5 items 1–20 list the behaviours that only the code defines today. Each one becomes a requirement or an open question.
- **Stale claims:** the [research 11 asset](../assets/research-11-docs-and-decisions.md) lists them. Correct each one per an owner ruling, or list it as open.
- **Spec writer:** an Opus session that has read the code. It is the examiner.

## Owner rules for this spec (ticket 17)

- State behaviour at the level that a player observes. Pointer, anchor and phase-order rules are implementation, and they stay out.
- **Swipe switch:** "After a swipe switch, the game is as that Swipe's generation left it."
- **Retry:** prior decision 6, as written: "A new Swipe redoes the last generation, with its stored inputs." The spec does not add a rewind-point rule.
- **Notices:** no notice reaches an LLM (owner, 2026-10-11; ticket 02). The channel is a design choice.
- **Storage:** "A write names its Game, and one save of a Message, its Swipe and its state is one unit" (research 16 §3.4).
- **Behaviour rulings** (research 15 §3): 1 (as the swipe-switch sentence above), 2 (an impersonate redo can move the player), 6, 11, 13 (first half), 14, 16 and 17. Restate each at the observable level, with its source. The module-shape decisions (3, 4, 5, 7, 8, 9, 12 and 15) stay out. `gg01` decision 3 is module shape.
- **Terms:** restate each `CONTEXT.md` term by what it must do, not by when it is saved.
- **Black boxes:** the narrator, Quantifier and Options calls (their inputs, result shape and failures), Trigger evaluation, the world rules of the engine commit, prompt assembly, storage (with the storage requirement) and the HTTP views.
- **Open questions:** number every point where no spec, doc or ruling gives the behaviour, and leave it unanswered. The spec writer does not decide them.
- SillyTavern and Marinara stay out of the spec. They go into ticket 22 as prior art.

## Output

- `../assets/clean-spec.md`, with the sections: terms, black boxes, requirements (numbered, each with its source), environment constraints, and open questions.
- `../assets/clean-run-prompt.md`: the task text for the clean agents. It lists the design items that research 15 §5 Step 3 names: the states and what each holds, what binds to a Swipe, what each Retry form cuts, restores and runs again, the writers during a generation, where notices and the location live, an assumptions list, and the questions for the owner. It asks for a design with no code and no language types.
- A coverage table in the Answer. Each input scenario maps to a requirement, or it is dropped with a reason.

## Done when

- Both files exist.
- `python .scratch/turn-state-model/assets/research-15-leakcheck.py` has run on both files, and the Answer records the result.
- Every input scenario and every §3.5 item maps to a requirement or an open question, or it is dropped with a reason.

If the spec does not fit one session, split the work by spec file and record the split here.

## Answer

Written on 2026-10-12 by an Opus session that has read the code (the examiner). One session was enough; no split.

**Files.**

- [`clean-spec.md`](../assets/clean-spec.md): 5,630 words. Sections: about, terms, the HTTP interface, black boxes, 83 requirements (R-01 to R-92, numbered with gaps between groups), 8 environment constraints, 17 open questions.
- [`clean-run-prompt.md`](../assets/clean-run-prompt.md): 671 words. It asks for a design with no code and no language types, in 12 sections: summary; records; states; Swipes; Retry and Retrigger per kind; writers; notices; the room report; restart; a requirement trace (forced or free); assumptions; questions for the owner. Each OQ-nn must land in the assumptions or the questions.

**Leak check (tested).** `python .scratch/turn-state-model/assets/research-15-leakcheck.py` on both files: 0 distinct `src/` names, 0 hits, 0 `src/` paths, in each file. To keep it at zero, the spec names no form field and no debug route that is also a `src/` name, and cites the feature specs by title ("Retry 9.2"), not by file name. A `rg` pass for mechanism words (snapshot, pointer, anchor, gate, slot, phase, pipeline, registry, persist, commit, claim, heal) found three phrasings that presupposed stored records. They are reworded (R-46, R-72, R-91).

**Terms.** The spec restates World, Persona, Character, Game, Action, Guided Generation (as Guide), Impersonate, Message, Swipe, Retry, Trigger, Quantifier (as Analysis) and Options by what they must do. It adds Story, Input, Narration, Opening Narration, Event Narration, Game state, Status, Generation, Busy, World rules, Trigger context, Always-on toggle, Retrigger, Swipe switch, Notice, Room report and Restart. It does not use Snapshot, Action Pipeline and Agent, because the glossary defines them by the current mechanism (research 15 §3). Narrator Mode and Display name are outside scope B.

### Coverage: input scenarios

Each input scenario maps to a requirement or a black-box contract, or is dropped with a reason. A reason in parentheses that named a mechanism ("the Quantifier re-ran", "the gate is busy", "the anchor has no snapshot") became an observable outcome or was dropped.

| Spec | Scenario or property | Maps to |
|---|---|---|
| Actions | 1.1, 1.2, I.2 | R-02 |
| | 1.4 | R-07, R-11, R-13, §3.4 |
| | 1.5 | R-03 |
| | 1.6 | R-11, R-14, §3.4 |
| | 1.7, 1.8 | R-15, R-16, §3.4 |
| | 1.9, 1.10 | R-01, R-04, R-05 |
| | 1.12 | R-01 for the engine commands. The text-check part is dropped: the check routes are outside scope B (spec §2) |
| | 1.13 | Dropped: the answer shape of the confirm route is a view. The Action it sends is R-01 |
| | 1.14 | R-11, R-22 |
| | 2.1 | R-20 |
| | 2.2, 2.3, I.4 | R-19, §3.1 |
| | 2.4 | R-17 |
| | 2.5 | R-12, §3.2 |
| | 3.3 | R-09 |
| | 6.1, 6.2, 6.3 | R-08, R-41 |
| | I.1 | R-10 |
| | I.3 | R-07 |
| Retry | 9.1 | R-41 |
| | 9.2, 9.6 | R-23, R-52 |
| | 9.3, 9.4 | R-51, R-75 |
| | 9.5 | R-52, R-43, OQ-02 |
| | 9.8 | R-23, R-25 |
| | 10.1, 10.4 | R-41, R-54 |
| | 10.2 | R-57 |
| | 10.5, 10.7 | R-55 |
| | 10.6 | R-56 |
| | 10.8 | R-24 |
| | 11.1 | R-44, OQ-05 |
| | 11.2 | R-45 |
| | 11.3, 11.4, I.7 | R-46, R-91 |
| | 11.5, 11.6, 11.7 | R-19, R-20, R-49 |
| | 11.8 | R-58 |
| | 11.9, 11.10, I.9 | R-47, R-48 |
| | 12.1, I.8 | R-70 |
| | 22.1 | R-41, R-59, R-60. Its last line is corrected per prior decision 2 |
| | 22.2 | R-18, R-61 |
| | 22.3, 22.4 | R-05, R-59, §3.1 |
| | 22.5 | R-23, R-53 |
| | I.1 | R-10, R-50 |
| | I.2 | First sentence: R-43, OQ-02. Second sentence: R-59, R-61 |
| | I.4, I.5 | R-41, R-42, R-51 |
| | I.6 | R-18, R-52, R-57, R-60. Corrected per prior decision 2 |
| Retrigger | 13.1, 13.2 | R-62, R-63 |
| | 14.1–14.4, 14.6 | R-65 |
| | 14.5 | R-66 |
| | 15.1, I.8 | R-70 |
| | I.3 | R-64, OQ-04 |
| | I.7 | R-46, R-91 |
| | I.9 | R-17, R-67. "Set before the LLM call" is dropped: it is an order rule |
| Swipe Switching | 36.1 | R-37 |
| | 36.2 | R-38, OQ-01 |
| Options | 24.1, 24.2, 24.4 | R-26, R-27, R-28 |
| | 24.5, 24.18 | R-29, R-83 |
| | 24.6 | R-31, the Always-on toggle term |
| | 24.13 | R-06 |
| | 24.15 | R-30 |
| | 24.16 | R-32 |
| | 24.17, 24.20 | R-33, R-83 |
| | 24.10 | §3.3 (a black-box contract) |
| | 24.11, 24.19 | Dropped: Narrator Mode is outside scope B (research 16 puts it at C) |
| | 24.14 | Dropped: icons are a view |
| Story Log | 8.1, 8.2, 8.4 | R-76 |
| | 8.3 | R-50 |
| Reset | 7.1, 7.2 | R-77 |
| Games | 17.1, 18.1 | R-78, R-79 |
| | 18.3 | R-80, R-92 |
| Dashboard | 39.1 | R-71, R-72, R-92 |
| | 39.3 | R-72, §3.7 |
| Failure Display | 38.3, 38.5, 38.7 | R-21, §3.1 |

Research 16 counted 10 of the 15 Options scenarios. This spec keeps 11 as requirements (it adds 24.13, which is an Action), puts 24.10 into a black-box contract, and drops 3.

### Coverage: research 16 §3.5, items 1–20

| Item | Maps to |
|---|---|
| 1. Which state a swipe switch restores | R-36 (the owner's sentence), R-37, R-38; OQ-01 (the offered set); OQ-17 (a Swipe that no generation wrote) |
| 2. The rewind point of a continue or post-impersonate Retry | R-40, prior decision 6 as written. The owner left its reading to the design (ticket 17); not an open question. The prompt asks each design to state its reading per kind (section 5) |
| 3. Re-impersonate runs the full tail | R-60 |
| 4. Event Retry and Retrigger run a presence pass that cannot move the player | R-14, R-57, R-63 |
| 5. A Retry or a Retrigger can add Messages | R-43, OQ-02; R-64; OQ-04 for notices |
| 6. The Input's pointer is the claim save | Dropped: a pointer rule is implementation (owner, ticket 17). Its observable effect is in items 7 and OQ-14 |
| 7. `last_trigger` after a main Retry | R-40 (the design's reading), R-62, R-67; OQ-14. The prompt asks what a Retrigger can do after each kind of Retry |
| 8. The writer rule | R-70, R-73, R-74; OQ-08, OQ-09 |
| 9. The Retry target when notices trail | OQ-04 |
| 10. Every Message has Swipes | R-34, R-41, R-59, R-61 |
| 11. Most Snapshots belong to no Message | Dropped: a storage mechanism. The spec states no records (§3.6) |
| 12. The event-Retry anchor can be a notice | Dropped: an anchor rule is implementation. Its observable effect is OQ-04 |
| 13. Arrival | OQ-10; R-73 and R-74 apply to it |
| 14. `/status/reset-generating` | OQ-07 |
| 15. The edit route | R-75; OQ-08 |
| 16. Writes of cancelled work after a game switch | R-74; OQ-09 |
| 17. The status lives in every Snapshot | Dropped: a storage mechanism. Its observable parts are R-71, R-72, R-92 and OQ-13 |
| 18. Delete and edit act on a trailing notice | OQ-04 |
| 19. `pending_location` seeds the opening | OQ-16 |
| 20. Every write targets the global current Game | R-74, R-90 |

### Behaviour rulings, owner rules and stale claims

| Source | Maps to |
|---|---|
| Prior decision 1 and the owner's swipe-switch sentence | R-36 |
| 2 | R-11, R-60 |
| 6, as the owner confirmed it | R-34, R-40, the Swipe term |
| 11 | R-47, R-55 |
| 13, first half | R-17, R-84 |
| 14 | R-22 |
| 16 | R-47, R-48 |
| 17 | §3.6, E-01, R-92 |
| Owner notice rule (2026-10-11) | R-82; the channel is the design's (R-83) |
| Owner storage rule (ticket 17) | R-90, R-74 |
| Research 11 drifts 1–2 (I.6, 22.1) | Corrected per prior decision 2: R-60 |
| Drift 3 (I.6 event clause, 10.2) | The room claim kept, the reason dropped: R-14, R-57 |
| Drift 4 (I.2) | Open: OQ-02. No ruling exists |
| Drift 5 (Retrigger 13.2) | The room claim kept, the reason dropped: R-63 |
| Drift 6 (Retrigger I.3) | R-64 says "story Message"; notices: OQ-04 |
| Drifts 7–10 (glossary Retry, Swipe, Snapshot, Game) | Restated in §1. Snapshot is not used; R-36 and R-91 say what a restore must give |
| Drifts 11–12 (Storage Design, Storage reference) | R-36. "Restores that exact state" against 36.1 and 36.2: OQ-01 |
| Drifts 13–14 (two plans) | Not inputs. The opening's room report: OQ-16 |
| Drift 16 (the superseded preset pin) | R-59, §3.1: the preset that is active when the call starts |

### Points for the leak review (ticket 19)

Inferred risks, most important first:

1. **Black-box contracts from the code.** The examiner wrote these from the code, not from a spec: the Quantifier fallback and the presence-only mode (§3.2); the world rules (a new room for an unknown destination, at most one Trigger, a one-shot Trigger counts as fired after its event Narration; §3.4); "prompt assembly does not filter" (§3.5); the controls on the last entry (§3.7). They are facts about black boxes, but they carry the current shape.
2. **R-55** restates prior decision 11 as "starts from the world that the replaced event Narration started from". It is the requirement closest to a rewind-point rule. Its sources are prior decision 11 and scenarios 10.5 and 10.7.
3. **R-71** names the four busy steps. It fixes that a separate analysis step exists (Dashboard 39.1, the Game Flow doc).
4. **R-30**, second sentence, extends 24.15 to the Impersonate redo and the user regen through the glossary's Options entry.
5. **R-70** counts `/options` as busy work, from Retry I.8 ("the same way `POST /action` does").
6. **E-08** (saved Games exist) has no source. It is an environment fact.
7. **OQ-06, OQ-12 and OQ-17** come from this session's reading of the code, not from research 16 §3.5.
