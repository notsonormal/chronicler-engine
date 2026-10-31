# Fix the prompt text

Type: task (AFK)
Status: open
Blocked by: 18

## Question

Fix the prompt text faults that the layout tickets decided:

- every card list in Character id order;
- the present-card label `Context:` renamed `Scenario:`;
- a layer with no content renders nothing;
- the Quantifier block `<LatestNarration>` renamed `<LatestMessage>`, with no name prefix;
- placeholders rendered in the card text of the Quantifier and Options calls;
- the Options closing line with no count;
- the answer shapes of each parser listed in `agent_system.md`.

Sources: [Decide which parts of a Narrator-role call an author can change](06-the-layout-model.md) (fixes 1, 3, 4) and [Decide which parts of a Quantifier or Options call an author can change](10-quantifier-options-layout.md) (fixes 1, 3, 5, 6, and the parser list).

Not here:
- The Options count field (ticket 10 fix 4) needs a new preset field. It goes with [Put the call parameters on the presets](22-call-parameters-on-presets.md).
- The history cut (ticket 06 fix 2) is [Cut the history in blocks](20-cut-the-history-in-blocks.md).

Done when the check trial shows the new prompt text and `python build.py` passes.

## Decisions

Choices made when the work was ticketed ([Write the build plan for the engine changes](09-build-plan.md)), corrected by the plan review of 2026-10-31:

- **Each box sorts its card lists where it renders them.**
  - The Narrator sorts `<KnownCharacters>` and `<CharactersPresent>` in `render_character_cards_layer`. The Quantifier sorts `<available_character_ids>` and `<PreviousCharactersPresent>` in `quantifier/prompt.rs`. Options sorts `<CharactersPresent>` in `options/prompt.rs`.
  - All three call one associated function, `CharacterCard::in_id_order(cards: &[CharacterCard]) -> Vec<&CharacterCard>` in `domain/model/character.rs`. It returns references, so a Job clones no card.
  - The Swipe state keeps its order. Reason: a seam contract pins the rule in each box, and a Swipe state stored before the change also renders sorted.
  - The `Relationships:` sub-list keeps the order of the card data. `<available_rooms>` keeps the map order.
- **A layer with a blank body renders nothing, not even its tags.** "Blank" means empty after trimming.
  - Today only `<PlayerInput>` can have a blank body. Every other layer always writes a label or a fallback sentence, and `<Persona>` and `<Guide>` already drop when empty. So the change is one blank check in `render_user_layer`, through a small `LayerRenderer` helper that any layer can use. The other layers keep their code and their bytes.
  - `prompt_system.md` states the rule for every layer, so a new layer follows it.
  - The fallback sentences ("No characters in this world.", "No Characters are present in this Room.", "(No history yet - this is the start of the conversation)") stay, because they tell the model a fact (owner, 2026-10-31, O4 in [Settle the open questions of the engine tickets](17-settle-engine-questions.md)). So the card lists and the history do not use the helper.
- **`Scenario:`.** `CONTEXT.md` also uses **Scenario** for the start of a Game. The label is text for the model, and it names the card field `scenario`, so the owner's choice stands. The docs say "the card's scenario field".
- **`<LatestMessage>`** holds the saved text only, with the indent of today: `"<LatestMessage>\n  {text}\n</LatestMessage>\n\n"`. A migration renames the word `LatestNarration` to `LatestMessage` in every stored preset, user copies included, as migration `due(31)` did for another tag.
- **Card text placeholders.** Both agent prompts render card text with `TemplateVars::from_persona`, as the Narrator does, so one card reads the same in all three calls. Their preset text then also renders the `{{persona_*}}` placeholders. Both prompt contexts replace `player_name: &str` with `persona: &PersonaCard`. The name comes from `persona.sheet.name`.
- **The Options closing line** becomes: "Based on the scene above, respond with options in the format the system instructions specify. Each option is a possible player action — something <Persona name> could do next — not a story outcome." The Persona name is rendered, as today. The count still reaches the model through `{{option_count}}` in the seed Options preset and in the fallback system prompt.
- **The parser shapes in `agent_system.md`** state only what an author's preset must ask the model for. The parser's search order is code, not a contract.

