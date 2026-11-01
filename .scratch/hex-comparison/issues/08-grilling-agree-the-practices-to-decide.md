# Agree the practices to decide

Type: grilling (HITL)
Status: resolved
Blocked by: 01, 02, 03, 04, 05, 06, 07, 09

## Question

Which practices from the seven research assets and the [findings asset](../assets/findings.md) get a decision ticket, and of which type?

## Context

- **Inputs.** Start from `assets/candidates.md`, the output of [Prepare the candidate list for the grilling](./09-task-prepare-the-candidate-list.md). It holds the unique practices by theme, the checked claims, a proposal for each practice, and the depth gaps. Open the [findings asset](../assets/findings.md) and the assets `assets/research-01-*.md` to `assets/research-07-*.md` only to look into one practice.
- **Docs organized as feature maps.** The map's **Not yet specified** has an entry for this, with the owner's reason: it is hard to decide how to organize the docs and which docs to create. Take it together with candidates 2 and 3. `pstack` is a reference for this entry only. Do not research it further (owner, 2026-11-01).
- **Check the proposals with the owner, one theme at a time.** The owner confirms or changes each one: keep for a decision, already covered, or drop. A drop is a decision ("drop with a reason"). Record it in this ticket's answer.
- **Judge the depth** with the owner, using the depth part of `assets/candidates.md`. The owner found the first look too shallow. If an area is still too shallow, create one more research ticket for it, and do not create decision tickets for that area.
- **For each kept practice,** create a decision ticket. Give it:
  - a type: research, prototype, grilling or task
  - its `Blocked by` line
  - the candidate numbers or asset sections that it covers
  The pairs agreed on 2026-11-01 stay together: candidates 1 and 4, and candidates 2 and 3.
- **Then** create the outside-check ticket, blocked by all the decision tickets (see the map's Notes).
- **Clear** the lines in the map's **Not yet specified** that now live as tickets.

## Answer

Grilled with the owner on 2026-11-01, in 8 rounds, one theme at a time. The input was [candidates.md](../assets/candidates.md). Every answer matched the proposal. The owner then said that his confidence was low after so many questions. A review of the answers, sorted by weight and with the agent's confidence, kept them all. The owner also found too many HITL tickets. So each kept practice now gets an **AFK ticket** that measures it and ends with a recommendation. One grilling ticket confirms the recommendations together. Only "land without the owner" stays a grilling of its own.

### Results by practice

| Result | Practices | Ticket |
|---|---|---|
| Kept | C-42, C-43, C-44: one work list, plan conventions, a decision record folder | [10](./10-research-agent-use-of-docs-and-plans.md) measures how agents read the plans, then [16](./16-grilling-confirm-the-recommendations.md) |
| Kept | C-30, C-31: the seam rule, and env and clock reads | [11](./11-task-measure-env-clock-ban-and-hook-traits.md), then 16 |
| Kept | C-12, C-13, C-14: proof records, proof limits, known gaps | [12](./12-research-where-evidence-lands.md), then 16. Does not wait for ticket 10 (owner) |
| Kept | C-20: the **work report** | [13](./13-task-draft-the-work-report.md), then 16 |
| Kept | C-21: checks scoped to the change | [14](./14-research-docs-only-gate-cost.md), then 16 |
| Kept | C-15: golden-image UI tests | [15](./15-task-golden-image-test.md), then 16 |
| Kept | C-19, with C-17, C-18 and C-50 | [17](./17-grilling-decide-whether-anything-lands-without-the-owner.md). If C-19 drops, C-17 and C-50 drop, and C-18 counts as covered by `code-review`, with its record in the work report |
| Waits for ticket 10 | C-01, C-02, C-03, C-09, C-10, C-11, C-32 | Map entry **Docs organized as feature maps** |
| Covered | C-06, C-07, C-22, C-34, C-35, C-40, C-41, C-45, C-46, C-47, C-48 | — |
| Dropped | C-04, C-05, C-08, C-16, C-23, C-24, C-25, C-26, C-27, C-28, C-29, C-33, C-36, C-37, C-38, C-39, C-49, C-51, C-52 | — |

The reason for each covered or dropped practice is in the Proposal column of `candidates.md`, part 2. The owner confirmed each one. One exception: C-35 is covered for capability folders, and Hex's form that puts code in the parent file is dropped, because `check_mod_purity` fails it.

### The doc practices

- **Why they wait.** An AI writes the docs, and an AI reads them. The owner does not know how useful they are to an agent, and wants no rewrite that rests on a hypothetical. Feature maps appealed to him for the same reason that Diátaxis did: a focused, logical structure. The ticket's rule applies here: this area was too shallow, so it gets one research ticket and no decision ticket yet.
- **The owner's first view, from rounds 1 and 2.** It stands until ticket 10 reports.
  - C-01 (one always-loaded instruction file) is one answer to "where do the invariants live", not a decision of its own.
  - C-02 (invariants with exact values) is kept for a decision. Specs already state values, for example "default 10, from 1 to 100" in `docs/specs/browser_autoplay.md`. So "Values stay in the code" conflicts only with the Diátaxis docs (inferred).
  - C-32 follows C-02.
  - C-03, C-10 and C-11 were paused in round 2 and moved here in round 3, without a direct answer. The proposals were: C-03 becomes a constraint on any doc shape; C-10 starts from the spec scenario ids; C-11 means that any adopted doc shape gets a mechanical check.
- **What Chronicler already has (known).** `docs/specs/` is most of a feature map. `chronicler-ui-investigator` already uses `docs/specs/browser_*.md` as its selector vocabulary. The missing parts are: one page for each capability (Autoplay is split over two specs, by test surface), proof records, known gaps, and a rule for which docs exist.

### Depth

The depth is enough for themes C, D, E, F and G. The single gap that can change a result changes only a design. For example, C-18 would need Chronicler's own review criteria. Themes A and B were too shallow on one point: whether agents use the docs at all. Ticket 10 covers it.

### Terms of this map

- "Spec" means only a Chronicler `docs/specs/` file.
- Hex's `AGENTS.md` is "the Hex instruction file".
- The **work report** is the durable record that one agent session leaves when it finishes a change: which checks ran, which did not, and the review verdict.
- "Run" alone is not used, because `CONTEXT.md` reserves it for an Autoplay run.

`CONTEXT.md` does not change. These terms concern how the repo works, not the game domain.

### Facts found in this session

- `playwright-rs` 0.9.0 has `to_have_screenshot`, with a baseline file, `max_diff_pixels`, `threshold`, animations turned off, masks and `update_snapshots`. So golden images need no research ticket.
- The permission config allows `git commit`. `AGENTS.md:415` is the only barrier to landing.
- Session logs, October 2026, `read` calls only: 888 sessions. 351 read `src/`, 122 a Reference doc, 100 a spec. Git: 93 of 234 October commits touch `docs/diataxis/`.
- 9 of the 48 commits since 2026-08-01 that changed CSS or templates are `fix` commits. Some are visual, and a UI review found several of them (inferred from the subjects).
- The 6 corrections of `candidates.md` part 7 are now in the findings asset and in research 02, 03 and 05.
