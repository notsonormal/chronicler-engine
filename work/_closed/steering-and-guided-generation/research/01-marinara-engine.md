# Research: Marinara-Engine AI Steering

One-line summary: Marinara-Engine implements three distinct steering surfaces—guided generation (transient per-turn guide), narrator/system-style history injection (persistent `narrator` role), and impersonate (force the model to write the next turn as the user persona)—with explicit prompt-layer rules that map cleanly to chronicler_engine's `LayerRenderer` / `PromptContext` model.

## Repo overview

Marinara-Engine is a local TypeScript/Node monorepo (packages: `client`, `server`, `shared`) for AI chat, roleplay, and game modes. It is not a SillyTavern fork; it has its own prompt assembler, message schema, and HTTP API.

Because the local `git clone` command is blocked by the permission system, I downloaded the latest `main` archive from GitHub and extracted it to `tmp/marinara-engine/`. The GitHub API reported the HEAD commit at that time as `bf103aa97676e8aed1659415df8ed26cada42453`.

```
tmp/marinara-engine/
packages/client/   React + Tailwind UI
packages/server/   Fastify backend, prompt assembler, storage
packages/shared/   Zod schemas, constants, macro utilities
```

## Guided generation

### What it is

A transient per-generation instruction. The user types a direction (e.g., a `/guided` command or a guided-regenerate prompt), the client sends it as `generationGuide` + `generationGuideSource`, and the server injects it as a final `system` message. It is **not** persisted as a chat message.

### Schema / request fields

`packages/shared/src/schemas/chat.schema.ts`:

```typescript
export const generateRequestSchema = z.object({
  ...
  generationGuide: z.string().nullable().optional().default(null),
  generationGuideSource: z.enum(["narrator", "guide", "game_start"]).nullable().optional().default(null),
  ...
});
```

`packages/shared/src/utils/generation-guide.ts`:

```typescript
export type GenerationGuideSource = "narrator" | "guide" | "game_start";

export function buildNarratorInstructionMessage(direction: string): string {
  return `[Narrator instruction — do not include a reply from {{user}}. Instead, write the next part of the narrative steering it toward the following: ${direction.trim()}]`;
}

export function buildGuidedGenerationInstructionMessage(direction: string): string {
  return `[Guided generation instruction — do not include a reply from {{user}}. Instead, write the next generated message steering it toward the following: ${direction.trim()}]`;
}
```

### Prompt format / block

The server-side builder wraps the raw text with a plain English prefix:

`packages/server/src/routes/generate/generate-route-utils.ts` (lines 93-115):

```typescript
export function buildGenerationGuideInstruction(
  generationGuide: unknown,
  promptMacroContext: MacroContext,
): string | null {
  const rawGenerationGuide = typeof generationGuide === "string" ? generationGuide.trim() : "";
  if (!rawGenerationGuide) return null;

  const normalizedGenerationGuide = resolveMacros(
    rawGenerationGuide,
    {
      ...promptMacroContext,
      variables: { ...promptMacroContext.variables },
    },
    { trimResult: false },
  ).trim();

  return normalizedGenerationGuide
    ? `Take the following into special consideration for your next message: ${normalizedGenerationGuide}`
    : null;
}
```

The client, for `/guide` input, sends the bracketed form already:

`packages/client/src/components/chat/ChatInput.tsx` (lines 1677-1685):

```typescript
generationGuide: buildGuidedGenerationInstructionMessage(guideText),
generationGuideSource: "guide",
```

Game-start uses the same field without the bracket wrapper:

`packages/client/src/components/game/GameSurface.tsx` (lines 2080-2082):

```typescript
const GAME_START_GENERATION_GUIDE =
  "Begin the game now with the first visible GM VN narration/dialogue segment. This is an invisible startup trigger, not a player action. Do not mention a start command.";
```

### Where it is injected

After the assembler produces `finalMessages`, the server appends the guide as a `system` message at the very end of the prompt, before any character-specific instructions but after the assembled preset/character/history context.

