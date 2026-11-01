# Assessment A: Design Review — Game tab

Target: the Game tab at `http://127.0.0.1:3001/` (tab `BB12B32F`). Mode: Operate, with a strong Read part.
Method: live inspection at 1280x856 and 390x844, plus a source read of `assets/index.html`, `assets/styles.css`, `assets/dashboard-*.js`, `src/adapters/driving/http/templates.rs` and `src/adapters/driving/http/layout/handlers/endpoints.rs`. The detector was not run.

Screenshots (all in `tmp/impeccable/critique/`):

- `a-desktop-initial.png`: 1280 wide, story log at the bottom (last Message, swipe controls).
- `a-desktop-top.png`: top of the story log (Front Gates).
- `a-desktop-middle.png`: middle of the story log (Library & Study).
- `a-desktop-slash.png`: slash menu open after a typed `/`.
- `a-mobile-bottom.png`: 390x844, bottom of the story log.
- `a-games-tab.png`: Games tab, for the consistency check only.

State at inspection: Game "Redmist Estate — 31 Oct 2026 (7)", 41 Messages, Persona Julian, room Mansion Kitchen, status Ready, Options dock empty (height 0). I saw no generation, no failure and no Options. Those states are judged from source, and they are labelled **inferred**.

---

## 1. Design-specificity verdict

**Verdict: a generic LLM-chat shell with a few authored details. A player could not tell from the layout that this is a game.**

- The story log uses the messenger pattern: left and right bubbles, an asymmetric corner radius (`.log-entry.input { border-bottom-right-radius: 4px }`), a timestamp on every Message, and max-width 85% (`a-desktop-top.png`). Any chat front-end uses the same layout. For a long reading session, this layout reads as a chat transcript and not as a story.
- The visual language is a default dark tool UI. It has a system font stack, Google-blue outlined buttons (`rgb(138,180,248)`), `#121212`/`#1f1f1f` greys and Lucide icons. Nothing in the type, colour or composition is specific to interactive fiction or to Redmist Estate.
- **Authored details that are specific to Chronicler:**
  - The inline Room heading (`.location-header`, green) on the entry where the Persona enters a Room. This shows the engine-owned state in the story (`a-desktop-middle.png`: "Library & Study - 23:37").
  - The Characters-present portraits.
  - The Autoplay cycle budget.
  - Swipes with Retry at the end of the arrows.
- **Against SillyTavern / Marinara Engine expectations:**
  - A SillyTavern user expects an avatar and a name on each Message, swipe arrows with a right-arrow-to-generate (present), arrow-key swipes (absent), and quote colouring (present: `q` is orange italic).
  - Marinara Engine users expect a visible scene and state panel. Chronicler holds more state than either product (Room, Characters present, Triggers). But the Game tab shows less of that state than the two products do. The Room name is only in the `alt` of the sidebar image. Every Room shows the same estate exterior (`data/images/` holds one location picture, so this is partly a data limit). The Persona name appears nowhere on the Game tab.
- **Missed opportunity:** the thing that makes Chronicler different (the engine keeps the state) is almost not shown. The sidebar could be the signature of the product. Now it is a picture and two 80px thumbnails above about 400px of empty column (`a-desktop-initial.png`).

## 2. Nielsen heuristics

