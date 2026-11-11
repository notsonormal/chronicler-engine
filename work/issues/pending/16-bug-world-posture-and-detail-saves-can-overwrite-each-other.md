# The World detail save and the World posture save can overwrite each other

Source: ticket 04 of the state-machine map, D5 ([catalogue](../../wayfinder-maps/state-machines-next/assets/catalogue.md), row D5; [dispositions](../../wayfinder-maps/state-machines-next/assets/04-dispositions.md))

## The defect

Both World write routes read the whole World, change some fields in memory, and write the whole row back. The read and the write are two separate `Storage` calls with no lock or version between them:

- The detail save, `update_world_handler` (`src/adapters/driving/http/worlds/handlers/worlds.rs:165-217`): `get_world`, then `update_world`.
- The posture save, `update_world_posture_handler` (`worlds.rs:236-297`): `get_world`, then `update_world`.

`update_world` writes every World column and the map (`src/adapters/driven/storage/worlds.rs:297-335`). So if two requests run at the same time, the one that writes last restores the old values of the fields that the other one changed.

The Edit dialog starts both: the three posture selects save on change, and the form saves on submit (`src/adapters/driving/http/worlds/templates/worlds.rs`, `hx-post` on the selects and on the form). Two posture requests can also finish out of order. Each one posts all three selects, so the last write wins and the screen can show a different triple from the stored row.

Compare the settings: `Storage::update_settings` does the read, the change and the write under one backend lock (`src/adapters/driven/storage/settings.rs:104-116`).

## Impact (inferred)

Low. The window between the read and the write is short, and one person uses the dialog. No test holds a request between the two calls, so the loss is not shown. If it happens, a posture change or a detail change is lost with no error, and the dashboard shows the saved flag.

## Done when

- A posture save writes only the three posture columns, and a detail save does not write them. Or both saves run the read and the write under one lock. The choice is made with bug 04 (the map write outside a transaction), which changes the same function.
- A test holds a posture save between its read and its write, runs a detail save, and finds both changes in the row.
