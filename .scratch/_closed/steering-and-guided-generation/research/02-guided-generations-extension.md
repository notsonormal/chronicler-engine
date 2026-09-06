# Research: GuidedGenerations-Extension AI Steering

One-line summary: GuidedGenerations-Extension is a SillyTavern client-side extension that implements guided generation (transient `ephemeral` `/inject` of a bracketed instruction, flushed after one generation) and impersonation (a perspective prompt prepended to chat history via a direct LLM call or ST's `/impersonate`, written to the input box for review) — but it has **no narrator/omniscient-voice message type**; its "persistent guides" are non-ephemeral depth-based prompt injections stored in chat metadata, never saved as history rows.

## Repo overview

A SillyTavern v3 extension (manifest `manifest.json`, `loading_order: 100`, `requires: []`). Pure client-side JS/CSS injected into the SillyTavern UI; it owns no server, no storage, no prompt assembler of its own. Every steering action is expressed as a SillyTavern **STScript** slash-command string executed via `context.executeSlashCommandsWithOptions(...)`, or as a direct LLM call through a helper (`scripts/utils/llmClient.js`) that reuses SillyTavern's own `prepareOpenAIMessages` prompt manager.

Downloaded as `main` branch archive (clone blocked by permissions) to `tmp/guided-generations-extension/GuidedGenerations-Extension-main/`. HEAD at fetch: `main`. Structure:

```
index.js                      entry: button wiring, event listeners, auto-trigger orchestration
manifest.json                 ST extension manifest (v3)
prompts.json                  default prompt templates (editable, overridable per-setting)
settings.html                 settings panel markup
scripts/
  guidedResponse.js           🐕 Guided Response button
  guidedSwipe.js              👈 Guided Swipe button
  guidedContinue.js           Guided Continue variant
  guidedImpersonate.js        👤 1st-person impersonate
  guidedImpersonate2nd.js     👥 2nd-person
  guidedImpersonate3rd.js     🗣️ 3rd-person
  inputRecovery.js            restore original input after a guided action
  persistentGuides/
    runGuide.js               generic generate-then-inject runner
    clothesGuide.js  stateGuide.js  thinkingGuide.js
    situationalGuide.js  rulesGuide.js  customAutoGuide.js
    customGuide.js  editGuides*.js  flushGuides.js  showGuides.js
  utils/
    llmClient.js              direct LLM call path (reuses ST prompt manager)
    presetUtils.js  promptManager.js  swipeHelpers.js  groupSelection.js
  tools/                      corrections, spellchecker, editIntros, separatedThinking, tracker
```

The three steering surfaces map to the destination as follows — and one does **not** exist here:

| Destination surface | GG-Extension analogue | Mechanism class |
|---|---|---|
| Guided Generation (transient) | Guided Response / Guided Swipe | `ephemeral=true` `/inject`, flushed post-generation |
| Narrator Action (permanent, in history) | **none** — persistent guides are prompt-time injections, not history rows | (negative finding) |
| Impersonate | Guided Impersonate 1st/2nd/3rd | perspective prompt + direct LLM call or `/impersonate` |

## Guided generation

### What it is

A transient, per-generation instruction the user types into the input box. The extension wraps it in a bracketed prompt, injects it into the chat context as an ephemeral injection, triggers a generation (new message or new swipe), then flushes the injection. The instruction is **never** saved as a chat message.

### Two surfaces, one mechanism

Both Guided Response and Guided Swipe build the same STScript core — only the generation trigger differs.

`scripts/guidedResponse.js` (single-character path):

```javascript
stscriptCommand =
    `// Single character logic|
/inject id=instruct position=chat ephemeral=true scan=true depth=${depth} role=${injectionRole} ${filledPrompt}|
/trigger await=true|
`;
```

`scripts/guidedSwipe.js` (after the same `/inject` line, verbatim):

```javascript
const stscriptCommand = `/inject id=instruct position=chat ephemeral=true scan=true depth=${depth} role=${injectionRole} ${filledPrompt} |`;
```

…then separately calls `context.swipe.right()` to create a new swipe on the last (AI) message:

```javascript
// guidedSwipe.js, generateNewSwipe()
debugLog("[Swipe] Calling context.swipe.right() to trigger new swipe generation...");
await context.swipe.right();
```

### Prompt format / block

The wrapper is plain text in square brackets, identical for response and swipe. `prompts.json`:

```json
"promptGuidedResponse": "[Take the following into special consideration for your next message: {{input}}]",
"promptGuidedSwipe":    "[Take the following into special consideration for your next message: {{input}}]",
"promptGuidedContinue": "[Continue the story based on the following input: {{input}}]"
```

`{{input}}` is substituted with the user's textarea contents via `fillPromptTemplate(promptTemplate, { input: originalInput })` before injection.

### How transience is enforced

Three layers, all in the slash-command layer — none in storage:

1. **`ephemeral=true`** on the `/inject` command. SillyTavern removes ephemeral injections from `chatMetadata.script_injects` after one generation.
2. **Explicit `/flushinject instruct`** in the `finally` block of `guidedSwipe.js`:
   ```javascript
   } finally {
       ...
       debugLog('[Swipe] Cleaning up injection (finally block)');
       await executeSTScriptCommand('/flushinject instruct');
   }
   ```
3. The injection lives under `chatMetadata.script_injects.instruct` — a prompt-time injection, **not** a row in `chat`. Verification loop in `guidedSwipe.js` polls for it:
   ```javascript
   if (currentContext.chatMetadata?.script_injects?.instruct) {
       debugLog(`[Swipe] Injection found after attempt ${i + 1}.`);
       injectionFound = true;
   ```

### Retry vs new generation

Same inject, different trigger: new generation uses `/trigger await=true`; retry/swipe uses `context.swipe.right()`. There is **no replay-from-stored-metadata** path (contrast Marinara's `generationReplay`) — the caller re-supplies the instruction each time by reading the input box, so the guide is re-injected on every press.

### Auto-trigger interaction (notable)

When SillyTavern's `GENERATION_AFTER_COMMANDS` event fires and auto-guides are enabled, the extension **saves, flushes, and re-injects** the ephemeral `instruct` injection around the auto-guide cycle so a guided response isn't eaten by the auto-guide's own generation (`index.js`, ~lines 1804–1872):

```javascript
if (context?.chatMetadata?.script_injects?.instruct) {
    savedInstructInjection = JSON.parse(JSON.stringify(context.chatMetadata.script_injects.instruct));
    await context.executeSlashCommandsWithOptions("/flushinject instruct", { displayCommand: false, showOutput: false });
}
// ... run thinkingGuide/stateGuide/clothesGuide/customAutoGuide ...
if (savedInstructInjection ...) {
    const { value, depth, scan } = savedInstructInjection;
    const injectionRole = extension_settings[extensionName]?.injectionEndRole ?? 'system';
    const re_inject_command = `/inject id=instruct position=chat ephemeral=true scan=${scan} depth=${depth} role=${injectionRole} ${value}`;
    await context.executeSlashCommandsWithOptions(re_inject_command, { displayCommand: false, showOutput: false });
}
```

### Configurable role and depth

A single global setting picks the injection role (`settings.html`):

```html
<label for="gg_injectionEndRole">Send Injections as:</label>
<select id="gg_injectionEndRole" name="injectionEndRole">
    <option value="system">System</option>
    <option value="assistant">Assistant</option>
    <option value="user">User</option>
</select>
```

Default `system` (`index.js:203`). Depth is per-surface (`index.js:287–296`): `depthPromptGuidedResponse: 0`, `depthPromptGuidedSwipe: 0` — i.e. injected at the very end of chat (depth 0 = after the last message).

## Narrator / system-style injection — **not present**

This is the key negative finding for the ticket. The extension has **no** omniscient-voice narrator message type and **no** permanent instruction saved into chat history. Grepping the whole tree, the only occurrences of "narrator" are in `prompts.json` `editIntros` rewrite templates referring to the **user** as narrator ("where `{{user}}` is the narrator using I/me") — unrelated to a system-voice injection.

What the extension calls "Persistent Guides" is the closest analogue, and it is a **different mechanism class** from the destination's Narrator Action:

### Persistent Guides — non-ephemeral depth injections

`scripts/persistentGuides/runGuide.js` is the generic runner. A guide (a) generates context via a separate `/gen` LLM call (or direct call), (b) injects the result wrapped in a bracketed template via `/inject` **without** `ephemeral=true`, so it persists in `chatMetadata.script_injects` across generations until explicitly `/flushinject <id>`.

Per-guide `finalCommand` (e.g. `clothesGuide.js`):

```javascript
const injectionPrompt = await getPromptValue('persistentGuides.clothesInjection', '');
const finalCommand = `/inject id=clothes position=chat scan=true depth=${depth} role=${injectionRole} ${injectionPrompt} |`;
```

Injection wrappers (`prompts.json` `persistentGuides`):

```json
{
  "clothesInjection":   "[Relevant Informations for portraying characters {{pipe}}]",
  "stateInjection":     "[Relevant Informations for portraying characters {{pipe}}]",
  "thinkingInjection":  "[Characters are currently thinking: {{pipe}}]",
  "situationalInjection":"[Current Situation: {{pipe}}]",
  "rulesInjection":     "[Rules for current scene {{pipe}}]",
  "customAutoInjection":"[{{pipe}}]",
  "movedPreviousInjection":"[Relevant Informations for portraying characters {{pipe}}]"
}
```

`{{pipe}}` is substituted with the generated guide content in `runGuide.js`:

```javascript
const injectionScriptWithContent = injectionScript.replace(/\{\{pipe\}\}/g, capturedGuideOutput);
```

Guide IDs and defaults (`index.js:287–296`): `clothes` (depth 1), `state` (depth 1), `thinking` (depth 0), `situational` (depth 1), `rules` (depth 0), `customAuto` (depth 1).

### Update semantics: `move` vs `flush`

`runGuideScript` takes `previousInjectionAction`:

```javascript
if (previousInjectionAction === 'move') {
    // read existing injection value, re-inject wrapped in movedPreviousInjection
    initCmd = `... /listinjects return=object | /let injections {{pipe}} | ... /inject id=${guideId} position=chat scan=true depth=4 ${movedInjectionPrompt} |`;
} else if (previousInjectionAction === 'flush') {
    initCmd = `/flushinject ${guideId} |`;
}
```

`clothesGuide`/`stateGuide`/`customAutoGuide` use `'move'` (accumulate/refresh in place); `situationalGuide`/`rulesGuide`/`thinkingGuide` use `'flush'` (replace).

### Why this is not the destination's Narrator Action

- **Not persisted as history.** Guides live in `chatMetadata.script_injects`, rendered into the prompt at assembly time, never stored as `chat[]` message rows. There is no distinct message bubble, no display rendering, no "from the omniscient voice" framing.
- **Depth-positioned, not chronologically placed.** A guide sits N messages from the end regardless of when it was created; a narrator message would sit at a fixed point in history.
- **Generated content, not author instruction.** Guides are LLM-generated summaries (clothes/state/thinking) re-injected; the destination's Narrator Action is an author-written directive.

So for the Narrator Action surface, GG-Extension offers **nothing directly portable**. The portable idea is indirect: a non-ephemeral, depth-keyed injection layer is an *alternative design* to a persisted narrator message — it keeps history clean at the cost of no rendering distinction. Ticket 04 should weigh this trade-off explicitly rather than inherit it silently.

## Impersonate

### What it is

The user types a brief outline, picks a perspective (1st/2nd/3rd person), and the extension generates a full message written **as `{{user}}`** (the active user persona) — not as a character. The result lands in the input box for review/editing; it is not auto-saved as a chat message by the extension.

### Two execution paths

`scripts/guidedImpersonate.js`:

```javascript
const useDirectCall = await shouldUseDirectCall(profileValue, presetValue);
if (useDirectCall) {
    const completion = await requestCompletion({
        profileName: profileValue,
        presetName: presetValue,
        prompt: filledPrompt,
        debugLabel: 'impersonate:1st',
        includeChatHistory: true,
        includeIdentityContext: true,
    });
    if (completion && completion.trim() !== '') {
        textarea.value = completion;
        textarea.dispatchEvent(new Event('input', { bubbles: true }));
        setLastImpersonateResult(completion);
    }
} else {
    const stscriptCommand = `/impersonate await=true ${filledPrompt} |`;
    await context.executeSlashCommandsWithOptions(stscriptCommand);
    setLastImpersonateResult(textarea.value);
}
```

- **Direct call** (`shouldUseDirectCall` true when a per-tool profile/preset is configured, `scripts/utils/llmClient.js:723`): builds messages via SillyTavern's prompt manager with `includeChatHistory: true` and `includeIdentityContext: true`, prepends the impersonate prompt as a `user` message at the front of the (newest-first) message list (`llmClient.js:659–670`):
  ```javascript
  const { prompt = '', includeChatHistory = true } = options;
  let resolvedBaseMessages = includeChatHistory
      ? helpers.setOpenAIMessages?.(context?.chat || []) || []
      : [];
  if (rawPrompt) {
      resolvedBaseMessages = [{ role: 'user', content: rawPrompt }, ...(resolvedBaseMessages || [])];
  }
  ```
  Identity context (`includeIdentityContext: true`) keeps character description/personality/scenario and world-info in the params; the impersonate-specific preset is applied.
- **Slash fallback**: `/impersonate await=true {prompt}` delegates to SillyTavern's built-in impersonate (whose internals are ticket 03's scope).

### Prompt format

`prompts.json`:

```json
"promptImpersonate1st": "Write in first Person perspective from {{user}}. {{input}}",
"promptImpersonate2nd": "Write in second Person perspective from {{user}}, using you/yours for {{user}}. {{input}}",
"promptImpersonate3rd": "Write in third Person perspective from {{user}} using third-person pronouns for {{user}}. {{input}}"
```

`{{user}}` = active user persona name; `{{input}}` = the user's outline. The perspective is the only structural difference between the three buttons — a single parameterised prompt would be equivalent.

### Persona target

Always `{{user}}` — the active user persona. There is no character-target impersonation and no persona registry; the extension relies on SillyTavern's persona system for the identity. Character identity context is **included** (not suppressed) — contrast Marinara, which suppresses preset sections during impersonate.

### Output handling — review, not save

The direct-call path writes the completion to the input textarea and stores it as `lastImpersonateResult`; it does **not** push a message into `chat[]`. The slash path likewise fills the input box (ST's `/impersonate` semantics). A toggle-restore guard undoes the last impersonate if the user presses the button again with the generated text still in the box:

```javascript
if (lastGeneratedText && currentInputText === lastGeneratedText) {
    textarea.value = getPreviousImpersonateInput();
    textarea.dispatchEvent(new Event('input', { bubbles: true }));
    return; // Restoration done, exit
}
```

So impersonate **replaces the AI voice for one turn** by writing as the user, and the user explicitly sends (or edits then sends) the result. No preset suppression is performed; the perspective instruction alone carries the intent.

## What is portable to chronicler_engine

Mapped to the current `LayerRenderer` + `PromptContext` + `MessageType` model (paths re-verified against the tree at fetch time):

### Guided generation

- A **transient injection layer** in `LayerRenderer`, keyed by an ephemeral flag on `PromptContext`, removed after one assemble-and-generate cycle. The bracketed wrapper is portable verbatim: `[Take the following into special consideration for your next message: {input}]`.
- Retry (swipe) vs new generation share one inject-then-generate path; the only divergence is the pipeline entry point (`action.rs` vs `retry.rs`/`retrigger.rs`). No replay metadata is needed — the caller re-supplies the guide each press, which is simpler than Marinara's `generationReplay` and avoids a storage schema change.
- Configurable role (system/user/assistant) and depth (recency position) are clean per-generation parameters on `PromptContext`. Default role `system`, default depth 0 (end of history) match GG.
- The auto-trigger save/flush/re-inject dance is a SillyTavern-specific hazard (auto-guides run inside the same generation event). chronicler_engine's pipeline has explicit phases, so this hazard does not port — but ticket 04 should confirm no equivalent re-entrancy exists in the action pipeline.

### Narrator Action

- **Nothing directly portable.** GG has no narrator. The indirect contribution is the *alternative design* of non-ephemeral depth-keyed injections (persistent guides) as a history-clean substitute for a persisted narrator message. Ticket 04 should decide between (a) a new `MessageType::Narrator` persisted in history (Marinara's approach) and (b) a non-ephemeral steering injection layer (GG's approach) — they are mutually exclusive on the "is it a message?" question.
- The bracketed-wrapper convention (`[Current Situation: ...]`, `[Rules for current scene ...]`) is a reusable rendering hint if chronicler chooses injection-style narrator content, but the destination specifies distinct UI rendering, which injections cannot provide.

### Impersonate

- An `impersonate` request flag that (a) includes full chat history + identity context, (b) appends a perspective prompt as the final user turn, (c) writes the result to the input buffer for user review rather than auto-saving. The 1st/2nd/3rd-person perspective is a single enum parameter on the prompt, not three code paths.
- **Divergence from Marinara to flag at 04:** GG writes to the input box for review; Marinara saves as a `user` message directly. GG's "review before send" is a UX choice that costs an extra user gesture but prevents accidental bad impersonations from entering history. chronicler should pick one explicitly.
- GG does **not** suppress preset sections during impersonate (Marinara does). Ticket 04 should decide whether chronicler's impersonate suppresses the narrator/scene sections or relies on the perspective instruction alone — the two researched repos disagree.

## Open gaps

- **SillyTavern `/impersonate` internals** not traced — GG delegates to it in the slash fallback. That is ticket 03's scope (SillyTavern core). Until 03 lands, the slash-path impersonate semantics are inferred from GG's usage, not verified in ST source.
- **`scan=true` keyword-scan behaviour** is SillyTavern-internal and not traced. GG always sets it `true`; the effect on prompt assembly is undocumented here.
- **`position=chat` vs other positions** (e.g. `relative`, `absolute`) not enumerated — GG uses only `position=chat`. Whether chronicler's equivalent needs a position concept beyond "end of history" is unresolved.
- **Direct-call preset content** for impersonate (`presetImpersonate1st`) is user-configured; its default output-format content was not inspected. GG ships no default preset for impersonate (the internal helper preset is hidden for impersonate per README), so impersonate runs against the user's active preset.
- **Per-tool profile/preset switching** (`shouldUseDirectCall`, `llmClient.js`) is an ST-specific affordance with no chronicler analogue; noted but not portable.
