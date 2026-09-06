# Research: SillyTavern Core AI Steering

One-line summary: SillyTavern core implements three steering surfaces—a transient `/inject` (with `ephemeral=true`) stored in chat metadata and flushed after one generation, persistent narrator messages saved as real chat rows tagged `extra.type === 'narrator'` and rendered as `system` turns to the model, and impersonation through a trailing `system` control prompt that streams the result into the input box for review rather than saving a chat message.

## Repo overview

SillyTavern core is the upstream JavaScript/Electron web app fetched at `tmp/sillytavern-core/`. The parent supplied HEAD `8172dcd0ee672d3cd9a5e5f7af134f91a45cd2b8`.

The relevant steering code is in:

- `public/scripts/slash-commands.js` — `/inject`, `/flushinject`, `/impersonate`, `/sys`, `/sysgen`, `/sysname`, `/comment`, `/note`.
- `public/scripts/openai.js` — OpenAI/chat-completion prompt manager: `prepareOpenAIMessages`, `preparePromptsForChatCompletion`, `populateChatCompletion`, `populationInjectionPrompts`, impersonation prompt assembly.
- `public/scripts/script.js` — extension prompt registry (`setExtensionPrompt`, `getExtensionPrompt`, `extension_prompt_types`, `extension_prompt_roles`), `doChatInject`, `Generate`, `StreamingProcessor`.
- `public/scripts/system-messages.js` — `system_message_types` including `NARRATOR` and `COMMENT`.
- `public/scripts/authors-note.js` — persistent author's note (`/note`) as a depth-keyed extension prompt.

## Guided generation

### What it is

A transient, per-generation instruction. The user (or an extension) runs `/inject id=<id> position=... depth=... role=... scan=... ephemeral=true <text>`. The text is injected into the next outgoing prompt and removed after that generation finishes. It is **not** saved as a chat message row.

### Command definition

`public/scripts/slash-commands.js:2892`:

```javascript
SlashCommandParser.addCommandObject(SlashCommand.fromProps({
    name: 'inject',
    returns: t`injection ID`,
    callback: injectCallback,
    namedArgumentList: [
        SlashCommandNamedArgument.fromProps({
            name: 'id',
            description: t`injection ID`,
            typeList: [ARGUMENT_TYPE.STRING],
            isRequired: false,
            enumProvider: commonEnumProviders.injects,
        }),
        new SlashCommandNamedArgument(
            'position', t`injection position`, [ARGUMENT_TYPE.STRING], false, false, 'after', ['before', 'after', 'chat', 'none'],
        ),
        new SlashCommandNamedArgument(
            'depth', t`injection depth`, [ARGUMENT_TYPE.NUMBER], false, false, '4',
        ),
        new SlashCommandNamedArgument(
            'scan', t`include injection content into World Info scans`, [ARGUMENT_TYPE.BOOLEAN], false, false, 'false',
        ),
        SlashCommandNamedArgument.fromProps({
            name: 'role',
            description: t`role for in-chat injections`,
            typeList: [ARGUMENT_TYPE.STRING],
            isRequired: false,
            enumList: [
                new SlashCommandEnumValue('system', null, enumTypes.enum, enumIcons.system),
                new SlashCommandEnumValue('assistant', null, enumTypes.enum, enumIcons.assistant),
                new SlashCommandEnumValue('user', null, enumTypes.enum, enumIcons.user),
            ],
        }),
        new SlashCommandNamedArgument(
            'ephemeral', t`remove injection after generation`, [ARGUMENT_TYPE.BOOLEAN], false, false, 'false',
        ),
        SlashCommandNamedArgument.fromProps({
            name: 'filter',
            description: t`if a filter is defined, an injection will only be performed if the closure returns true`,
            typeList: [ARGUMENT_TYPE.CLOSURE],
            isRequired: false,
            acceptsMultiple: false,
        }),
    ],
    unnamedArgumentList: [
        new SlashCommandArgument(
            t`text`, [ARGUMENT_TYPE.STRING], false,
        ),
    ],
    helpString: t`Injects a text into the LLM prompt for the current chat. Requires a unique injection ID (will be auto-generated if not provided). Positions: "before" main prompt, "after" main prompt, in-"chat", hidden with "none" (default: after). Depth: injection depth for the prompt (default: 4). Role: role for in-chat injections (default: system). Scan: include injection content into World Info scans (default: false). Hidden injects in "none" position are not inserted into the prompt but can be used for triggering WI entries. Returns the injection ID.`,
}));
```