## Facts

Known on 2026-10-31.

The Narrator-role prompt:

- The user half is a fixed array of eight render calls in `LayerRenderer::render_and_fit` (`src/application/prompting/assembler.rs`), joined with `"\n\n"`. A layer that returns `""` is dropped by one `filter`. The post-history splice is a `String` field of `LayerRenderer`, not a render method.
- Only `<Persona>` (on Impersonate), the post-history splice and `<Guide>` can return `""` today. `<GameState>` always writes `Current Room: `, `<WorldLore>` always writes `World: `, the card lists and the history write a fallback sentence. `<PlayerInput>` renders `<PlayerInput>\n\n</PlayerInput>` when the message is empty: on an Impersonate with no direction, and on a Continue after a Narration (`Narrator::user_message`).
- The layers differ in their bytes. For example, `<KnownCharacters>` and `<CharactersPresent>` have no newline after the open tag, and `<Persona>` has one. No test pins the bytes of `<GameState>`, the card lists, `<WorldLore>` or the history. `tests/http/autoplay/run.rs` (scenario 41.1) asserts `<PlayerInput>\n{input}\n</PlayerInput>`.
- `Context:` has one render site: the present card in `render_character_cards_layer`. No test, spec, doc or data file holds the label.
- `<KnownCharacters>` comes from `bundle.characters.values()` (`job.rs`), and `WorldBundle.characters` is a `HashMap`. `<CharactersPresent>` keeps the Swipe state order. All five card lists are `&[CharacterCard]`. The Character id is a `String`. No sort by id exists in `src/`.
- The free-function guardrail does not allow a top-level free function in `assembler.rs`.
- The mock provider (`providers/mock.rs`) falls back to the first line of the user prompt when `extract_player_input` finds no `<PlayerInput>` tag. That first line is `<GameState>`. So on a mock server, an Impersonate with no direction answers `[MockNarration] <GameState>`. No test asserts that text.
- `domain/model/utils/xml.rs::wrap_xml` always writes its tags and indents every line, so it does not fit the blank rule.

The Quantifier and Options prompts:

- `<LatestNarration>` is in `quantifier/prompt.rs` twice (the block and the closing sentence) and in `data/prompt_presets/quantifier/default.json` twice, in one `instructions` string. Ticket 10 says four times for the file. The file has two.
- The block is `"<LatestNarration>\n  {player_name}: {text}\n</LatestNarration>\n\n"`. The text is `HookInput.text`: a Narration, an Event Narration or an Impersonate Input.
- The seed writes a preset only when its id is absent or its stored row has no text (`process_preset_file`, `bootstrap/run.rs`). A default preset cannot be edited or deleted (`PromptPresetService::ensure_editable`). Migration `due(31)` (`storage/utils/plumbing.rs`) renamed a tag in four text columns of every stored preset with SQL `replace()`. Its test is `test_v31_renames_the_character_ids_tag_in_stored_presets`. `LATEST_VERSION` is 31.
- Both prompt builders render the card `description` raw in `prompt.rs`, and build `TemplateVars::new(player_name)`. `QuantifierPromptContext` and `OptionsPromptContext` carry `player_name: &str` and no Persona. `WorldView.persona` is a `&PersonaCard`, in reach in both `orchestration.rs` files.
- Constructions of the two contexts (each breaks when a field changes):
  - `QuantifierPromptContext`, 16: `quantifier/utils/orchestration.rs` (2), `quantifier/utils/orchestration_tests.rs` (6), `quantifier/prompt_tests.rs` (8).
  - `OptionsPromptContext`, 4: `options/utils/orchestration.rs` (1), `options/prompt_tests.rs` (3).
