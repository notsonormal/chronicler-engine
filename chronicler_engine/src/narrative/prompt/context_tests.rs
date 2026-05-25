use crate::narrative::prompt::budget;
use crate::narrative::prompt::budget::estimate_tokens;
use crate::narrative::prompt::context::{
    fit_messages_to_context, make_prompt_context, trim_history_to_budget,
};
use crate::narrative::prompt::types::PromptContext;

#[test]
fn test_context_fitting_no_trim_needed() {
    let system = "System prompt.";
    let user = "<GameState>Room</GameState>\n\n<ConversationHistory>\nNarrator: Hello\n</ConversationHistory>";
    let result = fit_messages_to_context(system, user, 4096, Some(1024));
    assert!(result.is_ok());
    let (s, u, max) = result.unwrap();
    assert_eq!(s, system);
    assert_eq!(u, user);
    assert!(max <= 1024);
}

#[test]
fn test_context_fitting_trims_oldest_history() {
    let system = "System prompt.";
    let mut history_lines = String::new();
    for i in 0..100 {
        history_lines.push_str(&format!(
            "Narrator: This is a long history entry number {i} with enough text to consume tokens.\n"
        ));
    }
    let user = format!(
        "<GameState>Room</GameState>\n\n<ConversationHistory>\n{history_lines}</ConversationHistory>"
    );

    let result = fit_messages_to_context(system, &user, 1024, Some(256));
    assert!(result.is_ok());
    let (_s, fitted_user, _max) = result.unwrap();

    assert!(fitted_user.contains("<ConversationHistory>"));
    assert!(
        !fitted_user.contains("number 0"),
        "Oldest history entry should be trimmed first"
    );
    assert!(
        fitted_user.contains("number 99"),
        "Newest history entries should be preserved"
    );
}

#[test]
fn test_context_fitting_caps_max_tokens() {
    let system = "System prompt with some length.";
    let user = "<GameState>Room</GameState>";
    let result = fit_messages_to_context(system, user, 4096, Some(4096));
    assert!(result.is_ok());
    let (_s, _u, max) = result.unwrap();
    assert!(max < 4096);
    let total = estimate_tokens(system)
        + estimate_tokens(user)
        + max as usize
        + budget::SAFETY_MARGIN_TOKENS as usize;
    assert!(
        total <= 4096,
        "Total tokens {total} exceed context window 4096"
    );
}

#[test]
fn test_context_fitting_system_overflow() {
    let system = "x".repeat(5000);
    let user = "User prompt.";
    let result = fit_messages_to_context(&system, user, 512, Some(256));
    assert!(result.is_err());
    let err = result.unwrap_err().to_string();
    assert!(err.contains("Context overflow"));
}

#[test]
fn test_trim_history_to_budget_no_history_tag() {
    let user = "<GameState>Room</GameState>\n\n<PlayerInput>look</PlayerInput>";
    let result = trim_history_to_budget(user, 100);
    assert_eq!(result, user);
}

#[test]
fn test_context_fitting_post_trim_overflow() {
    let system = "System.";
    let user = format!(
        "<GameState>{}</GameState>\n\n<ConversationHistory>\nNarrator: Hi\n</ConversationHistory>",
        "x".repeat(2000)
    );
    let result = fit_messages_to_context(system, &user, 512, Some(256));
    assert!(result.is_err());
    let err = result.unwrap_err().to_string();
    assert!(err.contains("Context overflow"));
}

#[test]
fn test_make_prompt_context() {
    let world = crate::test_support::TestWorld::minimal();
    let room = crate::test_support::TestMap::room("test_room");
    let player = crate::test_support::TestPlayer::standard();
    let npcs = vec![crate::test_support::TestNpc::named("npc1", "Npc")];
    let history = vec![];

    let context: PromptContext = make_prompt_context(
        &world,
        &room,
        &npcs,
        &npcs,
        &player,
        "hello",
        &history,
        String::new(),
    );

    assert_eq!(context.world.name, "Test World");
    assert_eq!(context.room.id, "test_room");
    assert_eq!(context.player.sheet.name, "Hero");
    assert_eq!(context.all_npcs.len(), 1);
    assert_eq!(context.user_message, "hello");
    assert!(context.history.is_empty());
}

#[test]
fn test_trim_history_to_budget_empty_content() {
    let user = "<GameState>Room</GameState>\n\n<ConversationHistory>\n\n</ConversationHistory>";
    let result = trim_history_to_budget(user, 10);
    assert!(result.contains("<ConversationHistory>"));
    assert!(
        result.contains("History truncated to fit context window") || result == user,
        "Empty content should be handled gracefully"
    );
}

#[test]
fn test_trim_history_to_budget_no_close_tag() {
    let user = "<GameState>Room</GameState>\n\n<ConversationHistory>\nNarrator: Hello";
    let result = trim_history_to_budget(user, 10);
    assert_eq!(result, user);
}

#[test]
fn test_context_fitting_user_already_fits() {
    let system = "Short.";
    let user = "<GameState>Room</GameState>";
    let result = fit_messages_to_context(system, user, 4096, Some(256));
    assert!(result.is_ok());
    let (s, u, max) = result.unwrap();
    assert_eq!(s, system);
    assert_eq!(u, user);
    assert!(max <= 256);
}
