---
target: the game tab
total_score: 23
max_score: 40
na_heuristics: 
p0_count: 0
p1_count: 3
target_identity: "url:http://127.0.0.1:3001/"
timestamp: 2026-11-01T14-25-57Z
slug: 127-0-0-1
---
Method: dual-agent (A: bf67a70b · B: b202dcf1)

Target: the Game tab, http://127.0.0.1:3001/ (1280x800 and 390x844). Mode: Operate, with a strong Read part.

## Design Health Score

| # | Heuristic | Score | Key Issue |
|---|-----------|-------|-----------|
| 1 | Visibility of System Status | 2 | A generation shows only as a 12px label in the bottom-right corner. The story log does not change while a Job runs. |
| 2 | Match System / Real World | 2 | Engine words reach the player: "Quantifying...", "Generating event...", "Retrigger Event". |
| 3 | User Control and Freedom | 3 | Swipes, Edit, Delete and Autoplay Stop work. A single Job has no Stop. |
| 4 | Consistency and Standards | 3 | Send and Autoplay have identical styles. Retry and "New options" share one icon. |
| 5 | Error Prevention | 2 | One click on Autoplay starts 10 paid Cycles, with no confirmation, next to Send. |
| 6 | Recognition Rather Than Recall | 2 | The cycles field shows a bare "10". The current Room and the Persona have no label. |
| 7 | Flexibility and Efficiency | 2 | Slash commands exist. No key for swipe or Retry. 41 Edit buttons come before the Action input in the tab order. |
| 8 | Aesthetic and Minimalist Design | 3 | Quiet layout. But the story text has the same size as the tab labels, and every Message carries a timestamp. |
| 9 | Error Recovery | 3 | From source only: the failure line names a remedy and keeps the raw text behind Details. |
| 10 | Help and Documentation | 1 | No help on the Game tab, except the slash descriptions and tooltips. |
| **Total** | | **23/40** | **Acceptable** |

## Design Specificity Verdict

**LLM assessment.** The Game tab is a generic LLM-chat shell: left and right bubbles, a timestamp on every Message, and a default dark tool style (system font, blue outlined buttons, grey panels). A player cannot tell from the layout that this is a game. Four details are specific to Chronicler: the green Room heading inside the story, the portraits of the Characters present, Swipes with Retry, and the Autoplay cycle budget. The main difference of the product, the state that the engine keeps, is almost not shown. The Room name is only the alt text of the picture, every Room shows the same estate picture, no label names the Persona, and about 400px of the side column is empty. Marinara Engine shows this kind of state in a tracker panel. Chronicler holds more state and shows less of it.

**Deterministic scan.** 85 findings on the rendered Game tab, in two rules. `line-length`: 78 warnings, every story paragraph (`.log-entry > span.text > p`), about 130 characters per line. Measured: every paragraph is 810px wide at 14px/21px. Cause: `assets/styles.css` gives `.log-entry { max-width: 85% }` and sets no measure on the text. `straggler`: 7 advisory notes (one word alone on the last line), only at the bottom scroll position. The CLI scan of `assets/index.html` found 0, which proves nothing, because the real markup is in Askama templates inside Rust strings. False positives: none. The detector and the design review agree on the reading measure. The detector found nothing else, because hierarchy, status and layout are outside its rules.

**Visual overlays.** The overlay injection worked in Agent B's tab and marked all 78 long paragraphs ("line length too long"). That tab was reloaded afterwards, so no overlay is visible now. The screenshot is `tmp/impeccable/critique/b-overlay.png`. The injected script wrote no console messages.

## Overall Impression

The plumbing is careful: live regions, a real ARIA slash menu, readable dialogue colour. The surface around it is a chat app, not a game. The biggest opportunity is to make the story read like a book and to make the side column show the state that only Chronicler has.

## What's Working

1. **Dialogue stands out.** Quoted speech renders in orange italic (`.log-entry .text q`). In a long Narration, the eye finds the speech at once. SillyTavern players expect this.
2. **The Room heading puts engine state into the story.** "Front Gates", "Library & Study" head the entry where the Persona enters a Room. This is the one place where the tab shows what a chat front-end cannot.
3. **Accessibility plumbing.** The slash menu is a real ARIA combobox. New Narration, Options and status go to separate polite live regions. Every icon button has an `aria-label`, and a skip link exists.

