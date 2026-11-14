# Find where the base story memory attaches in the engine

Type: research (AFK)
Status: resolved
Blocked by: 05

## Question

Where does the base story memory of [Choose the memory candidates to try](05-memory-candidates.md) attach in the code, and can it attach with no bolt-on? Give the facts and the design that [Build the base story memory](14-build-base-story-memory.md) needs.

## Context

- **The base (owner, 2026-11-11).** Read the Answer of [Choose the memory candidates to try](05-memory-candidates.md): summarization at the cut, the story memory in the prompt before the history, storage tied to the Swipe, a failed writer call loses nothing, and the trial pack shows the story memory. The first form is one summary text, rewritten at each cut from the previous text and the Messages that leave.
- **The room that Prompt flexibility kept** (map Notes): `HistoryCut` (`src/domain/model/history_cut.rs`) is the one history cut and fires no hook yet. A writer is a new Agent on `AfterNarrationAgent` or a new hook trait. Memory that depends on the story is pinned to the Swipe, and a new `SwipeState` field is an optional JSON key.
- **The cut is found inside the prompt assembly (inferred).** The assembler fits the history when it builds a call (`render_and_fit` in `src/application/prompting/assembler.rs`). So the engine may learn which Messages leave only while it builds the next Narration call. The writer call must run before that call is sent, or the first call after a cut has neither the Messages nor their summary.
- **The standing rules** (map Notes): a new API takes a `GameId` and adds no read of `ActiveGame`. The migration rules. The root `AGENTS.md`: never bolt a feature on. If no natural seam exists, say so and name the refactor.
- Read `docs/diataxis/reference/narrative/prompt_system.md`, `docs/diataxis/explanation/prompt_system_design.md`, `docs/diataxis/reference/narrative/agent_system.md` and `docs/diataxis/reference/game_flow.md` first.

## Output

An asset `assets/base-memory-seam.md`, and an Answer with its gist. Mark each statement known (with file and line), inferred or guessed. The asset holds:

- **The cut today.** Which code decides that Messages leave, at which step of a Job, for which calls, and what it records about the Messages of each block.
- **The writer call.** Where a model call that summarizes the leaving Messages can run so that the next call already has the story memory. Which seam fits: a step of the cut, a new hook, or an Agent on an existing hook. How the three calls with a history space (Narration, Impersonate Input, event Narration) share one story memory. What Autoplay, Retry, a swipe switch, a delete and an edit of an older Message do to it.
- **The storage.** Where the story memory lives so that it is tied to the Swipe: a `SwipeState` key or a table. What a Retry and a delete then do with no extra code. The migration, if one is needed.
- **The prompt.** How a `<StoryMemory>` block enters the layout before the history, its effect on the history space (`TokenBudget`), and its effect on the cache.
- **The failure rule.** How the existing rule for a failed writer applies. What happens to the Messages of a cut whose writer call failed: are they summarized at the next cut, or lost?
- **The connection.** Which Connection and which preset the writer call uses, and how the gate (`src/application/generation/gate.rs`) sees the call.
- **The records.** How the call record and the trial pack (`scripts/trial.py`) show the story memory of each Cycle.
- **The tests.** Which seam cases and specs exist for the cut, and where `tests/STRATEGY.md` puts the new ones.
- **The open choices.** Each choice that the code leaves open, with a recommendation. Mark a choice that only the owner can make.
- **The refactor, if one is needed.** If the base cannot attach cleanly, name the refactor that makes it clean, and its size.

The ticket changes no code.

## Answer

Resolved on 2026-11-11, read on `long-term-memory-2` at `4baba0a4`. The full evidence, with the code quotes and the open choices, is in [the asset](../assets/base-memory-seam.md).

- **The cut fires inside the prompt render, and nowhere else (known).** `render_cut_prompt` in `src/application/prompting/assembler.rs:188` computes the space (`:196`) and calls `HistoryCut::of` (`:197`). `rg -n HistoryCut src` finds the type in the assembler and its own tests only. The cut is a local value, never stored.
- **A model call cannot run at that moment (known).** The Narrator "holds no storage" (`src/application/narration/narrator.rs:27`), and `HistoryCut` is a pure function of token sizes (`history_cut.rs:23`). So the Job must plan the cut itself, before the Narrator call, and run the writer as its own step.
- **The writer step goes before `narrate`, and its text is saved with the new Swipe (known).** `Job::generate` today runs `save_input`, `narrate`, `save_text` (`job.rs:183-193`). The Swipe that the memory belongs to does not exist before `save_text` (`gate.rs:793`, `gate.rs:878`). So the writer returns text, the Job puts it in the prompt, and `save_text` writes it into the new Swipe's state.
- **The seam is a new hook trait, not `AfterNarrationAgent` (known).** That hook runs after the text save, and its `HookInput` (`agent.rs:93`) holds no previous memory and only 4 story entries. A new "before narration" hook with its own input and result is the shape the agent system describes.
- **The writer must not ride on the agent config (known).** The agent system promises zero agents are needed, and `seed_settings` writes `OnlyIfAbsent` (`storage/settings.rs:120`). An existing DB would never see a third agent entry. Build it in `wiring.rs` beside the Narrator.
- **Storage: a `SwipeState` optional key, and no migration (known).** `message_swipes.state` is JSON text (`plumbing.rs:639`), and `event_record` shows the pattern (`swipe_state.rs:23-30`). A Retry appends a Swipe (`gate.rs:878`), a switch changes the active index (`gate.rs:501`), and a delete cascades (`plumbing.rs:110`). So the memory's lifecycle needs no code.
- **The key holds the text and one Message id (`inferred`).** The marker (`covers_up_to`) is what lets the writer read only the entries that left since the last cut. Without it the writer's prompt grows with the story.
- **The block is the last of the before-layers, and its size must be a constant (known, then inferred).** `before_layers` (`assembler.rs:276`) fills the first user part, and `fixed_tokens` (`:237`) takes its room from the history space. Because the writer result is not known at plan time, the block needs a fixed reserve in `fixed_tokens` and in the overflow guard, and the writer's text is capped at that size.
- **A failed writer loses nothing, and the leaving Messages wait (inferred).** The previous memory stays, the Job continues (the rule of `runner.rs:70`), and the marker does not advance, so the next successful cut summarises them. The model misses them until then.
- **The connection: the Narration slot and a new `Role::Memory` (recommended).** A new `PresetType` needs a `Game` field and a migration (`game.rs:37`, `settings.rs:511`). The agent name and the connection are already separate (the Impersonate does this, `narrator.rs:79`). The role matters: a failed summary recorded as `narrator` would raise a false banner (`headers.rs:27`).
- **The refactor is small: expose the cut plan (about 30 lines in `assembler.rs` plus one wrapper in `narrator.rs`), and add one hook, one module and one `Step`.** No migration, no new preset type, no new settings field.
- **Tests:** the cut function keeps its plain unit test; the prompt cases go in `narrator_tests.rs` (the existing `narrator-history-*` cases); the writer is an Agent so its file is a seam contract with `CASE` tags; and the visible behaviour needs a new spec (`docs/specs/story_memory.md`, Scenarios 46.x) with a tier-1 test, because `ScriptedModel` records the prompt parts.
