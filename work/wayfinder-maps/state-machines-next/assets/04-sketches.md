# Ticket 04: review sketches

A sketch is a review aid. It is drawn from the code as it is today. It is not a doc and never goes into `docs/`. Ticket 06 shows these to the owner.

Marks: a **red state** is a state that is not valid and that can be reached. A **dashed arrow with `ADDED`** is a move that the disposition adds. A **`REMOVED`** note names an arrow or state that the disposition removes. Each sketch says which claims are known and which are inferred.

Machines with "leave it" have no sketch: C3, C4, C5, D1, D5, D6, E2, E3, E4. D5 would show only a race, which needs the To-check first.

## D2 Text check: mode and auto-check (explicit in the code)

Today (known): both fields are `pub`. The normaliser repairs the pair on read only. The default, the seed and `data/settings.json:70-71` all hold the red state.

```mermaid
stateDiagram-v2
    direction LR
    [*] --> DisabledOn : default value<br/>(settings_defaults.rs:4)
    [*] --> DisabledOn : seed of data/settings.json<br/>(load.rs:64-72)
    [*] --> DisabledOff : form, Disabled<br/>(set_mode_and_auto_check)
    [*] --> Checking : form, Spell, Grammar<br/>or SpellGrammar
    DisabledOn --> DisabledOn : read repairs the value<br/>(effective_enable_auto_check)
    DisabledOff --> Checking : form, mode chosen
    Checking --> DisabledOff : form, Disabled
    DisabledOn --> Checking : form, mode chosen

    DisabledOn --> DisabledOff : ADDED the constructor and the serde read<br/>make DisabledOn unwritable

    state "Disabled, auto-check ON (not valid, reachable)" as DisabledOn
    state "Disabled, auto-check off" as DisabledOff
    state "Spell, Grammar or SpellGrammar, auto-check on or off" as Checking

    classDef bad fill:#fdd,stroke:#c00,color:#600
    class DisabledOn bad
```

After the disposition: the red state has no entry arrow. The default and the seed come out as `DisabledOff`. No stored JSON changes shape.

## D3 Mode preset registry and the presets (explicit in the code, conditional on Q9)

Today: the check of `allows` is at the activation only. The edit of a preset and the seed make arrows that the check does not see. Path (a) is known; its effect is inferred.

```mermaid
stateDiagram-v2
    direction TB
    [*] --> Valid : seed of settings.json<br/>names stored presets
    [*] --> Dangling : seed names a missing preset<br/>(load.rs:60-72 checks Connections only)
    Valid --> Valid : activate preset for a mode<br/>(allows is checked, prompt_presets.rs:283)
    Valid --> Disallowed : edit removes the mode<br/>from the preset a bundle names<br/>(prompt_preset_service.rs:94-107)
    Disallowed --> Valid : edit adds the mode again,<br/>or activate another preset
    Valid --> Refused : ADDED edit that removes the mode<br/>of an active preset
    Refused --> Valid : the 400 leaves the preset as it was

    [*] --> Refused2 : ADDED seed with a missing preset id
    state "Seed refused, boot stops" as Refused2

    state "Every bundle names a stored preset that allows its mode" as Valid
    state "A bundle names a preset that does not allow its mode (not valid, reachable)" as Disallowed
    state "A bundle names a missing preset (not valid, reachable by seed)" as Dangling
    state "Edit refused (400)" as Refused

    classDef bad fill:#fdd,stroke:#c00,color:#600
    class Disallowed,Dangling bad
```

After the disposition: the two red states have no entry arrow. The arrow `Valid --> Disallowed` is **REMOVED**. The arrow `[*] --> Dangling` is **REMOVED**. The two `ADDED` arrows replace them. If the owner answers "no" to Q9, none of this is added and the doc line in `narrator_mode.md:34` is the rule.

## D4 Allowed Modes of a preset (explicit in the code)

Today (known): the list can be empty only through a seed file. The create and edit routes cannot make it. Q3 (both boxes clear) is a form question and stays open.

```mermaid
stateDiagram-v2
    direction LR
    [*] --> Both : create, both boxes clear<br/>(prompt_presets.rs:171-176)
    [*] --> Both : seed, key absent or null
    [*] --> Empty : seed with allowed_modes []<br/>(run.rs:217-227)
    [*] --> One : create, one box checked
    Both --> One : edit, one box checked
    One --> Both : edit, both boxes checked
    Both --> Both : edit, both boxes clear = no change (Q3)
    One --> One : edit, both boxes clear = no change (Q3)
    Empty --> Empty : edit, both boxes clear = no change (Q3)
    Empty --> One : edit, one box checked
    Empty --> Both : edit, both boxes checked

    [*] --> Both : ADDED seed with [] falls back<br/>to both modes and logs a warning

    state "Both modes" as Both
    state "One mode" as One
    state "No mode (not valid, reachable by seed)" as Empty

    classDef bad fill:#fdd,stroke:#c00,color:#600
    class Empty bad
```

After the disposition: `[*] --> Empty` is **REMOVED**, so `Empty` and its three arrows are **REMOVED**. The arrow `[*] --> Both` from the seed with `[]` is **ADDED**. The arrows marked "(Q3)" change only if the owner picks the answer "an error".

## E1 Migration ladder and the database version (explicit in the code, conditional on Q5)

Today (known): `migrate_up_to` (`plumbing.rs:25-800`) runs every step after the stored version, in one transaction. No step looks at a version above `LATEST_VERSION`.

```mermaid
stateDiagram-v2
    direction LR
    [*] --> Older : open a database<br/>with user_version less than 34
    [*] --> Current : open a database<br/>at user_version 34
    [*] --> Newer : open a database that a newer binary wrote<br/>(a downgrade of the binary)
    Older --> Current : the steps after the stored version<br/>run in one transaction (plumbing.rs:58-60)
    Current --> Current : no step is due
    Newer --> Newer : no step is due, no refusal<br/>(plumbing.rs:51-54)
    Newer --> Refused : ADDED the check at the top<br/>of migrate_up_to

    state "user_version below 34" as Older
    state "user_version 34 (valid)" as Current
    state "user_version above 34 (not valid, reachable by stored data)" as Newer
    state "Boot stops with an error that names both numbers" as Refused

    classDef bad fill:#fdd,stroke:#c00,color:#600
    class Newer bad
```

After the disposition: the arrow `Newer --> Newer` is **REMOVED**. The old code no longer reads or writes the newer schema.