### Storage

`/inject` writes into `chat_metadata.script_injects` under the user-supplied `id`, then registers the same data with `setExtensionPrompt` using a prefixed key. `public/scripts/slash-commands.js:3779`:

```javascript
const positions = {
    'before': extension_prompt_types.BEFORE_PROMPT,
    'after': extension_prompt_types.IN_PROMPT,
    'chat': extension_prompt_types.IN_CHAT,
    'none': extension_prompt_types.NONE,
};
const roles = {
    'system': extension_prompt_roles.SYSTEM,
    'user': extension_prompt_roles.USER,
    'assistant': extension_prompt_roles.ASSISTANT,
};

const id = String(args?.id ?? '') || Math.random().toString(36).substring(2);
const ephemeral = isTrueBoolean(String(args?.ephemeral ?? ''));

const defaultPosition = 'after';
const defaultDepth = 4;
const positionValue = args?.position ?? defaultPosition;
const position = positions[positionValue] ?? positions[defaultPosition];
const depthValue = Number(args?.depth ?? defaultDepth);
const depth = isNaN(depthValue) ? defaultDepth : depthValue;
const roleValue = typeof args?.role === 'string' ? args.role.toLowerCase().trim() : Number(args?.role ?? extension_prompt_roles.SYSTEM);
const role = roles[roleValue] ?? extension_prompt_roles.SYSTEM;
const scan = isTrueBoolean(String(args?.scan));
```

`public/scripts/slash-commands.js:3810`:

```javascript
const prefixedId = `${SCRIPT_PROMPT_KEY}${id}`;

if (!chat_metadata.script_injects) {
    chat_metadata.script_injects = {};
}

if (value) {
    const inject = { value, position, depth, scan, role, filter };
    chat_metadata.script_injects[id] = inject;
} else {
    delete chat_metadata.script_injects[id];
}

setExtensionPrompt(prefixedId, String(value), position, depth, scan, role, filterFunction);
saveMetadataDebounced();
```

The enum values are defined in `public/scripts/script.js:483`:

```javascript
export const extension_prompt_types = {
    NONE: -1,
    IN_PROMPT: 0,
    IN_CHAT: 1,
    BEFORE_PROMPT: 2,
};

export const extension_prompt_roles = {
    SYSTEM: 0,
    USER: 1,
    ASSISTANT: 2,
};
```

### Where injected

#### `position=chat`

`position=chat` maps to `extension_prompt_types.IN_CHAT`. The injection is spliced into the chat-history array at the requested depth. For OpenAI backends this happens in `public/scripts/openai.js:801` (`populationInjectionPrompts`):

```javascript
async function populationInjectionPrompts(prompts, messages) {
    let totalInsertedMessages = 0;

    const roleTypes = {
        'system': extension_prompt_roles.SYSTEM,
        'user': extension_prompt_roles.USER,
        'assistant': extension_prompt_roles.ASSISTANT,
    };

    const maxDepth = getExtensionPromptMaxDepth();
    for (let i = 0; i <= maxDepth; i++) {
        // Get prompts for current depth
        const depthPrompts = prompts.filter(prompt => prompt.injection_depth === i && prompt.content);

        const roleMessages = [];
        const separator = '\n';
        const wrap = false;

        // Group prompts by priority
        const extensionPromptsOrder = '100';
        const orderGroups = {
            [extensionPromptsOrder]: [],
        };
        for (const prompt of depthPrompts) {
            const order = prompt.injection_order ?? 100;
            if (!orderGroups[order]) {
                orderGroups[order] = [];
            }
            orderGroups[order].push(prompt);
        }

        // Process each order group in order (b - a = low to high ; a - b = high to low)
        const orders = Object.keys(orderGroups).sort((a, b) => +b - +a);
        for (const order of orders) {
            const orderPrompts = orderGroups[order];

            // Order of priority for roles (most important go lower)
            const roles = ['system', 'user', 'assistant'];
            for (const role of roles) {
                const rolePrompts = orderPrompts
                    .filter(prompt => prompt.role === role)
                    .map(x => x.content)
                    .join(separator);

                // Get extension prompt
                const extensionPrompt = order === extensionPromptsOrder
                    ? await getExtensionPrompt(extension_prompt_types.IN_CHAT, i, separator, roleTypes[role], wrap)
                    : '';
                const jointPrompt = [rolePrompts, extensionPrompt].filter(x => x).map(x => x.trim()).join(separator);

                if (jointPrompt && jointPrompt.length) {
                    roleMessages.push({ 'role': role, 'content': jointPrompt, injected: true });
                }
            }
        }

        if (roleMessages.length) {
            const injectIdx = i + totalInsertedMessages;
            messages.splice(injectIdx, 0, ...roleMessages);
            totalInsertedMessages += roleMessages.length;
        }
    }

    messages = messages.reverse();
    return messages;
}
```

