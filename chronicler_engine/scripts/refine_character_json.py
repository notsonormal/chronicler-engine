#!/usr/bin/env python3
import json
import re
import glob
from pathlib import Path

def extract_section(text, section_name):
    # Match "Section_name: " until the next newline or end of file
    pattern = rf"{section_name}:\s*(.*?)(?=\n\w+:|\n\n|\Z)"
    match = re.search(pattern, text, re.IGNORECASE | re.DOTALL)
    if match:
        return match.group(1).strip()
    return ""

def process_file(file_path):
    with open(file_path, "r", encoding="utf-8") as f:
        data = json.load(f)
    
    desc = data.get("description", "")
    if not desc: return

    # Extract sections
    personality = extract_section(desc, "Personality")
    appearance = extract_section(desc, "Appearance")
    background = extract_section(desc, "Background")
    goals = extract_section(desc, "Goals")
    
    scenario_parts = []
    if background: scenario_parts.append(background)
    if goals: scenario_parts.append(f"Goals: {goals}")
    
    # Clean up description to mainly be intro and appearance
    intro_match = re.match(r"(.*?)(?=\n\w+:)", desc, re.DOTALL)
    clean_desc = ""
    if intro_match:
        clean_desc = intro_match.group(1).strip()
        
    if appearance:
        clean_desc += f"\n\nAppearance: {appearance}"

    # Only overwrite if we successfully found stuff
    if personality:
        data["personality"] = personality
    if scenario_parts:
        data["scenario"] = "\n\n".join(scenario_parts)
    if clean_desc:
        data["description"] = clean_desc

    with open(file_path, "w", encoding="utf-8") as f:
        json.dump(data, f, indent=4)
    print(f"Refined -> {file_path}")

def main():
    target_dir = Path("/workspaces/mrn-general/chronicler_engine/data/characters")
    for file in target_dir.glob("*.json"):
        process_file(file)

if __name__ == "__main__":
    main()
