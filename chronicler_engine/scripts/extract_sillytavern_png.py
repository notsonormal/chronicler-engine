"""Extract embedded PNG images from SillyTavern character cards."""

import argparse
import base64
import json
import os
import sys
from pathlib import Path

try:
    from PIL import Image
except ImportError:
    print("Error: Pillow library required. Run 'pip install pillow'")
    sys.exit(1)


def extract_png(file_path, output_char_dir):
    try:
        img = Image.open(file_path)
        chara_str = img.info.get("chara")
        if not chara_str:
            print(f"[{file_path.name}] No 'chara' metadata found. Skipping.")
            return []

        raw_json = base64.b64decode(chara_str).decode("utf-8")
        data = json.loads(raw_json)
        cd = data.get("data", data)

        name = cd.get("name", "Unknown")
        char_id = name.lower().replace(" ", "_")

        npc_card = {
            "id": char_id,
            "name": name,
            "description": cd.get("description", "").replace("{{char}}", name),
            "personality": cd.get("personality", "").replace("{{char}}", name),
            "scenario": cd.get("scenario", "").replace("{{char}}", name),
            "example_dialogue": cd.get("first_mes", "").replace("{{char}}", name),
            "inventory": [],
        }

        char_out = Path(output_char_dir) / f"{char_id}.json"
        with open(char_out, "w", encoding="utf-8") as f:
            json.dump(npc_card, f, indent=4)
        print(f"[{file_path.name}] Character saved to -> {char_out}")

        world_rules = []
        book = cd.get("character_book")
        if book and "entries" in book:
            for entry in book["entries"]:
                content = entry.get("content")
                if content:
                    world_rules.append(content)

        return world_rules

    except Exception as e:
        print(f"[{file_path.name}] Error processing: {e}")
        return []


def main():
    parser = argparse.ArgumentParser(
        description="Extract SillyTavern JSON metadata from V2 PNG character cards."
    )
    parser.add_argument("input", nargs="+", help="Input PNG files or directories")
    parser.add_argument(
        "--char-out", default="data/characters", help="Output directory for character JSON files"
    )
    parser.add_argument(
        "--world-out", default="data/world", help="Output directory for world JSON files"
    )
    parser.add_argument(
        "--world-name", default="Extracted World", help="Name of the generated World Card"
    )
    args = parser.parse_args()

    os.makedirs(args.char_out, exist_ok=True)

    files_to_process = []
    for path_str in args.input:
        path = Path(path_str)
        if path.is_dir():
            files_to_process.extend(path.glob("*.png"))
        elif path.is_file():
            files_to_process.append(path)

    if not files_to_process:
        print("No PNG files found to process.")
        sys.exit(0)

    print(f"Found {len(files_to_process)} PNG files to process.")

    all_world_rules = set()
    for file in files_to_process:
        rules = extract_png(file, args.char_out)
        for r in rules:
            all_world_rules.add(r)

    if all_world_rules:
        os.makedirs(args.world_out, exist_ok=True)
        world_card = {
            "name": args.world_name,
            "description": "The world setting derived from character lorebooks.",
            "global_rules": list(all_world_rules),
        }

        world_filename = args.world_name.lower().replace(" ", "_") + ".json"
        world_out = Path(args.world_out) / world_filename

        with open(world_out, "w", encoding="utf-8") as f:
            json.dump(world_card, f, indent=4)
        print(f"Worldwide metadata ({len(all_world_rules)} rules) saved to -> {world_out}")


if __name__ == "__main__":
    main()