`packages/server/src/routes/generate.routes.ts` (lines 7080-7091, individual group mode):

```typescript
const generationGuideInstruction = buildGenerationGuideInstruction(input.generationGuide, promptMacroContext);
const buildRoleplayCharacterInstruction = (charName: string) =>
  groupTurnPromptEnabled && chatMode === "roleplay" ? `Respond ONLY as ${charName}.` : null;

if (useIndividualLoop) {
  sendProgress("generating");
  let runningMessages = [...finalMessages];

  if (generationGuideInstruction) {
    runningMessages.push({ role: "system", content: generationGuideInstruction });
  }
```

`packages/server/src/routes/generate.routes.ts` (lines 7153-7165, single/merged mode):

```typescript
} else {
  sendProgress("generating");
  let targetCharId = ...;
  const sentMessages = [...finalMessages];

  if (generationGuideInstruction) {
    sentMessages.push({ role: "system", content: generationGuideInstruction });
  }
```

### Lorebook scan also sees the guide

The guide is folded into the keyword/semantic scan as a synthetic `user` message so lorebooks can react to it, but that synthetic message is not part of the final prompt history.

`packages/server/src/services/generation/lorebook-generation-runtime.ts` (lines 30-45):

```typescript
export function buildLorebookScanMessagesWithGenerationGuide(
  messages: LorebookScanMessage[],
  input: {
    generationGuide?: string | null;
    generationGuideSource?: "narrator" | "guide" | "game_start" | null;
  },
  resolveContent: (value: string) => string = (value) => value,
): LorebookScanMessage[] {
  const guide = input.generationGuide?.trim();
  if (!guide || (input.generationGuideSource !== "narrator" && input.generationGuideSource !== "guide")) {
    return messages;
  }
  const resolvedGuide = resolveContent(guide).trim();
  return resolvedGuide ? [...messages, { role: "user", content: resolvedGuide }] : messages;
}
```

### Retry vs new generation

For a **new generation**, the client supplies `generationGuide` directly in the request.

For a **regeneration (swipe)**, the previous assistant message stores a `generationReplay` blob in its `extra`. On regeneration, the server replays the guide into the new request input.

`packages/server/src/routes/generate/generation-replay.ts` (lines 95-110):

```typescript
export function applyGenerationReplayToRegenerateInput(
  input: GenerationReplayInput,
  replay: GenerationReplay | null,
): boolean {
  if (!replay) return false;

  let applied = false;
  ...
  if (replay.impersonate !== true && !asNonEmptyString(input.generationGuide) && replay.generationGuide) {
    input.generationGuide = replay.generationGuide;
    input.generationGuideSource = replay.generationGuideSource ?? "guide";
    applied = true;
  }
```

The guide itself is stored in the previous assistant message's `extra.generationReplay` so it can be replayed, but it is **not** stored as a transcript message. The strip helper is used to recover the raw direction when a guide is converted into an impersonate user message:

`packages/shared/src/utils/generation-guide.ts` (lines 19-31):

```typescript
export function stripGenerationGuideInstruction(value: string): string {
  if (!value.endsWith("]")) return value;
  const prefixes = ["[Narrator instruction ", "[Guided generation instruction "];
  const prefix = prefixes.find((candidate) => value.startsWith(candidate));
  if (!prefix) return value;
  const marker = " following:";
  const markerIndex = value.indexOf(marker, prefix.length);
  if (markerIndex < 0 || value.indexOf("]", prefix.length) < markerIndex) return value;
  return value.slice(markerIndex + marker.length, -1).trim() || value;
}
```

### Transience enforcement

1. The guide is never saved as a `messages` row.
2. It is only appended to the in-memory `runningMessages` / `sentMessages` arrays at the end of prompt assembly.
3. The only durable artifact is the `generationReplay` object inside the generated assistant message's `extra`, and that is used only for regeneration replay.

## Narrator / system-style injection

### What it is

