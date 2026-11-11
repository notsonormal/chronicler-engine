# Research: map what each retry form restores and re-runs

Type: research
Status: resolved
Blocked by: —

## Question

For each retry form, which Message is the anchor, which Message or Swipe is the target, which Snapshot is restored, where the history is cut, which phases re-run (Narration, Quantifier, engine commit, Trigger, Options rewrite), and what a failed redo restores? Where do the code, the specs and the docs disagree?

## Scope

Retry forms (known from the code and the map; add any you find):

| Form | Entry |
|---|---|
| Main Retry | `RetryMode::ReNarrate`, Input anchor |
| Event Retry | `RetryMode::ReNarrate`, `RetryTarget.is_event` |
| Guided Retry | guided Narration as target and anchor |
| Re-impersonate | `RetryMode::ReImpersonate`, `retry_main_narration` |
| User regen | `RetryMode::UserRegen` |
| Retrigger | `action_pipeline/retrigger.rs` |
| Swipe switch | `MessageService::switch_swipe` (restores, re-runs nothing) |
| Options retry | on-demand Options, if it restores state |

For each form, record the anchor rule (`find_retry_anchor_msg`, `Message::is_input_or_narration`, `messages.last()`), the restored Snapshot, the phases re-run, the new Swipe it adds, and the failure rollback.

Then compare with the specs (`docs/specs/swipe_new.md`, `swipe_switch.md`, `retrigger.md`, `actions.md`), `docs/diataxis/reference/game_flow.md` §Retry Flow, and the Retry and Swipe entries in `CONTEXT.md`. List each disagreement.

Settle the seed §2 conflict as far as reading can: does re-impersonate re-run the Quantifier? Does scenario 22.1 pass only because its mock Quantifier returns no movement?

## Context

- Seed: [three pipeline defects with one root](../assets/seed-evidence.md) §1 (pointer table) and §2.
- [Make event Retry restore the trigger context and never roll the game back](../../dashboard-ui-review-phase2/issues/09-fix-event-retry-snapshot.md) changed event Retry. Read its Answer.
- The closed map [guided generations architecture](../../guided-generations-architecture/map.md) set the guided Retry and steering replay rules.

## Output

An asset `../assets/research-06-retry-tails.md`: one table with one row per retry form and one column per question above, then the list of disagreements with the source of each side. Label each fact known, inferred, or tested. Cite symbols, not line numbers.

## Feeds

[Model a generation's states, its Snapshot pointers and the game-state writers](01-model-a-generations-states.md), and the map's fog patch "Retry tails".

## Answer

Asset: [research-06-retry-tails.md](../assets/research-06-retry-tails.md). Facts are known from the code unless marked. The asset has the per-form table, the failure rollback, and 12 disagreements.

- **Five Retry forms share one entry.** `resolve_retry_target` picks the form from the last Input-or-Narration Message: event → event Retry; guided → guided Retry; other Narration → main Retry; impersonated Input → re-impersonate; plain Input → user regen. The redo's text is always a new Swipe on that Message. A Trigger in the redo adds a new event Message.
- **What each re-runs.** Main and guided: narration, Quantifier and engine commit, Trigger, reconciliation, Options rewrite. Event: Trigger, reconciliation, Options rewrite. Re-impersonate: all but the Options rewrite. User regen: narration only. Retrigger restores nothing and adds a new event Message. Swipe switch re-runs nothing.
- **What each restores.** Main: the anchor Input's Sc (pre-turn, previous `last_trigger` kept). Guided and re-impersonate: the target's S1 (pre-turn). Event: the event Swipe's S5. User regen: the Input's Sc.
- **Failed redo.** The click-time world comes back only when the redo persisted no Swipe. Otherwise the attempt's world stays.
- **New finding: the main-Retry anchor is the last Input row in the whole history.** A Retry of a continue Narration rewinds to an earlier Action's Input, drops the Messages in between from the prompt, and re-sends that earlier Input's text. A Retry of a Narration after an impersonated Input rewinds that impersonate Action's movement and Trigger. Mechanism known; effect inferred; no test covers it.
- **Seed §2 settled.** Re-impersonate re-runs the Quantifier: known, tested (`test_retry_flow_impersonate_mode_runs_full_tail` passed this session), and decided by ticket 01 of the closed guided-generations map. `swipe_new.md` I.6 and 22.1 are stale. The 22.1 test runs with an empty agent registry and has no room assertion, so it cannot see the difference.
- **Other disagreements:** I.6 says event Retry does not re-run the Quantifier, but reconciliation does (10.5 depends on it). I.2 says a Retry never adds a Message, but 9.5 expects one. `CONTEXT.md` puts the Swipe on "the last Message" of an "AI-generated" Message; the code skips System Messages and lets user regen swipe a typed Input. `game_flow.md` names three of the forms.
