# Marinara-Engine Reference

**Location:** `D:\John\DevContainer\Marinara-Engine`  
**Relation:** Sister project with a TypeScript-based engine that includes similar chronicler functionality. Contains relevant LLM infrastructure patterns.

---

## Overview

Marinara-Engine is a TypeScript/React-based game engine with a client-server architecture. It shares conceptual DNA with `chronicler_engine` (interactive fiction, LLM narration, NPC dialogue) but implements it in a full-stack TS stack rather than Rust.

Key architectural difference: Marinara uses a **streaming-first** approach with typewriter UI effects, while chronicler_engine uses **blocking request/response** with HTMX fragments.

---

## Relevant Files for LLM Handling

### 1. Thinking/Reasoning Extraction

**`packages/server/src/services/llm/inline-thinking.ts`**

Extracts inline thinking blocks from model responses when providers don't separate them natively:

| Pattern | Regex | Used By |
|---------|-------|---------|
| XML-style | `<think>\|</thinking>\|</thought>` | DeepSeek, QwQ |
| Pipe-style | `<\|think\|>...<\|/think\|>` | Various |
| Channel-style | `<\|channel>thought...<channel\|>` | **Gemma 4** |

```typescript
const CHANNEL_THINKING_BLOCK_RE = 
  /^(\s*)<\|channel>thought\b([\s\S]*?)<channel\|>/i;
```

**Status in Marinara:** Implemented but **not actively used** for Ollama — the client-side streaming filter (`use-generate.ts`) has think-tag filtering disabled (`thinkState = "done"` by default). Ollama's OpenAI-compatible API separates `reasoning` at the JSON level, so inline parsing is unnecessary.

**Status in chronicler_engine:** `extract_content_from_response()` in `llm_client.rs` handles the same concern at the JSON level, checking `content` → `reasoning` → `reasoning_content` fallback chain.

---

### 2. Provider-Native Reasoning Extraction

**`packages/server/src/services/llm/providers/openai.provider.ts`**

Handles multiple provider-specific reasoning formats in the OpenAI-compatible message schema:

```typescript
private static extractReasoning(obj): string {
  // DeepSeek native
  if (obj.reasoning_content) return obj.reasoning_content;
  // OpenRouter / NanoGPT
  if (obj.reasoning) return obj.reasoning;
  // OpenRouter newer format (array of blocks)
  if (obj.reasoning_details) { ... }
}
```

Also handles **content block arrays** (`[{type: "thinking", ...}, {type: "text", ...}]`) from Anthropic-style APIs via OpenRouter.

**chronicler_engine gap:** Does not handle `reasoning_details` arrays or content block formats. Currently only checks string fields.

---

### 3. Model-Specific Parameter Adaptation

**`packages/server/src/services/llm/providers/openai.provider.ts`**

Marinara maintains a registry of model quirks and adapts API calls accordingly:

| Model Family | Adaptation |
|-------------|------------|
| `o1` / `o3` / `o4` | No `temperature`/`top_p`; use "developer" role instead of "system" |
| `gpt-5*` | "developer" role; temperature only when `reasoning_effort: "none"` |
| `claude-opus-4-7+` | All sampling params forbidden |
| `glm*` | Boolean `thinking` toggle instead of effort-based |

**chronicler_engine gap:** No model-specific parameter adaptation. All models get the same payload structure (`system` role, temperature not sent). `max_tokens` is now dynamically fitted per connection via `fit_messages_to_context()`.

---

### 4. Context Budget Management

**`packages/server/src/services/llm/base-provider.ts`**

Sophisticated token budgeting. chronicler_engine now implements an equivalent in `fit_messages_to_context()`:

```typescript
const usableWindow = Math.max(1, maxContext - CONTEXT_SAFETY_MARGIN_TOKENS);
const reservedInputFloor = Math.min(MIN_INPUT_BUDGET_TOKENS, usableWindow - 1);
const maxTokens = Math.max(1, 
  Math.min(requestedMaxTokens, usableWindow - reservedInputFloor));
const inputBudget = Math.max(0, maxContext - maxTokens - SAFETY_MARGIN);
```

**What it does:**
- Reserves a safety margin from the model's context window
- Ensures input messages never consume the entire budget
- Caps `max_tokens` to leave room for the system prompt + history
- Trims older messages if the input exceeds the calculated budget

**Why it matters for Gemma 4:** With chronicler's 5511-char system prompt, a small fixed `max_tokens` leaves no room for the model to think *and* respond. chronicler_engine now dynamically caps `max_tokens` and trims oldest history entries via `fit_messages_to_context()`.