| # | Heuristic | Score | Key issue |
|---|---|---|---|
| 1 | Visibility of System Status | 2 | The state of a generation is a 12px label in a 240px column at the far bottom-right (`#status-display`, "Ready" in `a-desktop-initial.png`). The story log shows nothing while a Job runs (inferred from `templates.rs`). The current Room is not shown as text anywhere. |
| 2 | Match System / Real World | 2 | `CONTEXT.md` names are used well (Swipe, Autoplay, Persona). But the status leaks engine jargon to the player: "Quantifying...", "Generating event...", "Retrigger Event" (`endpoints.rs::step_label`, `retrigger-btn` title). The pending label "Thinking..." (`dashboard-action-area.js`) is a third word for the same wait. |
| 3 | User Control and Freedom | 3 | Swipes, Edit on every entry (Escape cancels, Ctrl+Enter saves), Delete of the last Message, and Stop for an Autoplay run are good. There is no Stop for a single Job: the docs render Stop only while an Autoplay run holds the Game. |
| 4 | Consistency and Standards | 3 | Mostly consistent. But Send and Autoplay have identical styles (`a-desktop-initial.png`). Retry and "New options" use the same `i-refresh-cw` icon. The Game tab uses icon-only buttons, and the Games tab uses text buttons in three colours (`a-games-tab.png`). |
| 5 | Error Prevention | 2 | Autoplay starts 10 paid Cycles with one click and no confirmation. The click target sits 104px from Send and has the same style. Delete has a native `confirm()`. Send stays enabled with an empty input (verified in the DOM; what the server does with an empty send is not verified). |
| 6 | Recognition Rather Than Recall | 2 | The slash menu shows on `/` and has descriptions (`a-desktop-slash.png`). But the placeholder "Enter an action..." does not say that slash commands exist. The cycles field shows a bare "10" with only a `title`. To know the current Room and the Persona, the player must remember them from far up the log. |
| 7 | Flexibility and Efficiency | 2 | Slash commands with arrow and Enter keys, and Ctrl+Enter in edit, are present. There is no key for swipe, Retry or focus of the Action input. To reach the Action input by keyboard, a player tabs through about 41 Edit buttons first (49 focusable elements in `#game-tab`). |
| 8 | Aesthetic and Minimalist Design | 3 | The layout is quiet and has no decoration noise. But the reading text has the same size as the chrome (14px). A timestamp on every Message is chat noise. The green Room heading is the loudest element on the screen. |
| 9 | Error Recovery | 3 | Inferred from source and docs, not seen live. The status failure line names a remedy and keeps the raw text behind Details (`failure_answer`). The Options failure shows inline in the dock with Details. A draft stays after a refusal. |
| 10 | Help and Documentation | 1 | There is no help on the Game tab. The only help is the slash descriptions and the `title` tooltips. Nothing tells a stranger what an Action is, what Autoplay costs, or what a Swipe is. |
| **Total** | | **23/40** | **Acceptable** |

## 3. Cognitive load

| Item | Result | Evidence |
|---|---|---|
| Single focus | Pass | The story log takes about 80% of the width (`a-desktop-initial.png`). |
| Chunking | Pass | The action bar has 4 controls and a status. |
| Grouping | **Fail** | The cycles field "10" sits between Send and Autoplay with an equal gap on each side. Nothing ties it to Autoplay (`a-desktop-initial.png`). The status sits about 220px away from the controls it describes. |
| Visual hierarchy | **Fail** | Send and Autoplay have equal weight. The story text (14px) has the same size as the tabs (14px). The green Room heading outranks the prose. |
| One thing at a time | Pass | |
| Minimal choices | Pass (borderline) | The action bar has 4 controls. The last entry has up to 5 (see below). |
| Working memory | **Fail** | The current Room, the Persona name and the meaning of "10" must be recalled. The Room heading of the current Room (Mansion Kitchen, offsetTop 7472) can be thousands of pixels above the view. |
| Progressive disclosure | Pass | The slash menu, the Details of a failure and the edit-on-demand work this way. |

**Result: 3 failures. The cognitive load is moderate.**

Decision points with more than 4 visible options:

- The last Message when a Retrigger is allowed has Edit, Delete, Retrigger, previous Swipe, and Retry or next Swipe (5 controls, all 24x24 icons). This is inferred from the `NarrationLogTemplate`; I did not see it, because Retrigger was not shown.
- The Options dock with N Options has N Option buttons, N "Edit before send" buttons and 1 "New options" button (2N+1 targets). With 3 Options, that is 7. This is inferred from `OptionsDockTemplate`; the dock was empty.

## 4. Emotional journey

