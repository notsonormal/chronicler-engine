import re
from pathlib import Path

files = [
    "src/storage/backend/snapshots.rs",
    "src/storage/backend/games.rs",
    "src/storage/backend/messages.rs",
    "src/storage/backend/swipes.rs",
    "src/storage/backend/presets.rs",
    "src/storage/backend/llm_messages.rs",
    "src/storage/backend/worlds.rs",
    "src/storage/backend/personas.rs",
    "src/storage/backend/characters.rs",
    "src/storage/backend/settings.rs",
]

for rel_path in files:
    path = Path(rel_path)
    content = path.read_text()
    lines = content.splitlines(keepends=True)
    new_lines = []
    i = 0
    while i < len(lines):
        line = lines[i]
        if "Backend::Test { .. } => unreachable!()," in line:
            test_indent = len(line) - len(line.lstrip())
            # Look at next non-empty line
            j = i + 1
            while j < len(lines) and lines[j].strip() == "":
                j += 1
            if j < len(lines) and "Backend::InMemory" in lines[j]:
                # Test is before InMemory: remove Test line,
                # copy InMemory block, then add _ => at end of match
                i += 1  # skip Test line
                inmem_indent = len(lines[j]) - len(lines[j].lstrip())
                new_lines.append(lines[j])  # InMemory line
                k = j + 1
                # Copy InMemory block until closing brace at same indent
                while k < len(lines):
                    stripped = lines[k].lstrip()
                    indent = len(lines[k]) - len(stripped)
                    if indent == inmem_indent and stripped.startswith("}"):
                        new_lines.append(lines[k])
                        k += 1
                        break
                    new_lines.append(lines[k])
                    k += 1
                # Now skip any blank lines until match closing
                while k < len(lines) and lines[k].strip() == "":
                    k += 1
                # Insert _ => before the match closing line
                if k < len(lines) and lines[k].strip().endswith("})"):
                    indent = len(lines[k]) - len(lines[k].lstrip())
                    new_lines.append(" " * (indent + 4) + "_ => unreachable!(),\n")
                    new_lines.append(lines[k])
                    i = k + 1
                    continue
            else:
                # Safe to replace with _ =>
                new_lines.append(" " * test_indent + "_ => unreachable!(),\n")
                i += 1
                continue
        new_lines.append(line)
        i += 1

    path.write_text("".join(new_lines))
    print(f"Updated {rel_path}")
