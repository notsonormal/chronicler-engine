"""Count story-truth facts in stored trial packs, one row per trial, beside the numbers the pack already has.

Every count is a plain text rule, so two runs of this script give the same numbers. A count is a signal, not a
verdict: each one has a note on what it can miss. Read-only.

    python3 tmp/fable-audit/evidence/trial_truth_counts.py tmp/lt01/redmist_julian/trial-1 tmp/lt03/redmist_julian/trial-1 ...
"""
import collections
import json
import re
import statistics
import sys

ROSTER = ["carla", "jezebel", "lisette", "louise", "gabriella"]
ROOM_WORDS = {
    "front_gates": ["gate"],
    "courtyard": ["courtyard"],
    "gardens": ["garden"],
    "entrance_hall": ["entrance hall", "foyer", " hall"],
    "library": ["library", "study"],
    "master_quarters": ["master quarters", "master bedroom", "bedroom"],
    "kitchen": ["kitchen"],
    "dining_room": ["dining"],
    "staff_quarters": ["staff quarters", "servants"],
    "financial_office": ["office"],
    "guest_wing": ["guest wing", "guest room"],
}
OFF_ESTATE = r"\b(bank|harbou?r|town|village|market|shop|street|church|inn|tavern|dock|quay|chandlery|registrar|almshouse|station|road to)\b"
MODERN = r"\b(phone|app|sunglasses|laptop|car|email|text message|screen|wifi|internet|camera|taxi)\b"
PERIOD = r"\b(carriage|farrier|bridle|lamplight|gasworks|shillings?|sennight|michaelmas|candle|gaslight|coachman|horses?|quill|telegram)\b"
TITLE_NAME = r"\b(?:Mr|Mrs|Miss|Ms|Dr|Captain|Father|Sister|Lady|Lord|Sir|Madame|Master)\.? ([A-Z][a-z]+)"
STOP_NAMES = {"julian", "redmist", *ROSTER}


def state_of(entry):
    state = entry["state"]
    return json.loads(state) if isinstance(state, str) else state


def kind_of(entry):
    return "Event" if "EventNarration" in entry["kind"] else entry["kind"].strip('"')


def words(text):
    return re.findall(r"[a-z']+", text.lower())


def ngrams(tokens, size):
    return {tuple(tokens[i:i + size]) for i in range(len(tokens) - size + 1)}


