# Grill: narrative perspective (first/second/third person) and impersonate

Type: grilling
Status: pending
Blocked by: (none)

## Question

Grill whether narrative perspective (first person "I went", second person "you go", third person "Jack went") and tense should be a configurable setting, and how impersonate's perspective relates to the narrator's. Today both perspectives are hardcoded into preset text and they disagree.

## Why this needs grilling

**Perspective is hardcoded in two presets, and they disagree.** Verified:
- System (narrator) preset: `"writing_style": "Third-person limited perspective, focused on the player character. Past tense narrative prose."` (`data/prompt_presets/system/default.json`).
- Impersonate preset: `"writing_style": "First-person perspective as {{user}}. Match the tense already in use."` + instructions `"First person (\"I\") for speech and internal thought"` (`data/prompt_presets/impersonate/default.json`).
- Quantifier preset: its *examples* use second person — `"You walk through the door into the kitchen."`, `"You examine the ancient vase carefully."` (`data/prompt_presets/quantifier/default.json`). This is inconsistent with the system preset's declared third-person.

So the engine already carries three perspectives across three presets: the narrator writes third-person, the quantifier's examples assume second-person, and impersonate writes first-person. Nothing checks they agree.

**No perspective/tense setting exists.** `AppSettings` (`src/domain/model/settings.rs:120-130`) has `active_system_prompt_preset_id` / `active_quantifier_prompt_preset_id` / `active_impersonate_prompt_preset_id` (which *preset* to use) but no `perspective`, `narrative_voice`, `point_of_view`, or `tense` field. Perspective is baked into preset *text*, configurable only by editing the preset (which requires prompt-engineering skill the settings panel is meant to spare the user).

**The impersonate-specific tension.** Impersonate writes as the player, so first person ("I went") is semantically intrinsic — you impersonate *yourself* in first person. But the conversation around it is third-person narration ("She went to the store"). So an impersonated turn produces a jarring first-person line in a third-person conversation. This may be the real root of "the response is in first person and that's strange" (the observation that opened this thread): the strangeness is not that impersonate is first-person, but that the narrator is third-person while impersonate is first-person, with no setting to align them.

## Questions to grill

1. **Scope: impersonate only, or narrator too?** The user framed this as "impersonate and perspective," but the system preset's perspective is also hardcoded. Is the goal a setting that controls the *narrator's* perspective (with impersonate following along), or one that controls impersonate independently? Argue which.

2. **Is impersonate's perspective semantically locked to first person?** Impersonate writes *as* the player. Third-person impersonate ("Jack went") is indistinguishable from narration-about-the-player. Second-person impersonate ("you went") is directing the player, not being the player. Does "impersonate" *mean* first-person by definition — in which case it is not configurable, and only the narrator's perspective is? Or is "write as the player, in the conversation's perspective" a coherent alternative?

3. **Second person as a supported mode.** Second-person narration ("You walk into the tavern") is a classic interactive-fiction voice (Zork, CYOA). The quantifier preset already assumes it in examples. Should the engine support second person as a first-class perspective option? If yes, does impersonate in a second-person story become "I walk" (first) or "You walk" (matching the narrator)?

4. **Where does the setting live — new field, macro, or preset-edit?** Three options:
   - **(a) New `AppSettings` field + macro.** A `NarrativePerspective` enum (`FirstPerson`/`SecondPerson`/`ThirdPerson`) on `AppSettings`, substituted into presets via a new `{{narrative_perspective}}` macro (the `TemplateVars`/`render_template` machinery already supports `{{user}}`/`{{persona_*}}` — `src/domain/model/template.rs`). One preset template serves all three perspectives. New setting + macro plumbing + storage migration.
   - **(b) Preset-edit only.** Perspective stays in preset text; the user edits the preset to change it. Zero new code, but requires prompt-engineering skill and is inconsistent with the settings panel's purpose. The three preset *ids* already let advanced users swap presets; is that enough?
   - **(c) Both.** A setting for the common case (perspective as a dropdown), presets for the expert case (full voice control). The setting injects via macro; a custom preset that ignores the macro wins.
   Grill the tradeoff: how much does a non-prompt-engineer user need to toggle perspective vs. full voice control?

5. **Tense alongside perspective?** The system preset fixes "Past tense." The impersonate preset says "Match the tense already in use" — it inherits tense but not perspective. Should tense be a setting too (Past/Present), or stay in the preset? Same macro mechanism could carry both.

6. **The quantifier inconsistency.** The quantifier preset's examples assume second-person narration, but the system preset produces third-person. Is this a latent bug (the quantifier misreads third-person narration because its examples teach second-person), or harmless because the quantifier keys off movement verbs not pronouns? Check the quantifier agent's actual parsing. If it is a bug, a perspective setting fixes it only if the quantifier examples are also generated from the setting.

7. **Coherence across the three presets.** If perspective becomes a setting, all three presets (system, impersonate, quantifier) must respect it, or the setting is cosmetic. The quantifier examples (Q6) and impersonate's "match the tense" instruction both already cross-reference the narrator's voice. Grill whether a single setting can drive all three coherently, or whether each preset needs its own perspective (which is just option b).

8. **Default.** If a setting is added, what is the default — third-person limited (current system behavior), or second person (current quantifier-example behavior, and the IF tradition)? The two current defaults disagree, so picking one is a behavior change for someone.

## Forcing instance

The user ran `/impersonate`, saw first-person output, and found it strange against the third-person conversation. The first-person is by design (ticket 09's impersonate preset); the strangeness is the unconfigurable narrator/impersonate perspective mismatch. This ticket decides whether to make perspective configurable and how impersonate fits.

## Notes for the session

- Read before grilling: `data/prompt_presets/system/default.json` (writing_style: third-person limited), `data/prompt_presets/impersonate/default.json` (writing_style: first-person, instructions "First person (\"I\")"), `data/prompt_presets/quantifier/default.json` (examples use second person — the inconsistency), `src/domain/model/settings.rs:120-180` (AppSettings — no perspective field), `src/domain/model/template.rs` (TemplateVars + render_template — the macro mechanism a `{{narrative_perspective}}` field would extend), `src/application/agents/quantifier/` (whether the quantifier parses pronouns or verbs — Q6).
- Related tickets: 09 (impersonate preset, first-person choice), 14 Q6 (superseded by 15), 15 (Dialogue-vs-Input — the message-type side of "impersonate reads as a player line"). This ticket is the *perspective* side of the same observation; 15 is the *type* side.
- Skills: `/grilling`, `/domain-modeling`.
- This is a settings/UX design decision, not a bug fix. Do not implement until the grilling resolves where the setting lives (Q4) and whether impersonate is locked (Q2).