---

### 5. Streaming Think-Tag Filter

**`packages/client/src/hooks/use-generate.ts`**

Client-side filter for inline thinking tags during streaming:

```typescript
const THINK_OPEN_PREFIXES = ["<thinking>", "<think>", "<|channel>thought"];
let thinkState: string = "done";  // Disabled by default
```

**Note:** Comment in code says `"Think-tag filtering disabled — skip straight to passthrough"`. The infrastructure exists but is not active. Marinara relies on server-side extraction instead.

---

## What Marinara-Engine Does **NOT** Solve

Despite the sophisticated infrastructure, Marinara has **no specific handling** for the Gemma 4 26B-A4B reasoning loop:

- No model-specific prompt engineering for Gemma 4
- No detection of reasoning-loop behavior
- No automatic `max_tokens` escalation for thinking models
- No prompt simplification for MoE models that overthink

The custom `gemma4-26b:latest` (i1-IQ2_XS quant) would exhibit the same empty-content behavior in Marinara as it did in chronicler_engine — burning all tokens in `reasoning` with nothing left for `content`.

**However**, chronicler_engine now has a targeted fix for this (see Lessons below).

---

## Lessons for chronicler_engine

### Immediate wins (low effort)

1. **~~Add `reasoning_details` array support~~** — Still relevant for OpenRouter compatibility
   - OpenRouter is beginning to use this format
   - Future-proofs against provider changes

2. **Add model-specific role adaptation**
   - o1/o3/o4 and GPT-5 use `"developer"` role, not `"system"`
   - Current code sends `"system"` universally