Marinara has a dedicated `narrator` message role. It is used for omniscient voice content that should persist in the transcript but not be spoken by a character: scene participation guides, scene descriptions, game session recaps/conclusions, and fork continuity context. The model sees these messages as `system` turns (or as history-time-stamped `user` turns in conversation mode), while the UI renders them as a distinct "Narrator" bubble.

### Storage role

`packages/server/src/db/schema/chats.ts` (lines 39-50):

```typescript
export const messages = fileTable("messages", {
  id: text("id").primaryKey(),
  chatId: text("chat_id")
    .notNull()
    .references(() => chats.id, { onDelete: "cascade" }),
  role: text("role", { enum: ["user", "assistant", "system", "narrator"] }).notNull(),
  characterId: text("character_id"),
  content: text("content").notNull().default(""),
  activeSwipeIndex: integer("active_swipe_index").notNull().default(0),
  extra: text("extra").notNull().default("{}"),
  createdAt: text("created_at").notNull(),
});
```

### Creation examples

Scene participation guide (visible to user):

`packages/server/src/routes/scene.routes.ts` (lines 380-387):

```typescript
if (plan.participationGuide) {
  await chats.createMessage({
    chatId: sceneChat.id,
    role: "narrator",
    characterId: null,
    content: plan.participationGuide,
  });
}
```

Hidden continuity context when forking a scene:

`packages/server/src/routes/scene.routes.ts` (lines 675-686):

```typescript
if (continuity) {
  copiedMessages.push({
    role: "narrator",
    characterId: null,
    content: continuity,
    extra: {
      displayText: null,
      isGenerated: true,
      tokenCount: null,
      generationInfo: null,
      hiddenFromUser: true,
    },
  });
}
```

Game session recap:

`packages/server/src/routes/game.routes.ts` (lines 7207-7224):

```typescript
if (recapText) {
  const recapMsg = await chats.createMessage({
    chatId: newChat.id,
    role: "narrator",
    characterId: null,
    content: recapText,
  });
```

### Rendering in the model's history

For roleplay/game, `narrator` rows are mapped to `role: "system"` with `contextKind: "history"` when the prompt is built:

`packages/server/src/routes/generate.routes.ts` (lines 1448-1456):

```typescript
return {
  id: typeof m.id === "string" ? m.id : null,
  role: m.role === "narrator" ? ("system" as const) : (m.role as "user" | "assistant" | "system"),
  content,
  contextKind: "history" as const,
  characterId: typeof m.characterId === "string" && m.characterId ? m.characterId : null,
  ...
};
```

In conversation mode, older narrator entries are formatted with an explicit `Narrator:` speaker prefix and wrapped in a date block:

`packages/server/src/routes/generate/conversation-history-runtime.ts` (lines 393-399):

```typescript
let author = "Character";
if (membershipEvent !== null) author = "System";
else if (raw.role === "narrator") author = "Narrator";
else if (msg.role === "user") author = args.personaName;
```

`packages/server/src/routes/generate/conversation-prompt-formatting.ts` (lines 39-43):

```typescript
return messages.map((message, idx) => {
  let content = `${message.author}: ${message.content}`;
  if (idx === 0) content = `<date="${date}">\n${content}`;
  if (idx === messages.length - 1) content = `${content}\n</date>`;
  return { role: message.role, content };
});
```

### Post-history system-role handling

After the assembler runs, any `system` messages that appear **after** the leading system block and are tagged as history or injection are kept at their exact transcript position (converted to `user` if needed). System messages that are part of the prompt block are merged into the latest user message so providers that dislike interleaved system/user/assistant turns still receive a clean transcript.

`packages/server/src/routes/generate/generate-route-utils.ts` (lines 499-523):

```typescript
export function appendNonLeadingSystemMessagesToLastUser<T extends PromptRoleMessage>(messages: T[]): T[] {
  ...
    if (cloned.role === "system") {
      const converted = { ...cloned, role: "user" as const };
      if (cloned.contextKind === "history" || cloned.contextKind === "injection") {
        result.push(converted as T);
        lastUserIndex = result.length - 1;
        continue;
      }
      if (lastUserIndex >= 0) {
        appendPromptMessageContent(result[lastUserIndex]!, converted);
      } else {
        result.push(converted as T);
        lastUserIndex = result.length - 1;
      }
      continue;
    }
```

