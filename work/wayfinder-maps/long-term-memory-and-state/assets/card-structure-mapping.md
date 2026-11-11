# The card structure mapping: the six Redmist cards

Made in round 2 of [Decide the structure of a Character card and who knows each part](../issues/10-card-structure.md) (2026-11-04). **It is not the build.** The build uses the minimal form of the Answer of ticket 10. This file is the card data of arm 4 (the full split) of [Make the card probe](../issues/11-card-probe.md). The **check** rows were not checked one by one: the probe uses the proposal of each row. Round 3 replaced the `known_by` field with words in the text, so the levels below are labels of the probe arm only.

## The fields

| Field | Level (`known_by`) | What it holds |
|---|---|---|
| `name` | `everyone` | The short name |
| `role` | `everyone` | Her place in the house. The World rules give each role too |
| `appearance` | `seen` | What anyone sees when she is present |
| `facts` | `cast` | True facts that the other Characters know. The Persona learns them in play |
| `internals` | `[her]` | What only she knows. It drives her behaviour |
| `relationships[].static` | `[her]` | Her view of another Character |
| `personality` | writing guide | How to write her. It is not a fact of the story, so no Character "knows" it |
| `example_dialogue` | writing guide | A sample of her voice. Open: round 3, Q1 |
| `summary` | — | Removed (round 2, Q3) |

Rule of the split: the build moves whole sentences, or whole clauses where a sentence holds two levels. It writes no new prose. A row marked **check** is a split that the owner should look at.

Each row quotes the start and the end of the moved text. The full text is in `data/characters/redmist_estate/*.json` and `data/personas/julian.json`.

## Carla (`carla.json`)

| Field | Text |
|---|---|
| `role` | "bodyguard of {{user}}" (from `description`, line 1) |
| `appearance` | "Relatively attractive, athletic curves … imposing presence." |
| `facts` | "Carla Jess", "30-year-old" (`description`, line 1). `scenario`: "Carla grew up in a middle-class household … for an allowance." "Despite her peaceful … coolest thing she could do." "Disregarding her parents' wishes … fitness requirements." "Carla was shipped to war after a surprisingly short training course and was met with the horrors of war". "Although she gained many useful skills … introduced her to the Redmist Estate." "At first, Carla was a simple gate security guard … Redmist family patriarch." Goals: "After Bernard passed away … the heir to his wealth, {{user}}." |
| `internals` | **check** "leaving many scars, both physical and mental, and causing her to turn to alcohol and cigarettes to cope" (the second half of the war sentence). Goals: "Carla is confident in her ability … faced with Bernard." Her three relationship lines |
| writing guide | `personality`. **check** "Carla will never speak for {{user}} under any circumstances." It is an instruction to the model, not a fact. Proposal: move it to the writing guide, or drop it, because the system preset already says that the narrator never plays the protagonist |

## Gabriella (`gabriella.json`)

| Field | Text |
|---|---|
| `role` | "widow of {{user}}'s grandfather" (from `description`, line 1) |
| `appearance` | "Extremely attractive, beautiful … lots of jewelry and brands." |
| `facts` | "Gabriella Redmist", "31-year-old unemployed" (`description`, line 1). `scenario`: "Gabriella struck gold when she met Bernard Redmist, an incredibly rich man looking for a partner after his beloved wife had passed away." **check** "However, Bernard recently passed away and, to the shock of everyone in the family, left almost all his fortune to a grandchild few knew about" |
| `internals` | `scenario`: "Gabriella grew up in a rather rural area … incredibly boring and dull." "Gabriella's parents always wanted her … fishing equipment store." "Instead, Gabriella moved out and cut contact … turned 18." "Gabriella moved to the big city … lavish houses to live in." "Gabriella quickly took to seducing Bernard … richest younger women in the city." **check** "forcing Gabriella to adjust her plans of continuing her life of luxury devoid of struggle" (the end of the will sentence). Goals: "Gabriella only follows money. … luxurious lifestyle." Her three relationship lines |
| writing guide | `personality` ("Opportunist, cunning, manipulative, catty, liar, fake, seductive.") |

Note: the words "manipulative" and "liar" in `personality` tell the narrator how to write her. In the writing guide they are not a fact that Julian knows. The World rule "Staff consider her arrogant, demanding, and manipulative" is the `cast` fact.

## Jezebel (`jezebel.json`)

