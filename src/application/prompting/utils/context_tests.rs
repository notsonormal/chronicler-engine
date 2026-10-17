use crate::application::prompting::assembler::PromptContext;
use crate::application::prompting::types::NpcContext;
use crate::application::prompting::utils::context::trim_history_to_budget;
use crate::domain::model::settings::{NarrativePerspective, NarrativeTense};

#[test]
fn test_trim_history_to_budget_no_history_tag() {
    let user = "<GameState>Room</GameState>\n\n<PlayerInput>look</PlayerInput>";
    let result = trim_history_to_budget(user, 100);
    assert_eq!(result, user);
}

#[test]
fn test_make_prompt_context() {
    let world = crate::test_support::TestWorld::minimal();
    let room = crate::test_support::TestMap::room("test_room");
    let player = crate::test_support::TestPersona::standard();
    let npcs = vec![crate::test_support::TestNpc::named("npc1", "Npc")];
    let history = vec![];

    let context: PromptContext = PromptContext::new(
        &world,
        NarrativePerspective::Third,
        NarrativeTense::Past,
        &room,
        NpcContext {
            all_npcs: &npcs,
            npcs_in_area: &npcs,
        },
        &player,
        "hello",
        &history,
    );

    assert_eq!(context.world.name, "Test World");
    assert_eq!(context.room.id, "test_room");
    assert_eq!(context.persona.sheet.name, "Hero");
    assert_eq!(context.npcs.all_npcs.len(), 1);
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