- **Peak:** a new Narration arrives. The log follows to the bottom only when it was already at the bottom (`dashboard-story-log.js`). The new text has no entrance and no mark of "new" (inferred). So the high point of the loop shows up without any moment of its own.
- **Valley: the wait of several seconds.** After Send, the reassurance is "Thinking...", then "Generating narration...", then "Quantifying...". This is 12px yellow text in the bottom-right corner, outside the reading focus (`#action-status`, `#status-display`, inferred from source). The story log, where the eyes are, does not change. A single Job has no Stop. For a paid call of several seconds, the player cannot see where the answer will show up and cannot cancel it.
- **Valley: Autoplay.** "Autoplay: cycle 3/10 Quantifying..." in the same small corner is the only progress (inferred, `busy_answer`). For a mode where the player only watches, the display of progress is the whole experience, and now it is a footnote.
- **End:** the screen ends on an empty Action input ("Enter an action...") and "Ready". The Options dock (inferred) improves this end because it gives the next step. Without Options, the end of each loop is a blank field.
- **Reassurance at high-stakes moments:** Delete has a confirm. The text-check preview says "Suggestions only — nothing changes unless you choose 'Send with edits'". This is good copy. Autoplay, the most expensive control, has no reassurance and no statement of cost.

## 5. Strengths

1. **Dialogue is readable in long prose.** Quoted speech renders as `<q>` in orange italic at body weight (`.log-entry .text q`). In a long Narration with many lines of speech, the eye finds the speech at once (`a-desktop-top.png`, `a-desktop-middle.png`). SillyTavern players expect this treatment.
2. **The Room heading puts the engine state into the story.** "Front Gates", "Main Courtyard", "Library & Study" each head the entry where the Persona entered the Room (`a-desktop-middle.png`). This is the one place where the Game tab shows what Chronicler does that a chat front-end does not. It is in the reading flow and not in a panel.
3. **The accessibility and state plumbing is careful.** The slash menu is a real ARIA combobox and listbox with `aria-selected`. New Narration, Options and status changes go to separate polite live regions (`#narration-announcer`, `#options-announcer`, `#status-announcer`). Every icon button has an `aria-label`. A skip link exists. Swipe controls show only on the last Message, and Retry takes the place of the next arrow on the last Swipe, as in SillyTavern (`a-desktop-initial.png`: "1 / 1" with the refresh control).

## 6. Priority issues

### [P1] The reading measure is too wide and the text is too small for a long reading session

- **What:** the story text is 14px with a 21px line height, in a box 810px wide. That is about 115–125 characters per line (`a-desktop-initial.png`, `a-desktop-top.png`). The prose uses the same system sans-serif and the same size as the tab bar.
- **Why it matters:** PRODUCT.md says that "Play is a long reading session" and "The story comes first". Lines past about 75 characters make the eye lose the start of the next line. At 14px, a session of an hour tires the eyes. This is the primary task of the primary surface.
- **Fix:** in `assets/styles.css`, set `.log-entry .text` to 16–18px with `line-height: 1.6`. Cap the measure with `max-width: 68ch` on the text, not 85% of the log. Consider a reading face for `.log-entry .text` only, and keep the UI chrome in the system font. Make the story a centred column, so the measure stays fixed when the window gets wider.
- **Suggested command:** `/impeccable typeset`

### [P1] A generation is not visible where the player looks

- **What:** while a Job runs, the only signal is a 12px label in a 240px column at the bottom-right (`#action-status` "Thinking...", then `#status-display` "Generating narration...", "Quantifying..."). The story log does not change until the Narration lands (inferred from `NarrationLogTemplate` and `dashboard-action-area.js`). A single Job has no Stop.
- **Why it matters:** each Job costs money and takes several seconds. The player reads in the centre of the log, and the signal sits in the far corner. A stranger who clicks Send sees the input clear and nothing in the story. This is the most frequent moment of the product, and it is the weakest one.
- **Fix:** add a pending row at the end of `#story-log` (a placeholder Narration with the step in player words, for example "The narrator is writing…" and "Updating the scene…"), rendered by the server from the Busy state. Then a screen load replaces it. Move the status next to the input, or into that row. Add Stop for a single Job if the engine can support it. Replace "Quantifying..." and "Generating event..." with player words, and keep the exact step names for the LLM Messages tab.
- **Suggested command:** `/impeccable clarify`, then `/impeccable animate` for the pending row.

