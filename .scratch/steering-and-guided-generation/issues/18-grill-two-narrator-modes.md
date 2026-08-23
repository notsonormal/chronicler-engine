# Grill: two narrator modes (novel/RP vs. IF/CYOA)

Type: grilling
Status: pending
Blocked by: (none)

## Question

Should the engine carry two distinct narrator postures — a novel/RP mode and an IF/CYOA mode — as a first-class mode setting? And if so, what shape does it take (preset bundle, a new `NarratorMode` setting, a different system preset, or a separate effort/map)?

This is the *narrator posture* axis, surfaced during ticket 16's perspective grilling. It is **orthogonal to perspective** (I/you/third): mode is how the narrator relates to the player's input; perspective is the pronoun of the player character. The two defaults *correlate* (novel→third, IF→second), which is why they felt like one thing, but they are two knobs.

## Why this needs grilling

**The engine runs novel mode only today.** Verified:
- `data/prompt_presets/system/default.json` instructions carry the "Agency Rule: Never write, assume, or infer the player's actions, thoughts, or feelings." The narrator is forbidden from writing the PC's action.
- That is the novel/RP posture: the player supplies rich prose, the narrator world-builds around it, never narrating the PC.
- IF/CYOA is the opposite posture: the player supplies a terse verb ("take vase", "go north"), and the narrator *elaborates* it ("You take the ancient vase."). The Agency Rule blocks exactly that elaboration.
- So a second narrator mode is a real, missing posture — not a perspective tweak.

**Mode is not currently a setting.** `AppSettings` (`src/domain/model/settings.rs`) has `active_system_prompt_preset_id` (which *preset* to use) but no `narrator_mode` / `narrative_mode` / `play_style` field. Mode is baked into the system preset's Agency Rule, configurable only by editing the preset or swapping it for another — which again requires prompt-engineering skill the settings panel is meant to spare the user.

**Perspective and mode are orthogonal but their defaults correlate.** A novel game defaults to third; an IF game classically defaults to second. If both become settings, they interact: does the mode set a default perspective (and can the user override)? Does the mode bundle a whole preset set (system + impersonate + quantifier)? Or does the mode simply change the system preset's Agency Rule while perspective is still an independent macro?

**Impersonate's posture also differs by mode.** In novel mode, impersonate writes the PC's rich line (a real feature — the player drafts as their character). In IF mode, the player's "message" *is* the terse command; "write as the player" may be redundant or undesired there. This needs resolving as part of the mode design, not the perspective ticket.

## Questions to grill

1. **Is there a second posture at all?** The premise is "novel mode (Agency Rule on, never narrate the PC) vs. IF mode (narrate/elaborate the PC's terse input)." Confirm whether a genuine second posture is wanted, or whether the existing system preset already serves both via player input style. Argue which.

2. **Mode as a setting, or as a preset-swap?** Three shapes (mirror of ticket 16's Q4):
   - **(a) New `AppSettings` field `NarratorMode` + preset bundle.** A `NarratorMode` enum (`Novel`/`InteractiveFiction`) on `AppSettings`, selecting a *bundle* (system + impersonate + quantifier presets) tuned for that mode. One mode = one coherent preset set. New field + storage migration + seed presets per mode.
   - **(b) Preset-edit only.** Mode stays a preset choice; advanced users edit/swap presets. Zero code, but requires prompt-engineering skill and lets presets drift (the original defect from ticket 16).
   - **(c) Both.** A mode dropdown for the common case (selects a bundle), custom presets for experts.
   Grill: how much does a non-prompt-engineer need to flip "novel vs. IF" as a single toggle vs. hand-tuning three presets?

3. **Does mode set a default perspective, and can the user override it?** Novel defaults to third; IF classically defaults to second. If ticket 16 lands a `NarrativePerspective` setting (Second/Third), does mode (a) force the correlated default and lock perspective, (b) set a default the user can still override, or (c) leave perspective entirely independent? Grill the coherence-vs-flexibility tradeoff.

4. **Impersonate in IF mode.** Does impersonate exist in IF mode at all? If the player's input *is* the terse command, "write as the player" is redundant. Options: disable impersonate in IF mode; repurpose it to draft a terse command in IF voice; keep it but note it's odd. Decide.

5. **Is this a separate map, or a ticket on this map?** The mode question may be larger than one ticket (it reshapes the narrator role, not just a setting). Decide whether this grilling resolves into a single ticket/decision on this map, or whether it graduates into a fresh wayfinder map of its own (the user's "whether it becomes a different map or whatever can be determined there").

6. **Scope boundary vs. this map's destination.** This map's destination is the three steering features (guide / narrator action / impersonate). Two-narrator-modes is adjacent but not named in the destination. Decide whether it is in scope (extends the narrator) or out of scope (a fresh effort). Ticket 16 chose to scope it *out* of its perspective grilling and into this ticket; this ticket decides whether it stays on the map at all.

## Forcing instance

The observation opened during ticket 16: the user noted "a novel style where the player is a character who might have complex dialogue and actions" vs. "a more IF choose-your-own-adventure style, were the player has simple actions and dialogue, and the game acts and extends them." The engine currently serves only the novel style; the IF style is latent and unsupported by the Agency Rule.

## Notes for the session

- Read before grilling: `data/prompt_presets/system/default.json` (Agency Rule — the novel posture), `data/prompt_presets/impersonate/default.json` (first-person-as-player posture), `data/prompt_presets/quantifier/default.json`, `src/domain/model/settings.rs` (AppSettings — no mode field), `src/domain/model/template.rs` (TemplateVars macro mechanism, shared with ticket 16's perspective macro).
- Related tickets: 16 (perspective — orthogonal axis, nearly decided: Second/Third, default Third, macro across presets), 04 (feature synthesis), 09 (impersonate preset), 15 (Dialogue-vs-Input — the type axis, also orthogonal to mode).
- Skills: `/grilling`, `/domain-modeling`.
- This is a narrator-design decision, larger in scope than ticket 16. Do not implement until the grilling resolves mode shape (Q2), the perspective interaction (Q3), and whether this stays on the map or becomes its own effort (Q5/Q6).
