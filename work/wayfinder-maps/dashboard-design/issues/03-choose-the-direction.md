# Choose the direction of the Game tab

Type: prototype
Status: open
Blocked by: none

## Question

Which direction does the Game tab take: a book page, a better chat, or a stage?

## Context

- The critique found that the Game tab reads as a generic LLM chat app, not as a game ([the Game tab critique](../assets/game-tab-critique/report.md), "Design Specificity Verdict").
- The three directions, from the critique's questions:
  - **Book page.** One centred column of prose. Inputs as indented lines that name the Persona. No bubbles and no timestamp on every Message.
  - **Better chat**, like SillyTavern. Keep the bubbles; fix the width, the size and the noise.
  - **Stage**, like Marinara Engine. A large scene picture and a strong state panel around the story.
- Each version also fixes the P1 reading issue in its own way: story text 16–18 px, a line of about 65–75 characters, a centred column. Use a self-hosted reading font where the version needs one (map Notes, "Fonts").
- Show each version with the real story of the probe DB, at 1280×800, with the visual sidebar and the action area in view. Add one shot at 1920 wide, because a measure that grows with the window was the cause of the P1 issue. Keep the phone rule: no version may block a phone layout.
- A version built in a worktree starts its probe server from a copy of the probe DB fixture (map Notes).
- The pick changes `docs/diataxis/explanation/dashboard_design.md`, "The SillyTavern lineage", unless it is "better chat". Record that in the answer, so the build ticket rewrites the section.
- The owner picks from screenshots (map Notes, "How decisions are made"). Use `/prototype` for throwaway versions; inject CSS and JavaScript into the probe page, or build in a worktree. [Reference apps](../assets/reference-apps.md) holds the measurements of the two reference apps.
- Not decided here: the exact type and colour ([Write `DESIGN.md`](04-write-design-md.md)), the action area ([Decide the action area](07-the-action-area.md)), the content of the visual sidebar ([Decide what the visual sidebar shows](08-the-visual-sidebar.md)).

Done when the owner has picked one direction from the screenshots, the screenshots are under `../assets/direction/`, and the answer records the pick and the owner's reasons.
