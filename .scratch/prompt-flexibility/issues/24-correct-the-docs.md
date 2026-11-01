# Correct the docs that are wrong today

Type: task (AFK)
Status: resolved
Blocked by: —

## Question

Correct the doc statements that earlier tickets found wrong and that need no code change:

- the 32 rows for `docs/external_applications/` that [How ST and ME let an author change a call](02-st-me-prompt-configuration.md) listed;
- three statements of `docs/diataxis/reference/narrative/prompt_system.md` that [List what reaches the model in each call](01-what-reaches-the-model.md) found.

Done when the statements match their sources and `python build.py` passes.

## Decisions

Choices made when the work was ticketed ([Write the build plan for the engine changes](09-build-plan.md)), corrected by the plan review of 2026-10-31:

- **All 32 rows are in scope.** 24 rows correct a wrong statement. 8 rows name an omission ("no mention of …"). An omission becomes one sentence in the section that already covers the topic, not a new section, so the third-party docs do not grow much.
- **The pinned source wins.** Check each corrected sentence against the source at its pinned commit before you write it. When the asset and the source disagree, write what the source says, and record the asset's error in the answer.
- **The full gate runs,** as the map's "Engine tickets" note asks, also for a docs-only ticket.

## Facts

Known on 2026-10-31.

- The 32 rows are in the section "Corrections that `docs/external_applications/` needs" of [ST and ME prompt configuration](../assets/st-me-prompt-configuration.md): 12 for `sillytavern_prompt_system.md`, 11 for `marinara_engine.md`, 9 for `marinara_engine_system_prompt.md`. Some rows hold more than one fact.
- The pinned sources are already on disk: `/tmp/research/st` at SillyTavern `06bde939fb1e9c4c8d8641d810f0a916b5bce127`, `/tmp/research/marinara` at Marinara Engine `457e827a3d3291be978121f8c50c98c5dd9d644d`. If they are gone, clone them there again.
- The checks of `docs/external_applications/`:
  - Vale skips it (`TRANSIENT_DIR_NAMES` in `scripts/vale_lint.py`).
  - `validate_docs.py` scans Markdown only under `docs/diataxis/`, and its DOC-anchor rule covers only `src/` and top-level `*.toml`. But its route scan does read `docs/external_applications/`. A path whose first segment matches an engine route prefix fails with `UNKNOWN_ROUTE`. Keep third-party API paths as the source writes them.
  - So no check catches a wrong fact there. The sources are the only check.
- `prompt_system.md` is under the Vale gate (sentences of at most 25 words, paragraphs of at most six sentences, no semicolons) and under the `validate_docs.py` rules (no `path:line` citation, no route string, valid front matter).
- The three `prompt_system.md` statements (ticket 01):
  - "from the active system-prompt preset": an Impersonate uses the impersonate preset (`Generation::preset_type`).
  - The sanitizer "replaces any `{{…}}` pattern with `[FILTERED]`": an empty `{{}}` passes. Only this clause is wrong. The sentence already says that the sanitizer works on the `<PlayerInput>` content, and `sanitize_for_prompt` has one caller.
  - "stores editable presets": a default (`is_default`) preset is refused on edit (`PromptPresetService::ensure_editable`). It can be copied (`POST /prompt-presets/:id/duplicate`).
- `assets/what-reaches-the-model.md` says that every seed file in `data/prompt_presets/` has `is_default: true`. That is wrong: `options/roadway_domains.json` has `false`. Do not copy that clause.
- The fourth statement of ticket 01, the scope of "Context Templates", goes with [Fix the prompt text](19-fix-the-prompt-text.md), because that ticket changes the scope.
- `docs/AGENTS.md` lists only `docs/diataxis/`, so no index changes.

## What already exists

| Need | Reuse |
|---|---|
| The replacement text of the 32 rows | The tables of the asset |
| The three `prompt_system.md` rows | `assets/what-reaches-the-model.md`, its list of rows that the code contradicts |
| The sources | `/tmp/research/st` and `/tmp/research/marinara` at the pinned commits |

## Steps