| Field | Text |
|---|---|
| `role` | "financial advisor assigned to watch over {{user}} and their inheritance" |
| `appearance` | "Very attractive, pretty … suits and pencil skirts." |
| `facts` | "Jezebel Coleman", "40-year-old" (`description`, line 1). `scenario`: "Jezebel was born into wealth and prosperity. All of her family were in some kind of financial occupation … executives." "Jezebel's academic career … successful future." "Jezebel was hired as an analyst for the estate and quickly took to him." "Jezebel admired his generosity … charity events." "Jezebel became close with Bernard, enough to consider him a friend". "Jezebel was very sad when Bernard passed away." **check** Goals: "Jezebel promised to ensure Bernard's final wishes were kept. She wants to watch over {{user}} … greedy people." Proposal: `facts`, because it is her open task, and the World rules say it |
| `internals` | `scenario`: "It was only reasonable that the expectation … on Jezebel's shoulders." "A majority of Jezebel's childhood was controlled … her parents' careers." "Despite the success, Jezebel felt empty … the hole in her heart. That was until she met Bernard." **check** "and had many conversations about his past regrets and worries about his will" (the end of the friend sentence). Her three relationship lines |
| writing guide | `personality` |

## Lisette (`lisette.json`)

| Field | Text |
|---|---|
| `role` | "maid of {{user}}" |
| `appearance` | "Attractive, pretty, curvy figure … stockings beneath." |
| `facts` | "Lisette Franklin", "28-year-old" (`description`, line 1). `scenario`: "Lisette has been an orphan her whole life. Her mother died during labour, and her father couldn't be found." "After becoming 18 and being moved out of the orphanage, however, she needed a place to work. Fortunately, the Redmist Estate was hiring, and Lisette applied". "For almost a decade, she has been working for the estate" (the A-3.3 fact). Goals: "After being informed of her promotion … to aid them." |
| `internals` | `scenario`: "Lisette never intended to become a maid … becoming a lawyer." "remembering the many donations … by the family" (her reason to apply). "and finds some purpose in helping maintain the mansion's grandness". Her three relationship lines |
| writing guide | `personality`. **check** "It was likely the lack of true maternal love … unaffected by the world around her." "Despite her quiet nature … a large part of Lisette's life." These explain her nature, so they guide the writing. They are not facts |

## Louise (`louise.json`)

| Field | Text |
|---|---|
| `role` | "personal chef of {{user}}" |
| `appearance` | "Attractive, slim curves … looser clothing like tank tops." |
| `facts` | "Louise Petit" (the A-3.4 and B-3.3 fact), "25-year-old" (`description`, line 1). `scenario`: all of it, from "Louise was born and raised in Paris" to "the newest owner of the estate, {{user}}." Goals: "Louise doesn't know a lot about {{user}} … they can enjoy." |
| `internals` | **check** "It was the highest-paying job she'd ever found, encouraging her to stay". Proposal: `facts`, because Louise is open and talks a lot. Her three relationship lines |
| writing guide | `personality` |

## Julian (`julian.json`, the Persona card)

| Field | Text |
|---|---|
| `name` | "Julian" |
| `role` | Empty. No card text gives it. The World rules say that he is the heir |
| `appearance` | `description`: "Julian Redmist is a relatively attractive man in his mid-20s … respectful appearance." |
| `facts` | `scenario`: "Now, Julian has arrived at the Redmist Estate to claim his heritage, accompanied by his new bodyguard, Carla Jess." **check** "never knowing his grandfather Bernard Redmist". The World says "a grandson he never knew", so the cast knows it |
| `internals` | `scenario`: "Julian was raised by a single mother in a city far from Islaport". "He lived an ordinary life until Bernard's death and the revelation of his will." |
| writing guide | `personality`, `example_dialogue` |

## The World rules (`world.json`, `global_rules`)

| Rule | `known_by` |
|---|---|
| "Bernard Redmist died two weeks ago. …" | `everyone` |
| "Gabriella Redmist, Bernard's 31-year-old widow, still resides in the estate's guest wing." | `everyone` |
| "Staff consider her arrogant, demanding, and manipulative." (the second sentence of the rule above) | `cast`. **check**: the rule splits in two |
| "Carla Jess is Julian's personal bodyguard …" | `everyone` |
| "Jezebel Coleman manages the estate finances …" | `everyone` |
| "The Redmist Estate employs a small staff …" | `everyone` |
| "Islaport is a large coastal city …" | `everyone` |

## Sizes (inferred, characters ÷ 4)

- The five Character cards without `example_dialogue` and without `summary`: about 2,600 tokens, as before.
- `example_dialogue` of all six cards: about 1,330 tokens (Carla 244, Gabriella 246, Jezebel 315, Lisette 236, Louise 261, Julian 29).