- `<available_character_ids>` comes from a `HashMap`. `<PreviousCharactersPresent>` and Options `<CharactersPresent>` keep the Swipe state order.
- The Options closing line says "respond with exactly {count} options" (`options/prompt.rs`). The legacy test `user_prompt_carries_room_character_and_history_sections` asserts "exactly 3 options". The parser keeps at most the count.
- No card in `data/worlds/` uses a `{{persona_*}}` placeholder today.
- The legacy test `test_quantifier_prompt_builder_empty_history` is the only case with an empty history. It finds the Persona name only through the `Hero: ` prefix. `test_quantifier_prompt_builder_basic` keeps its `Hero` assertion through its own preset text and the `sender` of its history.
- `agent_system.md` lists no parser shape today.
- The parser shapes:
  - The Quantifier reads a JSON object with `characters_present` (alias `npcs_in_room`) and `movement` (`type`, `destination`). With no JSON, it falls back to the Character ids found in the text, at Medium confidence, with no movement.
  - Options reads `<suggestion>` tags, a numbered list, or `Suggestion N:` lines. The first shape that matches wins.

## What already exists

| Need | Reuse |
|---|---|
| Placeholder render of card text | `TemplateVars::from_persona` and `TemplateVars::render` |
| A layer that drops when blank | `render_guide_layer` already trims and returns `""` |
| The tag rename in stored presets | The four-column `replace()` of `due(31)`, and its test |
| Seam test helpers | `layer()` and `Fixture` (`narrator_tests.rs`), `Turn` (`quantifier/agent_tests.rs`), `Scene` (`options/agent_tests.rs`), `ScriptedModel`, `make_history()` (`test_support/quantifier.rs`) |

## Steps

1. **The sort.** Add `CharacterCard::in_id_order` with a test in `domain/model/character_tests.rs`.
2. **The Narrator.** Add the seam cases to `narrator_tests.rs`, each with its `// CASE` line, then make them pass:
   - `narrator-cards-in-id-order`: a `CharacterContext` with ids `c`, `a`, `b` in both lists renders both lists as `a`, `b`, `c`;
   - `narrator-card-scenario-label`: a present card with a scenario renders `Scenario:` and no `Context:`.
3. **The Persona in the agent contexts.** Replace `player_name` with `persona: &PersonaCard` in `QuantifierPromptContext` and `OptionsPromptContext`. Update the 20 constructions listed in the Facts. Both builders build `TemplateVars::from_persona(context.persona)`.
4. **The Quantifier.** Add the cases to `quantifier/agent_tests.rs`, then change `quantifier/prompt.rs`:
   - `quantifier-cards-in-id-order`: both lists are in id order, from a `HashMap` and a present list out of order;
   - `quantifier-card-text-renders-placeholders`: a card description with `{{user}}` reaches the prompt with the Persona name.
5. **Options.** Add `options-cards-in-id-order` and `options-card-text-renders-placeholders` to `options/agent_tests.rs`, then change `options/prompt.rs`.
6. **The blank `<PlayerInput>`.**
   - Add `narrator-blank-player-input-renders-nothing`: an Impersonate with no direction has no `<PlayerInput>` tag, and its user message ends with the post-history splice.
   - Rewrite `narrator-user-message-after-narration-empty` for a Continue after a Narration: no `<PlayerInput>` tag. Its `layer()` helper panics when the tag is absent, so the case reads the user message directly.
   - Add the `LayerRenderer` helper (see Decisions), and use it in `render_user_layer`.
   - Check the mock answer of an Impersonate with no direction (see Facts). No test asserts it today.