def row(pack):
    story = json.load(open(f"{pack}/story.json"))
    summary = json.load(open(f"{pack}/summary.json"))
    out = collections.OrderedDict()
    out["pack"] = pack.replace("tmp/", "").replace("/redmist_julian", "").replace("/trial-1", "")
    out["entries"] = len(story)
    out["cached %"] = round(100 * summary["cached"] / summary["tokens_in"], 1)
    out["tokens out"] = summary["tokens_out"]

    rooms, presents = [], []
    for entry in story:
        state = state_of(entry)
        rooms.append(state["movement"]["current_room_id"])
        presents.append({c["id"] for c in state["scene"]["npcs_in_area"]})
    kinds = [kind_of(e) for e in story]
    narr = [i for i, k in enumerate(kinds) if k in ("Narration", "Event")]
    inputs = [i for i, k in enumerate(kinds) if k == "ImpersonateInput"]

    out["Rooms used"] = len(set(rooms))
    out["added Rooms"] = len(state_of(story[-1])["movement"]["dynamic_rooms"])
    out["Room changes"] = sum(1 for i in range(1, len(story)) if rooms[i] != rooms[i - 1])
    # An Input is the Persona's attempt. A state change on it happens before any Narration has answered.
    out["state changes on an Input"] = sum(
        1 for i in inputs if i and (rooms[i] != rooms[i - 1] or presents[i] != presents[i - 1]))

    # The text names a place off the estate while the record holds a map Room.
    out["Narr. off estate, map Room recorded"] = sum(
        1 for i in narr if len(re.findall(OFF_ESTATE, story[i]["text"], re.I)) >= 2 and not rooms[i].startswith("dynamic_"))
    # The text names one or more map Rooms by their words, and none of them is the recorded Room.
    def named_rooms(text):
        low = text.lower()
        return {room for room, keys in ROOM_WORDS.items() if any(k in low for k in keys)}
    out["Narr. names other Rooms only"] = sum(
        1 for i in narr if (n := named_rooms(story[i]["text"])) and rooms[i] not in n and not rooms[i].startswith("dynamic_"))

    named = [{c for c in ROSTER if c in story[i]["text"].lower()} for i in narr]
    out["Narr. with no roster Character"] = sum(1 for n in named if not n)
    out["Narr. names a Character not recorded present"] = sum(1 for i, n in zip(narr, named) if n - presents[i])
    # A Character recorded present who is named in none of the last five Narrations.
    stale = 0
    for position, i in enumerate(narr):
        if position < 4:
            continue
        recent = set().union(*named[position - 4:position + 1])
        stale += 1 if presents[i] - recent else 0
    out["Narr. with a present Character unnamed for 5"] = stale

    titled = collections.Counter()
    for entry in story:
        for name in re.findall(TITLE_NAME, entry["text"]):
            if name.lower() not in STOP_NAMES:
                titled[name] += 1
    out["titled names not in the World (3+ uses)"] = sum(1 for count in titled.values() if count >= 3)
    out["their uses"] = sum(count for count in titled.values() if count >= 3)

    text_all = " ".join(e["text"] for e in story)
    out["modern words"] = len(re.findall(MODERN, text_all, re.I))
    out["period words"] = len(re.findall(PERIOD, text_all, re.I))
    last_modern = max((i for i, e in enumerate(story) if re.search(MODERN, e["text"], re.I)), default=-1)
    first_period = min((i for i, e in enumerate(story) if re.search(PERIOD, e["text"], re.I)), default=-1)
    out["first period entry"] = first_period
    out["last modern entry"] = last_modern

    tokens = [words(story[i]["text"]) for i in narr]
    grams = collections.Counter()
    for t in tokens:
        grams.update(ngrams(t, 5))
    out["5-word phrases in 5+ Narrations"] = sum(1 for count in grams.values() if count >= 5)
    openers = collections.Counter(" ".join(t[:2]) for t in tokens if len(t) > 1)
    out["top 2-word opener"] = "%s (%d)" % openers.most_common(1)[0]
    out["Narr. ends on a question"] = sum(1 for i in narr if story[i]["text"].rstrip().rstrip('"”*').endswith("?"))
    out["Narr. words, median"] = int(statistics.median(len(t) for t in tokens))
    in_tokens = [words(story[i]["text"]) for i in inputs]
    out["Input words, median"] = int(statistics.median(len(t) for t in in_tokens)) if in_tokens else 0
    # An Impersonate Input that holds a quoted line and a named roster Character acting: the Persona narrates others.
    speaks_for_others = 0
    for i in inputs:
        text = story[i]["text"]
        if re.search(r"\b(%s)\b[^.\"“”]{0,60}\b(said|replied|answered|asked|laughed|smiled|nodded|whispered|murmured)\b" % "|".join(ROSTER), text, re.I):
            speaks_for_others += 1
    out["Inputs that write a Character's reaction"] = speaks_for_others
    out["Inputs"] = len(inputs)
    out["Narrations"] = len(narr)
    return out, titled


def main(packs):
    rows, names = [], {}
    for pack in packs:
        r, titled = row(pack)
        rows.append(r)
        names[r["pack"]] = titled.most_common(6)
    keys = list(rows[0].keys())
    print("| Count | " + " | ".join(r["pack"] for r in rows) + " |")
    print("|---|" + "---|" * len(rows))
    for key in keys[1:]:
        print(f"| {key} | " + " | ".join(str(r[key]) for r in rows) + " |")
    print()
    for pack, top in names.items():
        print(f"{pack}: " + ", ".join(f"{n} {c}" for n, c in top))


if __name__ == "__main__":
    main(sys.argv[1:])
