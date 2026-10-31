# Tools that evaluate characters, prompts and roleplay output

This asset gathers material for [05-evaluation-tools](../issues/05-evaluation-tools.md). It does not
pick criteria. Each entry says what the tool is, what it measures, how it measures it, and where the
claim comes from. `known` means the source states it. `inferred` means this note reads it from code,
docs or context.

Sources were read on 2026-10-31. SillyTavern was read at `06bde93` and Marinara Engine at `457e827`,
the pinned commits of the map. Marinara agent prompts come from
[Pasta-Devs/Marinara-Agents](https://github.com/Pasta-Devs/Marinara-Agents), which the Engine
downloads.

## Short answer

The owner's memory is correct. The SillyTavern extension is most likely
**SillyTavern-CharacterTools**, later replaced by **CardRefinery** (`inferred` from the stated
lineage). Marinara's near equivalent is the **Card Evolution Auditor** agent. Both use an LLM judge
that scores or edits the card. Neither one judges a whole story.

No tool found here scores a long, multi-cycle Autoplay story. The closest output judges are the
per-reply tools: ST-jeved in SillyTavern, and Continuity Checker and Prose Guardian in Marinara.

## SillyTavern: card evaluators

SillyTavern core has no card quality checker. The official extension list has Token Counter and
Duplicate Finder ([known](https://docs.sillytavern.app/extensions/)). The evaluators below are
third-party extensions.

| Tool | What it is | What it measures | How | Source |
|---|---|---|---|---|
| **CharacterTools** | Extension with a Score → Rewrite → Analyze pipeline | A rating and critique for each card field, such as description, personality, first message, scenario, example messages and system prompt. The Analyze stage compares the rewrite with the original and gives a verdict: `ACCEPT`, `NEEDS_REFINEMENT` or `REGRESSION` | LLM judge with editable prompts and optional JSON schemas. The person locks, accepts or reverts each stage | [README](https://github.com/Inktomi93/SillyTavern-CharacterTools) (`known`) |
| **CardRefinery** and Orbweaver's **Refinery** | The successor of CharacterTools, first as an extension, now a feature of the Orbweaver app | A score from 1 to 10 and a critique for each field. A drift check on the rewrite. A score for a whole library, so cards can be sorted | LLM judge. The person approves each change, and every accepted change is kept as a version | [README](https://github.com/Inktomi93/SillyTavern-CardRefinery), [wiki](https://github.com/Inktomi93/Orbweaver/wiki/Refinery) (`known`) |
| **Character Card Suggestions** | Extension that watches one chat | Durable facts in the chat that may belong in the card. Each proposal holds a reason, the evidence, the current text and replacement text | LLM auditor with an allowlist of text fields. It drops low-confidence, duplicate, no-op and dismissed proposals. The person applies selected edits | [README](https://github.com/zapers/SillyTavern-Character-Card-Suggestions) (`known`) |
| **CardQualifier** | Local browser page for card files | Card validity, identity and behaviour clarity, scenario and greeting use, whether examples teach a voice, token footprint, metadata and lorebook hygiene, quality hazards such as placeholder text, contradictions and tag spam. A Playability Audit reads connected causes, behaviour, unresolved tension and the link from greeting to card | Deterministic local rubric with a score from 0 to 100. An optional LLM reviewer drafts repairs for named findings only, and it does not score | [README](https://github.com/owler-tavern/cardqualifier) (`known`) |
| **Character Card Studio** | Extension that authors and audits cards | Static checks: required fields, field token budgets, lorebook problems, cross-field consistency and total tokens. It returns a score out of 100 with errors, warnings and notes. An AI review then scores concept clarity, first message, character voice and replay value | Local rule analysis first, then an LLM scorecard. The person fixes or ignores each item | [README](https://github.com/MMKAVERAPPA/SillyTavern-CharCardStudio) (`known`) |
| **Duplicate Finder** (official) | Extension that groups similar cards | Similarity between cards over the V1 fields (name, description, first message, personality, scenario, example dialogue), and the V2 fields if enabled. It measures duplication, not quality | Rule: sentence similarity or Levenshtein distance, with a slider threshold | [README](https://github.com/SillyTavern/Extension-DupeFinder), [docs](https://docs.sillytavern.app/extensions/) (`known`) |

## SillyTavern: output judges

| Tool | What it is | What it measures | How | Source |
|---|---|---|---|---|
| **ST-jeved** | Extension that measures each reply and acts on the answer | Whatever questions the user writes. A sensor returns a number from a scale, one option, or a percentage chance. Examples in the README: pacing, voice drift from the card, a reply that speaks for the player, and a running state list | A small decision model answers each sensor. A rule with thresholds, a cooldown and an exception then adds one prompt line, rerolls the reply, changes a list, or runs a script | [README](https://github.com/mossyfield/ST-jeved) (`known`) |
| **Prompt Inspector** (official) | Extension that shows the output prompt | Not quality. It shows the assembled prompt before the call, and lets the person edit it | Display only | [docs](https://docs.sillytavern.app/extensions/) (`known`) |

ST-jeved is the closest SillyTavern match to an output judge with automatic action. It is also the
closest match to an engine rule set, because the rules are deterministic and the model only answers
the questions (`inferred`).

## Marinara Engine

The Engine and its official agent packages hold no story-level scorer (`inferred`: the agent
reference and the `packages/` folder list no such feature). The evaluation work is per reply.

| Tool | What it is | What it measures | How | Source |
|---|---|---|---|---|
| **Card Evolution Auditor** | Post-processing agent, every 8 messages by default | Durable changes that now contradict or extend the saved card. The prompt names the durable kinds: job, home, body, powers, core beliefs, relationships, backstory, appearance. It names what to ignore: temporary mood, scene location, transient clothing and healed injuries | LLM judge. It returns `characterId`, `field`, a verbatim `oldText`, `newText` and a reason, or an empty list. The person approves each edit in a modal, and stale edits are marked. It never writes on its own | [agent reference](https://github.com/Pasta-Devs/Marinara-Engine/blob/457e827a3d3291be978121f8c50c98c5dd9d644d/docs/agents/built-in-agents.md), [prompt](https://github.com/Pasta-Devs/Marinara-Agents/blob/main/packages/card-evolution-auditor/agents.json), [approval flow](https://github.com/Pasta-Devs/Marinara-Engine/blob/457e827a3d3291be978121f8c50c98c5dd9d644d/docs/agents/approvals-and-agent-suite.md) (`known`) |
| **Continuity Checker** | Post-processing editor | Concrete spatial, timeline and physical logic errors in the latest reply. The prompt gives examples: a seated character walks before standing, someone five metres away is already beside the user, noon becomes night with no time skip | LLM rewrite with a list of changes. The person picks the fixes | [agent reference](https://github.com/Pasta-Devs/Marinara-Engine/blob/457e827a3d3291be978121f8c50c98c5dd9d644d/docs/agents/built-in-agents.md), [prompt](https://github.com/Pasta-Devs/Marinara-Agents/blob/main/packages/continuity/agents.json) (`known`) |
| **Prose Guardian** | Post-processing editor | Banned words, repetition and unwanted prose habits. It has banned, avoid and prefer lists | LLM rewrite with a list of changes. It keeps the events and the meaning | [agent reference](https://github.com/Pasta-Devs/Marinara-Engine/blob/457e827a3d3291be978121f8c50c98c5dd9d644d/docs/agents/built-in-agents.md), [prompt](https://github.com/Pasta-Devs/Marinara-Agents/blob/main/packages/prose-guardian/agents.json) (`known`) |
| **Tracker agents** (World State, Character Tracker, Beholder, Relationship Tracker, Persona Stats, Custom Tracker, Inventory Tracker, Quest Tracker, Memory Nag) | Post-processing agents that keep structured state and add it to the next prompt | State change after a reply: who is present, the place, date and time, mood, outfit, wounds, relationships, stats, items and quests | LLM extraction into a schema. Beholder validates its own snapshot. The person can edit and lock entries | [agent reference](https://github.com/Pasta-Devs/Marinara-Engine/blob/457e827a3d3291be978121f8c50c98c5dd9d644d/docs/agents/built-in-agents.md) (`known`) |

The trackers do not judge. They are the state record that a later judge could compare against. The
Engine's own answer to a stale state is to re-extract the state after each reply (`inferred`).

## Roleplay benchmarks

These are research benchmarks. They score model answers, not stories. All of them score short
dialogue turns (`known`), which matches the Impersonate call better than the Narration call
(`inferred`).

| Tool | What it is | What it measures | How | Source |
|---|---|---|---|---|
| **CharacterEval** | Chinese benchmark: 1,785 dialogues, 23,020 examples, 77 characters | Four dimensions and 13 metrics. Conversation ability: fluency, coherency, consistency. Character consistency: knowledge exposure, knowledge accuracy, knowledge hallucination, persona behaviour, persona utterance. Role-play attractiveness: human-likeness, communication skills, expression diversity, empathy. Personality back-testing: MBTI accuracy | Twelve metrics use a 5-point score. Twelve annotators set the human reference, which trains a reward model (CharacterRM) that does the main scoring. GPT-4 is also reported as a judge. MBTI is scored by rule | [paper](https://arxiv.org/abs/2401.01275), [code](https://github.com/morecry/CharacterEval) (`known`) |
| **CharacterBench** | Bilingual benchmark: 22,859 annotated samples, 3,956 characters, 25 categories | Six aspects and 11 dimensions. Memory consistency. Fact accuracy and boundary consistency. Attribute consistency and behaviour consistency. Emotional self-regulation and empathetic responsiveness. Morality stability and robustness. Human-likeness and engagement | Human annotators score the sample set. A trained judge model, CharacterJudge (Qwen2-7B-Chat), then scores new answers. The paper reports it beats GPT-4 as a judge | [paper](https://arxiv.org/abs/2412.11912), [code](https://github.com/thu-coai/CharacterBench) (`known`) |
| **InCharacter** | Interview method: 32 characters, 14 psychological scales | Personality fidelity. It interviews the character agent, then compares the measured traits with the traits people perceive in the character | Scripted interview questions, then psychological scale scoring. The comparison is the metric | [paper](https://arxiv.org/abs/2310.17976) (`known`) |
| **PersonaGym** | Framework: 200 personas, 10,000 questions | How faithfully an agent holds a persona across persona-relevant environments. `PersonaScore` is the summary | An automatic metric built on decision theory, aligned to human judgement | [paper](https://arxiv.org/abs/2407.18416), [site](https://personagym.com/) (`known`) |
| **RoleEval** | Bilingual question set: 6,000 multiple-choice questions, 300 characters | Role knowledge: personal information, relationships, abilities and experiences, at one hop and at several hops | Rule: answer accuracy | [paper](https://arxiv.org/abs/2312.16132), [data](https://github.com/Magnetic2014/RoleEval) (`known`) |
| **RoleLLM** / **RoleBench** | Framework and benchmark for 100 roles | Speaking-style imitation, answer accuracy and role-specific knowledge and memory | Rouge-L overlap, a GPT judge in the AlpacaEval style, and three human reviewers | [paper](https://arxiv.org/abs/2310.00746) (`known`) |

## Rubric and rule methods

| Method | What it gives | How | Source |
|---|---|---|---|
| **G-Eval** | A way to build an LLM judge from a rubric | A prompt holds the task and the criteria. The model writes its reasoning first, then a score. Token probabilities weight the score | [paper](https://arxiv.org/abs/2303.16634) (`known`) |
| **Verifiable instruction checks** (for example IFEval) | Rule checks on a reply against a written instruction, such as a word limit or a required format | A program checks the answer, with no model | [IFEval paper](https://arxiv.org/abs/2311.07911) (`known` method, `inferred` fit) |
| **Repetition metrics** (distinct-n, self-BLEU) | A number for repeated words or phrases across replies | Rule: n-gram overlap between replies | `known` metric, `inferred` fit |

## Which tools would catch the faults of the first run

This is a mapping only. It is not a criteria decision. Every mark here is `inferred`. The faults come
from [the review](impersonate-output-review.md) and [the transcript](autoplay-story-transcript.md).

| Fault from the first run | Tool that could catch it | How it would catch it |
|---|---|---|
| Inputs too long for one beat | ST-jeved with a scale sensor; IFEval-style rule; Prose Guardian, if a length habit is in its avoid list | A score or a word count fires a rule, then the rule nudges or rerolls |
| The same pattern in every Input | Prose Guardian (repetition); CharacterEval expression diversity; distinct-n | An LLM rewrite, or a number across replies |
| A beat done twice | Continuity Checker; ST-jeved with a history sensor | The reply is compared with earlier turns |
| Fact errors from the card | Card Evolution Auditor; CharacterTools or CardRefinery soul check; CharacterBench fact accuracy; RoleEval | The card is the reference, and the story text is compared with it |
| A stale game state | Marinara trackers; Continuity Checker; CardQualifier playability audit, for the card side | Structured state is re-extracted after each reply, or the reply is checked against it |

## Limits

- None of these tools runs on the Chronicler stack. Each one is a source of method, not a dependency
  (`known`).
- The card tools judge card text alone. They do not see a prompt or an output (`known`).
- The benchmark suites score one answer to one prompt. They do not score a story that grows over many
  cycles (`inferred`).
- ST-jeved and the Marinara editors judge one reply. A repeated beat or a drifting voice over six
  cycles needs a check across replies, and none of the found tools does that out of the box
  (`inferred`).
- The criteria for a good output stay open. This asset only lists what exists.
