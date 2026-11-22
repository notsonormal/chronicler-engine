"""The seven lost facts: write scene entries over the older stories P (lt01) and Q (lt03).

Their stored requests have no <ConversationHistory> block, so the tool cannot find their
history cuts. The probe cuts each story into parts of about 4,000 tokens of Messages (the
size of one cut of story A), from the first Message, and chains the calls as `append_cut` does.
"""
import json, sys, uuid
from pathlib import Path
sys.path.insert(0, "work/wayfinder-maps/long-term-memory-and-state/assets")
import memory_test as mt

CAND = mt.load_candidate("work/wayfinder-maps/long-term-memory-and-state/assets/candidates/scene_entries.json")


class OldPack(mt.Pack):
    def __init__(self, folder):
        self.folder = Path(folder)
        self.story = mt.trial.read_story(self.folder)
        self.by_id = {e["id"]: e for e in self.story}


def main(folder, out, last):
    pack = OldPack(folder)
    out = Path(out); out.mkdir(parents=True, exist_ok=True)
    model = mt.Model("deepseek-v4.1-flash", 0.0, f"ltm16-lost-{uuid.uuid4().hex[:8]}")
    mem = mt.Memories(pack, CAND, model, out)
    story = [e for e in pack.story if e["id"] <= last]
    previous, i = None, 0
    while i < len(story):
        part, size = [], 0
        while i < len(story) and (not part or size + mt.tokens_of(story[i]["text"]) <= 4000):
            size += mt.tokens_of(story[i]["text"]); part.append(story[i]); i += 1
        cut = i < len(story) and story[i]["id"] or part[-1]["id"] + 1
        rec = mem.stored_json(f"cut-{cut}", lambda: mem.scene_call(part, previous))
        print(cut, rec["first"], rec["last"], rec["failed"], len(rec["entries"]))
        if rec["failed"]:
            i = story.index(part[0])  # not expected; stop rather than loop
            break
        previous = rec["entries"][-1]
    print(model.cost)


if __name__ == "__main__":
    main(sys.argv[1], sys.argv[2], int(sys.argv[3]))
