# SillyTavern Chat Window Reference

> **Status:** historical/reference, not authoritative. HTML structure capture for comparison; chronicler design is defined in [`docs/system/dashboard.md`](../system/dashboard.md) + [`docs/system/ui_design.md`](../system/ui_design.md).

This document describes the HTML structure of the SillyTavern chat window, verified against the official source code at [SillyTavern/SillyTavern](https://github.com/SillyTavern/SillyTavern) (`release` branch).

## Source Verification

| Element | Source Location | Type |
|---------|----------------|------|
| `#message_template` | `public/index.html` line ~7336 | Static HTML template (cloned by JS) |
| Message rendering | `public/script.js` `updateMessageElement()` (~line 2611) | JavaScript |
| Message adding | `public/script.js` `addOneMessage()` (~line 2539) | JavaScript |
| Character tags | `public/scripts/tags.js` `applyCharacterTagsToMessageDivs()` | JavaScript (runtime) |
| Media templates | `public/index.html` lines ~2850-2950 | Static HTML templates |

## How Messages Are Built

SillyTavern uses a **jQuery HTML template** pattern. The `#message_template` in `index.html` is cloned via `messageTemplate.clone()` (stored at `public/script.js:447`), then populated by `updateMessageElement()` which sets attributes, fills text, and attaches event listeners. Messages are appended to `#chat` by `addOneMessage()`.

## Top-Level Structure

```
#sheld                          ← Draggable container (static HTML)
├── #sheldheader                ← Drag handle (fa-grip icon)
├── #chat                       ← Message list container (messages injected by JS)
│   ├── .mes (message 0)        ← Character message (cloned from template)
│   ├── .mes (message 1)        ← User message
│   └── .mes (message 2)        ← Character message (last)
└── #form_sheld                 ← Input form container (static HTML)
    ├── #dialogue_del_mes       ← Delete confirmation dialog
    └── #send_form              ← Message input form
```

## Message Template (`#message_template`)

The core template at `public/index.html:7336-7415` defines the base structure. Attributes like `mesid`, `ch_name`, `is_user`, `is_system`, `swipeid`, `timestamp`, `type`, and `force_avatar` are set dynamically by `updateMessageElement()` (`script.js:2642-2653`).

### `.mes` Attributes

| Attribute | Set By | Template Default | Runtime Value |
|-----------|--------|------------------|---------------|
| `mesid` | `updateMessageElement()` | `""` | Message index (integer) |
| `ch_name` | `updateMessageElement()` | `""` | `mes.name` |
| `is_user` | `updateMessageElement()` | `""` | `mes.is_user` (boolean) |
| `is_system` | `updateMessageElement()` | `""` | `!!mes.is_system` |
| `swipeid` | `updateMessageElement()` | `""` | `mes.swipe_id ?? 0` |
| `bookmark_link` | `updateMessageElement()` | `""` | `mes?.extra?.bookmark_link` |
| `force_avatar` | `updateMessageElement()` | *(not in template)* | `!!mes.force_avatar` |
| `timestamp` | `updateMessageElement()` | *(not in template)* | Formatted date string |
| `type` | `updateMessageElement()` | *(not in template)* | `mes.extra?.type ?? ''` |
| `title` | `updateMessageElement()` | *(not in template)* | `mes.title` (if set) |

### Dynamically Added Attributes (Not in Core Template)

| Attribute | Added By | Notes |
|-----------|----------|-------|
| `data-char-tags` | `applyCharacterTagsToMessageDivs()` in `tags.js:2610` | Comma-separated character tag names |
| `data-char-tag-*` | `applyCharacterTagsToMessageDivs()` in `tags.js:2707` | Individual tag attributes (one per tag) |
| `xdc-author_uid` | **Extension** (not in core ST) | Third-party extension attribute |
| `thoughts_rendered` | **Extension** (not in core ST) | Third-party extension attribute |

### Special CSS Classes

| Class | Set By | Purpose |
|-------|--------|---------|
| `lastInContext` | **Extension** (not in core ST) | Marks last message in AI context window |
| `last_mes` | `addOneMessage()` at `script.js:2586-2587` | Most recent message in chat |
| `last_swipe` | **Extension** (not in core ST) | Last swipe variant |
| `fade` | **Extension** (not in core ST) | Fade-in animation |
| `smallSysMes` | `updateMessageElement()` at `script.js:2677` | Small system message (`mes.extra.isSmallSys`) |
| `toolCall` | `updateMessageElement()` at `script.js:2681` | Has tool invocations |

### Message Internal Structure

```
.mes                                          ← Cloned from #message_template
├── .for_checkbox + .del_checkbox             ← Bulk delete selection
├── .mesAvatarWrapper
│   ├── .avatar > img                         ← Avatar (src set at runtime)
│   ├── .mesIDDisplay                         ← "#N" (set by updateMessageElement)
│   ├── .mes_timer                            ← Generation time (e.g., "8.6s")
│   └── .tokenCounterDisplay                  ← Token count (e.g., "123t")
├── .swipe_left.fa-solid.fa-chevron-left      ← Previous swipe button
├── .mes_block                                ← Main content area
│   ├── .ch_name.flex-container
│   │   ├── .name_text                        ← Character name (NOTE: template has "${characterName}" literal)
│   │   ├── .mes_ghost.fa-solid.fa-ghost      ← Ghost icon (invisible to AI)
│   │   └── small.timestamp                   ← Timestamp text + title (API + model name)
│   │   └── .mes_buttons                      ← Action buttons
│   │   │   ├── .extraMesButtonsHint          ← "..." toggle for extra buttons
│   │   │   ├── .extraMesButtons              ← Hidden by default, toggled
│   │   │   │   ├── .mes_translate            ← Translate message
│   │   │   │   ├── .sd_message_gen           ← Generate Image (core)
│   │   │   │   ├── .mes_narrate              ← Narrate/TTS (core)
│   │   │   │   ├── .mes_prompt              ← Custom prompt (core)
│   │   │   │   ├── .mes_hide / .mes_unhide   ← Hide/unhide from prompts
│   │   │   │   ├── .mes_media_gallery        ← Toggle media gallery view
│   │   │   │   ├── .mes_media_list           ← Toggle media list view
│   │   │   │   ├── .mes_embed                ← Embed file/image
│   │   │   │   ├── .mes_swipe_picker         ← Jump to swipe history
│   │   │   │   ├── .mes_create_bookmark      ← Create checkpoint
│   │   │   │   ├── .mes_create_branch        ← Create branch
│   │   │   │   └── .mes_copy                 ← Copy message text
│   │   │   ├── .mes_bookmark                 ← Open checkpoint chat (flag icon)
│   │   │   └── .mes_edit                     ← Edit message (pencil icon)
│   │   └── .mes_edit_buttons                 ← Edit-mode buttons (hidden by default)
│   │       ├── .mes_edit_done                ← Confirm edit
│   │       ├── .mes_edit_copy                ← Copy this message
│   │       ├── .mes_edit_add_reasoning       ← Add reasoning block
│   │       ├── .mes_edit_delete              ← Delete message
│   │       ├── .mes_edit_up                  ← Move message up
│   │       ├── .mes_edit_down                ← Move message down
│   │       └── .mes_edit_cancel              ← Cancel edit
│   ├── details.mes_reasoning_details         ← Collapsible reasoning block
│   │   ├── summary.mes_reasoning_summary
│   │   │   ├── .mes_reasoning_header_block
│   │   │   │   └── .mes_reasoning_header
│   │   │   │       ├── .mes_reasoning_header_title  ← "Thought for some time"
│   │   │   │       └── .mes_reasoning_arrow
│   │   │   └── .mes_reasoning_actions
│   │   │       ├── .mes_reasoning_edit_done
│   │   │       ├── .mes_reasoning_delete
│   │   │       ├── .mes_reasoning_edit_cancel
│   │   │       ├── .mes_reasoning_close_all
│   │   │       ├── .mes_reasoning_copy
│   │   │       └── .mes_reasoning_edit
│   │   └── .mes_reasoning                    ← Reasoning content (empty in template)
│   ├── .mes_text                             ← Message body (filled by getMessageTextHTML)
│   ├── .mes_media_wrapper                    ← Generated images/media
│   ├── .mes_file_wrapper                     ← Attached files
│   └── .mes_bias                             ← Bias/override display
├── .swipeRightBlock.flex-container
│   ├── .swipe_right.fa-solid.fa-chevron-right ← Next swipe button
│   └── .swipes-counter                        ← Swipe counter (e.g., "1/3")
└── [Extension-injected elements below .mes]   ← See Extension section
```

## Message Action Buttons — Core vs Extension

### Core Buttons (in `#message_template`)

| Button | Class | Icon | Purpose |
|--------|-------|------|---------|
| More Actions | `extraMesButtonsHint` | `fa-ellipsis` | Toggle extra buttons |
| Translate | `mes_translate` | `fa-language` | Translate message |
| Generate Image | `sd_message_gen` | `fa-paintbrush` | Image generation |
| Narrate | `mes_narrate` | `fa-bullhorn` | Text-to-speech |
| Prompt | `mes_prompt` | `fa-square-poll-horizontal` | Custom prompt (hidden by default) |
| Hide | `mes_hide` | `fa-eye` | Exclude from prompts |
| Unhide | `mes_unhide` | `fa-eye-slash` | Include in prompts |
| Media Gallery | `mes_media_gallery` | `fa-photo-film` | Toggle media display |
| Media List | `mes_media_list` | `fa-table-cells-large` | Toggle media list |
| Embed | `mes_embed` | `fa-paperclip` | Embed file/image |
| Swipe Picker | `mes_swipe_picker` | `fa-bookmark` | Jump to swipe (hidden by default) |
| Checkpoint | `mes_create_bookmark` | `fa-flag-checkered` | Create checkpoint |
| Branch | `mes_create_branch` | `fa-code-branch` | Create branch |
| Copy | `mes_copy` | `fa-copy` | Copy message text |
| Bookmark | `mes_bookmark` | `fa-flag` | Open checkpoint chat |
| Edit | `mes_edit` | `fa-pencil` | Edit message |

### Extension-Added Buttons (NOT in core template)

These are injected at runtime by third-party extensions. They do NOT appear in the SillyTavern core source code:

| Button | Class | Extension | Purpose |
|--------|-------|-----------|---------|
| Scene Marker End | `vecthare-scene-marker-end` | Vecthare | Mark scene END |
| Scene Marker Start | `vecthare-scene-marker-start` | Vecthare | Mark scene START |
| WTracker | `mes_wtracker_button` | WTracker | World tracker |
| WeatherPack | `mes_weatherpack_button` | WeatherPack | Weather extension |
| Ask AI | `mes_askai_button` | Ask AI | AI assistance |
| Generate Roadway | `mes_magic_roadway_button` | Magic Roadway | Roadway generation |
| Baby Bunny Mode | `CarrotKernel_baby_bunny_button` | CarrotKernel | Process as character sheet |
| Remember | `qvink_memory_remember_button` | Qvink Memory | Toggle long-term memory |
| Force Exclude | `qvink_memory_forget_button` | Qvink Memory | Exclude from memory |
| Edit Summary | `qvink_memory_edit_button` | Qvink Memory | Edit memory summary |
| Summarize | `qvink_memory_summarize_button` | Qvink Memory | AI summarization |

## Input Form (`#form_sheld`)

Verified against `public/index.html` lines ~8022-8060.

```
#form_sheld                                 ← Static HTML
├── #dialogue_del_mes                       ← Bulk delete confirmation
│   ├── #dialogue_del_mes_ok                ← "Delete" button
│   └── #dialogue_del_mes_cancel            ← "Cancel" button
└── #send_form
    ├── #file_form                          ← File attachment (hidden by default)
    │   ├── #file_form_input                ← File input (multiple, hidden)
    │   ├── #embed_file_input               ← Embed file input (multiple, hidden)
    │   └── #file_form_reset                ← Remove file button
    └── #nonQRFormItems
        ├── #leftSendForm
        │   ├── #options_button             ← Options menu (fa-bars)
        │   └── [Extension-injected]        ← Extensions inject here
        ├── #send_textarea                  ← Message input textarea
        │   └── .stih--buttons              ← Input history controls (extension)
        │       ├── .stih--arrows           ← Previous/Next input
        │       └── .stih--menuTrigger      ← Input history popup
        └── #rightSendForm
            ├── #stscript_continue          ← Continue script (play)
            ├── #stscript_pause             ← Pause script (pause)
            ├── #stscript_stop              ← Abort script (stop)
            ├── #mes_stop                   ← Abort request (circle-stop, hidden by default)
            ├── #mes_impersonate            ← AI impersonate (user-secret)
            ├── #mes_continue               ← Continue message (arrow-right)
            └── #send_but                   ← Send message (paper-plane)
```

### Input Form — Extension-Added Elements

These are NOT in the core `index.html` template:

| Element | Extension | Purpose |
|---------|-----------|---------|
| `#fawn-plot-btn` | Fawn's Plot Driver | Plot management |
| `#extensionsMenuButton` | Core extensions menu | Extensions panel toggle |
| `#quickPersona` | Core UI | Quick persona switcher |
| `.stih--buttons` / `.stih--arrows` | Input History extension | Input history navigation |
| `#gg-action-button-container` | Guided Generations | Guided generation tools |
| `#gg-menu-buttons-container` | Guided Generations | GG menu buttons |
| `#gg-qr-container` / `#qr--bar` | Guided Generations | Quick response buttons |
| `#gg-regular-buttons-container` | Guided Generations | Guided impersonate/swipe/continue |

## Key Patterns

### Internationalization (i18n)

Buttons use `data-i18n` attributes for translation:
- `[title]` prefix for tooltip translations: `data-i18n="[title]Send a message"`
- `[data-tooltip]` for tooltip attributes: `data-i18n="[data-tooltip]Open checkpoint chat"`
- Direct text content: `data-i18n="Delete"`
- Default text in template: `data-i18n="Thought for some time"` → "Thought for some time"

### Interactivity

All clickable elements use:
- `class="interactable"` — CSS hover/focus styles
- `tabindex="0"` — Keyboard focusable
- `role="button"` — Accessibility role

**Note:** The core template does NOT include `tabindex` or `role` attributes. These are added by the example HTML's runtime environment or extensions. The template uses bare `<div>` elements with `class` and `title`/`data-i18n` attributes only.

### Flex Layout

Heavy use of flexbox utility classes:
- `flex-container` — Display flex
- `flex1` — Flex: 1
- `justifySpaceBetween` — Justify-content: space-between
- `alignitemscenter` / `alignItemsBaseline` — Align items
- `flexFlowColumn` — Flex-direction: column
- `flexNoGap` / `flexGap5` — Gap control

### Extension Integration

Extensions inject buttons via DOM manipulation at runtime. Extension naming conventions:

| Prefix | Extension |
|--------|-----------|
| `vecthare-*` | Vecthare (scene markers) |
| `CarrotKernel_*` | CarrotKernel (baby bunny mode) |
| `qvink_memory_*` | Qvink Memory |
| `sd_*` | Stable Diffusion (core feature, not extension) |
| `lacommon--*` | LA Common (quick actions) |
| `gg-*` | Guided Generations |
| `mfc--*` | Message Flow Control |
| `stih--*` | Input History |
| `fawn-*` | Fawn's Plot Driver |

### Message Text Format

Message content in `.mes_text` is rendered by `getMessageTextHTML()` → `messageFormatting()` which uses Showdown (Markdown-to-HTML). The output uses:
- `<p>` for paragraphs
- `<em>` for narrative/action text (italicized)
- `<q>` for dialogue (quoted)
- `<br>` for line breaks
- Code blocks with copy buttons (added by `addCopyToCodeBlocks()`)

### Message Editing Flow

The edit functionality is key to understanding how SillyTavern handles text. The critical insight is that **message text is stored in a JavaScript data object, NOT in the HTML**.

#### Edit Mode Activation

When user clicks `.mes_edit` (pencil button), the `messageEdit()` function in `script.js` is called:

```javascript
// script.js line ~8159
const editTextArea = document.createElement('textarea');
editTextArea.id = 'curEditTextarea';
editTextArea.className = 'edit_textarea mdHotkeys';
editTextArea.dataset.macros = '';
messageText.append(editTextArea);

const text = trimSpaces(editMessage.mes || '');  // <-- Gets from DATA STORE
const $editTextArea = $(editTextArea);
$editTextArea.val(text);
```

#### Key Points

1. **Data Store**: Messages are stored in a JavaScript object (`editMessage.mes`), not in the HTML
2. **HTML is Display-Only**: The `.mes_text` div shows rendered HTML for display purposes
3. **Textarea Population**: When entering edit mode, the textarea is populated from the **data store**, NOT from the HTML text content
4. **Save Flow**: On save, textarea value goes back to the data store, then HTML is re-rendered

This is why editing in SillyTavern doesn't corrupt quotes or other markdown - you're editing the original markdown source, not the rendered HTML.

#### Edit Mode Buttons

When edit mode is active, the `.mes_buttons` cluster is replaced with `.mes_edit_buttons`:

| Button | Class | Purpose |
|--------|-------|---------|
| Done/Confirm | `mes_edit_done` | Save changes and re-render |
| Copy | `mes_edit_copy` | Copy message text to clipboard |
| Add Reasoning | `mes_edit_add_reasoning` | Add reasoning block |
| Delete | `mes_edit_delete` | Delete message |
| Move Up | `mes_edit_up` | Reorder message up |
| Move Down | `mes_edit_down` | Reorder message down |
| Cancel | `mes_edit_cancel` | Cancel editing, restore original |

#### Detection of Edit Mode

Other components detect edit mode by checking for the presence of `#curEditTextarea`:

```javascript
// RossAscends-mods.js line ~912
if ($('#curEditTextarea').length) {
    // Don't swipe while in text edit mode
    return;
}
```

### Runtime Attribute Setting

The `updateMessageElement()` function (`script.js:2611-2700`) performs these operations on the cloned template:

1. Sets `.mes` attributes: `mesid`, `swipeid`, `ch_name`, `is_user`, `is_system`, `bookmark_link`, `force_avatar`, `timestamp`, `type`
2. Sets avatar image `src`
3. Sets character name text
4. Sets timestamp text and title (with API + model info)
5. Sets message ID display (`#N`)
6. Sets token counter (if `mes.extra.token_count` exists)
7. Sets generation timer (if `mes.gen_started` and `mes.gen_finished` exist)
8. Updates reasoning UI (`updateReasoningUI()`)
9. Inserts SVG model icon (if `power_user.timestamp_model_icon` enabled)
10. Adds `smallSysMes` class for small system messages
11. Adds `toolCall` class for tool invocations
12. Appends media (`appendMediaToMessage()`)
13. Sets `.mes_text` HTML content
14. Adds copy buttons to code blocks
15. Updates swipe counter for non-user messages
16. Applies character tags (`applyCharacterTagsToMessageDivs()`)

## Cross-Reference with Prompt System

See [`system/prompt_system.md`](../system/prompt_system.md) for the 7-layer prompt construction system. The chat window HTML is the **presentation layer** — it displays the conversation history (Layer 5: Chat History) and user input (Layer 6: User Input). The prompt assembly happens server-side via API calls; the HTML reflects the rendered conversation state.

Key connections:
- `.mes_text` content → becomes part of chat history sent to LLM
- `mes_hide` / `mes_unhide` buttons → control whether a message is included in the prompt
- `mes_ghost` icon → marks message as invisible to AI (excluded from prompts)
- `.tokenCounterDisplay` → shows token count for budget management (see prompt system token budget section)
- `swipeid` / `.swipes-counter` → alternate message variants, each with different content for the prompt

## Differences from Example HTML

The `sillytavern_chat_window_example.html` file contains a **runtime snapshot** of a live SillyTavern instance with extensions loaded. Differences from the core template:

| In Example HTML | In Core Template | Notes |
|-----------------|------------------|-------|
| `tabindex="0"`, `role="button"` on all buttons | Not present | Added by runtime/accessibility layer |
| `class="interactable"` on all buttons | Not present | Added by runtime |
| `data-char-tags`, `data-char-tag-*` attributes | Not present | Added by `tags.js` at runtime |
| `xdc-author_uid` attribute | Not present | Third-party extension |
| `thoughts_rendered` attribute | Not present | Third-party extension |
| `lastInContext`, `fade`, `last_swipe` classes | `last_mes` only | Others added by runtime/extensions |
| Vecthare, CarrotKernel, Qvink Memory buttons | Not present | Third-party extensions |
| LA Common quick actions (`.lacommon--*`) | Not present | Third-party extension |
| Message Flow Control (`.mfc--*`) | Not present | Third-party extension |
| Guided Generations (`#gg-*`) | Not present | Third-party extension |
| Fawn's Plot Driver (`#fawn-plot-btn`) | Not present | Third-party extension |
| Input History (`.stih--*`) | Not present | Extension |
| `${characterName}` in template | Literal string | Template has unresolved template literal |
| SVG model icons in timestamps | Not present | Added by `insertSVGIcon()` when enabled |
| Inline `style` attributes | Not present | Added by runtime for visibility toggling |
