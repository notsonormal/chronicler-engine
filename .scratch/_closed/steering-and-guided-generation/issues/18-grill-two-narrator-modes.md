# Grill: two narrator modes (novel/RP vs. IF/CYOA)

Type: grilling
Status: resolved
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

## Resolution

Grilled across one round (Q1, Q2). The frontier collapsed after both answers.

### Decisions

1. **A genuine second narrator posture is wanted (Q1=A).** IF/CYOA mode — where the player writes terse commands and the narrator *elaborates* them ("take vase" → "You take the ancient vase") — is a real, missing posture, not a perspective tweak. The current system preset's Agency Rule materially blocks it (Fact 1: *"Never write, assume, or infer the player's actions, thoughts, or feelings… The player's speech lines must be in indirect speech."*). Forcing an IF player to write prose defeats the genre, so the existing preset cannot serve both postures by input style alone. Reinforcing signal from the user: a future auto-generation of pickable options for the player would suit the new mode — a classic IF/CYOA affordance with no home in novel mode.

2. **Out of scope for this map; spins off as a fresh wayfinder effort (Q2=A).** Two-narrator-modes is not named in this map's Destination (guide / narrator action / impersonate — all three now implemented and build-green). It reshapes the narrator role itself — bigger and architecturally different from a steering feature (new mode setting, preset bundles, Agency Rule surgery, perspective-interaction, impersonate-in-IF redesign). Loading it onto an effort at its finish line risks scope creep and blurs the "done" line. It deserves its own chart-the-map treatment, which forces the mode question through its own breadth-first grilling.

### What this ticket does NOT decide (graduated to the new map)

The mode-shape questions are the new map's to grill, not this ticket's:
- **Mode shape** (ticket 18 Q2): new `AppSettings` `NarratorMode` field + preset bundle vs. preset-edit-only vs. both.
- **Perspective interaction** (ticket 18 Q3): does mode set/lock the default perspective, or stay independent? Mode and perspective are orthogonal (ticket 16); their defaults correlate (novel→third, IF→second) but are independent knobs.
- **Impersonate in IF mode** (ticket 18 Q4): disable, repurpose to draft a terse command, or keep?
- **Options-autogeneration** (user-surfaced this session): an auto-generated pickable-options feature that would suit IF mode. Candidate fog toward the new map's destination — not yet sharp enough to ticket; the new map's charting decides whether it is in the destination or fog.

### Seed facts for the new map's first session

- **Current engine state:** novel-only. Agency Rule in `data/prompt_presets/system/default.json` (the novel posture). `AppSettings` (`src/domain/model/settings.rs`) has `active_system_prompt_preset_id` but no mode field. Perspective/tense settings landed (ticket 19: `NarrativePerspective` Second/Third default Third, `NarrativeTense` Past/Present default Past, via `{{narrative_perspective}}`/`{{narrative_tense}}` macros). Impersonate works in novel mode (tickets 09/17; output is `MessageType::Input`, no `sender`).
- **Orthogonality:** mode is orthogonal to perspective (ticket 16) and to the speaker axis (ticket 15). The new map should respect both — neither axis is re-litigated.
- **The new map's chart-the-map step 1 (name the destination) is a grilling, not a ticket.** It runs before the new map exists. Tickets are created in step 4, after the destination and frontier are settled.
