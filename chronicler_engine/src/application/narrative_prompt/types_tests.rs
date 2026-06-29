use crate::application::narrative_prompt::types::PromptLayer;

#[test]
fn test_prompt_layer_variants() {
    assert_eq!(PromptLayer::System as u8, 0);
    assert_eq!(PromptLayer::GameState as u8, 1);
    assert_eq!(PromptLayer::NpcCards as u8, 2);
    assert_eq!(PromptLayer::Player as u8, 3);
    assert_eq!(PromptLayer::WorldInfo as u8, 4);
    assert_eq!(PromptLayer::History as u8, 5);
    assert_eq!(PromptLayer::User as u8, 6);
}
