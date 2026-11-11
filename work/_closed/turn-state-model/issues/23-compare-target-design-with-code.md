# Compare the target design with the code

Type: research
Status: resolved
Blocked by: 22

## Question

Where do the target design and the current code agree, and where do they differ? For each difference: is it a defect in the code, a gap in the spec, or a free choice where the code's shape comes only from its history? What does the current code know that the spec does not say? This research is step 6 of the clean-room route. Its answer is the input to the fix-or-rewrite decision of this map and to the [platform choice](../../platform-choice/map.md) map.

## Method

- Make one row per element of `../assets/target-design.md`. The columns are: the design, the code (cite the symbol), the spec source, and the class.
- The classes come from the [research 15 asset](../assets/research-15-clean-room-method.md) §5 Step 4:
  - **forced:** both agree, and the spec states the constraint;
  - **shared, not forced:** both agree, and the spec is silent;
  - **code defect:** they differ, and the code breaks a requirement;
  - **spec gap:** they differ, and a clean design listed an assumption at that point;
  - **free choice:** they differ, and both meet the spec.
- The research assets 04–12 hold most of the code facts. Re-check each fact at the current tree, and label each fact known, inferred or tested.
- **The oracle gap.** List each behaviour that the code has and the spec does not state. A rewrite must close these gaps, because the spec is its test oracle.
- Where the design keeps a state that tickets 01–03 ask about, cite the function that writes that state today.

## Output

`../assets/research-23-design-vs-code.md`.

## Feeds

- [Model a generation's states, its Snapshot pointers and the game-state writers](01-model-a-generations-states.md), [Replace `System` Messages with an error channel outside the Message history](02-replace-system-messages.md) and [Decide where the room a generation arrives in lives](03-decide-where-the-room-lives.md). Each of them is re-stated, closed, or ruled out of scope after this ticket.
- The map's fog patches "Fix or rewrite", "Retry tails" and "The writer rule".

## Done when

- The asset exists, and every element of the target design has a row.
- The Answer recommends one of these for each of tickets 01, 02 and 03: re-state it, close it, or rule it out of scope.

## Answer

Done on 2026-10-12 at tree `9cbf3112`; the asset is [`research-23-design-vs-code.md`](../assets/research-23-design-vs-code.md), with the oracle-gap list as its appendix [`research-23-oracle-gap-list.md`](../assets/research-23-oracle-gap-list.md). A `scout` agent wrote the first oracle-gap list, and a read-only `reviewer` agent checked the rows against `src/`. All 9 review findings were checked in the code and fixed.

**The 26 design decisions** (asset §1, §2):

| Class | Decisions | Count |
|---|---|---|
| forced | D-19, D-23 | 2 |
| shared, not forced | D-12, D-20, D-26 | 3 |
| code defect | D-01 to D-04, D-06 to D-10, D-13, D-14, D-15, D-21, D-25 | 14 |
| spec gap | — | 0 |
| free choice | D-05, D-11, D-16, D-17, D-18, D-22, D-24 | 7 |

- **No spec gap.** At each of the design's four assumptions, the code does the same thing.
- **The defects have two main roots.** 5 defect rows come from the writer rule (one latest Snapshot with many writers, one process-wide Game id, no guard around a save, no work id). 4 come from where a Swipe's state is (the pointer holds the copy saved before the analysis, and nothing updates it). 2 come from notices and the dock. 3 are local.
- **§3 adds finer rows** for the derived values, the save units, the plans, the Swipe state of each kind, Retry, the failure lines, stop and switch, and the views. §4 names the function that writes each state today.
- **New facts, not in research 04–12** (§1): an event Retry and a Retrigger send the prompt stored when the Trigger fired, so an edit or a preset change does not reach them; a swipe switch does not give back the trigger context of the Swipe's own generation; a Trigger whose prompt cannot be built is dropped with no failure; `POST /action` answers `500` for every error; an empty event answer does not say that the event failed.
- **The oracle gap** (§5): of 140 code behaviours, 18 must be added to the spec (12 items), 11 need an owner ruling first, and 20 belong to specs outside the lifecycle. The other 91 are defects, covered, moot, internal, or inside a black box.
- **Input to the fix-or-rewrite decision** (§6): seven clusters of change, A to G, each with its spec rows, its change in today's shape and its dependencies. Clusters A (per-Swipe state) and D (writer rule) touch every ordering function and most save sites. Cluster G is local.

**Tickets 01–03** (asset §8):

| Ticket | Recommendation | Reason |
|---|---|---|
| [01](01-model-a-generations-states.md) | Close | The target design is the model. §4 names today's writer of each state. The owner ruled the pointer questions to be implementation (ticket 17). |
| [02](02-replace-system-messages.md) | Close | The dock record (D-06), R-82 to R-86, OR-34 and OR-35 decide the channel. OR-01 removes the migration. The work is cluster C. |
| [03](03-decide-where-the-room-lives.md) | Close | The room report is on the Swipe with its Swipe state (D-19), and a switch gives it back. The work is cluster A. The doc follow-ups go to the plan. |

**Fog patches:**

- **Retry tails:** cleared. The design states every Retry form (§4.5), and asset §3.5 states where today's code differs. The feature-spec edits (`swipe_new.md` I.6 and 22.1, event Retry's reconciliation pass) go to the plan with the edits in ticket 21's Answer.
- **The writer rule:** cleared. The design states the rule (D-07 to D-09), and cluster D states how far today's writers are from it.
- **Fix or rewrite:** graduated to [Decide: targeted fixes or a rewrite of the game-state lifecycle](24-decide-fix-or-rewrite.md). The 11 "decide first" rows of §5.3 go there too.

**For the [platform choice](../../platform-choice/map.md) map:** the input that map waits for now exists. §5.4 says what a rewrite oracle needs beyond the clean spec: the feature-spec scenarios, and specs for the four black boxes.