For text-completion backends the same idea is implemented in `public/scripts/script.js:5569` (`doChatInject`). A `system`-role injection is additionally tagged with `extra.type === system_message_types.NARRATOR` so it is rendered as a system turn:

```javascript
for (const role of roles) {
    const extensionPrompt = String(await getExtensionPrompt(extension_prompt_types.IN_CHAT, i, separator, role, wrap)).trimStart();
    const isNarrator = role === extension_prompt_roles.SYSTEM;
    const isUser = role === extension_prompt_roles.USER;
    const name = names[role];

    if (extensionPrompt) {
        roleMessages.push({
            name: name,
            is_user: isUser,
            mes: extensionPrompt,
            extra: {
                type: isNarrator ? system_message_types.NARRATOR : null,
            },
        });
    }
}
```

Depth `0` means "after the last message"; depth `4` means "4 messages back from the end".

#### `position=before` / `position=after`

`position=before` maps to `extension_prompt_types.BEFORE_PROMPT` and is placed at the start of the system-prompt block; `position=after` maps to `IN_PROMPT` and is placed at the end. In `public/scripts/openai.js:1439` any unknown extension prompt with one of those positions is merged into the prompt manager's system prompts:

```javascript
for (const key in extensionPrompts) {
    if (Object.hasOwn(extensionPrompts, key)) {
        const prompt = extensionPrompts[key];
        if (knownExtensionPrompts.includes(key)) continue;
        if (!extensionPrompts[key].value) continue;
        if (![extension_prompt_types.BEFORE_PROMPT, extension_prompt_types.IN_PROMPT].includes(prompt.position)) continue;

        const hasFilter = typeof prompt.filter === 'function';
        if (hasFilter && !await prompt.filter()) continue;

        systemPrompts.push({
            identifier: key.replace(/\W/g, '_'),
            position: getPromptPosition(prompt.position),
            role: getPromptRole(prompt.role),
            content: prompt.value,
            extension: true,
        });
    }
}
```

`public/scripts/openai.js:1131`:

```javascript
export function getPromptPosition(position) {
    if (position == extension_prompt_types.BEFORE_PROMPT) {
        return 'start';
    }

    if (position == extension_prompt_types.IN_PROMPT) {
        return 'end';
    }

    return false;
}
```

### Other native guided-generation equivalents

SillyTavern core has **no other transient** guided-generation mechanism besides `/inject`.

The closest alternatives are persistent, not ephemeral:

- **Author's Note (`/note`)** — a per-chat or per-character prompt stored in metadata and injected as an extension prompt at a configurable depth/role/interval. `public/scripts/authors-note.js:324`:

  ```javascript
  export function setFloatingPrompt() {
      const context = getContext();
      if (!context.groupId && context.characterId === undefined) {
          console.debug('setFloatingPrompt: Not in a chat. Skipping.');
          shouldWIAddPrompt = false;
          return;
      }

      // take the count of messages
      let lastMessageNumber = Array.isArray(context.chat) && context.chat.length ? context.chat.filter(m => m.is_user).length : 0;
  ```

  `public/scripts/authors-note.js:383`:

  ```javascript
  context.setExtensionPrompt(
      MODULE_NAME,
      String(prompt),
      chat_metadata[metadata_keys.position],
      chat_metadata[metadata_keys.depth],
      extension_settings.note.allowWIScan,
      chat_metadata[metadata_keys.role],
  );
  ```

  It is never saved as a chat message row.

- **`/comment`** — creates a chat row with `is_system: true` and `extra.type === system_message_types.COMMENT`. It is visible in the UI but filtered out of `coreChat` before prompt assembly, so it is display-only.

### Retry vs new generation

