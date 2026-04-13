#!/usr/bin/env python3
"""
Extract and process images from SillyTavern character cards.
Creates two versions:
- Original: Full image saved as-is
- Cropped: Portrait crop (top 40%) for profile images
"""

import sys
import base64
import json
import os
import argparse
from pathlib import Path

try:
    from PIL import Image
except ImportError:
    print("Error: Pillow library required. Run 'pip install pillow'")
    sys.exit(1)


def extract_images(file_path: Path, output_dir: Path) -> str | None:
    """Extract character image from PNG and save full + cropped versions."""
    try:
        img = Image.open(file_path)
        chara_str = img.info.get("chara")

        if not chara_str:
            print(f"[{file_path.name}] No 'chara' metadata found. Skipping.")
            return None

        raw_json = base64.b64decode(chara_str).decode("utf-8")
        data = json.loads(raw_json)
        cd = data.get("data", data)

        name = cd.get("name", "Unknown")
        char_id = name.lower().replace(" ", "_")

        # Handle different image formats in SillyTavern
        # The main image might be in different formats
        main_img = None

        # Try to get the avatar/selected image from JSON
        if "avatar" in cd:
            try:
                avatar_data = cd["avatar"]
                # Sometimes it's base64 encoded
                if isinstance(avatar_data, str):
                    # Check if it's base64 or a filename
                    if "," in avatar_data:  # Data URL
                        import re

                        match = re.search(r"data:image/(\w+);base64,(.+)", avatar_data)
                        if match:
                            fmt = match.group(1)
                            img_data = base64.b64decode(match.group(2))
                            main_img = Image.open(io.BytesIO(img_data))
            except Exception as e:
                print(f"[{file_path.name}] Avatar decode error: {e}")

        # Fallback: try to get from PNG directly - SillyTavern stores
        # the main image as the base PNG itself (we already have it as 'img')
        if main_img is None:
            main_img = img.convert("RGBA") if img.mode != "RGBA" else img.copy()

        # Create output directory
        os.makedirs(output_dir, exist_ok=True)

        # Save full version
        full_path = output_dir / f"{char_id}.png"
        main_img.save(full_path)
        print(f"[{file_path.name}] Full image saved to -> {full_path}")

        # Create cropped portrait version (top 40%)
        w, h = main_img.size
        crop_height = int(h * 0.4)
        cropped = main_img.crop((0, 0, w, crop_height))

        crop_path = output_dir / f"{char_id}_crop.png"
        cropped.save(crop_path)
        print(f"[{file_path.name}] Cropped portrait saved to -> {crop_path}")

        return char_id

    except Exception as e:
        print(f"[{file_path.name}] Error processing: {e}")
        import traceback

        traceback.print_exc()
        return None


def main():
    parser = argparse.ArgumentParser(
        description="Extract images from SillyTavern PNG character cards."
    )
    parser.add_argument(
        "input", nargs="+", help="Input PNG files or directories containing PNG files"
    )
    parser.add_argument(
        "--out", default="data/images", help="Output directory for images (default: data/images)"
    )
    args = parser.parse_args()

    output_dir = Path(args.out)

    files_to_process = []
    for path_str in args.input:
        path = Path(path_str)
        if path.is_dir():
            files_to_process.extend(path.glob("*.png"))
        elif path.is_file() and path.suffix.lower() == ".png":
            files_to_process.append(path)

    if not files_to_process:
        print("No PNG files found to process.")
        sys.exit(0)

    print(f"Found {len(files_to_process)} PNG files to process.\n")

    for file in files_to_process:
        extract_images(file, output_dir)

    print(f"\nDone! Images saved to: {output_dir}")


if __name__ == "__main__":
    main()
