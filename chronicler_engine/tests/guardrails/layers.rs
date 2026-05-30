use crate::Violation;

// ── Server Layer Boundary ──

pub fn check_server_layer_boundaries(file_path: &str, content: &str) -> Vec<Violation> {
    let mut violations = Vec::new();

    // Exceptions: mod.rs and debug.rs are allowed to reference GameState
    // (mod.rs for re-exports/legacy, debug.rs for debug DTOs)
    if file_path.ends_with("mod.rs") || file_path.ends_with("debug.rs") {
        return violations;
    }

    // Only check src/server/
    if !file_path.starts_with("server/") {
        return violations;
    }

    for (line_no, line) in content.lines().enumerate() {
        let line_num = line_no + 1;
        let trimmed = line.trim();

        // Skip comments
        if trimmed.starts_with("//") || trimmed.starts_with("*") {
            continue;
        }

        // Ban GameState references
        if trimmed.contains("GameState") && !trimmed.contains("GameStateSnapshot") {
            violations.push(Violation::error(
                file_path,
                line_num,
                "Server layer file references `GameState`. \
                 Server must use ApplicationService methods, not direct GameState access.",
            ));
        }

    }

    violations
}

// ── Test Layer Boundary ──

pub fn check_test_layer_boundaries(file_path: &str, content: &str) -> Vec<Violation> {
    let mut violations = Vec::new();

    // Only check tests/components/
    if !file_path.starts_with("components/") {
        return violations;
    }

    for (line_no, line) in content.lines().enumerate() {
        let line_num = line_no + 1;
        let trimmed = line.trim();

        // Skip comments
        if trimmed.starts_with("//") || trimmed.starts_with("*") {
            continue;
        }

        // Ban GameState construction (GameState::new)
        if trimmed.contains("GameState::new(") {
            violations.push(Violation::error(
                file_path,
                line_num,
                "Component test constructs `GameState` directly. \
                 Use `TestAppBuilder` instead.",
            ));
        }

        // Ban GameState imports (but allow GameStateSnapshot)
        if (trimmed.contains("use") || trimmed.contains("model::state::GameState"))
            && trimmed.contains("GameState")
            && !trimmed.contains("GameStateSnapshot")
        {
            violations.push(Violation::error(
                file_path,
                line_num,
                "Component test imports `GameState`. \
                 Use `TestAppBuilder` instead.",
            ));
        }
    }

    violations
}