### [P1] The action bar gives Autoplay the same weight as Send, and the budget field has no label

- **What:** `#submit-btn` and `#autoplay-btn` have identical styles (transparent fill, blue border, blue text) and nearly the same size (`a-desktop-initial.png`). Between them is `#autoplay-cycles` with the value "10" and only `title="Cycles to play"`. One click on Autoplay starts 10 Cycles (each Cycle has an Input and a Narration, and each has a Quantifier call) with no confirmation.
- **Why it matters:** this is Error Prevention and Hierarchy at the most expensive control in the dashboard. A stranger cannot tell what "10" means. A player who aims at Send can hit the wrong outlined button. On a phone, the "10" stands alone on its own row (`a-mobile-bottom.png`).
- **Fix:** make Send the only primary control (a filled button). Put Autoplay and its budget in one secondary group with a visible label, for example `Autoplay [10] cycles` in one bordered cluster, or a split button. Move the group away from Send. Show the budget in the button text ("Autoplay 10 cycles"). The `#autoplay-form` and `#autoplay-controls` markup in `assets/index.html` already allow this regrouping.
- **Suggested command:** `/impeccable layout`

### [P2] The visual sidebar shows little of the state the engine keeps

- **What:** the Room name is only in `alt` (`<img alt="Mansion Kitchen" src="Redmist Estate.png">`). Every Room shows the estate exterior, because the World has one location picture (a data limit). The portraits are 80x80 with names in 12px grey. About 400px of the column below them is empty (`a-desktop-initial.png`). The Persona (Julian) is not named anywhere on the Game tab. When the log is scrolled to Library & Study, the sidebar still shows the current state, with no sign that it does not match the passage on screen (`a-desktop-middle.png`).
- **Why it matters:** PRODUCT.md positions Chronicler by its engine-kept state. A first-time player sees a static picture and cannot answer "where am I, who is here, who am I". Working memory fails (see section 3).
- **Fix:** in `VisualSidebarTemplate`, add a visible caption with the Room name above or over the picture. Add a Persona line ("You play Julian"). Make the portraits larger, with names in readable text. Use the empty column for the Characters present and, later, the last event Narration. When the Room has no own picture, show the Room name as the main element and not a borrowed exterior picture.
- **Suggested command:** `/impeccable layout`

### [P2] The phone layout hides the Characters present and most of the tabs

- **What:** at 390x844, `.visual-sidebar` collapses to 103px with `overflow: hidden`. The portraits render at y=741, inside a box that ends at y=623, so they are clipped and not visible (`a-mobile-bottom.png`, measured with `getBoundingClientRect`). The tab bar scrolls sideways (scrollWidth 633 in a 390px bar), and "Games" and "LLM Messages" are off-screen with no sign. The story log is 435px tall. The cycles field takes a full row.
- **Why it matters:** PRODUCT.md wants a phone layout later and says "No layout may block a phone layout later". The current layout is not blocked, but the media queries at `styles.css` lines 1834 and 1854 produce a layout that loses content. A phone is not a target yet, so this is P2.
- **Fix:** below 768px, show the Room name and the portraits in one compact row (picture as a background strip, portraits as 40px chips) and not in a clipped column. Put the tab bar in a menu, or fade its edge to show that it scrolls. Put Autoplay behind a secondary control on phone.
- **Suggested command:** `/impeccable adapt`

## 7. Persona red flags

**Alex (power user, comes from SillyTavern)**

