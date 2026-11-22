"""Probe: one writer call over four cuts of story A (about 15,500 tokens of Messages), as at a larger history size."""
import json, sys, uuid
from pathlib import Path
sys.path.insert(0, "work/wayfinder-maps/long-term-memory-and-state/assets")
import memory_test as mt
pack = mt.Pack(mt.DEFAULT_PACK)
cand = mt.load_candidate("work/wayfinder-maps/long-term-memory-and-state/assets/candidates/scene_entries.json")
part = [e for e in pack.story if e["id"] < 615]
print("message tokens", sum(mt.tokens_of(e["text"]) for e in part))
for cap in (2000, 8000):
    cand["max_tokens"] = cap
    model = mt.Model("deepseek-v4.1-flash", 0.0, f"ltm16-big-{uuid.uuid4().hex[:8]}")
    mem = mt.Memories(pack, cand, model, Path("tmp/ltm16/big"))
    rec = mem.scene_call(part, None)
    Path(f"tmp/ltm16/big/cap-{cap}.json").write_text(json.dumps(rec, indent=1))
    print(cap, rec["failed"], rec["finish_reason"], rec["prompt_tokens"], rec["completion_tokens"], len(rec["entries"]))