`ephemeral=true` registers a one-time listener on `GENERATION_ENDED` and `GENERATION_STOPPED`. `public/scripts/slash-commands.js:3826`:

```javascript
if (ephemeral) {
    let deleted = false;
    const unsetInject = () => {
        if (deleted) {
            return;
        }
        console.log('Removing ephemeral script injection', id);
        delete chat_metadata.script_injects[id];
        setExtensionPrompt(prefixedId, '', position, depth, scan, role, filterFunction);
        saveMetadataDebounced();
        deleted = true;
    };
    eventSource.once(event_types.GENERATION_ENDED, unsetInject);
    eventSource.once(event_types.GENERATION_STOPPED, unsetInject);
}
```

`GENERATION_ENDED` is emitted from `hideStopButton()` in `public/scripts/script.js:3479`, which is called at the end of every generation. Therefore:

- **New generation:** the ephemeral inject is present for that generation, then removed.
- **Retry / swipe / regenerate:** the inject is present only if it was created before that specific generation attempt. It does not survive the end of the previous attempt. The caller must re-inject the guide for each retry (this is exactly what the GuidedGenerations extension does).

A manual `/flushinject <id>` can also remove any inject early. `public/scripts/slash-commands.js:3870`:

```javascript
function flushInjectsCallback(_, value) {
    if (!chat_metadata.script_injects) {
        return '';
    }

    const idArgument = value;

    for (const [id, inject] of Object.entries(chat_metadata.script_injects)) {
        if (idArgument && id !== idArgument) {
            continue;
        }

        const prefixedId = `${SCRIPT_PROMPT_KEY}${id}`;
        setExtensionPrompt(prefixedId, '', inject.position, inject.depth, inject.scan, inject.role);
        delete chat_metadata.script_injects[id];
    }

    saveMetadataDebounced();
    return '';
}
```

### Transience enforcement

1. The injection lives in `chat_metadata.script_injects`, not in `chat[]`.
2. `setExtensionPrompt` registers it only in the in-memory `extension_prompts` registry used at prompt-assembly time.
3. The ephemeral listener deletes the metadata entry and clears the registry entry after the next generation ends.

## Narrator / system-style injection

### What it is

A persistent omniscient-voice message inserted into the transcript. SillyTavern core provides `/sys` (alias `/nar`) for a manually authored narrator message, `/sysgen` to generate one via the LLM, and `/sysname` to set the display name.

### Creation

`public/scripts/slash-commands.js:1328`:

```javascript
SlashCommandParser.addCommandObject(SlashCommand.fromProps({
    name: 'sys',
    rawQuotes: true,
    callback: sendNarratorMessage,
    aliases: ['nar'],
    returns: t`Optionally the text of the sent message, if specified in the "return" argument`,
    ...
}));
```

The message builder is `sendNarratorMessage`. `public/scripts/slash-commands.js:6019`:

```javascript
export async function sendNarratorMessage(args, text) {
    text = String(text ?? '');
    const name = args.name ?? (chat_metadata[NARRATOR_NAME_KEY] || NARRATOR_NAME_DEFAULT);
    // Messages that do nothing but set bias will be hidden from the context
    const bias = extractMessageBias(text);
    const isSystem = bias && !removeMacros(text).length;
    const compact = isTrueBoolean(args?.compact);

    const message = {
        name: name,
        is_user: false,
        is_system: isSystem,
        send_date: getMessageTimeStamp(),
        mes: substituteParams(text.trim()),
        force_avatar: system_avatar,
        extra: {
            type: system_message_types.NARRATOR,
            bias: bias.trim().length ? bias : null,
            gen_id: Date.now(),
            isSmallSys: compact,
            api: 'manual',
            model: 'slash command',
        },
    };
    ...
}
```

`/sysgen` is the same end result but first generates the text via `generateQuietPrompt`. `public/scripts/slash-commands.js:5700`:

```javascript
export async function generateSystemMessage(args, prompt) {
    $('#send_textarea').val('')[0].dispatchEvent(new Event('input', { bubbles: true }));

    if (!prompt) {
        console.warn('WARN: No prompt provided for /sysgen command');
        toastr.warning(t`You must provide a prompt for the system message`);
        return '';
    }

    const trim = isTrueBoolean(args?.trim?.toString());

    // Generate and regex the output if applicable
    const toast = toastr.info(t`Please wait`, t`Generating...`);
    const message = await generateQuietPrompt({ quietPrompt: prompt, trimToSentence: trim });
    toastr.clear(toast);

    return await sendNarratorMessage(args, getRegexedString(message, regex_placement.SLASH_COMMAND));
}
```

