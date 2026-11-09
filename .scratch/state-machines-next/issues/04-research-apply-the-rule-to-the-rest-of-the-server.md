# Apply the rule to the rest of the server

Type: research
Status: resolved
Blocked by: 01

## Question

Apply the rule of ticket 01 to each machine of the prompt, the settings and forms, and the storage and boot. For each machine, what is its disposition and the evidence for it? Which product behaviour questions must the owner answer before the disposition holds?

## Machines

Catalogue rows. Ticket 02 can add rows.

- **Prompt.** C3 the call mode (`impersonate` and `guide`), C4 the history cut and the overflow guard, C5 the Agent registration for each Hook.
- **Settings and forms.** D1 Connection slots, D2 text check, D3 the mode preset registry, D4 Allowed Modes of a preset, D5 the World form and its two write routes, D6 the Connection form.
- **Storage and boot.** E1 the migration ladder, E2 the transaction (`Tx`), E3 the seeding precedence, E4 the boot resume-or-create.

## Behaviour questions already known (charting)

1. After the first boot, the database wins for every kind. An edit of a World, Character or Persona file in `data/` never reaches the stored row (scan 2; `bootstrap/load.rs:115-125`). `seed_world` says that it replaces, but boot calls it only for a World that is not stored. Is this what we want, or must a changed file reach a stored row that nobody edited?
2. A malformed `data/settings.json` stops the boot, also when the database already holds the settings. Do we want this?
3. When both Allowed Modes boxes are clear, create reads "both modes" and edit reads "no change". Which meaning do we want, or must it be an error?
4. Must the Connection form show the difference between an empty slot, which follows Narration, and a slot that names the same Connection?

## Behaviour questions added by ticket 02 (scan 2)

5. E1 the migration ladder. No step checks for `user_version > LATEST_VERSION` (`src/adapters/driven/storage/utils/plumbing.rs:17`, `:51-54`). A database that a newer binary wrote opens in an older binary, no step runs, and the older code reads and writes the newer schema. Must the boot refuse such a database?
6. E4 boot resume. The boot checks that the `--persona` argument names a stored Persona (`src/bootstrap/run.rs:86`), but it resumes the newest Game of the World whatever Persona that Game has (`run.rs:88`, `src/bootstrap/init_game.rs:13-22`). The argument then acts only when a new Game is made. Is that what the argument must do?
7. E4 "newest Game". The boot picks the Game with the newest Message (`src/adapters/driven/storage/games.rs:135-149`, `docs/diataxis/reference/storage.md:69-74`). The Games list orders by `updated_at` (`games.rs:18`, `:48`, `docs/specs/games.md:325`), which a rename or a config change moves and a new Message does not. So the top of the list need not be the Game a restart resumes. Must the two rules agree? Bug ticket 06 of `state-machines-work` fixes only the two texts that state a third order.

## To check (from scan 2)

Scan 2 labelled these inferred or unsure. Ticket 02 did not prove them.

- D1, D6: `pin_provider_order = true` with an empty `provider_order` is accepted by the form and by the settings (`src/domain/model/settings.rs:362-364`). The effect on the provider call is unsure (`scan2-domain.md`, *Connection slots*; `scan2-driving.md`, finding 14).
- E1: some ladder steps guard with `column_exists`, and some do not (`scan2-driven.md`, *Schema version ladder*).
- E2: a closure that swallows a failed write and returns `Ok` commits its other writes (`scan2-driven.md`, *Unit of record writes*).
- E3: the settings save uses `INSERT OR REPLACE`, which writes `created_at` again on every save (`src/adapters/driven/storage/settings.rs:69-80`). No reader of the settings `created_at` was found, so the effect is unsure.

## Output

- `assets/04-dispositions.md`: the disposition table in the form that ticket 01 decides (one row for each machine, with the columns of its Answer), with the evidence for each machine.
- A review sketch of each machine whose disposition is not "leave it": a Mermaid state diagram in `assets/04-sketches.md`, drawn from the code as it is today. Mark each arrow that the disposition adds or removes. Ticket 06 shows the owner these sketches, so the owner can check each machine against intent. A sketch is a review aid, not a doc: it never goes into `docs/`.
- The behaviour questions: the four above, checked again, plus any new ones. Each one names its machine and its file:line.
- Each plain defect goes to `.scratch/state-machines-work/issues/` as a ticket. The overflow guard of C4 is already there.

## Answer

Resolved 2026-11-09. Details: [`assets/04-dispositions.md`](../assets/04-dispositions.md) and [`assets/04-sketches.md`](../assets/04-sketches.md).

Of 13 machines, 9 get "leave it" (C3, C4, C5, D1, D5, D6, E2, E3, E4) and 4 get "explicit in the code" (D2, D3, D4, E1).

- **No model-based test, TLA+ model or diagram here.** The four explicit forms are value constraints: an enum in place of the pair of D2, a checked constructor in D4, and a refusal in D3 and E1. None has moves between states, so none has a transition table to generate a test or a diagram from. No machine here has two or more places that make its moves, and no order difference that a client can see is more than inferred. (Reading of the ticket 01 rule by the parent session: "a machine that becomes explicit also gets a model-based test" applies when the explicit form has a transition table.)
- **D2 is the clear case.** The shipped `data/settings.json:70-71` and the default value both hold the pair `Disabled` with auto-check on, and a normaliser hides it on read.
- **D4.** Only a seed file with `"allowed_modes": []` can store an empty Allowed Modes list.
- **D3 and E1 depend on the owner:** questions 9 (refuse an edit that breaks a mode default) and 5 (refuse a database from a newer binary).
- **D5** has a possible lost update between the posture save and the detail save. The path is known, the effect is inferred, so the rule gives "leave it", and the defect is bug 20 of `state-machines-work`.
- **Ten behaviour questions** go to ticket 06. Three are new: 8 (an unknown `agent_type` is swallowed at `bootstrap/wiring.rs:147-151`), 9 (the D3 refusals), 10 (the shipped auto-check value).
- **Four draft work tickets** (A to D) wait for ticket 06. A (D2) is the cheapest and the most certain.
