# Product

<!-- impeccable:product-schema 1 -->

## Platform

web

## Users

Now: one person, the author. The author uses the dashboard in two equal ways:

- **Player.** The author reads and steers the story of a Game, often for a long time. Sometimes the author watches an Autoplay run.
- **Tuner.** The author edits Worlds, prompt presets, Connections and settings. The author reads LLM messages to find why a generation went wrong.

Later: other players. The goal is a public release, because Chronicler is a game. A released player plays first and tunes less. How much tuning a released player does is not decided.

## Product Purpose

Chronicler Engine is an interactive fiction engine. An LLM writes the story of each Game. The engine keeps the state of each Game in storage. Success means that the player stays inside the story, and that the tuner can see and fix what the model did.

## Positioning

Chronicler belongs to the same family as SillyTavern and Marinara Engine. The author models the product on these two applications. Chronicler is different because the engine, not the model, keeps the game state. A World is an authored template with a map of Rooms, Characters and Triggers. After each generation, the Quantifier reads the new text and records the Characters present and the destination of the Persona. A Trigger fires an event Narration when the state of a Game meets its condition.

## Operating Context

- The server runs locally. The player opens the dashboard in a desktop browser.
- Each model call goes to a Connection: a provider, an address and a model. A turn costs money and takes several seconds.
- Play is a long reading session. The story log grows to many Messages in one Game.
- An Autoplay run plays the Persona by itself for a set number of Cycles while the player watches.

## Capabilities and Constraints

- **Tabs:** Game, Settings, Prompt Presets, Worlds, Games and LLM Messages.
- **Game tab:** the story log, the location picture and the portraits of the Characters present. The Action input with slash commands (`/guide`, `/impersonate`, `/options`). Swipes, Retry, Options, Autoplay and the status of the Busy state.
- **Rendering:** the server renders every part of the screen as an HTML fragment. The Askama templates are in Rust source (`src/adapters/driving/http/`). HTMX loads the parts into a static shell (`assets/index.html`). The styles are plain CSS in `assets/`. No JavaScript framework. The rules of the shell are in `docs/diataxis/reference/frontend/dashboard.md`.
- **Terminology:** product copy uses the names in `CONTEXT.md` (Game, World, Persona, Character, Message, Swipe, Action, Autoplay run) and no competing word.
- **Devices:** desktop first. A phone layout is wanted, but not built yet.
- **Open decisions:** how much of the tuning surface a released player sees. Whether a release needs accounts or stays a local install.

## Brand Commitments

- The name is Chronicler Engine.
- The `CONTEXT.md` names are part of the product voice.
- No visual commitment. The current dark theme is not binding.

## Evidence on Hand

- One shipped World, Redmist Estate (`data/worlds/redmist_estate`), with Characters, Personas and 19 images in `data/images/` (location pictures and Character portraits).
- Prompt presets in `data/prompt_presets/`.
- No players other than the author, no testimonials and no usage data. Future work must not invent them.

## Product Principles

1. **The story comes first.** On the Game tab, the story is the main content, and every control serves it.
2. **Playing and tuning are equal.** The tuning surfaces must make the work of the model visible and easy to correct.
3. **Ready for a stranger.** A new player must understand each screen without the author present.
4. **Desktop first, phone later.** No layout may block a phone layout later.