The display name defaults to `"System"` and can be changed per-chat with `/sysname`. `public/scripts/slash-commands.js:3768`:

```javascript
const NARRATOR_NAME_KEY = 'narrator_name';
const NARRATOR_NAME_DEFAULT = 'System';
```

### Storage role

A narrator message is a **real chat row** in `chat[]`. It is persisted with the rest of the chat via `saveChatConditional`. It is not a separate table or metadata blob.

It is `is_user: false`, usually `is_system: false` (unless the content is only a bias string), and carries `extra.type === system_message_types.NARRATOR`. `system_message_types` is defined in `public/scripts/system-messages.js:18`:

```javascript
export const system_message_types = {
    HELP: 'help',
    WELCOME: 'welcome',
    EMPTY: 'empty',
    GENERIC: 'generic',
    NARRATOR: 'narrator',
    COMMENT: 'comment',
    SLASH_COMMANDS: 'slash_commands',
    FORMATTING: 'formatting',
    HOTKEYS: 'hotkeys',
    MACROS: 'macros',
    WELCOME_PROMPT: 'welcome_prompt',
    ASSISTANT_NOTE: 'assistant_note',
    ASSISTANT_MESSAGE: 'assistant_message',
};
```

A message can also be converted to narrator via `/messagerole system`. `public/scripts/slash-commands.js:5841`:

```javascript
message.extra = message.extra || {};
if (role === 'system') {
    message.extra.type = system_message_types.NARRATOR;
} else {
    delete message.extra.type;
}
message.is_user = role === 'user';
```

### Rendering in the model's history

For OpenAI/chat-completion backends, `setOpenAIMessages` maps narrator rows to `role: 'system'`. `public/scripts/openai.js:580`:

```javascript
// 100% legal way to send a message as system
if (chat[j].extra?.type === system_message_types.NARRATOR) {
    role = 'system';
}
```

It also suppresses the usual name prefix for narrator rows. `public/scripts/openai.js:586`:

```javascript
switch (oai_settings.names_behavior) {
    case character_names_behavior.NONE:
        break;
    case character_names_behavior.DEFAULT:
        if ((selected_group && chat[j].name !== name1) || (chat[j].force_avatar && chat[j].name !== name1 && chat[j].extra?.type !== system_message_types.NARRATOR)) {
            content = `${chat[j].name}: ${content}`;
        }
        break;
    case character_names_behavior.CONTENT:
        if (chat[j].extra?.type !== system_message_types.NARRATOR) {
            content = `${chat[j].name}: ${content}`;
        }
        break;
    ...
}
```

For text-completion / non-OAI backends, `formatMessageHistoryItem` omits the speaker prefix for narrator rows. `public/scripts/script.js:5774`:

```javascript
function formatMessageHistoryItem(chatItem, isInstruct, forceOutputSequence) {
    const isNarratorType = chatItem?.extra?.type === system_message_types.NARRATOR;
    const characterName = chatItem?.name ? chatItem.name : name2;
    const itemName = chatItem.is_user ? chatItem.name : characterName;
    const shouldPrependName = !isNarratorType;

    ...
    let textResult = chatItem?.name && shouldPrependName ? `${itemName}: ${chatItem.mes}\n` : `${chatItem.mes}\n`;
```

### Display-only system/comment messages

`system_message_types.COMMENT` and `GENERIC` messages are created with `is_system: true` and are **filtered out** of the prompt context. `public/scripts/script.js:4434`:

```javascript
let coreChat = chat.filter(x => !x.is_system || (canUseTools && Array.isArray(x.extra?.tool_invocations)));
```

`/comment` creates such a row. `public/scripts/slash-commands.js:6112`:

```javascript
async function sendCommentMessage(args, text) {
    const compact = isTrueBoolean(args?.compact);
    const message = {
        name: COMMENT_NAME_DEFAULT,
        is_user: false,
        is_system: true,
        send_date: getMessageTimeStamp(),
        mes: substituteParams(String(text ?? '').trim()),
        force_avatar: comment_avatar,
        extra: {
            type: system_message_types.COMMENT,
            gen_id: Date.now(),
            isSmallSys: compact,
            api: 'manual',
            model: 'slash command',
        },
    };
    ...
}
```

