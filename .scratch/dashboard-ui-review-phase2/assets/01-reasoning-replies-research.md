# Reasoning-as-reply: how SillyTavern and Marinara Engine handle `content: null` with `reasoning` present

**Status:** research brief for the decision "should Chronicler Engine treat reasoning text as the answer or as a failure?"
**Date:** 2026-10-10
**Method:** shallow clones read directly (read-only), plus primary provider docs.
**Repos inspected**

| Project | Commit | Clone |
| --- | --- | --- |
| [SillyTavern](https://github.com/SillyTavern/SillyTavern) | `06bde939fb1e9c4c8d8641d810f0a916b5bce127` (2026-09-14) | `tmp/research-repos/st` |
| [Marinara Engine](https://github.com/Pasta-Devs/Marinara-Engine) | `c56501495a8baf76c258e76534c8728a9b7859bb` (2026-10-07) | `tmp/research-repos/marinara` |

Every file/line reference below is from those two revisions. Epistemic labels: **[known]** = read directly in the cited source; **[inferred]** = reasoned from cited source but not stated; **[guessed]** = not verified.

---

## 1. Direct answer

**Neither project ever shows reasoning text to the user as the assistant's reply. [known]**

- **SillyTavern** extracts the reply only from `delta.content` / `message.content` / `choices[0].text`, with an implicit `?? ''`. Reasoning is routed into a *separate* `state.reasoning` accumulator and rendered in a **collapsible `<details class="mes_reasoning_details">` "thinking" block** attached to the message, never into `message.mes`. [known]
- **Marinara Engine** does the same, and goes further: when the final visible text is empty it **refuses to save the message** and replaces it with a *diagnostic error sentence* derived from `finish_reason` and `usage.completion_tokens_details.reasoning_tokens` — e.g. "The model used its whole output budget (2048 of 2048 output tokens, 2041 of them reasoning) before writing any visible text. Raise Max Tokens or lower Reasoning Effort, then try again." [known]
- The only place either project promotes reasoning to the answer is **Marinara's local llama.cpp sidecar** (`formatStreamOutput`: `content.trim() || reasoning.trim()`), used for scene analysis / tracker prompts, not for the narrative reply. [known]
- **SillyTavern actively suppresses the problem at the source** for OpenRouter: it sends `reasoning: { exclude: !includeReasoning }`, so a user who has "Request model reasoning" off never receives the reasoning field at all. [known]
- **The two projects diverge on `finish_reason`.** SillyTavern never reads it (it appears nowhere in its production code — only in test mocks). Marinara does, and uses `finish_reason === "length"` plus reasoning-token usage as the primary signal that a reasoning-only reply is *budget exhaustion*, not an answer. [known]

Primary-source confirmation: OpenRouter documents that the `content: null` + `reasoning` shape is **exactly** the `max_tokens` exhaustion case, and gives the detection formula. There is **no documented case of a model returning its final answer only in `reasoning`**. [known]

---

## 2. SillyTavern

### 2.1 Where content is extracted (streaming)

`public/scripts/openai.js:3222` — `getStreamingReply(data, state, { chatCompletionSource, overrideShowThoughts })`. This is the single reply extractor for every chat-completion source; `public/scripts/custom-request.js:518` reuses it for custom endpoints.

| Source | Reasoning goes to | Reply returned |
| --- | --- | --- |
| `CLAUDE` (`:3226-3231`) | `state.reasoning += data?.delta?.thinking` | `data?.delta?.text \|\| ''` (`:3230`) |
| `MAKERSUITE`/`VERTEXAI` (`:3232-3247`) | `parts.filter(x => x.thought)` → `state.reasoning` (`:3236-3238`) | `parts.filter(x => !x.thought).map(x => x.text)` (`:3247`) |
| `COHERE` (`:3248-3249`) | — | `delta.message.content.text \|\| delta.message.tool_plan` |
| `DEEPSEEK` (`:3250-3253`) | `delta.reasoning_content` → `state.reasoning` (`:3251`) | `data.choices?.[0]?.delta?.content \|\| ''` (`:3253`) |
| `XAI` (`:3254-3258`) | `delta.reasoning_content` → `state.reasoning` (`:3256`) | `delta.content \|\| ''` (`:3258`) |
| `OPENROUTER` (`:3259-3288`) | `delta.reasoning` **??** `delta.reasoning_content` **??** `message.reasoning` **??** `message.reasoning_content` (`:3265-3270`), plus `reasoning_details` harvested for encrypted signatures (`:3272-3287`) | `delta.content ?? message.content ?? choices[0].text ?? ''` (`:3288`) |
| `CUSTOM`, `ZAI`, `AIMLAPI`, `NANOGPT`, `MOONSHOT`, `POLLINATIONS`, `SILICONFLOW`, `CHUTES`, `ELECTRONHUB`, `WORKERS_AI`, `FIREWORKS`, `COMETAPI` (`:3289-3296`) | `delta.reasoning_content ?? delta.reasoning` (`:3291-3293`) | `delta.content ?? message.content ?? choices[0].text ?? ''` (`:3295`) |
| `MISTRALAI` (`:3297-3303`) | `delta.content[0].thinking[0].text` → `state.reasoning` (`:3299`) | content block array → text parts (`:3301-3302`) |
| default (`:3304-3305`) | — | `delta.content ?? message.content ?? choices[0].text ?? ''` |

**Never once does a reasoning field appear on the right-hand side of the reply expression.** [known] And the accumulator is nested under `if (show_thoughts)` — with the setting off, reasoning is not even retained. [known]

The `??` (not `||`) is deliberate: a `content: ""` chunk returns `''` and does **not** fall through to `reasoning`. [known]

### 2.2 Where content is extracted (non-streaming)

`public/script.js:6276-6300` — `extractMessageFromData`. The `'openai'` branch (`:6292`):

```js
return data?.content?.filter(p => p.type === 'text')?.map(p => p.text)?.join('\n\n')
    ?? data?.choices?.[0]?.message?.content
    ?? data?.choices?.[0]?.text ?? data?.text
    ?? data?.message?.content?.[0]?.text
    ?? data?.message?.tool_plan ?? '';
```

`refusal` and `tool_plan` are fallbacks; **reasoning is not** — not even for DeepSeek or Z-AI. [known]

Non-streaming reasoning is extracted separately in `public/scripts/reasoning.js:112-170` — `extractReasoningFromData`, which has explicit per-source branches (`DEEPSEEK`/`XAI` → `message.reasoning_content`; `OPENROUTER` → `message.reasoning ?? message.reasoning_content`; `ZAI`, `CUSTOM`, … → `message.reasoning_content ?? message.reasoning`; `MAKERSUITE`/`VERTEXAI` → `thought` parts; `CLAUDE` → `type === 'thinking'` parts; `MISTRALAI` → `content[0].thinking[]`). [known]

### 2.3 What happens when content is empty and only reasoning exists

Three distinct behaviours, all "reasoning is not the reply":

1. **Streaming, normal chat turn** — `text` stays `''`. The message is created with `mes = ''` and `extra.reasoning = <reasoning>`. The collapsible thinking block renders (`public/index.html:7442`, `public/style.css:435-521`), and `public/style.css:1222` exists specifically to give that block bottom margin when `.mes_text:empty` — i.e. an *empty visible message with a visible thinking block* is an expected, styled state. [known]
2. **Streaming/non-streaming with tool calls** — the empty bubble is **kept** precisely *because* reasoning exists: `public/script.js:5413` and `public/script.js:5543`
   ```js
   const shouldDeleteMessage = type !== 'swipe' && ['', '...'].includes(getMessage) && !reasoning;
   ```
   and `public/script.js:3608` `const shouldHide = ['', '...'].includes(this.result) && !this.reasoningHandler.reasoning;`. So reasoning presence is the *reason an empty message survives* so the thought block can be inspected. [known]
3. **`generateRaw` (quiet/agent-style generations)** — empty extracted text is a hard failure: `public/script.js:4147` `throw new Error('No message generated');` regardless of reasoning. [known]

Also `public/scripts/reasoning.js:1569-1572`: the auto-parse event handler explicitly **bails out** when the message body is empty:
```js
if (!message.mes || message.mes === '...') {
    console.debug('[Reasoning] Message content is empty or a placeholder', idx);
    return null;
}
```
[known]

### 2.4 Per-model / per-provider special rules

- **OpenRouter reasoning exclusion (the big one).** `src/endpoints/backends/chat-completions.js:2306-2314`:
  ```js
  const includeReasoning = Boolean(request.body.include_reasoning);
  bodyParams = { transforms: …, plugins: …,
      reasoning: { exclude: !includeReasoning } };
  ```
  fed from `public/scripts/openai.js:2820` `'include_reasoning': Boolean(settings.show_thoughts)`. With "Request model reasoning" off, OpenRouter is told not to return reasoning at all, so the whole `content: null` + `reasoning` situation cannot arise from a reasoning-only budget. [known]
- **DeepSeek route**: `chat-completions.js:1137` `'thinking': { type: request.body.include_reasoning ? 'enabled' : 'disabled' }`; the endpoint handler `sendDeepSeekRequest` (`:1071-1175`) merely forwards the SSE stream (`forwardFetchResponse`, `:1152`) — parsing is client-side. [known]
- **Z-AI route**: `chat-completions.js:2551-2562` `bodyParams = { thinking: { type: request.body.include_reasoning ? 'enabled' : 'disabled' } }`. [known]
- **Setting semantics are documented in the UI itself** (`public/index.html:2142-2156`): label "Request model reasoning" / "Allows the model to return its thinking process", with two conditional notes — for `deepseek,zai,moonshot`: *"This setting enables or disables reasoning generation by the model."* and for everything else: *"This setting affects visibility only."* [known]
- **Reasoning-effort side rules** — `openai.js:2619-2621`: when `show_thoughts` is off and effort is "min" on OpenRouter, ST sends `effort: 'none'`; `openai.js:1730-1732` gates `reasoning_effort` to `OPENAI_REASONING_EFFORT_MODELS` only. [known]
- **Reasoning templates replace the old hardcoded think-tag regex.** `default/content/presets/reasoning/` ships `Think XML` (`<think>` / `</think>`, default), `DeepSeek`, `Gemma 4` (`<|channel>thought` / `<channel|>`), `OpenAI Harmony` (`<|start|>assistant<|channel|>analysis<|message|>` … `final`), `Blank`. `public/scripts/reasoning.js:1461-1490` `parseReasoningFromString` builds `^\s*?<prefix>(.*?)<suffix>` and **returns `{ reasoning, content }` with the block removed from `content`**; `:1428-1438` `removeReasoningFromString` applies it. Defaults are conservative: `public/scripts/power-user.js:274-283` — `auto_parse: false`, `add_to_prompts: false`, `max_additions: 1`. [known]
- **Reasoning prefix / "continue on reasoning"** — `public/scripts/reasoning.js:762-772` in `PromptReasoning.addToMessage`: when `isPrefix && !content` (i.e. the last assistant message has *no visible text, only reasoning*), ST writes the reasoning prefix **without** its closing suffix (`prefixIncomplete = true`) so the model resumes inside the think block, and `getLatestPrefix()` (`:687-696`) hands that prefix to the auto-parser on the next MESSAGE_RECEIVED. This is the closest either project comes to treating "empty content + reasoning" as a first-class state — but it is a *prompt-side continuation*, never a user-visible reply. [known]
- **Ollama** is a `textgenerationwebui` source, not a chat-completion source. `src/endpoints/backends/text-completions.js:29-65` `parseOllamaStream` reads `json.response` and `json.thinking` into **separate** fields of the forwarded chunk (`{ choices: [{ text, thinking }] }`); `public/scripts/textgen-settings.js:1337` accumulates `choices[0].reasoning ?? choices[0].thinking` into `state.reasoning` while `text` comes only from `choices[0].text ?? data.content` (`:1330-1337`). `extractReasoningFromData` handles `textgen_types.OLLAMA` via `data?.thinking` (`reasoning.js:121-124`). [known]
- **Gemini/MakerSuite** keeps thought parts (`part.thought`) out of the text parts, and captures `thoughtSignature` on text parts only (`openai.js:3272-3287`, `reasoning.js:179-212`). [known]

### 2.5 Does SillyTavern read `finish_reason`?

**No.** `rg finish_reason` over the entire repo (excluding `node_modules`) matches only `tests/util/mock-server.js:34` and `tests/mock-server.test.js:29` — test fixtures. No production code path inspects it. [known]

Consequence **[inferred]**: SillyTavern cannot distinguish "reasoning-only because truncated" from "reasoning-only for some other reason"; it simply renders an empty message with a thinking block and lets the user retry.

---

## 3. Marinara Engine

### 3.1 Where content is extracted

`packages/server/src/services/llm/providers/openai.provider.ts` is the single OpenAI-compatible provider used for `openai`, `openrouter`, `nanogpt`, `xai`, `mistral`, `cohere`, `arli`, `zai`, `custom`, `openai-chatgpt`, `local-sidecar` (`:117-128`).

**Streaming** (`:1494-1512`): `reasoning = extractReasoning(delta ?? message)` → `options.onThinking(reasoning)`; then
```js
const content = delta?.content ?? message?.content;
const blocks = OpenAIProvider.extractContentBlocks(content);
if (blocks) { … if (blocks.text) yield blocks.text; }
else if (typeof content === "string" && content) { yield content; }
else if (refusal) { yield refusal; }
```
Reasoning is never yielded as content. Note the generator yields only when content is truthy, so a `content: null` chunk yields **nothing**. [known]

**Non-streaming** (`:1708-1717`): `resolvedContent = blocks ? (blocks.text || null) : (choice.message.content ?? null)`, with a **refusal** fallback (`:1719-1721`) and **not** a reasoning fallback. `finishReason` is captured from `choice.finish_reason` (`:1725`). [known]

`extractReasoning` (`:353-375`) recognises `reasoning_content` (DeepSeek), `reasoning` (OpenRouter/NanoGPT), and `reasoning_details[]` with `type: "reasoning.text"` → `text` / `type: "reasoning.summary"` → `summary`. [known]

`extractOpenAICompatibleContentBlocks` (`:66-97`) handles OpenRouter's Anthropic-style block array, splitting `type: "thinking"` blocks from `type: "text"` blocks. **If the array contains only thinking blocks, `text` is `""` and `resolvedContent` becomes `null`** (`:1715`) — i.e. Marinara also produces the empty-content + reasoning state, and still treats it as empty. [known]

### 3.2 Empty content + reasoning = a diagnostic failure, not a reply

`packages/server/src/services/generation/empty-response-reason.ts` (whole file, 78 lines) — the doc comment is explicit [known]:

> Reasoning models spend one output budget on thinking and on text. When the budget runs out mid-thought the reply is empty, and the provider says so: finish_reason "length", completion tokens at the cap, nearly all of them reasoning. Z.AI also reports "sensitive" (content policy) and "model_context_window_exceeded" (docs.z.ai chat-completion reference); any other reason is quoted verbatim rather than hidden.

`describeEmptyModelResponse(context)` takes `{ finishReason, usage: { completionTokens, completionReasoningTokens }, maxTokens, hadThinking }` and returns:

- `finish_reason === "sensitive"` → **"The provider stopped the reply for content policy (finish reason \"sensitive\") and returned no text."** (`:50-52`)
- `finish_reason === "model_context_window_exceeded"` → context-window advice (`:53-55`)
- `budgetSpent` (`finish_reason === "length"` **or** `hadThinking && completionTokens >= maxTokens`) → **"The model used its whole output budget (N of M output tokens, K of them reasoning) before writing any visible text. Raise Max Tokens or lower Reasoning Effort, then try again."** (`:45-62`)
- else `hadThinking` → **"The model finished reasoning (K reasoning tokens, finish reason \"X\") but returned no visible text. No output-limit exhaustion was reported. Retry, and inspect the debug response if this repeats; changing the thinking display does not change the model request."** (`:63-70`)
- else `finish_reason` present → generic empty-with-reason (`:71-73`); else the generic constant (`:74-77`)

The call site is the main generation route, `packages/server/src/routes/generate.routes.ts:9668-9700`:

```js
// Guard: don't save empty responses — the model returned nothing useful.
if (!fullResponse.trim() && !roleplayActivity.length && !currentRoleplayMedia.length) {
    const emptyResponseMessage = gmVerbRefusals.length ? … : describeEmptyModelResponse({
        finishReason,
        usage,
        maxTokens: sentOutputBudget(effectiveMaxTokensForSend, conn.maxTokensOverride),
        hadThinking: providerThinking.trim().length > 0 || fullThinking.trim().length > 0,
    });
    logger.warn({ … providerThinkingLength, fullThinkingLength, finishReason, completionReasoningTokens … },
        "[generate] Empty response after post-processing");
```

`packages/server/src/routes/connections.routes.ts:1833-1841` uses the same function for the **connection "Send" test** (`hadThinking: (usage?.completionReasoningTokens ?? 0) > 0`), and `packages/server/src/services/advanced-memory.ts:1096-1101` for the summary model. [known]

So Marinara's answer to "content empty, reasoning present" is: **surface a precise, actionable error; never promote the reasoning.** [known]

`packages/server/src/services/agents/agent-executor.ts:3633` and `:3705` additionally strip leading think blocks from agent JSON responses before parsing (`extractLeadingThinkingBlocks(text).content`), with comments naming the failure mode: *"with reasoning_format \"none\" a local runtime leaves thinking inline in content, and a fenced block inside the thinking region would win the fence regex and poison every downstream heuristic (#5537)"* and *"A reasoning level chosen on the connection can make a local endpoint answer with its thinking inline (#7131)."* [known]

### 3.3 Inline reasoning is lifted into a separate `extra.thinking` channel

`packages/server/src/routes/generate.routes.ts:8703-8712`:

```js
// Some models inline reasoning blocks instead of using provider-native
// thinking channels. Lift those blocks into message.extra.thinking.
const inlineThinking = extractLeadingThinkingBlocks(fullResponse, customThinkingTags);
if (inlineThinking.stripped) {
    if (inlineThinking.thinking) fullThinking = fullThinking ? fullThinking + "\n\n" + inlineThinking.thinking : inlineThinking.thinking;
    fullResponse = inlineThinking.content;
    contentReplaced = true;
}
```

`packages/shared/src/utils/thinking-tags.ts:28-36` defines the recognised pairs (a superset of SillyTavern's presets): `<thinking>`, `<think>`, `<thought>`, `<|think|>`, `<|channel>thought`, `[thinking]`, `[think]`, `[thought]`, plus up to 20 user-defined pairs. `extractLeadingThinkingBlocks` (`:114-141`) only strips **leading** blocks; `createInlineThinkingStreamFilter` (`:147-201`) does the same during streaming so raw markers never flash, used at `packages/client/src/hooks/use-generate.ts:1600-1609` and reset at `:2411`, `:2539`, `:2592`, `:2681`. [known]

Reasoning is rendered in a separate collapsible panel on the client: `packages/client/src/components/chat/ChatMessage.tsx:1990` (`showThinking` state), `:2335-2337` (inline-thinking option for roleplay), `:4211` and `:4701` (thinking panel); `packages/client/src/components/chat/ConversationMessage.tsx:982`. [known]

### 3.4 The one place reasoning *is* promoted to the answer

`packages/server/src/services/sidecar/sidecar-inference.service.ts:105-107`:

```js
function formatStreamOutput(content: string, reasoning: string): string {
  return content.trim() || reasoning.trim();
}
```

Used by `streamChatCompletion` (`:227`), whose only callers are `analyzeScene` (`:653`) and `runTrackerPrompt` (`:680`) — the **local llama.cpp sidecar** on `127.0.0.1`, not the main narrative reply. [known]
And `:456-462`, the sidecar connection test:
```js
const content = extractContentText(message?.content).trim();
const reasoning = extractContentText(message?.reasoning_content).trim();
const output = content || reasoning;
if (!output) throw new Error("The local sidecar test returned an empty response.");
```
[known]

**Inferred:** these two sites are deliberate — for a local sidecar whose tiny model sometimes answers in the thinking channel, JSON/prose extraction from either channel is more useful than failing, and there is no user-facing story text at stake. This is *not* a precedent for the narrative path.

### 3.5 Explicit regression proof that reasoning is never promoted

`scripts/regressions/deepseek-reasoning-output.regression.ts` drives `OpenAIProvider` against a synthetic DeepSeek-shaped server across three wire shapes (`separate`, `combined`, `non-stream`) × `captureThinking` on/off × visible text `"The experiment is ready."` **and `""`**, asserting at `:83`:

```js
assert.equal(chatText, text, "capturing reasoning must never discard or promote the visible answer");
```

with `:57` `assert.equal(complete.content ?? "", text)` and the fixture at `:19`/`:28` returning exactly the incident shape (`message: { content: null, reasoning_content: … }`). [known]

### 3.6 Per-model special rules (requests)

`openai.provider.ts:882-1000` (`applyChatCompletionsReasoning`) and `packages/server/src/services/llm/providers/glm-request-compat.ts`:

- Claude-strict models → `body.reasoning = { effort }` (OpenRouter endpoint) else `body.reasoning_effort` (`:882-890`)
- GPT-6 always-reasoning models → forced `low` (`:891-895`)
- xAI configurable reasoning → `reasoning_effort` mapped by model (`:896-905`)
- GLM → `applyGlmThinkingParameters`; GLM 5.3 mandatory-reasoning models get `thinking.type: "enabled"` with effort mapped to `low`/`high`/`max` and a reasoning-off request becomes the lightest level instead of a rejected disable (`CHANGELOG.md:1263`, `:2501`; also `:1294`, `:1287`, `:173`, `:442`)
- Custom Gemini/OpenRouter (`:965-979`) → merges into `body.reasoning = { …existing, effort }`; `effort: "none"` when disabled and supported
- NanoGPT Kimi K3 → `low` instead of `none` (`:952-959`)
- Local sidecar reasoning-off → `body.reasoning_format = "none"` + `chat_template_kwargs.enable_thinking = false` (`:928-933`)
- `:997-1004` `normalizeStrictClaudeRequest` rewrites `effort: "none"` → `"low"` rather than deleting reasoning
- Model lists/regexes live in `packages/shared/src/constants/model-lists.ts` and `generation-parameter-relevance.ts` (`:219-298`), including the explicit note that for `provider !== "openrouter"` reasoning effort is hidden, and per-model `glm` / `grok` / `gemini-3` gating. [known]

**Notable difference from SillyTavern:** Marinara **does not** send OpenRouter `reasoning.exclude`. It sends `reasoning: { effort }` (or `effort: "none"`), i.e. it keeps reasoning visible and handles the empty case with the diagnostic error instead of preventing it. [known]

Prefill: `packages/server/src/routes/generate/generate-route-utils.ts:1159` sends `reasoning_content: options.assistantReasoningPrefill.trimEnd()` on the final assistant message, and `packages/shared/src/types/prompt.ts:248` documents it as *"Optional reasoning_content prefill on the final assistant message."* `CHANGELOG.md:1530`: *"Added Assistant Reasoning Prefill alongside the existing visible assistant prefill: compatible OpenAI-style endpoints can now continue hidden reasoning from reasoning_content on a partial assistant message."* — the direct analogue of SillyTavern's `PromptReasoning` prefix. [known]

**Historical direction of travel (relevant to the decision):** every CHANGELOG entry touching reasoning-only output *tightens* rather than loosens:
- `:1029` — "Added Z.AI text connections with GLM 5.3 model metadata, supported reasoning levels, and **useful explanations when reasoning exhausts the output budget** (#5963, #5968)."
- `:173` — "…Before, they kept reasoning anyway and could spend the whole reply budget on it, **ending with a message to raise Max Tokens** (#6961)."
- `:1274` — "Professor Mari's plans and questions no longer vanish into hidden reasoning on local custom connections (#5721): … so the model **answers in the visible reply instead of burying its brainstorming in the reasoning channel**." (fix = disable hidden reasoning on local endpoints so the answer lands in `content`)
- `:2510` — turn-game narration leaking chain-of-thought: "**narration now strips inline reasoning** like the main generation pipeline, **falls back to the factual event line when the output was all reasoning**" (#3427)
- `:1886` — "Kept the reasoning action visible when a provider reports hidden reasoning-token usage but omits the displayable summary, and **explained the missing summary** in the Model Thoughts panel."
- `:1104`, `:1187` — storyboard planning "**reports empty final answers or exhausted output limits** when planning still fails" / "retry unusable local planner output once without reasoning"
[all known]

Net: **Marinara never treats reasoning-only as success anywhere in a user-visible reply path; it converts it into a tailored error or a factual fallback.** [known]

### 3.7 Does Marinara read `finish_reason`?

**Yes.** `openai.provider.ts:1490` (`if (choice0?.finish_reason) finishReason = choice0.finish_reason;`), `:1725` (`finishReason: toolCalls.length > 0 ? "tool_calls" : (choice?.finish_reason ?? "stop")`), `:1929` (streaming return), returned in the usage object alongside `completionReasoningTokens` from `usage.completion_tokens_details.reasoning_tokens` (`:1174`, Responses API `:2969`). Consumed at `generate.routes.ts:9674-9677`, `connections.routes.ts:1836-1840`, `advanced-memory.ts:1096-1101`, and logged at `generate.routes.ts:9691`. [known]

---

## 4. Primary-source provider evidence

### 4.1 OpenRouter — `content: null` is the documented `max_tokens` exhaustion case

[openrouter.ai/docs/use-cases/reasoning-tokens](https://openrouter.ai/docs/use-cases/reasoning-tokens), section **"Reasoning tokens and max_tokens"** [known]:

> **Reasoning tokens count against `max_tokens`**
> On most providers, the request's `max_tokens` limit (or `max_completion_tokens`, which shares the same budget) applies to reasoning and visible output combined. **If the limit is small enough that the model spends all of it reasoning, the response returns `finish_reason: "length"` with an empty `content`, and the reasoning tokens are still billed.** With `reasoning.exclude: true` the `reasoning` field is omitted as well, so the response carries no text at all and the only signal left is `finish_reason` plus `usage`.

> **To detect this case, subtract `usage.completion_tokens_details.reasoning_tokens` from `usage.completion_tokens`.** The difference is the number of visible output tokens, and it is 0 or near 0 when reasoning consumed the budget. For example, a request with `max_tokens: 300` that returns `completion_tokens: 302` and `reasoning_tokens: 301` produced a single visible token.

This is exactly the incident (`finish_reason: "length"`, content null, 8771 chars of reasoning). [known]

Also from the same page [known]:
- *"Reasoning tokens will appear in the `reasoning` field of each message, unless you decide to exclude them."*
- *"You can also use `reasoning_content` as an alias — it functions identically to `reasoning`."*
- Legacy: `include_reasoning: true` ≡ `reasoning: {}`; `include_reasoning: false` ≡ `reasoning: { exclude: true }`.
- `reasoning_details[]` types: `reasoning.text` (`text`, `signature`), `reasoning.summary` (`summary`), `reasoning.encrypted` (`data`, may stream as `[REDACTED]`). Non-streaming: `choices[].message.reasoning_details`; streaming: `choices[].delta.reasoning_details`.
- Anthropic: *"`max_tokens` must be strictly higher than the reasoning budget to ensure there are tokens available for the final response after thinking."*

**No documented case anywhere on that page (or elsewhere I found) where a model returns its final answer only in `reasoning`.** The only two documented `content`-empty paths are (a) max_tokens exhausted by reasoning, and (b) `reasoning.exclude: true` where reasoning is also absent — which cannot produce the incident shape. [known for (a)/(b) as documented; **inferred** that no other path exists, based on absence of documentation + both projects' code assuming the same]

### 4.2 Ollama — reasoning and answer are separate fields by contract

[docs.ollama.com/capabilities/thinking](https://docs.ollama.com/capabilities/thinking) [known]:

> Thinking-capable models emit a `thinking` field that separates their reasoning trace from the final answer.
> **The reasoning output and answer use separate fields. Chat returns `message.thinking` and `message.content`. Generate returns `thinking` and `response`.**

### 4.3 Z.AI / GLM — `reasoning_content` is a peer of `content`, and `finish_reason` has "sensitive"

[docs.z.ai/guides/capabilities/thinking](https://docs.z.ai/guides/capabilities/thinking) [known]: `thinking.type` = `enabled`/`disabled`; `reasoning_effort` = `max`/`high`/`low` (GLM-5.2+). **GLM-5.3 and GLM-5.3-FLASH no longer support disabling thinking** (an error occurs if `thinking.type: "disabled"` is sent) — so on those models reasoning is mandatory and cannot be suppressed at the request level.

The documented response example populates **both** `message.content` and `message.reasoning_content`; the documented client pattern prints both, and the doc's own streaming sample switches to `delta.content` only once `delta.content` appears:
```
# Process thinking process (if any)
if hasattr(delta, 'reasoning_content') and delta.reasoning_content: …
# Process answer content
if hasattr(delta, 'content') and delta.content: …
```
[docs.z.ai/api-reference/llm/chat-completion](https://docs.z.ai/api-reference/llm/chat-completion) [known] lists `finish_reason` as `stop`, `tool_calls`, `length`, `sensitive`, `model_context_window_exceeded`, `network_error` — which confirms the strings Marinara's `describeEmptyModelResponse` special-cases. `reasoning_content` is documented as *"For thinking mode only. The reasoning contents of the assistant message, before the final answer."*

Z.AI also documents **Preserved Thinking**: `clear_thinking` (default `true`) controls whether prior-turn `reasoning_content` is cleared; setting it `false` requires forwarding historical reasoning *"full, unmodified, and correctly ordered."*

### 4.4 DeepSeek — `content` is `''` on tool-calling turns; `reasoning_content` precedes the final answer

[api-docs.deepseek.com/guides/thinking_mode](https://api-docs.deepseek.com/guides/thinking_mode) [known]:

> Thinking mode … before outputting the final answer, the model will first output a chain-of-thought reasoning to improve the accuracy of the final response.
> In thinking mode, the chain-of-thought content is returned via the `reasoning_content` parameter, **at the same level as `content`**.
> In each turn of the conversation, the model outputs the CoT (`reasoning_content`) and the final answer (`content`).
> If the request **carries the `tools` parameter**: the `reasoning_content` of all previous turns should be passed back… If your code does not correctly pass back `reasoning_content`, the API will return a 400 error.

The doc's own sample output shows the documented **empty-content-with-reasoning** state, and it is always a tool-calling turn: `reasoning_content="…get tomorrow's date first…" content='' tool_calls=[…]` (Turn 1.2, Turn 2.1), while the final answer turn has both populated (Turn 1.3). [known]

[api-docs.deepseek.com/api/create-chat-completion](https://api-docs.deepseek.com/api/create-chat-completion) [known]: `reasoning_content` is a **nullable** field on the assistant message — *"For thinking mode only. The reasoning contents of the assistant message, before the final answer."*; also usable as the Beta Chat Prefix Completion CoT input; `usage.completion_tokens_details.reasoning_tokens` is exposed. Same shape on `delta.reasoning_content` for streaming.

**Inferred:** for DeepSeek the only documented `content == ''` + `reasoning_content != ''` case is *intermediate tool-calling turns*, which our engine (no tool calls) cannot reach — meaning for DeepSeek-shaped traffic an empty content is either truncation or a genuine model fault.

### 4.5 Third-party corroboration of the "GLM puts the answer in reasoning" claim

The claim behind commit `1d4aeace` for Z-AI/GLM is reproducible as a *client bug*, not a provider contract. From the search results:
- [NousResearch/hermes-agent#16533](https://github.com/NousResearch/hermes-agent/issues/16533) — "Z.AI / GLM via `zai` provider never returns `reasoning_content` — Hermes sends `extra_body.reasoning` (OpenRouter-style) but Z.AI expects `extra_body.thinking={"type":"enabled"}`" (a request-shape bug: reasoning never returned at all).
- [code-yeongyu/oh-my-openagent#980](https://github.com/code-yeongyu/oh-my-openagent/issues/980) — "Z.ai GLM models return empty responses … Z.ai's GLM models default to 'thinking mode' which outputs to `reasoning_content` instead of `content`. oh-my-opencode expects standard `content` responses. **Direct API test WITHOUT thinking parameter (FAILS):** Returns `{"content":"", "reasoning_content":"..."}`" — i.e. a client that did not read the reasoning field *reported* empty responses. [third-party report — **inferred** reliability; not a Z.AI doc statement]
- The Z.AI docs themselves never show a response with empty `content` and non-empty `reasoning_content`.

**Inferred verdict:** the "GLM returns null content and puts the response in reasoning" story is most consistent with (a) clients that ignored `reasoning_content` entirely, or (b) GLM-5.3-class mandatory-reasoning models spending a small `max_tokens` budget before emitting content. Neither is a documented "final answer lives only in reasoning" contract. I could not find **any** first-party doc from OpenRouter, Z.AI, DeepSeek, or Ollama stating that the final answer may appear only in a reasoning field. [**inferred**, marked as a material gap]

### 4.6 Material gaps

- I could not reach Z.AI's community forum or GitHub for a first-party statement about `content: null` with non-empty `reasoning_content` outside the max_tokens case.
- SillyTavern's release notes are not in-repo (no `CHANGELOG.md` at this revision), so I cannot cite a first-party ST statement of intent; the code is the evidence.
- I did not exercise live API calls (read-only research, no credentials), so the truncation explanation is documented-and-corroborated, not independently reproduced.

---

## 5. Epistemic summary

| Claim | Status |
| --- | --- |
| SillyTavern never returns reasoning as the reply; `getStreamingReply` returns only `content`/`text` | **known** (`public/scripts/openai.js:3222-3306`) |
| SillyTavern keeps reasoning in a separate `state.reasoning` / `extra.reasoning` + collapsible block | **known** (`reasoning.js:112-170`, `index.html:7442`, `script.js:6682`, `style.css:435-521`) |
| SillyTavern sends OpenRouter `reasoning: { exclude: true }` when thoughts are off | **known** (`chat-completions.js:2306-2314`, `openai.js:2820`) |
| SillyTavern does not read `finish_reason` anywhere in production code | **known** (rg over whole repo; only test mocks) |
| SillyTavern's per-source reasoning field names (DeepSeek `reasoning_content`, OpenRouter `reasoning`/`reasoning_content`, Ollama `thinking`, Gemini `thought` parts, Mistral `content[].thinking`) | **known** (`reasoning.js:112-170` incl. `121-124` for textgen OpenRouter/Ollama, `openai.js:3249-3303`, `text-completions.js:29-65`) |
| SillyTavern strips *leading* think-tag blocks into `extra.reasoning` via templates, default `auto_parse: false` | **known** (`reasoning.js:1461-1490`, `power-user.js:274-283`, `default/content/presets/reasoning/`) |
| Marinara treats empty content as a failure with a `finish_reason`/reasoning-token-aware message | **known** (`empty-response-reason.ts:1-78`, `generate.routes.ts:9668-9700`) |
| Marinara's OpenAI-compatible provider never promotes reasoning to content (stream or non-stream) | **known** (`openai.provider.ts:1494-1512`, `:1708-1731`) + regression `deepseek-reasoning-output.regression.ts:83` |
| Marinara *does* promote reasoning in the local sidecar scene-analysis/tracker path | **known** (`sidecar-inference.service.ts:105-107`, `:456-462`) |
| Marinara reads `finish_reason` and reasoning-token usage | **known** (`openai.provider.ts:1490`, `:1725`, `:1174`) |
| Marinara lifts inline `<think>`-family blocks into `extra.thinking` | **known** (`generate.routes.ts:8703-8712`, `thinking-tags.ts:28-141`) |
| OpenRouter documents `content: null` as the `max_tokens` exhaustion case | **known** (OpenRouter reasoning-tokens docs, "Reasoning tokens and max_tokens") |
| No provider documents "final answer only in reasoning" | **inferred** (absence of evidence; see §4.5–4.6) |
| The Z-AI/GLM `1d4aeace` rationale describes a client-side mis-read or a mandatory-reasoning budget case | **inferred** (third-party issues only) |
| A reasoning-first fallback is defensible because "some provider needs it" | **not supported by any evidence found** |

---

## 6. Implications for the rule

### Where Chronicler Engine stands today (read-only observation, for contrast)

- `src/adapters/driven/llm/transport/utils/response.rs:6-27` `extract_content_from_response` returns `content → reasoning → reasoning_content`. **Note [known]:** `message.get("content").and_then(Value::as_str)` matches `Value::String("")`, so the fallback fires **only on JSON `null` or a missing `content` key** — an empty-string content already short-circuits to `Ok("")` and never reaches reasoning. (The "Add `is_non_empty()` helper" wording in commit `1d4aeace` describes the pre-refactor code, not the current tree.)
- Nothing in `src/` reads `finish_reason` or usage/reasoning-token counts. [known]
- `src/application/prompting/sanitize.rs:7-35` already strips leaked `<thought>`, `<|channel>thought…<channel|>`, and `<|turn>` markers — i.e. the engine already has an "inline reasoning leaked into content" defence, but it has no *separate* reasoning channel.

### Options

**Option A — always fail (remove the reasoning fallback).**
Behaviour: `content` null/missing ⇒ `Err(LlmFailure::ParseError)`, matching SillyTavern's `generateRaw` ("No message generated") and Marinara's empty-response guard.
- Pro: consistent with both reference implementations; cannot leak chain-of-thought into story text (the incident class is eliminated by construction); simplest to reason about and test.
- Con: turns the *old* Z-AI/GLM complaint into a hard failure. But per §4.5 that complaint is unsupported, and SillyTavern — which motivated `1d4aeace` — never actually accepted reasoning as a reply; the cited precedent was a misreading. [inferred]
- Risk if wrong: a provider that genuinely only fills `reasoning` becomes unusable rather than degraded.

**Option B — fail only on `finish_reason == "length"` (or reasoning tokens ≥ max_tokens).**
Behaviour: keep the fallback, but reject it when truncation explains it. Requires plumbing `finish_reason` + `usage.completion_tokens_details.reasoning_tokens` out of the transport (currently not parsed).
- Pro: matches OpenRouter's *documented* detection formula almost exactly, and is what Marinara's `budgetSpent` computes; directly addresses the reported incident.
- Con: adds transport/domain surface (a parsed `ChatCompletionMeta`), and the residual case — non-`length` reasoning-only replies — still leaks chain-of-thought. Given no provider documents such a case, that branch is dead weight plus risk.

**Option C — per-model allowlist.**
Behaviour: accept reasoning as the answer only for an explicit list/pattern of models (e.g. GLM variants) and/or only for configured providers.
- Pro: preserves a capability for a model that demonstrably needs it, without a blanket rule; easy to extend.
- Con: **no evidence found that any model on our configured list (openai/gpt-4o-mini, sao10k/l3.3-euryale-70b, mistralai/mistral-nemo, deepseek/deepseek-v4-flash, Ollama gemma variants) needs it.** Building an allowlist with an empty membership is unjustified complexity, and a model-name allowlist is exactly the kind of guess that decays silently. Also: model identity is user-configurable and can collide with namespaced aliases.

**Option D — keep reasoning separate (a real thinking channel).**
Behaviour: never substitute; on empty content, surface an actionable error (the Marinara `describeEmptyModelResponse` shape), and — separately — capture reasoning into a distinct field (log/forensics/UI), like SillyTavern's `extra.reasoning` and Marinara's `extra.thinking`.
- Pro: strictly the union of both projects' actual behaviour; preserves the useful diagnostic detail (`finish_reason`, reasoning chars/tokens) instead of discarding it; makes "why was this reply empty?" answerable from stored forensics; the sanitizer already exists for inline leakage.
- Con: the largest change (needs a reasoning field through transport → domain → forensics/view; a UI decision about where to show it).

### Recommendation (research position, not a scope decision)

**Option D, with Option B's signals folded into the error text — i.e. never promote reasoning, and make the failure diagnostic.**
Reasoning: both reference implementations converge on "reasoning is never the reply", and Marinara's whole reasoning-budget CHANGELOG history is a sequence of moves *away* from letting reasoning stand in for an answer. OpenRouter — the one provider in our stack that produced the incident — documents `content: null` as budget exhaustion and hands us the exact detection formula (`completion_tokens - reasoning_tokens == 0`), so the "accept it" branch is provably answering the wrong question. If a real model ever needs Option C, the evidence will be a first-party doc statement plus a stored forensics record showing repeated reasoning-only successes; neither exists today. [inferred recommendation]

Smallest defensible first step **[inferred]**: drop the `reasoning` / `reasoning_content` fallback from `extract_content_from_response` so `content: null` becomes a `ParseError`, and log `finish_reason` if it is cheap to surface — that alone kills the incident class while leaving the door open for B/C/D later. The cost of not doing D immediately is that the empty-reply failure stays under-explained; the cost of doing nothing is that chain-of-thought keeps reaching players.

---

### Files inspected (evidence index)

**SillyTavern (`06bde939`)**
`public/scripts/openai.js` (3222-3306, 1730-1732, 2619-2621, 2820, 394, 507) · `public/script.js` (3608, 4066, 4147, 5284, 5413, 5543, 6276-6300, 6661-6753) · `public/scripts/reasoning.js` (112-212, 671-785, 1428-1490, 1561-1600) · `public/scripts/custom-request.js` (4, 139, 168-184, 483, 509-530) · `public/scripts/textgen-settings.js` (1300-1337) · `public/scripts/power-user.js` (274-283) · `public/index.html` (2142-2156, 7442) · `public/style.css` (430-521, 1222) · `src/endpoints/backends/chat-completions.js` (1071-1175, 1137, 2306-2314, 2551-2562) · `src/endpoints/backends/text-completions.js` (29-65) · `default/content/presets/reasoning/*.json` · `tests/util/mock-server.js:34` · `tests/mock-server.test.js:29`

**Marinara Engine (`c5650149`)**
`packages/server/src/services/generation/empty-response-reason.ts` · `packages/server/src/routes/generate.routes.ts` (556, 8703-8712, 9668-9700, 10267) · `packages/server/src/routes/connections.routes.ts` (1833-1841) · `packages/server/src/routes/generate/generate-route-utils.ts:1159` · `packages/server/src/services/llm/providers/openai.provider.ts` (66-97, 117-128, 353-375, 433-553, 882-1004, 1174, 1490, 1494-1512, 1708-1731, 1929) · `packages/server/src/services/sidecar/sidecar-inference.service.ts` (105-107, 170-188, 227-390, 456-462) · `packages/server/src/services/agents/agent-executor.ts` (3633, 3705) · `packages/server/src/services/advanced-memory.ts` (1096-1101) · `packages/shared/src/utils/thinking-tags.ts` · `packages/shared/src/types/prompt.ts:248` · `packages/client/src/hooks/use-generate.ts` (1598-1609, 1978, 2411, 2539, 2592, 2681) · `packages/client/src/components/chat/ChatMessage.tsx` (1990, 2335-2337, 4211, 4701) · `scripts/regressions/deepseek-reasoning-output.regression.ts` · `CHANGELOG.md` (173, 1029, 1104, 1187, 1263, 1274, 1287, 1294, 1530, 1886, 2501, 2510, 2521)

**Primary docs**
https://openrouter.ai/docs/use-cases/reasoning-tokens · https://docs.ollama.com/capabilities/thinking · https://docs.z.ai/guides/capabilities/thinking · https://docs.z.ai/guides/capabilities/thinking-mode · https://docs.z.ai/api-reference/llm/chat-completion · https://api-docs.deepseek.com/guides/thinking_mode · https://api-docs.deepseek.com/api/create-chat-completion
