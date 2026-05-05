import json
import jsonschema
import sys
from pathlib import Path


def load_json(path):
    with open(path, "r", encoding="utf-8") as f:
        return json.load(f)


def validate_group(files, schema, label):
    """Validate a group of JSON files against a schema. Returns error count."""
    errors = 0
    for file_path in files:
        try:
            data = load_json(file_path)
            jsonschema.validate(instance=data, schema=schema)
            print(f"PASS: {file_path}")
        except jsonschema.exceptions.ValidationError as e:
            print(f"FAIL: {file_path}\n{e.message}")
            errors += 1
    return errors


def check_asset(base_dir, asset_path, context):
    """Check that an asset path exists. Returns 1 on missing asset, 0 otherwise."""
    if asset_path:
        full_path = base_dir / asset_path
        if not full_path.exists():
            print(f"FAIL: {context}\n  Asset not found: '{asset_path}'")
            return 1
    return 0


def validate_all():
    base_dir = Path(__file__).parent.parent
    data_dir = base_dir / "data"
    schemas_dir = data_dir / "schemas"

    if not schemas_dir.exists():
        print("Schemas directory not found.")
        sys.exit(1)

    world_schema = load_json(schemas_dir / "world.schema.json")
    map_schema = load_json(schemas_dir / "map.schema.json")
    char_schema = load_json(schemas_dir / "character.schema.json")

    errors = 0

    # Validate worlds, maps, personas, and characters
    errors += validate_group(data_dir.glob("worlds/*/world.json"), world_schema, "worlds")
    errors += validate_group(data_dir.glob("worlds/*/map.json"), map_schema, "maps")
    errors += validate_group(data_dir.glob("personas/*.json"), char_schema, "personas")
    errors += validate_group(data_dir.glob("characters/**/*.json"), char_schema, "characters")

    if errors > 0:
        print(f"Data validation failed with {errors} errors.")
        sys.exit(1)

    # Relational Validation
    print("Running relational validation...")
    for world_file in data_dir.glob("worlds/*/world.json"):
        world_data = load_json(world_file)
        world_dir = world_file.parent
        world_id = world_data.get("id")

        # Load map to collect room IDs
        map_file_name = world_data.get("map_file", "map.json")
        map_file = world_dir / map_file_name
        valid_room_ids = set()

        if map_file.exists():
            map_data = load_json(map_file)
            for region in map_data.get("overworld", {}).get("regions", []):
                for room in region.get("rooms", []):
                    valid_room_ids.add(room.get("id"))
        else:
            print(f"FAIL: Map file missing for world {world_id}: {map_file}")
            errors += 1
            continue

        # Check starting_room_id
        starting_room_id = world_data.get("starting_room_id", "start")
        if starting_room_id not in valid_room_ids:
            print(f"FAIL: {world_file}\n  starting_room_id '{starting_room_id}' not found in map")
            errors += 1

        # Check player_file
        player_file_name = world_data.get("player_file", "player.json")
        player_file = data_dir / "personas" / player_file_name
        if not player_file.exists():
            print(f"FAIL: {world_file}\n  player_file '{player_file_name}' not found in data/personas/")
            errors += 1

        errors += check_asset(base_dir, world_data.get("default_room_image"), f"{world_file}")

        # Check characters for this world
        chars_dir_name = world_data.get("characters_dir", world_id)
        chars_dir = data_dir / "characters" / chars_dir_name

        if map_file.exists():
            for region in map_data.get("overworld", {}).get("regions", []):
                for room in region.get("rooms", []):
                    errors += check_asset(
                        base_dir,
                        room.get("image_path"),
                        f"Map {map_file.name} Room {room.get('id')}"
                    )
                    for npc_id in room.get("npcs", []):
                        npc_file = chars_dir / f"{npc_id}.json"
                        if not npc_file.exists():
                            print(
                                f"FAIL: Map {map_file.name} Room {room.get('id')}\n"
                                f"  References missing NPC '{npc_id}' ({npc_file})"
                            )
                            errors += 1

        if chars_dir.exists():
            for char_file in chars_dir.glob("*.json"):
                char_data = load_json(char_file)
                errors += check_asset(base_dir, char_data.get("profile_image"), f"{char_file}")
                errors += check_asset(base_dir, char_data.get("headshot_image"), f"{char_file}")

                triggers = char_data.get("triggers", [])
                for idx, trigger in enumerate(triggers):
                    room_id = trigger.get("room_id")
                    if room_id and room_id not in valid_room_ids:
                        print(f"FAIL: {char_file}")
                        print(f"  Trigger[{idx}] references non-existent room_id: '{room_id}'")
                        errors += 1

    # Also check persona assets
    for char_file in data_dir.glob("personas/*.json"):
        char_data = load_json(char_file)
        errors += check_asset(base_dir, char_data.get("profile_image"), f"{char_file}")
        errors += check_asset(base_dir, char_data.get("headshot_image"), f"{char_file}")

    if errors > 0:
        print(f"Relational validation failed with {errors} errors.")
        sys.exit(1)
    else:
        print("All data files validated successfully.")


if __name__ == "__main__":
    validate_all()
