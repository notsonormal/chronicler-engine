# Reference apps: SillyTavern and Marinara Engine

How the two reference applications in `PRODUCT.md` lay out the story, read from their source on 2026-11-01. Chronicler values are measured on the probe server. The characters-per-line values of the two other apps are calculations, not measurements.

## The story text

| | Chronicler | SillyTavern | Marinara Engine |
|---|---|---|---|
| Story column width | 85 % of the story log, no cap (`.log-entry { max-width: 85% }`). 810 px at a 1280 px window | Half the window (`--sheldWidth: 50vw`), centered. 640 px at 1280 px | At most 928 px (`--mari-roleplay-message-column-width: min(58rem, …)`), centered |
| Text size | 14 px, fixed | 15 px × a font scale (`--mainFontSize: calc(var(--fontScale) * 15px)`) | 16 px default |
| Characters per line at 1280 px | About 130 (measured) | About 75–80 (calculated) | About 95–105 (calculated) |
| Reader controls | None | "Chat Width" slider (`power_user.chat_width`, default 50) and a font scale | Chat Font Size 12–48 px, chat position Left, Center or Right |

## The scene

| | Chronicler | SillyTavern | Marinara Engine (Roleplay mode) |
|---|---|---|---|
| Place | Location picture in a fixed right column | Wallpaper behind the chat | Full background image behind the story column, crossfades on change |
| Characters | 80 px portraits under the picture | Avatar beside each message | Avatar beside each message, plus expression sprites on a stage |
| Game state | Portraits and picture only | None by default | Tracker Panel or HUD: date, time, location, present characters, inventory, quests, stats |

## Sources

- SillyTavern: `public/style.css` (`--sheldWidth`, `--mainFontSize`, `#sheld`) and `public/scripts/power-user.js` (`chat_width`, `font_scale`), branch `release`, <https://github.com/SillyTavern/SillyTavern>.
- Marinara Engine: `packages/client/src/styles/globals.css` (`.rpg-chat-area`), `docs/appearance/appearance-settings.md` and `docs/roleplay/getting-started.md`, branch `main`, <https://github.com/Pasta-Devs/Marinara-Engine>.
- Marinara Engine is AGPL-3.0. Take ideas and behaviour from it, never code.
- The repo's own notes: `docs/external_applications/sillytavern_chat_window.md` and `docs/external_applications/marinara_engine.md`.
