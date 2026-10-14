# Research 15: method and inputs for a clean-room design of the generation model

Ticket: [Research: method and inputs for a clean-room design of the generation model](../issues/15-research-clean-room-design.md).

Labels: **known** = read in a source (a web page, a repo file or a tool's documentation). **inferred** = follows from known facts, not run. **tested** = a command run in this session shows it. **guessed** = not supported by a source.

Tree: `9cbf3112` plus the work tree of the parallel sessions.

Supporting assets:

- [research-15-web-sources.md](research-15-web-sources.md) — the web research for §1 and §2, with quotes and URLs. A `researcher` agent wrote it. This session re-fetched the key quotes; its header lists which.
- [research-15-leakcheck.py](research-15-leakcheck.py) — the leak check for §3 and §5. Run it from the repository root.

## Summary

- **The method transfers as a reasoning tool, not as a proof of independence.** *NEC v. Intel* inferred "forced by constraints" from agreement between an implementation and a walled-off clean implementation. Here the spec writer, the owner and possibly the model sit on both sides of the wall. The output of a run is therefore a list of requirement-forced and history-forced decisions. The disagreements are the main result. (§1)
- **No named software practice exists for this use.** The nearest prior art is the IETF and W3C "two independent implementations" rule, spec audits by a second implementation, and the 2026 chardet rewrite with an LLM clean team. The chardet case shows the four failure modes to plan for: a spec leak, training-data recall, a design document in place of a behaviour spec, and a clean team that converges for reasons other than the requirements. (§2)
- **The specs are the only near-clean input.** `docs/AGENTS.md` defines a spec as "the behaviour a client observes through HTTP and browser interactions". Three of the six specs in scope contain no `src/` identifier, and a fourth contains only the Quantifier's answer key (tested). But `actions.md` and `swipe_new.md` encode mechanism in domain words, and `retrigger.md` and `reset.md` read storage functions in their Then-steps. The Diátaxis docs keep seam identifiers by rule, so they are mixed or contaminated. Of the 18 prior decisions, 8 are behaviour rulings that a spec can carry; 8 are module-shape decisions that must stay out. (§3)
- **The two premise sources give different answers to four of the five questions of this map (inferred).** SillyTavern binds only text and a per-swipe metadata bundle to a swipe, and swipes only the last message. Marinara binds every rewinding store to the pair (message, swipe), and regenerates any assistant message. So "behave as the premise source does" is not a complete requirement. (§4)
- **A strong wall is cheap and tested.** A `pi` run with `--no-tools --no-context-files --no-skills --no-extensions --no-approve` has no tools and no project context, in four model families (tested). Three channels stay open: the cwd path in the system prompt, `ask_owner`, and training data. Three of four models describe SillyTavern's swipe storage from training data; none knows this repo or Marinara's swipe binding (tested). (§5)
- **Ticket 01 is on the frontier now.** All its blockers are resolved. A run before ticket 01 delays the main grilling ticket by about four sessions (inferred). A run after tickets 01–03 checks rulings that were made with no reference point. (§6)

This asset chooses nothing. §7 lists the owner's decisions.

## Method and commands

Files read in this session: the ticket and its map; tickets 01 and 16; the [seed](seed-evidence.md); the assets of [research 11](research-11-docs-and-decisions.md), [13](research-13-sillytavern-swipe-state.md) and [14](research-14-marinara-swipe-state.md); `CONTEXT.md`; `docs/specs/swipe_new.md`, `retrigger.md`, `reset.md`, `swipe_switch.md` in full; the headings of `actions.md` and `failure_display.md`; the head and headings of each file in `docs/external_applications/` and of the four closed prior-art files; `docs/AGENTS.md` (loaded by an extension, see §5); the Herdsman agent-definition schema; `pi --help`; the head of the `pi-on-demand-context` and `mrn-context` extensions; `.pi/APPEND_SYSTEM.md`; `.pi/extensions/pi-permission-system/config.json`; `.pi/agents/judge-glm.md`.

Commands run:

- `python .scratch/turn-state-model/assets/research-15-leakcheck.py <file>...` over every candidate input. The check builds 2,690 distinctive names from `src/`: declarations, struct fields, multi-word enum variants and file stems that are snake_case with an underscore (6+ characters) or CamelCase with two or more words. It subtracts the JSON keys under `data/`, because an author sees those. It counts the names and the `src/` paths in each input.
- The isolation probe, four times, from the repository root:
  `pi --no-session --no-context-files --no-skills --no-extensions --no-prompt-templates --no-mcp --no-approve --no-tools -e <provider extension> --model <model> -p "<probe>"`, with the models `opencode-go/deepseek-v4.1-flash`, `opencode-go/glm-5.3-flash`, `opencode-go/kimi-k2.7-code` and `anthropic/claude-opus-5-5`. The probe asked for the callable tools, any context file, every system-prompt line that names a project, path, skill or style rule, any knowledge of this repo, and SillyTavern's and Marinara's swipe binding from training data alone.
- `grep '^Status:\|^Blocked' .scratch/turn-state-model/issues/*.md` for the frontier.

---

## 1. The method

Full quotes and URLs: [web sources](research-15-web-sources.md) §Method, §Phoenix BIOS, §NEC v. Intel.

### Roles, wall and contamination

| Element | Legal clean room | Source | Label |
|---|---|---|---|
| Examiner ("dirty" team) | Examines the original and writes a specification | Wikipedia, *Clean-room design*: "having someone examine the system to be reimplemented and having this person write a specification" | known (re-fetched) |
| Specification | States what the system does, not how. Phoenix: "described everything it did as completely as possible without using or referencing any actual code" | Computerworld 2001; Wikipedia, *Phoenix Technologies* ("technical specifications for the BIOS APIs") | known (not re-fetched) |
| Reviewer | "This specification is then reviewed by a lawyer to ensure that no copyrighted material is included." | Wikipedia only. No primary Phoenix or NEC record of a lawyer review was found | known for Wikipedia; unconfirmed for Phoenix and NEC |
| Clean team | "implemented by a team with no connection to the original examiners". Phoenix used one engineer who knew the TMS9900, not the 8086 | Wikipedia, *Clean-room design* and *Phoenix Technologies* | known (the first quote re-fetched; the Phoenix detail not re-fetched) |
| The wall | A separation of knowledge, evidenced by records. NEC's clean-room file held "every single piece of paper which Mr. Davidian saw" | Harvard JOLT 3:209, quoting NEC's post-trial brief | known (not re-fetched) |
| Contamination | Access, not similarity. Davidian "never had access to any microcodes created by either Intel or NEC" | Harvard JOLT | known (not re-fetched) |

A legal clean room proves a negative: no access. Its strength is procedural. Phoenix insured the process with a $2M policy, not the result (inferred from the Wikipedia account).

### The convergence test of *NEC v. Intel*

**Known (re-fetched, Wikipedia):** NEC hired an independent contractor "who was only given access to specifications but ended up writing code that had certain similarities to both NEC's and Intel's code. From this evidence, the judge concluded that similarity in certain routines was a matter of functional constraints resulting from the compatibility requirements". The court's own words, through 10 Computer L.J. 453, are in the web sources (not re-fetched). The decision of 6 February 1989 was not appealed. The clean room started after the suit, as a litigation fallback (web sources, not re-fetched; Wikipedia confirms that NEC hired the contractor during the trial).

**Does the test transfer? Partly (inferred).** The transferable part is an asymmetry:

| The clean design and the code… | Legal reading | Reading for this map |
|---|---|---|
| agree, and the spec states a constraint that forces the shape | forced by function, not copied | forced by the requirements |
| agree, and the spec is silent | not addressed by NEC | shared premise, convention or recall; not evidence of forcing |
| differ | not forced | a free choice, a defect in the code, or a gap in the spec |

The limits, most severe first (inferred, from the web sources §Does the convergence test transfer):

1. **One spec writer, one owner.** NEC's independence premise does not hold. The spec writer here has read the code, and the owner who answers questions has too. That is the channel the NEC wall closed.
2. **Training data.** It is the LLM form of "the examiner had access". A wall cannot close it (§5 measures it).
3. **Correlated designers.** Knight and Leveson (IEEE TSE, 1986) found that 27 versions written independently from one spec failed together far more often than independence predicts. Agreement between designs is therefore weaker evidence than it looks (known, re-fetched through the Jas paper's citation).
4. **Re-derivation for other reasons.** A clean team can reach the old shape through the domain, the vocabulary or the scenarios, with no leak.
5. **Granularity.** NEC judged the whole and discounted similar trivial routines. A comparison must pick its unit before the run.

---

## 2. Prior art

Full quotes and URLs: [web sources](research-15-web-sources.md) §Prior art, §Failure modes seen, §Related methods.

### Non-legal uses of a specification-only implementation

| Prior art | What it uses a second implementation for | Label |
|---|---|---|
| IETF RFC 2026 §4.1.2, RFC 6410 §2.2 | A standard advances only with "two independent and interoperable implementations from different code bases". A feature that lacks two implementations is removed. A second implementation is an instrument for spec quality. | known |
| W3C Candidate Recommendation, WHATWG stages | "at least two independent interoperable implementations" before a spec advances | known |
| Jas (arXiv 2606.07828, 2026) | Five ports of one application from one spec, mostly written by AI. "The most informative divergences are not implementation bugs but specification gaps." | known (re-fetched) |
| FACET second implementation; SPECA (arXiv 2602.07513) | A second implementation from the spec text alone, to find where readers of the spec disagree | known from the researcher's sources only (the FACET quote came from a search index; neither was re-fetched) |
| "Zero-based design", "greenfield thought experiment" | Consulting and UX phrases. No software practice was found that hides the current design from a designer to expose anchoring. | known (phrases); not found (practice) |

**Inferred:** this map would re-purpose the clean-room method. It inherits no validated protocol, and no evidence of when a result means something.

### LLM agents as the clean team

| Case | What happened | Label |
|---|---|---|
| chardet 7.0.0 (March 2026) | The maintainer rewrote the library with Claude Code, "in an empty repository with no access to the old source tree". He conceded: "that separation did not exist here", and measured similarity (JPlag, at most 1.29%) in place of the process. | known (re-fetched through Willison) |
| The same case, the leak | The agent's plan shows that it read `metadata/charsets.py` of the original. | known (re-fetched, Willison) |
| The same case, training data | "Claude itself was very likely trained on chardet". No wall closes this. | known (re-fetched, Willison) |
| The same case, the spec | The maintainer wrote "a design document specifying the architecture and approach I wanted". The clean side received a design, not a behaviour spec. | known (re-fetched, Willison) |
| Ronacher, *AI and the Ship of Theseus* | An AI port of one of his libraries "ended up choosing a different design… the functionality was the same, but the path it took to get there was different". The port "worked … by going via the test suite". | known (re-fetched) |
| Anthropic C compiler (February 2026); METR and Epoch AI *MirrorCode* (April 2026) | Agents rebuild large programs from a spec. Ars Technica: a C compiler is near-ideal because the spec, the test suites and a reference compiler already exist. "Most real-world software projects have none of these advantages." | known (not re-fetched) |

### Failure modes to plan for

1. **A leak into the clean side.** chardet: the agent read an original file.
2. **Training-data recall.** chardet; measured for this repo in §5.
3. **A spec that is a design.** chardet's design document. The clean team then re-states the given design.
4. **A spec too thin to design from.** NEC's clean engineer finished in 15 days because the instruction set, an emulator and a test suite already existed (inferred from The Chip Letter). Ars Technica makes the same point for the C compiler.
5. **Convergence for other reasons.** Knight and Leveson; the vocabulary of the spec.
6. **Ambiguity read as freedom.** Differential testing "cannot say which implementation is correct" (arXiv 2608.18039, not re-fetched).
7. **Over-claiming the result.** A run here shows design reasons, not independence.

**Inferred, from the researcher and the Jas and NEC records:** keep the clean agent's reasoning: its plan, its questions, and every point where it says the spec is silent. Those points are the spec gaps. NEC kept "every single piece of paper" that its engineer saw.

### Related methods, to separate them

| Method | Difference from this use | Label |
|---|---|---|
| N-version programming (Avizienis, 1985) | Runs N versions with a voter for fault tolerance. Knight and Leveson showed that the failures correlate. Its cost argument changes when AI writes the versions (Jas). | known |
| Cleanroom software engineering (Mills, Dyer, Linger; IBM) | A different method with a similar name: formal specification, incremental development, statistical testing. It has no wall and no second team. | known |
| Second-system effect (Brooks, 1975) | A hazard of building a successor: the designer adds every deferred idea. A clean design can show this hazard, so the comparison must not reward extra features. | known (definition); inferred (the hazard here) |

---

## 3. The input inventory

**Classes.** *Clean*: states required behaviour only, or describes only an outside system; it names no Chronicler code and no Chronicler mechanism. *Mixed*: required behaviour plus Chronicler mechanism (identifiers, step order, storage shape). *Contaminated*: its main content is Chronicler's current mechanism.

**Leak check.** "Ids" is the number of distinct `src/` names that the input contains, and the rate is hits per 1,000 words (tested). A zero does not make an input clean: mechanism can be written in domain words. Stale and loose claims come from [research 11](research-11-docs-and-decisions.md) and were not re-checked.

### Glossary, specs and Diátaxis docs

| Input | Class | Current? | Ids (rate) | Notes and source |
|---|---|---|---|---|
| `CONTEXT.md` | mixed | partly stale: **Retry**, **Swipe**, **Snapshot**, **Game** (research 11, drifts 7–10); no generation entry | 0 (0.0) | No code names. But several entries define a term by the current mechanism: **Snapshot** "message-aligned and persisted with its corresponding Message"; **Game** holds "the generation gate"; **Action Pipeline** "the ordered sequence of phases"; **Agent** "a pipeline step that runs at a defined phase" (known, read). |
| `docs/specs/actions.md` | mixed | current (research 11: 1.5, 1.14, I.2, I.4 match) | 1 (1.9): `npcs_in_room` | `npcs_in_room` is the Quantifier's answer key, an LLM interface name. 22 Given-steps configure a "narrator backend" or a "quantifier backend" per call, so the scenarios fix the Quantifier as a separate LLM call (tested count). A title names an internal step: "trigger continuation re-runs the quantifier". |
| `docs/specs/swipe_new.md` | mixed | partly stale: I.2, I.6, 22.1, and the reason in 10.2 (research 11, drifts 1–4) | 0 (0.0) | Mechanism in domain words: "the anchor message has no snapshot" (11.3), "the generation gate is busy" (12.1), "(the Quantifier re-ran on the redo)" (9.2), "starts from the Snapshot of the Swipe it retries" (10.5). 16 backend Given-steps (tested count). |
| `docs/specs/swipe_switch.md` | clean | current; silent on which state a switch restores (research 11) | 0 (0.0) | Two scenarios. Both are observable: the status is idle; the options strip is gone. |
| `docs/specs/retrigger.md` | mixed | partly stale: the reason in 13.2, and I.3 (research 11, drifts 5–6) | 9 (28.2) | Then-steps read storage functions: `message_service.load_or_fresh().movement.current_room_id`, `message_service.load_messages()`; properties name `snapshot_id`, `last_trigger`, `process_action`. |
| `docs/specs/failure_display.md` | clean | current (research 11) | 0 (0.0) | Observable lines and disclosures only. |
| `docs/specs/reset.md` | mixed | not checked by research 11 | 2 (44.2) | Then-steps read `message_service.load_messages()`. |
| `docs/diataxis/reference/game_flow.md` | mixed | current; silent on the re-impersonate and user-regen tails (research 11) | 18 (34.2) | Reference docs keep "seam identifiers" by rule (`docs/AGENTS.md` §No code-indexer docs). §Retry Flow states invariants; §Phase Flow, §Mutation Order and §Cancellation describe mechanism. |
| `docs/diataxis/reference/storage.md` | contaminated | partly stale: §Relationships (research 11, drift 12) | 11 (24.3) | The storage shape. One invariant sentence (a swipe switch restores "that exact state") is stale. |
| `docs/diataxis/explanation/storage_design.md` | mixed | §Per-swipe state binding states the intent that the owner ruled right; the code does not meet it (research 11, drift 11) | 17 (11.8) | The per-Swipe intent is a requirement. The rest explains the storage mechanism. |
| `docs/diataxis/explanation/two-state-channels.md` | contaminated | current | 3 (11.4) | Explains the registry and the persisted status. |
| `docs/diataxis/explanation/architecture.md` (not in the ticket's list) | mixed | current for the deployment contract | 9 (16.9) | Source of prior decision 17: one process against one database. |

### Owner rulings and prior decisions

Source: [research 11](research-11-docs-and-decisions.md) §Binding prior decisions. The asset itself is contaminated: 86 distinct names, 76.8 per 1,000 words (tested). Its rulings split by content (inferred, from the decision texts):

| Kind | Decisions | Can a clean spec carry it? |
|---|---|---|
| Behaviour ruling | 1 (a Swipe's state is what its own generation produced, Quantifier included), 2 (an impersonate redo can move the player: full tail), 6 (every Message has Swipes; a Swipe stores its inputs; a new Swipe redoes the last generation), 11 (an event Retry starts from the state of the Swipe it redoes), 13 first half (a failed trigger continuation adds no story-log Message), 14 (the room name shows on the Narration that arrives), 16 (failed-Retry rollback rule), 17 (one process, one database) | Yes, after a rewrite without identifiers. |
| Module shape | 3 (pointer captured pre-tail), 4 (one narration-generation module; no redo flag), 5 (no redo-policy seam), 7 (one gated dispatcher; its preset pin is superseded), 8 (`PromptContext` fields), 9 (stored inputs attached in `push_message`), 12 (the anchor check), 15 (Swipe fields; the failed-redo enum) | No. Each is the current shape that a run would test. |
| Process | 10 (one commit), 18 (plan, don't do) | Not applicable. |

Decision 3 also has a behaviour consequence (a swipe switch restores the pre-Quantifier state) that conflicts with decision 1. A clean spec cannot carry both. The conflict is ticket 01's first question (research 11).

### Code facts

| Input | Class | Current? | Ids (rate) |
|---|---|---|---|
| [Seed](seed-evidence.md) | contaminated | current, except the event-retry row (changed by p2-09; ticket 01 §Context) | 26 (53.9) |
| Research assets 04, 05, 06, 07, 08, 09, 10, 12 | contaminated by design | current at their tree | 53–193 distinct; 37.1–131.3 per 1,000 words |
| Research 11 asset | contaminated | current | 86 (76.8) |
| This map, tickets 02 and 03 | mixed | current | map 23 (17.3); ticket 02 14 (27.3); ticket 03 11 (14.6) |

### Premise sources

| Input | Class | Current? | Ids (rate) | Notes |
|---|---|---|---|---|
| [Research 13 asset](research-13-sillytavern-swipe-state.md) (SillyTavern) | clean of Chronicler code; describes SillyTavern's mechanism | current at SillyTavern 1.19.0, commit `06bde939` | 2 (0.7): `display_text`, `system_prompt` | Both names are SillyTavern's own, a false positive. §9 "Patterns" is headed by this map's ticket titles ("…its Snapshot pointers and the game-state writers"), which carry the map's framing. |
| [Research 14 asset](research-14-marinara-swipe-state.md) (Marinara) | clean of Chronicler code; describes Marinara's mechanism | current at Marinara 2.5.0, commit `c5650149` | 0 (0.0) | Its "Patterns" section has the same ticket-title headings. |
| `docs/external_applications/marinara_engine.md` | mixed | stale: "historical/reference, not authoritative"; §File Mapping maps to an old layout (`narrative::llm_client`, `server::fragments::action_handler`, `prompt.rs`) | 7 (7.5) | Topic is LLM transport and prompt assembly, not swipe state. |
| `docs/external_applications/marinara_engine_system_prompt.md` | clean | historical | 0 (0.0) | A preset text. Off the topic of this map. |
| `docs/external_applications/sillytavern_chat_window.md` and `sillytavern_chat_window_example.html` | clean | historical | 0 (0.0); the HTML was not checked | HTML structure of the chat window. Off the topic. |
| `docs/external_applications/sillytavern_prompt_system.md` | clean | historical | 0 (0.0) | Prompt layering. Off the topic. |
| `_closed/steering-and-guided-generation/research/01-marinara-engine.md` | mixed | older upstream commit | 10 (5.7) | Maps Marinara onto Chronicler's `LayerRenderer` and `PromptContext`. Topic is steering. |
| `_closed/steering-and-guided-generation/research/03-sillytavern-core.md` | mixed | older upstream commit (`8172dcd0`) | 8 (4.5) | Same mapping; topic is steering. |
| `_closed/steering-and-guided-generation/research/04-design-synthesis.md` | contaminated | stale: uses the deprecated terms "Narrator Action" and "Replay blob" | 22 (27.8) | A Chronicler design, "re-verified against current chronicler source". |
| `_closed/guided-generations-architecture/research/marinara-terminology-cross-reference.md` | mixed | current for terms | 4 (3.2): `RetryMode`, `ReImpersonate`, `ReNarrate`, `UserRegen` | Terminology only. |

**Inferred:** the repo already has a layer that is meant to be the clean spec: `docs/AGENTS.md` defines the specs as "the behaviour a client observes through HTTP and browser interactions". A clean spec can start from `docs/specs/`. It needs four edits: remove the identifiers (`retrigger.md`, `reset.md`); turn mechanism phrases into observable outcomes; correct or open the stale claims of research 11; and add the behaviour rulings that no spec carries (decision 1 is the main one).

---

## 4. The role of the premise sources

### The question table

Sources: research 13 (R13) and research 14 (R14), by section; the Chronicler rulings come from research 11 (R11) and the seed. This table re-states those assets; it adds no new repo reading.

| Question | SillyTavern | Marinara Engine | Chronicler: owner ruling, or a spec or doc rule (source named) |
|---|---|---|---|
| Which state is bound to a Swipe? | The alternative text and a per-swipe metadata bundle (`swipe_info[i].extra`: reasoning, memory summary, translation, timings). The message-level fields mirror the active swipe. `swipe()` never touches `chat_metadata`, so chat variables and chat-scoped stores survive a switch. No world state exists. (R13 §1b, §2a, §2b) | Every store that must rewind is a row keyed by `(messageId, swipeIndex)`: tracker state (place, characters, stats), map position, turn-game state. Chat variables follow by replaying the swipe's `macroVariableChanges`. The inventory moves only when it still equals what the outgoing telling left. The journal, the discovered map and the NPC list are chat-global and do not rewind. (R14 Q2, Q3) | A Swipe holds the state its own generation produced, Quantifier included; the docs are right and the code is wrong (seed §1; R11 decision 1). Conflicting earlier ruling: captured pre-tail (R11 decision 3). An event Retry starts from the state of the Swipe it redoes (R11 decision 11). Open: one pointer or two (ticket 01). |
| What does a Retry cut and restore? | A swipe adds an alternative to the **last** message only. Its prompt is the history minus the target (`coreChat.pop()`). Regenerate deletes the last row in memory and appends a new row. Continue appends in place. Nothing is restored, because no world state exists. Older messages can be viewed, branched and deleted, not regenerated. (R13 §3) | Regenerate adds a swipe to **any** assistant message. It cuts the target and the later messages from the prompt only, never from storage. The new telling starts from the previous swipe's row, with no fallback to the latest row. It restores the stored guide and the speaker. An empty or aborted reply saves nothing, so no rollback exists. (R14 Q4) | A Retry adds a Swipe and re-applies the Swipe's stored inputs (R11 decision 6). Re-impersonate runs the full tail (decision 2). An event Retry restores the state of the Swipe it redoes (decision 11). A failed Retry that added no Swipe leaves the click-time world; one that added a Swipe keeps the world it wrote (decision 16). Swiping is bounded to the last Message (`storage_design.md`). No ruling: the rewind point of a Retry of a continue or post-impersonate Narration; whether a main Retry clears the stored trigger context (map fog "Retry tails"). |
| Which writers may run during a generation? | One client-side flag pair (`is_send_press \|\| is_group_generating`); no server lock. Each call site guards itself: swipe and message-move are blocked, edit is allowed, delete is blocked in the UI but open through slash commands. The stale-save guard is a per-chat integrity slug on a whole-file write, not a generation id. (R13 §5) | One generation per chat (`409`). Post-processing agents run after the reply, anchored to the `(message, swipe)` captured at save time, and can outlive the reply. A swipe switch is not refused; it is serialized per chat, and agent runs for an abandoned swipe abort. Edit and delete have no generation guard. Queues per message and per chat serialize writes. A write ordinal orders a per-anchor row against the chat-global cache. (R14 Q5) | A second Action or Retry during a generation gets "Still thinking..." and starts nothing (`swipe_new.md` 12.1, I.8). One process against one database (R11 decision 17). No ruling: whether edit, delete, swipe switch, reset and arrival narration must hold the slot; which Game a write belongs to (R11; map fog "The writer rule"). |
| Where do notices live outside the Message history? | Errors are toasts and console lines, never rows. `/comment` and help rows are `is_system` rows that the prompt filter drops. Narrator rows reach the model as `system`. `IGNORE_SYMBOL` hides a row from prompts in place. Each "last message" rule tests the last row on its own. (R13 §6) | Failures are SSE frames shown as toasts and are never stored. A notice that must persist but stay out of the prompt is a field on the message's `extra` (`gameDiceTurn`). Membership notices are ordinary rows, and they do reach the model. (R14 Q6) | A failed trigger continuation adds no story-log Message; the status reports it (R11 decision 13). A generation error shows as one clamped line with the raw text in a disclosure (`failure_display.md`). Open: ticket 02. |
| Where does the location live? | Core stores none. The nearest datum is a per-chat background file, which one quiet LLM call can choose from a closed list. (R13 §7) | Per telling, in the tracker row (`location`) and the map-position row (`currentLocationId`), both keyed by `(message, swipe)`. The transcript carries no location. The client writes the move into the player's own text (`*moves to <place>*`). The journal of places is chat-global and append-only. (R14 Q7) | The room name shows on the Narration that arrives in the room, as a header on its Swipe (R11 decision 14; `actions.md` 1.14). Deferred: whether the header must agree with the state a swipe switch restores (ticket 03). |

**Inferred from the table.** The two sources agree on three points: failures stay out of the history; one generation runs at a time per chat (client-side in SillyTavern, server-side in Marinara); edit has no generation guard. They disagree on what binds to a swipe, which message a regenerate may target, whether a regenerate deletes, and where the location lives. Chronicler's rulings sit near Marinara on state binding (decision 1) and near SillyTavern on last-message-only swiping. Neither source has a counterpart for a Quantifier pass that moves the player, or for a Trigger with an event continuation.

### The three roles and their costs

| Role | What the clean team gets | Cost | Effect on the comparison |
|---|---|---|---|
| **Requirements**: "behave as the premise source does, unless an owner ruling says otherwise" | The spec names one source per question, or one source for all | The sources disagree on four of five questions, so someone must pick a source per question. That is a design choice: either the owner makes it, or a code-aware spec writer does, which leaks. The spec grows by the chosen source's model. Chronicler-only concepts (Quantifier movement, Triggers, rooms) still need owner rulings. (inferred) | An agreement between the clean design and the code can come from the shared premise, not from the requirements. Both "independent" designs then copy the same original, and the NEC inference weakens. (inferred) |
| **Prior art**: the clean team may read the premise designs | The research 13 and 14 assets, without their "Patterns" sections | Moves the anchor from Chronicler's code to the premise designs. Two disagreeing designs may anchor less than one, but the clean team can adopt one whole. The two assets are 6,839 and 4,804 words of context. (inferred; word counts tested) | A difference from the code that matches a premise source is a known alternative, not an invention. The design must say which source each choice follows. (inferred) |
| **Excluded**: the clean team never sees them | Nothing about either source | Proven answers are re-invented or missed: Marinara's `(message, swipe)` key, SillyTavern's integrity slug. Exclusion is only partial: three of four probed models describe SillyTavern's `swipes` array, `swipe_id` and the untouched `chat_metadata` from training data; none knows Marinara's binding (tested, §5). The word "swipe" itself is SillyTavern's (inferred). | The cleanest test of what the requirements force. An agreement on a SillyTavern-like point can still be recall. (inferred) |

---

## 5. The procedure sketch

One run, with this repo's agent tooling. Each step names its actor.

### Step 1. The spec writer writes the specification

- **Actor:** a code-aware session of this map (it has read the code). It is the examiner.
- **Inputs:** the clean and mixed rows of §3, under the premise-source role that the owner chose.
- **Rules (from §2, inferred):**
  - Write a behaviour spec, not a design. Phoenix: describe what it does "without using or referencing any actual code". The chardet design document is the failure.
  - Start from `docs/specs/`. Remove the identifiers. Turn each mechanism phrase into an observable outcome: "the Quantifier re-ran" becomes "the player can move on the redo".
  - Restate each behaviour ruling of §3 as a numbered requirement with its source. Leave out the module-shape decisions.
  - List conflicts and open questions as open, numbered, and unanswered. The spec writer may not decide them: decision 1 against decision 3, the rewind point, the writer rule, the notice channel, the room.
  - Restate glossary terms by what they must do, not by when they are saved. Example: a Snapshot is defined by what a restore must give back.
  - Fix the black boxes: the narrator, Quantifier and Options calls, and the HTTP interface, if the scope keeps it (ticket 16 sets the scope).
- **Output:** `spec.md` with sections for terms, black boxes, requirements, environment constraints and open questions.

### Step 2. The reviewer checks the specification for leaks

- **Actor:** the reviewer, on the dirty side. It may read the code to find leaks.
- **Mechanical check (tested in this session):** run [research-15-leakcheck.py](research-15-leakcheck.py) on the spec, the task text and the agent-definition body. The target is zero `src/` names, zero `src/` paths and zero `.rs` names.
  - Known false positives: names that an outside system shares (`display_text`, `system_prompt` in research 13); interface names that the LLM sees (`npcs_in_room`).
  - Known false negatives: mechanism written in domain words. `swipe_new.md` scores zero and still says "the anchor message has no snapshot" and "the generation gate is busy".
- **Semantic check:** a fresh-context reviewer from a different model family than the spec writer, for example the `judge-glm` or `judge-kimi` seat. It flags each sentence that a client cannot observe through HTTP or the browser, and each sentence that names an internal step, an order of internal steps or a storage shape. This is the repo's own test for a spec (`docs/AGENTS.md`).
- **Owner sign-off:** the owner confirms the requirements, not the absence of leaks. Each owner edit is logged as an owner ruling.

### Step 3. The clean agent designs behind the wall

**The allow-list (by premise-source role; inferred from §3 and §4).** The clean agent reads only what its prompt embeds:

| Role | Allowed |
|---|---|
| every role | the spec and the task text, after Step 2 |
| prior art | add the research 13 and 14 assets, without their "Patterns" sections |
| requirements | nothing more: the spec carries the chosen source's behaviour as requirements |
| excluded | nothing more |

Not on the list in any role: `CONTEXT.md` as it is (its terms enter the spec rewritten, see Step 1); the Diátaxis docs; the specs as they are; `docs/external_applications/`; the closed prior-art research; every code-fact asset; this map and its tickets.

**The wall that was tested.** The probe (§Method and commands) used `--no-tools --no-context-files --no-skills --no-extensions --no-prompt-templates --no-mcp --no-approve`, plus only the provider extension of the model. Four model families gave the same result (tested):

- no callable tool;
- no `AGENTS.md`, `CLAUDE.md`, `CONTEXT.md` or `APPEND_SYSTEM.md` in context;
- no skill line in the system prompt;
- the system prompt still names the cwd, `/workspace/chronicler-engine`.

**The Herdsman form (inferred from the schema doc; not run).** A definition with `noTools: true` (or `tools: ["write"]` to write the design to a file; `write` cannot read), `noSkills: true`, `noExtensions: true`, `extensions` set to the provider extension only, a pinned `model`, `inheritProjectContext: false`, `inheritGlobalContext: false`, and no `agents`. The spec, and the premise assets if the role allows them, go in `files`; Herdsman embeds strict UTF-8 text. How Herdsman maps `inheritProjectContext: false` to `pi` flags is not stated in the doc (unknown). A dry run with a canary question, like this session's probe, would test it.

**Model choice (inferred).** All four probed models report no knowledge of this repo. A family other than the spec writer's family reduces the shared-writer channel of §1 limit 1. A second clean agent from a third family gives an N-version comparison: where the two clean designs differ from each other, the point is free or the spec is silent. That doubles the run and the comparison, and the agreements still correlate (Knight and Leveson).

**Leak paths and their closure:**

| # | Leak path | How it reaches the agent | Closed by | Label |
|---|---|---|---|---|
| 1 | Root `AGENTS.md`: its structure index names `src/` modules (72 distinct names; 217 `src/` paths and `.rs` file names) | `pi` loads context files from the cwd and its parents at start | `--no-context-files` | tested (probe; leak check) |
| 2 | `docs/AGENTS.md` (14 names) and `tests/AGENTS.md` (27 names; 116 paths and `.rs` file names) | The global extension `pi-on-demand-context` loads the `AGENTS.md` files on the path of any directory that a file tool touches. It loaded `docs/AGENTS.md` into this session after one `read` of `docs/specs/swipe_new.md`. | `--no-extensions`, or no file tools | tested (the load, observed here); inferred (the closure) |
| 3 | `.pi/APPEND_SYSTEM.md` (style rules and a pointer to `CONTEXT.md`); the project extension `.pi/extensions/mrn-context` (a `<pi-note>` that names the newest `logs/build_*.log`) | project-local resources | the probe's flag set (`--no-approve`, `--no-context-files`, `--no-extensions`) | tested for `APPEND_SYSTEM.md`: the probes quoted none of its rules, but the flag that closes it was not isolated; inferred for `mrn-context` |
| 4 | Skills in `.agents/skills/` (for example `codebase-design`, `improve-codebase-architecture`) and `~/.agents/skills/` | skill discovery lists them in the system prompt | `--no-skills` | tested |
| 5 | `src/`, `tests/`, `docs/diataxis/`, `.scratch/`, `tmp/`, `logs/`, `old-docs/`, `target/` | any file tool (`read`, `bash`, `grep`, `find`, `ls`); `read` accepts absolute paths, so a cwd outside the repo does not close this path | no file-reading tools | tested |
| 6 | git history | `bash` (`git log -p`) | no `bash` | tested |
| 7 | Session history: the `session_search`, `recall` and `session_read` tools, and 591 MB of transcripts in `~/.pi/agent/sessions/` | extension tools, or `read` by absolute path | `--no-extensions` and no file tools | tested |
| 8 | The web: `web_search` and `fetch_content` can reach `github.com/notsonormal/chronicler-engine` (public or private: unknown) and the premise repos | extension tools (`pi-web-access`) | `--no-extensions` | tested |
| 9 | Local clones of the premise sources: `tmp/sillytavern-13`, `/tmp/pi-github-repos/Pasta-Devs/Marinara-Engine` | file tools | no file tools | tested (both exist; no tools) |
| 10 | Chronicler names inside `docs/external_applications/marinara_engine.md` (§File Mapping, §Prompt Architecture) | an allow-listed input | leave the file out, or strip the two sections | tested (leak check) |
| 11 | Chronicler names in the closed prior-art research (22, 10, 8 and 4 names) | an allow-listed input | leave the files out, or strip them | tested (leak check) |
| 12 | This map's framing in research 13 and 14: the "Patterns" headings repeat the ticket titles | an allow-listed input | strip the "Patterns" sections | known (read) |
| 13 | The task text, the definition body and the spec | written by a code-aware session | Step 2 on all three | inferred |
| 14 | The cwd in the system prompt | `pi` puts the cwd in the system prompt. A fresh Herdsman delegation runs in the controller's cwd. | Start the clean agent in a neutral temporary directory (a raw `pi` run, or a Herdr pane with its own `cwd`) | tested (all four probes quote the path); known (Herdsman `agent.md`: "Fresh delegation runs in the calling controller's cwd.") |
| 15 | Extension discovery switched back on: when a definition inherits the controller's model and an extension supplies that model's provider, discovery stays on even with `noExtensions: true`. Every extension then loads: session search, web, on-demand context. | a Herdsman launch rule | Pin the model in the definition and list only its provider extension | known (the rule, schema doc); inferred (that every discovered extension then loads) |
| 16 | `ask_owner`: the owner has seen the code | stays available to a managed agent even with `noTools: true` | Log every question and answer. Count each answer as an owner ruling in the comparison. | known (schema doc) |
| 17 | Training data | the model has read public code | Cannot be closed. Choose a model with no recall of this repo. Record what it recalls of the premise sources. | tested (probe, below) |

**The training-data probe (tested, one run per model; a model's own report).**

| Model | Knows this repo? | SillyTavern: storage of swipe alternatives | SillyTavern: does a switch change `chat_metadata`? | Marinara: how state binds to swipes |
|---|---|---|---|---|
| `opencode-go/deepseek-v4.1-flash` | no | the `swipes` array with `swipe_id` | declined to say | no |
| `opencode-go/glm-5.3-flash` | no | per message: `swipes`, `swipe_id`, `swipe_info` | no | no |
| `opencode-go/kimi-k2.7-code` | no | the `swipes` array with `swipe_id` | no | no |
| `anthropic/claude-opus-5-5` | no | the `swipes` array, `swipe_id`, the parallel `swipe_info` | no | no |

Each SillyTavern answer matches research 13 §1a and §2a (known). So the models carry SillyTavern's core swipe model, and no model carries Marinara's.

**What the clean agent writes (from the ticket; plan, don't do).** A design, not code:

1. the states of a generation and what each holds;
2. what binds to a Swipe, and what a swipe switch restores;
3. what each Retry form cuts, restores and runs again;
4. who may write game state during a generation;
5. where notices and the location live;
6. an assumptions list: every point where the spec is silent and the design chose;
7. the questions it would ask the owner.

Keep its whole transcript (§2). After the run, run the leak check on the design as well. A distinctive `src/` name in it is a question for the wall, not a proof: a common name such as `GenerationStatus` can be invented again (inferred).

### Step 4. A code-aware session compares the design with the code

One row per design element. Columns: the clean design, the code (cite the symbol), the spec source, and the class.

| Class | Rule |
|---|---|
| Forced | The two agree, and the spec states the constraint that forces the shape. Record the other causes too: shared premise, vocabulary, recall. |
| Shared, not forced | The two agree, and the spec is silent. Not evidence of forcing. |
| Code defect | They differ, and the code breaks a spec requirement. |
| Spec gap | They differ, and the clean design listed an assumption at this point. |
| Free choice | They differ, and both meet the spec. The code's shape at this point is history. |

The disagreement list is the main result; the agreements are a sanity check (web sources §Does the convergence test transfer).

### What cannot be made clean

- **The owner's rulings and grilling answers.** The owner has seen the code. Every ruling in the spec, and every `ask_owner` answer, carries some of its shape.
- **The spec writer's selection.** A code-aware session chooses which requirements exist. The semantic review catches leaked mechanism; it does not catch a missing requirement.
- **The vocabulary.** `CONTEXT.md` defines **Snapshot**, **Action Pipeline** and **Agent** by the current mechanism (§3). A spec that says "Snapshot" presupposes that state is kept as stored copies. A spec that says "Quantifier" presupposes a separate analysis pass after narration.
- **The interface.** The specs fix the routes (`POST /swipe/new`, `POST /retrigger`, `POST /message/:id/swipe/:index`). A separate Retrigger route is a design choice.
- **The scenario shape.** 38 Given-steps in `actions.md` and `swipe_new.md` configure a narrator or Quantifier backend per call (tested count). They presuppose one LLM call per role.
- **Training data.** SillyTavern's swipe model (tested).

---

## 6. Order in the map

**Known (statuses):** ticket 01 is blocked by 04, 05, 06, 07, 11, 13 and 14, and all seven are resolved. Ticket 01 is on the frontier now. Ticket 03 waits on 01. A run takes about four sessions: spec, review, clean agent, comparison (inferred estimate).

| Order | What the spec can hold | What the comparison shows | Cost |
|---|---|---|---|
| **Before ticket 01** | The specs and the behaviour rulings so far (§3). The conflict between decisions 1 and 3, the rewind point, the writer rule, the notice channel and the room stay open, so the clean agent must choose there. | Ticket 01 is grilled next to a reference model that has no code shape. The clean choices on the open questions show the option space. | Delays ticket 01 and ticket 03 by about four sessions. On the open questions the clean design's choices are free by construction, so they are options, not evidence. The owner can anchor on the clean design in place of the code: a new anchor. (inferred) |
| **After tickets 01–03** | The specs and every ruling, including the model of ticket 01: a complete spec. | The shape that the ruled requirements imply. It feeds "Fix or rewrite" directly, and its disagreements point at history-shaped code. | Tickets 01–03 are decided with no reference point, which is the anchoring that the run is meant to counter. Their rulings are written in the code's terms (pointers, Snapshots), so the spec carries the code's shape and agreements are weak evidence. (inferred) |
| **In place of part of ticket 01** | As "before ticket 01". The clean agent proposes the state model: the states, the rewind and result binding, the writers. | Ticket 01 reviews a clean proposal against the code and the owner's intent, in place of building the model from the seed's S0–S5 diagram. | Ticket 01 changes shape. Its "done when" still needs a code-aware session to cite the function that writes each state. The owner's review answers still carry code knowledge. One model is written, not two. (inferred) |

A hybrid is possible: a run before ticket 01, and a second comparison after ticket 03. It costs two comparisons and a spec update (inferred).

**The rewind point.** [Research 16](research-16-clean-room-collapse.md) §4.2 reads prior decision 6 ("a new swipe redoes the last generation") as a fix for the rewind point (its rows O-1 and O-2). Research 11 lists the rewind point as unsettled. If the owner accepts that reading before the run, the rewind point leaves the open list of the first and third rows (inferred).

---

## 7. Decisions for the owner

This asset decides none of these. Each is a grilling decision.

1. Whether to run a clean-room design at all.
2. The scope of the run: the question of [Research: which known problems a clean-room design would collapse, at each scope](../issues/16-research-clean-room-collapse.md).
3. The role of the premise sources: requirements, prior art or excluded (§4).
4. The order: before ticket 01, after tickets 01–03, or in place of part of ticket 01 (§6). Ticket 01 is on the frontier now, so the first and third options close when a session takes ticket 01.
5. The wall: the tested raw `pi` run, or a Herdsman definition after a dry run (§5 Step 3); one clean agent or two.
6. Who reviews the spec, and whether the owner answers the clean agent's questions during the run or after it.
