# Research: Marinara Engine terminology cross-reference (for ticket 06)

One-line summary: Marinara Engine — the SillyTavern-lineage project the
`guided-generations` branch modeled on — does **not** use "Steering" as a
concept name, uses **"Regenerate"** (not "retry") as the umbrella for
swipe-creation, and calls the replay carrier **"Stored guidance"** in the UI
(`generationReplay` in code). The branch's "Steering" umbrella and "retry"
grouping are branch inventions, not inherited terminology.

## Sources

- Fresh clone: `https://github.com/Pasta-Devs/Marinara-Engine` (v2.4.3), at
  `/tmp/pi-github-repos/Pasta-Devs/Marinara-Engine`.
- Prior research artifact (older Marinara, commit `bf103aa` ~v2.4.2):
  `.scratch/steering-and-guided-generation/research/01-marinara-engine.md`.
  Code-level findings there still hold against v2.4.3 (verified:
  `generation-replay.ts`, `generation-guide.ts` unchanged at the same paths).
  The user-facing docs were reorganized: `explaining_docs/` → `docs/`, and the
  steering doc consolidated into `docs/chats/guided-and-impersonate.md`.

This is a terminology cross-reference, not a recommendation to copy. Marinara
is a different product (Node/TypeScript, SillyTavern-lineage, multi-mode
chat/roleplay/game). Its terms are a reference point for what the branch
inherited vs. invented.

## Findings against ticket 06's four doubts

### Doubt A — "retry" as the umbrella

**Marinara uses "Regenerate," not "retry," as the umbrella for swipe-creation.**

`docs/chats/messages.md` §"Regenerate, continue, and retry" defines three
distinct actions:

- **Regenerate** — "makes a new swipe." Works on AI messages and, in
  Roleplay/Conversation, on user messages made by Impersonate. This is the
  umbrella term for alternate-take creation.
- **Continue** (`/continue`) — extends the same message, not a new swipe.
- **Retry** — "Empty-Send retry": a **narrow** action. When the input box is
  empty and the last message is the user's, the Send button retries to get a
  fresh reply. Explicitly "not the same as /continue."

So in Marinara, "retry" names one specific empty-send behavior, not a category
covering narration-redo, impersonate-redo, and user-input-redo. "Regenerate" is
that category.

**Bearing on 06:** confirms the user's instinct (ticket 01 grilling, Q1). The
chronicler branch's `RetryMode { ReNarrate, ReImpersonate, UserRegen }` groups
three things under "retry" that Marinara — the lineage the branch modeled on —
groups under "regenerate." The branch's "retry" umbrella is a mis-inheritance.

### Doubt B — "Steering" as the umbrella

**Marinara does not use "Steering" as a concept name.**

The canonical doc is `docs/chats/guided-and-impersonate.md`, titled "Guided
Generation and Impersonate." It presents two **separate** features in separate
sections:

- **Guided generation** — transient, out-of-character direction; does not post
  a visible message.
- **Impersonate** — the AI writes your reply as your persona; posts a user
  message.

"Steer" appears only as a verb ("steer a chat," "steers the reply"). There is
no "Steering" concept unifying the surfaces. The chronicler branch's
**Steering** umbrella (Guided Generation + Narrator Action + Impersonate) is a
branch invention, not inherited from Marinara.

**Bearing on 06:** the branch unified three surfaces under a name Marinara
never uses. The grilling should ask whether that unification earns its place —
Marinara's separate-feature framing is the prior art.

### Doubt C — "replay blob" as the carrier

**Marinara's code calls it `generationReplay`; the UI calls it "Stored
guidance." Not "replay blob."**

`packages/server/src/routes/generate/generation-replay.ts` defines
`GenerationReplay { impersonate?, generationGuide?, generationGuideSource?,
impersonatePresetId?, ... }`. The chronicler branch's `GenerationReplay` is a
direct port (same fields).

The user-facing term is **"Stored guidance"** (`docs/chats/guided-and-
impersonate.md` §"Reading Stored guidance") — a scroll icon on the message
that shows the direction that produced the reply, labeled by source
(`/guided`, "Guided regenerate", "Game start").

**Bearing on 06:** the *concept* is inherited and has prior art. The branch's
*name* "replay blob" is worse than both Marinara names: it is internal jargon,
whereas `generationReplay` (code) and "Stored guidance" (UI) are clearer. If
the concept stays, the name should at least match one of these.