The preceding comment explains the policy:

```typescript
/**
 * Provider-safe role normalization for strict prompt presets.
 *
 * System blocks before chat history stay as provider system messages. Once
 * conversation turns have started, later system blocks are appended to the
 * latest user message so the request remains system/user/assistant/user...
 * without making post-history preset sections removable during context fitting.
 * History-context system messages are already positioned in the transcript, so
 * they become user messages in place instead of moving to the latest user turn.
 * Depth injections follow the same rule because they are also inserted at an
 * exact history position.
 */
```

### UI display

`packages/client/src/components/chat/ChatMessage.tsx` (lines 1450-1452, 1613, 2623, 2674):

```typescript
const isNarrator = message.role === "narrator";
```

```typescript
const ttsSpeakerName =
  message.role === "narrator"
    ? "Narrator"
    : message.characterId
      ? characterMap?.get(message.characterId)?.name
      : undefined;
```

```tsx
<div
  className={cn(
    "mari-message mari-message-narrator rpg-narrator-msg group mb-4 px-2",
    ...
  )}
>
  ...
  <div className="mb-1 flex items-center gap-2 text-[0.625rem] font-semibold uppercase tracking-widest text-amber-400/70">
    <span className="h-px flex-1 bg-amber-400/20" />
    {localizeUi("ui.chat.chatmessage.narrator")}
    <span className="h-px flex-1 bg-amber-400/20" />
  </div>
```

## Impersonate

### What it is

Impersonate forces the model to write the next turn as the user's persona (the active user character / persona card), not as the AI character. The generated text is saved as a `user` message in the chat.

### Request schema

`packages/shared/src/schemas/chat.schema.ts` (lines 82-86):

```typescript
// Impersonate overrides (applied only when impersonate=true)
impersonatePresetId: z.string().nullish(),
impersonateConnectionId: z.string().nullish(),
impersonateBlockAgents: z.boolean().optional().default(false),
impersonatePromptTemplate: z.string().optional(),
```

### Guardrails

`packages/server/src/routes/generate.routes.ts` (lines 787-809):

```typescript
if (requestChatMode === "conversation" && input.impersonate) {
  return reply.status(400).send({ error: "Impersonate is not available in Conversation mode" });
}
...
if (input.continueMessageId) {
  if (input.impersonate) {
    return reply.status(400).send({ error: "Cannot continue a message while impersonating" });
  }
```

### Persona target selection

The impersonate instruction uses the active chat persona (name + description) resolved by `resolveActivePersonaCandidate`. It does **not** target a character; it targets the user persona.

`packages/server/src/routes/generate.routes.ts` (lines 1529-1546):

```typescript
const persona = resolveActivePersonaCandidate(allPersonas, chat.personaId, chatMode);
if (persona) {
  personaId = persona.id as string;
  personaName = persona.name;
  personaPhoneticName = typeof persona.phoneticName === "string" ? persona.phoneticName : "";
  personaDescription = cardPromptText(persona.description);

  personaFields = {
    phoneticName: personaPhoneticName,
    personality: cardPromptText(persona.personality),
    scenario: cardPromptText(persona.scenario),
    backstory: cardPromptText(persona.backstory),
    appearance: cardPromptText(persona.appearance),
  };
}
```

### Instruction builder

Default template:

`packages/shared/src/constants/impersonate.ts` (lines 1-14):

```typescript
export const DEFAULT_IMPERSONATE_PROMPT = [
  `<instruction>`,
  `You are now writing as {{user}}, the user's character.`,
  `Study {{user}}'s previous messages in the conversation and replicate their voice, mannerisms, speech patterns, and style as closely as possible.`,
  `Character description: {{persona_description}}`,
  `Additional direction for this reply: {{impersonate_direction}}`,
  `Write a single in-character response from {{user}}'s perspective. Do NOT break character or add meta-commentary. Respond exactly as {{user}} would.`,
  `</instruction>`,
]
  .filter(Boolean)
  .join("\n");