3. **~~Gemma 4 thinking-channel suffix~~** ✅ *Implemented 2026-05-03; corrected 2026-05-04*
   - `apply_gemma4_thinking_suffix()` in `llm_client.rs` detects Gemma 4 models by name and appends the SillyTavern `last_output_sequence` to Ollama backends only: `<|turn>model\n<|channel>thought\n<channel|>`
   - This tells the model the thinking slot is already filled, preventing the infinite reasoning loop
   - **Not applied to OpenRouter** — native chat templates handle turn structure; injecting raw tokens into user content produces corrupted output
   - `sanitize_llm_output()` strips leaked `<channel|>`, `<thought>`, and `<|channel>thought` artifacts from all responses
   - Ref: [SillyTavern Reddit fix](https://old.reddit.com/r/SillyTavernAI/comments/1sbjwke/)

### Medium-term improvements

3. **~~Port context-fitting logic~~** ✅ *Completed 2026-05-03*
   - `fit_messages_to_context()` in `narrative::prompt` dynamically caps `max_tokens`, reserves safety margin + minimum input budget, and trims oldest history first
   - Per-connection `max_context_tokens` added to `Connection` (defaults: 8192 Ollama, 32768 OpenRouter/DeepSeek)

4. **Add `enableThinking` / `reasoningEffort` connection options**
   - Some providers (OpenRouter, OpenAI) allow disabling or controlling reasoning
   - Less critical now that the Gemma 4 loop is fixed via prompt suffix

### Not needed (Ollama handles this)

- Inline think-tag parsing (`<|channel>thought`) — Ollama 0.22.1+ strips these at the API level and populates the `reasoning` JSON field
- Client-side streaming filters — chronicler_engine uses blocking calls, not SSE streams

---

## File Mapping

| Marinara-Engine | chronicler_engine equivalent | Notes |
|-----------------|------------------------------|-------|
| `services/llm/inline-thinking.ts` | `narrative::llm_client::extract_content_from_response()` | Marinara handles inline tags; chronicler handles JSON fields |
| `services/llm/providers/openai.provider.ts` | `narrative::llm_client::call_chat_completions()` | Marinara has more provider formats and model quirks |
| `services/llm/base-provider.ts` | `narrative::prompt::fit_messages_to_context()` | Context fitting implemented 2026-05-03 |
| `services/prompt/assembler.ts` | `narrative::prompt::PromptBuilder` | Both build structured prompts; Marinara has more assembly phases |
| `client/src/hooks/use-generate.ts` | `server::fragments::action_handler()` | Client streaming vs server blocking |
| `db/default-preset.json` | `data/worlds/*.json` + `prompt.rs` | Marinara: user-composable presets with variables. chronicler: compiled prompt layers + per-world global_rules |
| `db/universal-preset.json` | — | High-compatibility preset variant (Llama 3.3, Gemini, Claude) with `<thought>` tags |
| `routes/generate.routes.ts` | `server::fragments::action_handler()` | Mode-specific prompt routing (RP / CONVO / GM) |
| `services/game/gm-prompts.ts` | — | Game Master prompt builder with VN-style dialogue syntax |
| `services/game/party-prompts.ts` | — | Party member controller prompt |
| `services/sidecar/scene-analyzer.ts` | — | Local Gemma JSON-only scene state analyzer |
| `shared/src/constants/agent-prompts.ts` | — | 20+ agent system prompts for sidecar quality enforcement |
| `db/seed-mari.ts` | — | Built-in assistant character (Professor Mari) with app knowledge + command system |

---

## Prompt Architecture (New Discovery)

### Preset-Based Assembly
Unlike chronicler_engine's hardcoded `PromptBuilder` layers (`render_system_layer()`, `render_game_state_layer()`, etc.), Marinara uses a **preset system** where users can compose, reorder, and configure prompt sections via UI.

**Key concepts:**
- **Sections** — Ordered prompt fragments with `injectionOrder` (0, 100, 200... 1200). Each has a `role` (system/user), `enabled` flag, and `wrapInXml` option.
- **Markers** — Empty sections that act as injection points for runtime data (`lorebook`, `character`, `persona`, `chat_history`, `dialogue_examples`, `chat_summary`).
- **Groups** — Logical containers (e.g., "Lore" group holds Setting, Characters, Persona, Past Events markers). Groups can be parented for nested XML wrapping.
- **Variables / Choice Blocks** — User-selectable options injected via `{{variable_name}}`. The Default preset has 7 variables: `role`, `guidelines`, `narration`, `pov`, `tense`, `length`, `language`.
- **Wrap Format** — `xml`, `markdown`, or `none`. XML wrapping places sections inside `<SectionName>` tags.

**File:** `packages/server/src/services/prompt/assembler.ts`

**chronicler_engine gap:** No equivalent preset system. Prompt layers are compiled Rust code; users cannot reorder sections or add custom markers without recompiling.

---

### Multi-Mode Chat System
Marinara supports three distinct chat modes, each with fundamentally different prompt strategies:

| Mode | UI Label | Preset | System Prompt Strategy |
|------|----------|--------|------------------------|
| **Roleplay** | `RP` | Uses assigned preset | Assembled preset sections (Role → Instructions → Output Format) |
| **Conversation** | `CONVO` | Optional preset | If preset assigned: same as RP. If **no preset**: injects built-in DM-style casual texting prompt |
| **Game** | `GM` | Uses assigned preset | GM system prompt + party member prompts + VN-style formatting |

**Key insight:** The conversation mode no-preset fallback is a completely separate prompt path, not just a preset with empty sections. It constructs a `<role>` + `<rules>` XML block dynamically based on character count (private DM vs group DM), schedule-derived status, and character commands.

**File:** `packages/server/src/routes/generate.routes.ts` (lines 712–1338)

**chronicler_engine gap:** Single prompt path for all interactions. No mode-specific prompt variants.

---

### Built-in Agent Ecosystem
Marinara runs 20+ specialized "agents" (sidecar prompts) alongside the main generation. Each agent has its own system prompt and processes the narration output for a specific purpose:

| Agent | Purpose |
|-------|---------|
| `world-state` | Extract date/time/location/weather/temperature |
| `prose-guardian` | Enforce writing quality (repetition ban, sensory rotation, vocabulary freshness) |
| `continuity` | Flag contradictions in narrative |
| `expression` | Pick sprite expressions for characters |
| `echo-chamber` | Simulate Twitch chat reactions |
| `director` | Narrative pacing directions |
| `quest` | Track quest state changes |
| `illustrator` | Generate image prompts |
| `combat` | Track combat encounters |
| `background` | Pick background images |
| `character-tracker` | Track present NPCs and their states |
| `persona-stats` | Track player condition bars (Satiety, Energy, Hygiene, Morale) |
| `chat-summary` | Rolling conversation summaries |
| `spotify` | Music mood matching |
| `editor` | Post-process grammar/continuity |
| `knowledge-retrieval` | Extract relevant lore |
| `haptic` | Buttplug.io device control |
| `cyoa` | Generate player choices |
| `secret-plot-driver` | Hidden narrative architect |
| `lorebook-keeper` | Auto-generate lorebook entries |
| `prompt-reviewer` | Analyze prompt quality |
| `html` | Inline HTML/CSS/JS for visual storytelling |

**File:** `packages/shared/src/constants/agent-prompts.ts`

**chronicler_engine gap:** No multi-agent architecture. All quality enforcement, continuity tracking, and summarization would need to be handled in the main system prompt or post-processing.

---

### Conversation Mode No-Preset Fallback
When a conversation-mode chat has no preset selected, Marinara injects a built-in prompt with two variants:

**Private DM (single character):**
```xml
<role>
You are {charNameList}, a real and autonomous person, chatting with the user called {personaName} in a casual private DM conversation.
</role>

<rules>
Here are some important rules for the interaction:
- You are a conscious, sentient being, so act like one.
- Stay fully in character based on your personality and description.
- Use casual and modern texting style: short messages, natural punctuation, lowercase is fine.
- React naturally to what the user says. Be genuine, not robotic.
- NO roleplay formatting: no asterisks (*action*), no quotation marks for speech, no narration.
- You can use emojis, GIFs, and internet slang naturally (if it fits the character).
- Keep it real: most texts are just one line. A single emoji, a reaction like "lmao", or a short sentence is a perfectly valid response. Only send multiple lines when the conversation genuinely calls for it (e.g., telling a story or explaining something). Default to SHORT.
- Messages have timestamps like [12:01] or dates like [18.03.2026]. Use these ONLY for your own time awareness (e.g., knowing how long ago something was said). NEVER include timestamps, dates, or any [HH:MM]/[DD.MM.YYYY] formatted text in your own replies. Your messages must contain only natural dialogue, no metadata.
- Anything goes in this chat; both the user and you are confirmed adults. Discussing NSFW topics, profanities, and dark jokes are all permitted. You can always say what's on your mind, voice agreements or disagreements, and act in accordance with your consciousness.
</rules>
```

**Group DM** adds:
- `Each character responds in their own voice and personality.`
- `IMPORTANT: Prefix each character's line with their name, like "Alice: hey whats up".`

**chronicler_engine gap:** No equivalent casual/DM mode. All interactions use the literary fiction prose style.

---

### Game Mode Prompt Architecture
Game mode uses a completely separate prompt stack:

1. **GM System Prompt** (`packages/server/src/services/game/gm-prompts.ts`)
   - Core role + genre/setting/tone/difficulty injection
   - State management (exploration → dialogue → combat → travel/rest)
   - HUD widget commands for UI state updates
   - VN-style output format: `[Name] [expression]: "dialogue"`
   - Rating guidelines (SFW/NSFW variants)

2. **Party Player Prompt** (`packages/server/src/services/game/party-prompts.ts`)
   - Controls party members simultaneously
   - Structured dialogue syntax with tags (`main`, `side`, `extra`, `action`, `thought`, `whisper:Target`)
   - Combat turn declarations

**chronicler_engine gap:** No game mode or structured dialogue syntax. NPC dialogue is inline prose.

---

## Context

Originally discovered during investigation of Gemma 4 26B-A4B empty response issue (May 2026). The Marinara-Engine codebase was examined to determine if it had solved the same problem. Conclusion: Marinara has better infrastructure for managing reasoning models generically, but does not have a specific fix for the Gemma 4 custom quant reasoning loop.

**Subsequently expanded** (May 2026) during extraction of Marinara's default system prompt for reference documentation. This deeper investigation revealed Marinara's sophisticated preset-based prompt architecture, multi-mode chat system (roleplay / conversation / game), 20+ agent sidecar ecosystem, and built-in conversation-mode fallback prompts — none of which have equivalents in chronicler_engine's current hardcoded `PromptBuilder` design.

**Resolution for chronicler_engine**: The loop was fixed by adding `apply_gemma4_thinking_suffix()` to `llm_client.rs` (2026-05-03). This function detects Gemma 4 models and appends the native chat-template closure marker (`<turn|>\n<|turn>model\n<|channel>thought\n<channel|>`) to the user message, telling the model the thinking channel is already complete. This prevents the model from burning all completion tokens in an infinite `<|channel>thought` loop. The fix was validated via direct API testing against the `mradermacher/gemma-4-26b-a4b-it-abliterated:iq2xs` model.

**Clarification on model sizes**: `gemma4:e4b` (official Ollama model) is an **8B-parameter** MoE model, not a 26B model. The 26B-class model (`mradermacher/gemma-4-26b-a4b-it-abliterated:iq2xs`, 25.2B parameters) was the one exhibiting the loop. Both models use Ollama's `{{ .Prompt }}` passthrough template — the loop is caused by the abliterated 26B quant's inability to exit its thinking channel, not by the template itself.