1. Apply the 32 rows. Check each sentence against the pinned source first (see Decisions). Do not carry the asset's `file:line` numbers into the docs.
2. Correct the three `prompt_system.md` statements. Read `docs/AGENTS.md` first: a Reference doc states what is, with no plan and no value. In the sanitizer sentence, change only the "any `{{…}}` pattern" clause. Keep each sentence under 25 words, and use no semicolon.
3. Run `python build.py validate-docs` and `python build.py vale`.
4. **Doc review.** Run `/document-review` (`.agents/skills/document-review/SKILL.md`) in a `reviewer` subagent, as a diff review of the changed files: the three files of `docs/external_applications/` and `prompt_system.md`. The brief gives the skill path, the diff scope, the pinned source paths and the asset. The reviewer checks each changed sentence against the pinned source (step 4 of the skill) and against the wording rules (step 5). Verify each finding. Fix each valid finding in the changed text. List each rejected finding with its reason in the answer. A finding under **Pre-existing** (a keep-or-delete verdict on text this ticket did not change) is not fixed here, because a rewrite of the third-party docs is not in scope. The answer lists it.
5. Run `python build.py`.

## Failure modes

| Code path | Failure | Result | Handled by |
|---|---|---|---|
| A corrected third-party sentence | It is wrong in a new way | No check catches it | The pinned source is checked before each sentence, and the doc review checks it again |
| A corrected third-party sentence | It holds a path that looks like an engine route | `validate-docs` fails with `UNKNOWN_ROUTE` | Keep the source's API paths as they are |
| A `prompt_system.md` sentence | It passes 25 words or holds a semicolon | `vale` fails | Split it |
| The asset | One of its rows is wrong | The doc would copy the error | The source wins, and the answer records it |

## Not in scope

- The "Context Templates" statement ([Fix the prompt text](19-fix-the-prompt-text.md) owns it).
- The rows of `what-reaches-the-model.md` that the code does not contradict.
- Any new section in the third-party docs, and any rewrite of their style.
- Fixing the assets themselves. The answer records their errors.

## Output

Resolved when `python build.py` passes. The answer lists each correction that the asset got wrong, and what the source says instead.

## Answer

Resolved on 2026-11-01, on the branch `prompt-flexibility` (not committed). The owner added step 4 (the doc review) when the work started. A `strong-implementer` applied the 32 rows, and a `reviewer` ran `/document-review` on the diff. `python build.py` passes (`logs/build_20261101_134223.log`). No code changed, so no comment pass and no `/code-review` ran.

**`prompt_system.md`.** An Impersonate uses the impersonate preset, and every other generation uses the system preset. The sanitizer replaces each non-empty `{{…}}` pattern. A default preset is read-only, and a copy of it is editable.

**The 32 rows.** All are applied. ME system prompt row 8 needed no change: the Parameters block matches the source.

**Where the asset was wrong** (the doc review confirmed each against the pinned source):

- ST row 5: `enabled` is in the `prompt_order` entry, not the prompt. `forbid_overrides` applies only to `main` and `jailbreak`.
- ST row 6: the utility prompts also include `impersonation_prompt`. `continue_postfix`, `assistant_prefill` and `assistant_impersonation` are preset keys outside the "Utility Prompts" block.
- ST row 7: the depth prompt is always in-chat and has only a depth and a role. The position of the Author's Note is in-chat only by default.
- ST row 9: the macros (`{{char}}`, `{{scenario}}` and others) work in both APIs. Only the Handlebars-style blocks (`{{#if}}`) are text-completion only.
- ME row 3: the assembler never reads `parentGroupId`, so the built prompt has no nested group wrappers. The Lore group also holds Dialogue Examples.
- ME row 6: `agent-prompts.ts` holds four constants, not three (`DEFAULT_CHAT_SUMMARY_COMBINE_PROMPT` too). The chat-summary and secret-plot ids are retired agents. The constants serve runtimes.
- ME row 7: `maxTokens` is in the agent `settings`. The temperature comes from the agent's connection.

**Doc review.** 17 findings, all accepted and fixed: 6 accuracy, 11 wording. Two of them go past the rows: the ME section `role` enum also has `assistant`, and `wrapInXml` is not read by the assembler, so it is no longer named.

**Pre-existing, not fixed** (no row names them, and a rewrite is not in scope):

- `sillytavern_prompt_system.md`: "Prompt Manager UI Concepts" repeats the layer list. The Token Budget keys are `openai_max_context` and `openai_max_tokens`. The WI entry fields are `comment`, `key` and `keysecondary`. The V1 example field is `mes_example`.
- `marinara_engine.md`: the Multi-Mode table still names three sections. The agent table lists retired agents (`chat-summary`, `secret-plot-driver`, `prompt-reviewer`). A Conversation chat with a preset does not run the assembler (inferred). The engine type is `PromptAssembler`, not `PromptBuilder`.
- `marinara_engine_system_prompt.md`: the `{{length}}` and `{{guidelines}}` defaults differ from the pinned `defaultChoices` in a few words.
- `prompt_system.md`: "Game Master call" is not a `CONTEXT.md` term.