```

Builder:

`packages/server/src/services/conversation/impersonate-prompt.ts` (lines 80-111):

```typescript
export function buildImpersonateInstruction({
  customPrompt,
  direction,
  personaName,
  personaDescription,
}: BuildImpersonateInstructionArgs): string {
  const normalizedCustomPrompt = normalizeText(customPrompt);
  const impersonationDirection = normalizeDirection(direction);
  const personaLabel = normalizeText(personaName) || "{{user}}";
  const description = normalizeText(personaDescription);

  if (normalizedCustomPrompt) {
    const resolvedCustomPrompt = renderImpersonateTemplate(normalizedCustomPrompt, {
      direction: impersonationDirection,
      personaName: personaLabel,
      personaDescription: description,
    });
    return normalizedCustomPrompt.includes("{{impersonate_direction}}")
      ? resolvedCustomPrompt
      : buildCustomImpersonateInstruction(resolvedCustomPrompt, impersonationDirection);
  }

  return renderImpersonateTemplate(DEFAULT_IMPERSONATE_PROMPT, {
    direction: impersonationDirection,
    personaName: personaLabel,
    personaDescription: description,
  });
}
```

### Injection position and role

The impersonate instruction is appended as a final `user` message:

`packages/server/src/routes/generate.routes.ts` (lines 5001-5007):

```typescript
if (input.impersonate && followUpIteration === 0) {
  const impersonateInstruction = buildImpersonateInstruction({
    customPrompt: input.impersonatePromptTemplate || chatMeta.impersonatePrompt,
    direction: input.userMessage,
    personaName,
    personaDescription: resolvePromptMacros(personaDescription),
  });
  finalMessages.push({ role: "user", content: impersonateInstruction });
}
```

No real user message is saved for the turn:

`packages/server/src/routes/generate.routes.ts` (lines 938-939):

```typescript
// Save user message — skip for impersonate (no real user message to save)
if (!input.impersonate && (input.userMessage || input.attachments?.length || input.pendingSpatialTransition)) {
```

### Interaction with the system prompt / output format / persona card

**Preset selection:** when impersonating, the candidate list prefers an impersonate-specific prompt preset, then falls back to the normal preset candidates. This lets users have a custom "write as user" preset.

`packages/server/src/routes/generate/prompt-preset-selection.ts` (lines 46-53):

```typescript
export function buildGenerationPromptPresetCandidates(args: {
  ...
  impersonate?: boolean;
  impersonatePromptPresetId?: unknown;
  ...
}): PromptPresetCandidate[] {
  const candidates: PromptPresetCandidate[] = [];
  const seen = new Set<string>();

  if (args.impersonate) {
    pushUnique(candidates, seen, asNonEmptyString(args.impersonatePromptPresetId), "impersonate");
  }
  pushUnique(candidates, seen, asNonEmptyString(args.requestPromptPresetId), "request");
```

**Preset section suppression:** unless the selected preset is the impersonate-specific one, the assembler drops normal preset sections (character card, system prompt, output format, etc.) during impersonate so they do not conflict with the "write as user" instruction. Only markers such as `chat_history` and `lorebook` remain.

`packages/server/src/services/prompt/assembler.ts` (lines 329-334):

```typescript
    if (input.impersonate === true && input.preserveImpersonatePresetSections !== true && section.isMarker !== "true") {
      continue;
    }
```

`packages/server/src/routes/generate.routes.ts` (lines 2182-2185):

```typescript
impersonate: input.impersonate === true,
preserveImpersonatePresetSections: input.impersonate === true && presetSource === "impersonate",
deferCharacterMacros,
```

**Output format / assistant prefill:** the assistant prefill is skipped so the model cannot prime an assistant-style continuation. The impersonate instruction is the last user turn.

`packages/server/src/routes/generate/generate-route-utils.ts` (lines 882-884):

```typescript
  const shouldAppendGoogleUserRegeneration =
    !options.impersonate && options.isGoogleProvider && !!options.regenerateUserMessage;
  const assistantPrefill = options.assistantPrefill.trim();
  const shouldAppendAssistantPrefill = !options.impersonate && !!assistantPrefill;
```

### Saving the result

The generated response is stored as a `user` message, not an `assistant` message.

`packages/server/src/routes/generate.routes.ts` (lines 6832-6838):

```typescript
savedMsg = await chats.createMessage({
  chatId: input.chatId,
  role: input.impersonate ? "user" : "assistant",
  characterId: input.impersonate ? null : targetCharId,
  content: fullResponse,
});
```

### Does it replace or augment the narrator voice?

It **replaces** the normal AI voice for one turn. The normal preset sections are suppressed, no assistant prefill is added, and the response is saved with the user role. The impersonate instruction itself supplies the persona description and direction, so the model writes as the user persona rather than as the narrator or character.

## What is portable to chronicler_engine

- **Transient guide layer.** Add a `Guide` or `Steering` layer to `PromptAssembler` / `LayerRenderer` that accepts a per-request `generationGuide` string from `PromptContext` and appends it as a final `system` message. It must never be written to `MessageHistory`/`MessageEntry` as a transcript turn. Store it only in the generated message's metadata (e.g., swipe extra) so regenerations can replay it.
- **Regeneration replay metadata.** Store a small `GenerationReplay` blob in the generated message extra/snapshot that preserves the guide, impersonate flag, and impersonate template. On regenerate, replay it into the request.
- **Narrator message type.** Add a `Narrator` variant to `MessageType` (or mark `System` messages with a display flag). Persist them as `narrator` rows, map them to `System` when building the LLM prompt, and render them with a distinct UI style (amber narrator header). This is useful for scene summaries, recaps, and participation guides.
- **History-safe system/narrator handling.** Keep leading system blocks intact; convert post-history narrator/system messages to `user` in place, or merge them into the latest user turn, depending on provider strictness. The Marinara rule (`contextKind` = history → keep in place, prompt → merge) maps cleanly to a prompt post-processor.
- **Impersonate mode.** Add an `impersonate` request flag that suppresses normal character preset sections (unless an impersonate-specific preset is selected), appends a user-role instruction built from the active persona, skips the assistant prefill, and saves the response as a `user` message.
- **Persona macro templates.** Use the same `{{user}}`, `{{persona_description}}`, and `{{impersonate_direction}}` placeholders in guide and impersonate templates; resolve them through the existing `PromptContext` macro resolver.

## Open questions / gaps

- The `buildNarratorInstructionMessage` helper in `packages/shared/src/utils/generation-guide.ts` exists, but I did not find a client call site that uses it. The client appears to use `buildGuidedGenerationInstructionMessage` for all guided input. It is unclear whether the `narrator` generation-guide source is reachable from the UI or only programmatically.
- The server does **not** strip the bracketed wrappers (`[Guided generation instruction ...]` / `[Narrator instruction ...]`) before wrapping the guide with `Take the following into special consideration...`. The bracketed text is therefore sent verbatim inside the system message. The `stripGenerationGuideInstruction` function is only used when converting a guide into an impersonate user message on regeneration.
- I did not confirm the exact UI path that creates a plain `narrator` message manually. The code shows automated creation via `/scene` and `/game/session`, but manual creation may be missing or in a different package.
- I did not trace the full prompt preset content for an `impersonate`-source preset, so I cannot confirm whether output-format instructions are actually included when `preserveImpersonatePresetSections` is true. The code only preserves sections if `presetSource === "impersonate"`.
- I did not verify whether `system` messages created manually by users (distinct from `narrator`) are rendered differently in the model history; the schema allows both roles, but the UI focus is on `narrator`.