### Impersonate regeneration (confirms user's Q1 answer)

**Marinara regenerates impersonate messages through the same Regenerate path,
via the replay blob — no separate "retry impersonate."**

`docs/chats/guided-and-impersonate.md` §Impersonate: "You can redo a message
that Impersonate wrote. The Regenerate action works on user messages that were
created by Impersonate, so you can get a different version."

`generation-replay.ts::applyGenerationReplayToRegenerateInput` replays the
blob into the regenerate request; the blob's `impersonate: true` flag routes
it back through the impersonate path. There is no `retry_reimpersonate`
equivalent that re-implements the impersonate choreography separately.

**Bearing on 06:** directly confirms the user's Q1 answer — impersonate retry
belongs with the impersonate generation flow (carried by the replay blob),
not grouped with narration retry. The chronicler branch's
`retry_reimpersonate` (which duplicates `phase_narrate`'s impersonate branch)
is the divergence from the prior art.

## Additional finding (beyond 06's four doubts, but relevant)

**`/narrator` changed meaning between Marinara versions: the player command
became transient; the engine-side message role stayed.**

- **Older Marinara** (local cache, `explaining_docs/guides/explanation/ai_steering.md`):
  `/narrator` was a **separate permanent** player command — "visible steering,"
  adds a permanent `narrator`-role message to history. Distinct from Guided
  Generation, which was "invisible/transient."
- **Current Marinara (v2.4.3)**: `/narrator` is an **alias of `/guided`**
  (`docs/chats/guided-and-impersonate.md:23`, `docs/chats/slash-commands.md:66`)
  — transient, no permanent message. Two near-duplicate steering commands
  collapsed into one transient surface.
- **The `narrator` message role survived.** It is still a first-class role in
  v2.4.3, used for engine-generated permanent content: scene summaries, game
  session recaps, TTS speaker routing, and conversation-history narrator
  bubbles (`CHANGELOG.md` lines 656, 804, 1649; `docs/chats/messages.md`).
  What changed is the *player command*, not the *role*. Players can no longer
  mint a permanent narrator directive via a slash command; the engine still
  writes permanent narrator messages for its own content.

**No recorded reason in the repo.** I checked the three places it would live:

| Source | Result |
|---|---|
| `git log --grep=narrator` (all branches) | No commits mention the alias change |
| `CHANGELOG.md` | Every "narrator" mention is the *message role* (TTS voice, scene summaries, bubbles), not the slash command |
| PR #5230 (staging → main merge, commit `34442e2`) | `docs/chats/slash-commands.md` was created fresh in this merge with `/narrator` already an alias of `/guided`; no prior version shows the transition, no dedicated commit or PR rationale |

The granular history was squashed in the staging branch.

**Inferred reason (not recorded — my guess).** The older design had two
overlapping steering surfaces (transient Guided Generation vs permanent
Narrator Command). Folding `/narrator` into `/guided` collapses them into one
transient surface. This is consistent with a simplification, but the repo
does not say so. I infer from the shape of the change.

**Bearing on 06.** The chronicler branch's **Narrator Action** (per
`CONTEXT.md`: "a permanent author directive, persisted as a Narrator message
in history, rendered without a sender prefix") matches the **older**
Marinara *player command*, which current Marinara demoted to a transient
alias. But Marinara kept the `narrator` *role* for engine-generated permanent
content. So the 06 grilling should split the question:

- If Narrator Action is a **player command** that mints a permanent narrator
  message, current Marinara prior art argues against it (demoted to
  transient `/guided`).
- If Narrator Action is an **engine content type** (recap, scene summary),
  current Marinara keeps it — the `narrator` role still carries permanent
  engine content.

The branch's `CONTEXT.md` describes Narrator Action as a player-authored
permanent directive, which is the demoted case. The grilling decides whether
the branch keeps it, demotes it, or re-scopes it.

## What this cross-reference does NOT settle

- Whether the chronicler branch should drop "Steering" — only that the term is
  invented, not inherited. The grilling decides if the unification earns its
  place on chronicler's own merits.
- Whether "replay blob" should be renamed vs. retired — only that the concept
  has prior art and better names exist.
- Whether Narrator Action should stay permanent — only that current Marinara
  made it transient.

These are the grilling's job (ticket 06, HITL). This asset is the fact base.
