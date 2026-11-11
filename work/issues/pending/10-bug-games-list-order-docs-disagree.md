# Two texts say the Games list is ordered by the newest Message, but it is ordered by `updated_at`

Source: ticket 02 of the state-machine map, the second scan (scan 2, 2026-11-09): `scan2-driven.md`, *"Newest Game" ordering: two definitions* ([catalogue](../../wayfinder-maps/state-machines-next/assets/catalogue.md), row E4)

## The defect

`list_games` and the saved-Games page order by `updated_at DESC, id DESC` (`src/adapters/driven/storage/games.rs:18`, `:48`). `insert_message` and `insert_swipe` do not touch `games.updated_at`. `docs/specs/games.md:325` agrees with the code: "newest `updated_at` first".

Two texts say something else:

- `docs/specs/changes.md:33`: "A new or a removed Message raises `games` because the Games list is ordered by the newest Message of each Game."
- The doc comment of `ChangeCounters::kinds_of_write` (`src/adapters/driven/storage/changes.rs:180-181`) says the same.

## Impact (known)

No wrong behaviour today. A new Message raises the `games` number (`changes.rs:184-186`) with a reason that is false, so the dashboard reloads the Games part for nothing. A reader who trusts the two texts expects a Game with a new Message to move to the top of the list. It does not.

## Done when

- `changes.md:33` and the comment at `changes.rs:180` give the real order, or the raise of `games` on a new Message is removed if nothing else needs it.
- Apply ticket 04 decides whether the list and the boot pick must use one rule (behaviour question there). This ticket fixes only the false texts.
