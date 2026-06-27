import os

SRC = "E:/John/Github/mrn-general/chronicler_engine/src/storage/backend"

OP_MAP = {
    "Operation::SaveSnapshot": '"save_snapshot"',
    "Operation::LoadLatestSnapshot": '"load_latest_snapshot"',
    "Operation::LoadSnapshotById": '"load_snapshot_by_id"',
    "Operation::ListGames": '"list_games"',
    "Operation::CreateGame": '"create_game"',
    "Operation::DeleteGame": '"delete_game"',
    "Operation::GetGame": '"get_game"',
    "Operation::InsertMessage": '"insert_message"',
    "Operation::DeleteMessage": '"delete_message"',
    "Operation::LoadMessageRows": '"load_message_rows"',
    "Operation::GetActiveSwipeIndex": '"get_active_swipe_index"',
    "Operation::UpdateActiveSwipe": '"update_active_swipe"',
    "Operation::SoftDeleteMessage": '"soft_delete_message"',
    "Operation::RestoreSoftDeleted": '"restore_soft_deleted"',
    "Operation::PurgeSoftDeleted": '"purge_soft_deleted"',
    "Operation::InsertSwipe": '"insert_swipe"',
    "Operation::UpdateSwipeText": '"update_swipe_text"',
    "Operation::ShiftSwipeIndices": '"shift_swipe_indices"',
    "Operation::LoadSwipesForMessages": '"load_swipes_for_messages"',
    "Operation::CountSwipesForMessage": '"count_swipes_for_message"',
    "Operation::ListPresets": '"list_presets"',
    "Operation::GetPreset": '"get_preset"',
    "Operation::SavePreset": '"save_preset"',
    "Operation::DeletePreset": '"delete_preset"',
    "Operation::SaveLlmMessage": '"save_llm_message"',
    "Operation::ListLatestLlmMessages": '"list_latest_llm_messages"',
    "Operation::ListWorlds": '"list_worlds"',
    "Operation::GetWorld": '"get_world"',
    "Operation::SeedWorld": '"seed_world"',
    "Operation::UpdateWorld": '"update_world"',
    "Operation::GetWorldById": '"get_world_by_id"',
    "Operation::DeleteWorld": '"delete_world"',
    "Operation::ListPersonas": '"list_personas"',
    "Operation::GetPersona": '"get_persona"',
    "Operation::SeedPersona": '"seed_persona"',
    "Operation::ListCharacters": '"list_characters"',
    "Operation::GetCharacter": '"get_character"',
    "Operation::SeedCharacter": '"seed_character"',
    "Operation::GetSettings": '"get_settings"',
    "Operation::SaveSettings": '"save_settings"',
    "Operation::SeedSettings": '"seed_settings"',
}

NEED_GAME_ID = {"messages.rs", "snapshots.rs"}

def fix_file(path):
    fname = os.path.basename(path)
    if fname == "core.rs":
        return
    with open(path, "r", encoding="utf-8") as f:
        text = f.read()
    original = text

    for op, replacement in sorted(OP_MAP.items(), key=lambda x: len(x[0]), reverse=True):
        text = text.replace(op, replacement)

    lines = text.splitlines()
    cleaned = []
    for line in lines:
        if line.strip().startswith("use ") and "Operation" in line:
            line = line.replace(", Operation", "").replace("Operation, ", "")
            line = line.replace("{, ", "{").replace(", }", "}").replace(",,", ",")
            line = line.replace("{}", "")
        cleaned.append(line)
    text = "\n".join(cleaned)

    text = text.replace("|backend, _game_id|", "|backend|")
    text = text.replace("|backend, game_id|", "|backend|")

    if fname in NEED_GAME_ID:
        lines = text.splitlines()
        new_lines = []
        prev = None
        for line in lines:
            stripped = line.lstrip()
            if stripped.startswith("self.with_backend_mut("):
                if prev is None or not prev.lstrip().startswith("let game_id = self.game_id()"):
                    indent = line[:len(line) - len(stripped)]
                    new_lines.append(indent + "let game_id = self.game_id();")
            new_lines.append(line)
            prev = line
        text = "\n".join(new_lines)

    if text != original:
        with open(path, "w", encoding="utf-8") as f:
            f.write(text)
        print(f"Fixed {fname}")

for entry in os.listdir(SRC):
    if entry.endswith(".rs"):
        fix_file(os.path.join(SRC, entry))