- No key for swipe or Retry. SillyTavern uses ←/→ on the last Message. Here the controls are `.swipe-btn` icons of 24x24 only (`a-desktop-initial.png`).
- No key to focus `#command-input`. Tab order goes through every `.edit-btn` in the log first (49 focusable elements in `#game-tab`).
- Delete uses a native `confirm()` (`hx-confirm="Delete this message?"`), which blocks the window for a low-risk delete of the last Message.
- To change Narrator Mode or the presets of the Game, Alex must go to the Games tab (`a-games-tab.png`: Mode, Perspective, Tense, System, Quantifier, Impersonate, Options selects). None of them are on the Game tab.

**Sam (screen reader, keyboard only)**

- About 41 `Edit` buttons with the same name "Edit" are in the tab order before the Action input. A screen reader lists "Edit, Edit, Edit…", with nothing to tell which Message each one edits.
- Entries carry no speaker. An Input and a Narration differ only by side and background shade (`#272727` against `#1f1f1f`). The story log `div` has no list semantics, and each entry has no heading or label such as "Julian" or "Narrator".
- `#command-form input:focus` sets `outline: none` (`styles.css` line 523) and uses a border change instead. A ring comes back only under `forced-colors`. The visibility of that border change in the normal theme is not verified.
- Good: the live regions announce the new Narration, the Options and the status. The slash menu is a proper combobox.

**Casey (phone)**

- The portraits of the Characters present are clipped and not visible (`a-mobile-bottom.png`).
- "Games" and "LLM Messages" are off-screen in the tab bar.
- The edit, swipe and Retry targets are 24x24, below 44x44.
- Send is in the thumb zone, which is good. Autoplay is centred below an unlabeled "10".

**Project persona: the player-author who also tunes**

- A bad Narration has no path from the entry to its LLM message. The entry template has Edit, Delete, Retrigger and swipe controls, and no "View LLM message" (`NarrationLogTemplate`). The author must switch to the LLM Messages tab and match rows by time.
- The Swipe counter shows "1 / 1" but nothing about which Connection or preset wrote the Swipe.
- The status failure line is clamped to two lines in a 240px column (`#status-display .error-disclosure-text`). Two lines is a narrow window for a tuner who needs to read the cause (inferred).

**Project persona: a first-time released player**

- The first view is a chat transcript with no statement of who the player is, where the Persona is, or what to type. The placeholder "Enter an action..." does not say "What does Julian do?" and does not mention `/`.
- "Autoplay" with a bare "10", "Retrigger Event" (zap icon) and "Quantifying..." need the author to explain them.
- The header shows "Redmist Estate — 31 Oct 2026 (7)", a date-based Display name, and not the story or the Persona.
- A new Game, with only the opening Narration, was not inspected. Its empty state is unknown.

## 8. Minor observations

- A literal `</br>` shows at the end of the Library & Study Narration (`a-desktop-middle.png`). The model output carries markup that the sanitizer escapes and does not remove.
- A timestamp on every Message ("23:35", "23:36"…) is chat-app noise in a story. The Room heading entries format it as "- 23:35", with a hyphen that the other entries do not have.
- The previous-Swipe arrow renders disabled at "1 / 1". The control could be left out until a second Swipe exists.
- The green Room heading (`rgb(166,214,160)`, 17.6px bold) is the most saturated text on the screen, and it is louder than the prose it introduces.
- The header splits "Chronicler Engine" (muted) and the Display name with no separator. On a phone, they wrap onto two lines (`a-mobile-bottom.png`).
- `.image-label` is `aria-hidden` and repeats the `alt`. This is correct, but the label is 12px grey on a tertiary background.
- "Write as the player persona" in the slash menu uses a lowercase "persona". `CONTEXT.md` capitalises Persona.

## 9. Questions to consider

1. What if the story log were a page of a book and not a chat? One centred 68ch column, Inputs as indented lines that carry the Persona name, Narration as plain prose, and no bubbles and no timestamps.
2. What if the sidebar became the state panel of the engine (Room, Characters present, Persona, the last fired Trigger)? That is the one thing SillyTavern cannot show, and here it is the image that the World does not have.
3. Is Autoplay an action next to Send, or a mode that the player enters? A "watch" mode with its own budget, its own progress display and Stop would end the equal weight with Send and give the watcher something to look at.
