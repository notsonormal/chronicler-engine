import json
import sys

sys.stdout.reconfigure(encoding="utf-8")

import os

coverage_path = "tmp/coverage/coverage.json"
if not os.path.exists(coverage_path):
    coverage_path = "coverage.json"  # fallback for local dev

with open(coverage_path) as f:
    data = json.load(f)

totals = data["data"][0]["totals"]["lines"]
print("=" * 60)
print("COMBINED COVERAGE (lib + fragment_tests):")
print(f"Lines: {totals['covered']}/{totals['count']} = {totals['percent']:.1f}%")
print("=" * 60)

files = data["data"][0]["files"]

print("\nFILES BELOW 80% COVERAGE:")
print("-" * 60)

for f in files:
    filename = f["filename"]
    short_name = (
        filename.split("chronicler_engine/")[-1] if "chronicler_engine/" in filename else filename
    )

    summary = f.get("summary", {})
    line_info = summary.get("lines", {})
    covered = line_info.get("covered", 0)
    total = line_info.get("count", 0)
    if total > 0:
        pct = (covered / total) * 100
        if pct < 80:
            print(f"[LOW] {short_name}: {covered}/{total} = {pct:.0f}%")