7. **`<LatestMessage>`.**
   - Add `quantifier-latest-message-block` to `quantifier/agent_tests.rs`, with an empty history so the empty-history path stays covered. The block is exactly `"<LatestMessage>\n  {text}\n</LatestMessage>\n\n"`, and the closing sentence names `<LatestMessage>`.
   - Change the format string and the closing block in `quantifier/prompt.rs`, and both occurrences in `data/prompt_presets/quantifier/default.json`.
   - Delete the legacy tests that pin the old shape: `test_quantifier_prompt_uses_latest_narration_tag`, `test_quantifier_prompt_references_latest_narration_in_query` and `test_quantifier_prompt_builder_empty_history`.
   - Migration: replace `LatestNarration` with `LatestMessage` in the four text columns of every stored preset, after `due(31)`, with a comment on why in the shape of `due(31)`. Raise `LATEST_VERSION`. Add `test_v32_renames_the_latest_narration_tag_in_stored_presets` to `plumbing_tests.rs` (use the next free number if another ticket took 32).
8. **The Options closing line.** Add `options-closing-line-names-no-count` to `options/agent_tests.rs`. It asserts the new closing sentence itself, not the absence of a digit. Remove the "exactly 3 options" assertion from the legacy test. Change `options/prompt.rs`.
9. **Docs.**
   - `docs/diataxis/reference/narrative/prompt_system.md`:
     - the Layer 2 row: every card list is in Character id order;
     - "The remaining six layers are always present" becomes the one rule: a layer with a blank body renders nothing, not even its tags;
     - "Context Templates": placeholders render in preset text, in World, Room, Persona, Character and rule text, and in the card text of the Quantifier and Options calls. This also fixes a mismatch that [List what reaches the model in each call](01-what-reaches-the-model.md) found.
   - `docs/diataxis/explanation/prompt_system_design.md`:
     - the sentence that says the user half carries "the `<PlayerInput>`" adds "when it has content";
     - the reasons: a random card order has no meaning, breaks the provider cache and makes two trials of one setup differ. An empty tagged block at the end of the prompt reads as an empty instruction where the model starts to write. The label `Scenario:` keeps the meaning of the card field.
   - `docs/diataxis/reference/narrative/agent_system.md`: the answer shapes of each parser (see Facts and Decisions).
10. **Check.** Run `python scripts/trial.py run scripts/trial_setups/redmist_julian.json --cycles 20` on a probe server built from this branch. Read the pack:
    - every card list is in id order in every call, also after the Quantifier changes the Characters present;
    - an Impersonate with no direction has no `<PlayerInput>`;
    - the Quantifier block is `<LatestMessage>` with no name prefix, and card text has no raw `{{user}}`;
    - the cached input share against trial B (1 %), as a report, not a gate.

    Options does not run in this setup (Options always on is off in the World), so the seam cases cover Options.
11. Run `python build.py`. The mock answer changes, so the browser tier must pass too.

## Failure modes

| Code path | Failure | Result | Handled by |
|---|---|---|---|
| Card sort at render | A Swipe state stores another order | None. Each box sorts what it renders | The seam cases |
| Card sort | Two present entries with the same id | The sort is stable, so their order stays | Accepted |
| Blank layer | The user message holds only spaces | It renders nothing, where today it renders the spaces | The trim. Intended |
| Blank layer | An Impersonate with no direction on a mock server | The mock answers `[MockNarration] <GameState>` | Accepted. No test or reader depends on it |
| Tag migration | A user copy names the tag in other words (`latest narration`) | Not renamed, so the preset points at a block that is gone | Accepted. Same limit as `due(31)`. The wording is authoring |
| Options closing line | The model returns another number of Options | The parser keeps at most the count | Existing code |
| Card text | A card holds `{{option_count}}` | It renders 3 | Accepted |

## Not in scope

- The Options count field, and any preset field ([Put the call parameters on the presets](22-call-parameters-on-presets.md)).
- The history cut ([Cut the history in blocks](20-cut-the-history-in-blocks.md)).
- The order of `Relationships:` and of `<available_rooms>`.
- A shared wrapper for every layer. Only a layer that can be blank uses the helper.
- The wording of the presets (the authoring effort).

## Output

Resolved when the check pack shows the new text and `python build.py` passes. The answer records the CASE ids, the migration step, and the check results with the pack path.