So SillyTavern distinguishes three things:

- `NARRATOR` — real chat row, sent to model as `system`.
- `COMMENT` / `GENERIC` — real chat row, not sent to model, display-only.
- Extension prompts (injects, author's note) — not chat rows, prompt-time only.

### UI display

The narrator message uses the system avatar (`system_avatar`) and the configured name. In `updateMessageElement` the DOM element gets `type="narrator"`. The default stylesheet does not define a unique narrator color; the visual distinction comes from the system avatar, the name, and the `type` attribute. The `compact` flag adds the `smallSysMes` class.

## Impersonate

### What it is

`/impersonate [optional prompt]` triggers `Generate('impersonate', ...)` and asks the model to write the next turn as the active user persona (`{{user}}`). The generated text is written to the input textarea for review; the user must explicitly send it.

### Slash command definition

`public/scripts/slash-commands.js:344`:

```javascript
SlashCommandParser.addCommandObject(SlashCommand.fromProps({
    name: 'impersonate',
    callback: async function (args, prompt) {
        const options = prompt?.toString()?.trim() ? { quiet_prompt: prompt.toString().trim(), quietToLoud: true } : {};
        const shouldAwait = isTrueBoolean(args?.await?.toString());
        const outerPromise = new Promise((outerResolve) => setTimeout(async () => {
            try {
                await waitUntilCondition(() => !is_send_press && !is_group_generating, 10000, 100);
            } catch {
                console.warn('Timeout waiting for generation unlock');
                toastr.warning(t`Cannot run /impersonate command while the reply is being generated.`);
                return '';
            }

            // Prevent generate recursion
            $('#send_textarea').val('')[0].dispatchEvent(new Event('input', { bubbles: true }));

            outerResolve(new Promise(innerResolve => setTimeout(() => innerResolve(Generate('impersonate', options)), 1)));
        }, 1));
        ...
    }
    ...
}));
```

### Persona target

There is **no persona picker**. The target is always the active user persona, represented by the `{{user}}` macro and `name1`. The prompt template is `oai_settings.impersonation_prompt`. `public/scripts/openai.js:104`:

```javascript
const default_impersonation_prompt = '[Write your next reply from the point of view of {{user}}, using the chat history so far as a guideline for the writing style of {{user}}. Don\'t write as {{char}} or system. Don\'t describe actions of {{char}}.]';
```

### Prompt builder and position

In `preparePromptsForChatCompletion`, the impersonation prompt is built as a `system` message with identifier `'impersonate'`. `public/scripts/openai.js:1362`:

```javascript
const impersonationPrompt = oai_settings.impersonation_prompt ? substituteParams(oai_settings.impersonation_prompt) : '';

// Create entries for system prompts
const systemPrompts = [
    // Ordered prompts for which a marker should exist
    { role: 'system', content: formatWorldInfo(worldInfoBefore), identifier: 'worldInfoBefore' },
    { role: 'system', content: formatWorldInfo(worldInfoAfter), identifier: 'worldInfoAfter' },
    { role: 'system', content: charDescription, identifier: 'charDescription' },
    { role: 'system', content: charPersonalityText, identifier: 'charPersonality' },
    { role: 'system', content: scenarioText, identifier: 'scenario' },
    // Unordered prompts without marker
    { role: 'system', content: impersonationPrompt, identifier: 'impersonate' },
    { role: 'system', content: quietPrompt, identifier: 'quietPrompt' },
    { role: 'system', content: groupNudge, identifier: 'groupNudge' },
    { role: 'assistant', content: bias, identifier: 'bias' },
];
```

The optional user-provided slash prompt becomes the `quietPrompt` system message. Both are collected into `controlPrompts`, which is appended **after** the chat history and dialogue examples. `public/scripts/openai.js:1213`:

```javascript
const controlPrompts = new MessageCollection('controlPrompts');

const impersonateMessage = await Message.fromPromptAsync(prompts.get('impersonate')) ?? null;
if (type === 'impersonate') controlPrompts.add(impersonateMessage);

// Add quiet prompt to control prompts
// This should always be last, even in control prompts. Add all further control prompts BEFORE this prompt
const quietPromptMessage = await Message.fromPromptAsync(prompts.get('quietPrompt')) ?? null;
if (quietPromptMessage && quietPromptMessage.content) {
    ...
    controlPrompts.add(quietPromptMessage);
}
```

`public/scripts/openai.js:1336`:

```javascript
chatCompletion.freeBudget(controlPrompts);
if (controlPrompts.collection.length) chatCompletion.add(controlPrompts);
```

So the impersonation instruction is a **trailing `system` message**, not a user message, and not prepended before the history.

### Interaction with system prompt / persona card / output format

`/impersonate` does **not** suppress the main system prompt, character description, personality, scenario, world info, or jailbreak/output-format prompts. The only special casing found is:

- Group nudge is skipped. `public/scripts/openai.js:890`:

  ```javascript
  const noGroupNudgeTypes = ['impersonate'];
  if (selected_group && prompts.has('groupNudge') && !noGroupNudgeTypes.includes(type)) {
      groupNudgeMessage = await Message.fromPromptAsync(prompts.get('groupNudge'));
      chatCompletion.reserveBudget(groupNudgeMessage);
  }
  ```

- `force_name2` is forced to `false`. `public/scripts/script.js:4492`:

  ```javascript
  if (isImpersonate) {
      force_name2 = false;
  }
  ```

- No user message is saved for the turn, and no attachments are added. `public/scripts/script.js:4340`:

  ```javascript
  if (type !== 'regenerate' && type !== 'swipe' && type !== 'quiet' && !isImpersonate && !dryRun && !depth) {
      is_send_press = true;
      textareaText = String($('#send_textarea').val());
      $('#send_textarea').val('')[0].dispatchEvent(new Event('input', { bubbles: true }));
  }
  ```

  `public/scripts/script.js:4380`:

  ```javascript
  const noAttachTypes = [
      'regenerate',
      'swipe',
      'impersonate',
      'quiet',
      'continue',
  ];
  ```

### Saving the result

The generated text is **not saved as a chat message**. The `StreamingProcessor` clears the input box at generation start and writes each streamed chunk into the textarea. `public/scripts/script.js:3570`:

```javascript
async onStartStreaming(text) {
    ...
    if (this.type == 'impersonate') {
        this.sendTextarea.value = '';
        this.sendTextarea.dispatchEvent(new Event('input', { bubbles: true }));
    } else {
        await saveReply({ type: this.type, getMessage: text, fromStreaming: true });
        ...
    }
    ...
}
```

`public/scripts/script.js:3616`:

```javascript
if (isImpersonate) {
    this.sendTextarea.value = processedText;
    this.sendTextarea.dispatchEvent(new Event('input', { bubbles: true }));
}
```

When streaming finishes, `finalizeIntermediaryMessage` emits `IMPERSONATE_READY` instead of saving a message. `public/scripts/script.js:3739`:

```javascript
if (this.type !== 'impersonate') {
    await eventSource.emit(event_types.MESSAGE_RECEIVED, this.messageId, this.type);
    await eventSource.emit(event_types.CHARACTER_MESSAGE_RENDERED, this.messageId, this.type);
} else {
    await eventSource.emit(event_types.IMPERSONATE_READY, text);
}
```

The user can then edit and send the text normally.

### Does it replace or augment the narrator/AI voice?

It **replaces** the AI character voice for one turn by instructing the model to write as `{{user}}`. However, it does so as an appended system instruction while keeping the full character/world context intact. The generated text does not become a narrator or assistant message; it becomes pending user input.

## What is portable to chronicler_engine

| ST core surface | chronicler_engine mapping | Notes |
|---|---|---|
| `/inject` ephemeral guide | Add a transient `Guide`/`Steering` layer to `LayerRenderer`, driven by a `PromptContext` field (e.g. `generation_guide: Option<String>`, `guide_role`, `guide_depth`). Render it as a final `system`/`user`/`assistant` message after `History`/`User` layers. Do **not** persist it as a `MessageEntry`. | ST supports depth-keyed insertion and three roles; chronicler's current `History` layer is a rolling transcript appended at the end, so depth would be a new concept. |
| Narrator (`/sys`) | Add a `Narrator` variant to `domain::model::state::message_types::MessageType` (current variants are `Narration`/`Dialogue`/`System`/`Input`). Persist narrator rows in history; in `LayerRenderer::render_history_layer` render them as `system` turns with no speaker prefix. UI can style them distinctly (ST uses system avatar + `type="narrator"`). | Aligns with Marinara's `narrator` role, not with GG-Extension (which has no narrator row). |
| Author's Note (`/note`) | If chronicler wants a persistent-but-not-in-history steering layer, add a non-ephemeral depth-keyed injection that lives only in `PromptContext`/preset, similar to ST's `setFloatingPrompt`. This is mutually exclusive with "it is a message row." | A design choice for ticket 04. |
| `/impersonate` | Add an `impersonate` flag plus optional prompt override to `PromptContext`. `LayerRenderer` appends an impersonation system instruction built from the active persona (using the existing `{{user}}` macro). The pipeline writes the generated text to a pending input buffer rather than saving it as a `MessageEntry`. | Diverges from Marinara, which saves as a `user` message and suppresses normal preset sections. ST core keeps all context and writes to the input box for review. |

### Concrete chronicler integration points

- `src/application/prompting/types.rs` — add `PromptLayer::Guide` and/or extend `PromptContext` with `generation_guide`, `impersonate_prompt`, and flags.
- `src/application/prompting/assembler.rs` — in `LayerRenderer::render_and_fit`, render the guide/impersonate layer after history and user input, then let `fit_messages_to_context` enforce the budget.
- `src/domain/model/state/message_types.rs` — add `MessageType::Narrator`. History rendering can treat `Narrator` entries like `System` for the LLM but style them separately in the UI.
- Retry / swipe replay: if chronicler wants regeneration replay, store the guide/impersonate metadata in the generated message/swipe extra (Marinara's `generationReplay` pattern). If not, require the caller to re-supply the guide each time (GG-Extension pattern).

### Key divergences from the other researched systems

| Topic | Marinara | GG-Extension | ST core |
|---|---|---|---|
| Transient guide | Append as final `system` message in-memory only | `/inject ephemeral=true` in chat metadata, flushed after one generation | Same as GG, but native to core; also supports `before`/`after`/`chat` positions and role selection |
| Narrator | Real `narrator` row → `system` | **none** | Real chat row with `extra.type === 'narrator'` → `system` |
| Author's note | Not traced | Non-ephemeral `/inject` persistent guides | Native `/note` extension prompt, depth-keyed, not a message row |
| Impersonate instruction | Final `user` message | Direct call: final `user` message; slash path delegates to ST core | Trailing `system` control prompt |
| Impersonate context | Suppresses normal preset/character sections unless impersonate preset selected | Keeps full active preset/character context | Keeps full active preset/character context |
| Impersonate output | Saved as `user` message | Written to input box for review | Written to input box for review |

Ticket 04 must decide which of the three impersonate semantics to adopt, because they differ on role, context suppression, and output handling.

## Open questions / gaps

- **Non-OpenAI path.** This trace focused on the OpenAI/chat-completion prompt manager. The text-completion path (`getTextGenGenerationData`, `createRawPrompt`) was not fully traced; it may treat `/inject` positions and impersonate differently.
- **Prompt-manager ordering for `before`/`after` injects.** We confirmed that unknown extension prompts are merged into the prompt collection, but the exact precedence of a user-defined `/inject` relative to `main`, `jailbreak`, `nsfw`, etc. depends on the user's prompt-manager configuration and was not exhaustively traced.
- **World-Info `scan=true` effect.** The `scan` flag is stored and forwarded to `setExtensionPrompt`, but the exact keyword/semantic scan code that consumes it was not traced.
- **Narrator UI distinctiveness.** The default stylesheet does not define a unique narrator bubble style; distinction comes from the system avatar, the configurable name, and the `type="narrator"` DOM attribute. It is unclear whether a dedicated narrator rendering class exists in community themes.
- **Server-side conversion.** `src/endpoints/backends/chat-completions.js` and `src/prompt-converters.js` were not inspected; prompt assembly for OpenAI backends is primarily client-side in `public/scripts/openai.js`.
- **Impersonate prompt interaction.** The default impersonation prompt and the optional slash prompt are emitted as two separate `system` messages in `controlPrompts` (default first, optional last). It is untested whether models treat the ordering as intended.
- **`/sysgen` quiet generation internals.** `/sysgen` calls `generateQuietPrompt` and then `sendNarratorMessage`, but the quiet-generation path itself was not traced in detail.
