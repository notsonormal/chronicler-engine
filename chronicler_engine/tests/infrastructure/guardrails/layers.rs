use crate::Violation;

/// Guardrail: `messages.rs` must not reference the `message_swipes` table.
pub fn check_messages_swipes_separation(file_path: &str, content: &str) -> Vec<Violation> {
    let mut violations = Vec::new();

    if !file_path.ends_with("storage/backend/messages.rs") {
        return violations;
    }

    for (line_no, line) in content.lines().enumerate() {
        let line_num = line_no + 1;
        let trimmed = line.trim();

        if trimmed.starts_with("//") || trimmed.starts_with("*") {
            continue;
        }
        if trimmed.contains("FROM message_swipes")
            || trimmed.contains("INTO message_swipes")
            || trimmed.contains("UPDATE message_swipes")
            || trimmed.contains("JOIN message_swipes")
            || trimmed.contains("DELETE FROM message_swipes")
        {
            violations.push(Violation::error(
                file_path,
                line_num,
                "Storage module `messages.rs` references `message_swipes` table",
            ));
        }
    }

    violations
}

pub fn check_server_layer_boundaries(file_path: &str, content: &str) -> Vec<Violation> {
    let mut violations = Vec::new();

    if file_path.ends_with("mod.rs") || file_path.ends_with("debug.rs") {
        return violations;
    }

    if !file_path.starts_with("server/") {
        return violations;
    }

    for (line_no, line) in content.lines().enumerate() {
        let line_num = line_no + 1;
        let trimmed = line.trim();

        if trimmed.starts_with("//") || trimmed.starts_with("*") {
            continue;
        }

        if trimmed.contains("GameState") && !trimmed.contains("GameStateSnapshot") {
            violations.push(Violation::error(
                file_path,
                line_num,
                "Server layer file references `GameState`",
            ));
        }
    }

    violations
}

pub fn check_test_layer_boundaries(file_path: &str, content: &str) -> Vec<Violation> {
    let mut violations = Vec::new();

    if !file_path.starts_with("tests/components/") {
        return violations;
    }

    for (line_no, line) in content.lines().enumerate() {
        let line_num = line_no + 1;
        let trimmed = line.trim();

        if trimmed.starts_with("//") || trimmed.starts_with("*") {
            continue;
        }

        if trimmed.contains("GameState::new(") {
            violations.push(Violation::error(
                file_path,
                line_num,
                "Component test constructs `GameState` directly",
            ));
        }

        if (trimmed.contains("use") || trimmed.contains("model::state::GameState"))
            && trimmed.contains("GameState")
            && !trimmed.contains("GameStateSnapshot")
        {
            violations.push(Violation::error(
                file_path,
                line_num,
                "Component test imports `GameState`",
            ));
        }
    }

    violations
}