## Priority Issues

**[P1] The story text is too wide and too small for long reading**
- Why it matters: PRODUCT.md says play is a long reading session and the story comes first. At about 130 characters per line, the eye loses the start of the next line. At 14px, an hour of reading tires the eyes. Both reference apps read better: SillyTavern uses half the window at 15px, Marinara Engine caps its column at 928px at 16px.
- Fix: in `assets/styles.css`, give `.log-entry .text` 16–18px with `line-height: 1.6`, and cap the measure at about 68ch. Center the story column, so the measure stays the same in a wider window. Add `text-wrap: pretty`. Consider a reading face for the story text only.
- Suggested command: /impeccable typeset

**[P1] A generation is not visible where the player looks**
- Why it matters: each Job costs money and takes seconds. The player reads in the story log, but the only signal is a 12px label in the far corner, and the story does not change until the Narration lands. A stranger who clicks Send sees the input clear and nothing else. This is the most frequent moment of play.
- Fix: render a pending row at the end of `#story-log` from the Busy state, in player words ("The narrator is writing…", "Updating the scene…"). Move the status next to the input or into that row. Keep the exact step names for the LLM Messages tab. Add Stop for a single Job if the engine can support it.
- Suggested command: /impeccable clarify, then /impeccable animate

**[P1] Autoplay has the same weight as Send, and its budget has no label**
- Why it matters: Autoplay is the most expensive control. It looks exactly like Send, sits 104px from it, and starts 10 Cycles with one click. A stranger cannot tell what "10" means.
- Fix: make Send the only filled, primary button. Group Autoplay and its budget into one secondary control with visible text ("Autoplay · 10 cycles"), away from Send. `#autoplay-form` in `assets/index.html` already allows the regrouping.
- Suggested command: /impeccable layout

**[P2] The side column shows little of the state that the engine keeps**
- Why it matters: PRODUCT.md positions Chronicler by its engine-kept state. A first-time player cannot answer "where am I, who is here, who am I".
- Fix: in the visual sidebar template, add a visible Room name caption, a Persona line ("You play Julian"), larger portraits with readable names, and use the empty column for state. When a Room has no own picture, lead with the Room name instead of a borrowed picture.
- Suggested command: /impeccable layout

**[P2] The phone layout hides content**
- Why it matters: PRODUCT.md wants a phone layout later. At 390px, `.visual-sidebar` clips the portraits (`overflow: hidden`), the Games and LLM Messages tabs are off-screen with no sign, and touch targets are 24px.
- Fix: below 768px, show the Room name and the portraits in one compact row, put the tab bar in a menu or fade its edge, and move Autoplay behind a secondary control.
- Suggested command: /impeccable adapt

## Persona Red Flags

**Alex (power user from SillyTavern):** no arrow keys for swipe or Retry; no key to focus the Action input; Narrator Mode and presets are only on the Games tab.

**Sam (screen reader, keyboard):** 41 buttons all named "Edit" before the Action input; entries carry no speaker name, so an Input and a Narration differ only by side and shade; the input focus style removes the outline (`#command-form input:focus { outline: none }`).

**Casey (phone):** portraits clipped and invisible; two tabs off-screen; 24x24 targets; the "10" stands alone on its own row.

**The player-author who tunes:** a bad Narration has no link to its LLM message; the author must match rows by time on the LLM Messages tab.

**A first-time released player:** nothing says who they play, where they are, or that `/` opens commands; "Autoplay 10", "Retrigger Event" and "Quantifying..." need the author to explain them.

## Minor Observations

- A literal `</br>` shows in the Library & Study Narration (verified). The model output carries markup that the page shows as text.
- A timestamp on every Message is chat noise in a story. Room heading entries add a stray hyphen ("- 23:35").
- The green Room heading is the loudest text on the screen, louder than the prose.
- The disabled previous-Swipe arrow shows at "1 / 1".
- The slash menu writes "persona" in lowercase. `CONTEXT.md` capitalises Persona.

## Questions to Consider

- What if the story log were a page of a book, not a chat: one centred column, Inputs as indented lines with the Persona name, no bubbles, no timestamps?
- What if the side column became the state panel of the engine: Room, Characters present, Persona, last fired Trigger?
- Is Autoplay a button next to Send, or a "watch" mode with its own progress display and Stop?
