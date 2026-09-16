//! Tests for options prompt construction.

use crate::application::agents::options::types::OptionsPromptContext;
use crate::application::agents::options::prompt::OptionsPromptBuilder;
use crate::domain::model::state::message_types::{MessageEntry, MessageType};
use crate::test_support::fixtures::{TestMap, TestNpc, TestPersona};

fn make_context<'a>(
    map: &'a crate::domain::model::map::MapDef,
    npcs: &'a [crate::domain::model::character::NpcCard],
    override_text: Option<String>,
) -> OptionsPromptContext<'a> {
    let room = map
        .get_room_by_id("start")
        .expect("single_room map has 'start'");
    OptionsPromptContext {
        room,
        npcs_in_area: npcs,
        recent_history: &[],
        player_name: "Julian",
        options_prompt_override: override_text,
        option_count: 3,
    }
}

fn make_room_npcs() -> Vec<crate::domain::model::character::NpcCard> {
    vec![TestNpc::named("npc_1", "Carla")]
}

#[test]
fn fallback_system_prompt_substitutes_macros() {
    let map = TestMap::single_room("start");
    let npcs = make_room_npcs();
    let (system, _user) = OptionsPromptBuilder::new(make_context(&map, &npcs, None)).build();

    assert!(!system.contains("{{user}}"), "{{user}} must substitute");
    assert!(
        !system.contains("{{option_count}}"),
        "{{option_count}} must substitute"
    );
    assert!(system.contains("Julian"));
    assert!(system.contains("3"));
    assert!(system.contains("<suggestion>"));
}

#[test]
fn override_system_prompt_is_rendered_with_macros() {
    let map = TestMap::single_room("start");
    let npcs = make_room_npcs();
    let (system, _user) = OptionsPromptBuilder::new(make_context(
        &map,
        &npcs,
        Some("Offer {{option_count}} bold moves for {{user}}.".to_string()),
    ))
    .build();

    assert_eq!(system, "Offer 3 bold moves for Julian.");
}

#[test]
fn whitespace_only_override_falls_back() {
    let map = TestMap::single_room("start");
    let npcs = make_room_npcs();
    let (system, _user) =
        OptionsPromptBuilder::new(make_context(&map, &npcs, Some("   ".to_string()))).build();

    assert!(system.contains("<suggestion>"), "fallback prompt expected");
}

#[test]
fn user_prompt_carries_room_npc_and_history_sections() {
    let map = TestMap::single_room("start");
    let room = map.get_room_by_id("start").expect("room exists");
    let history = vec![
        MessageEntry {
            text: "The hall is silent.".to_string(),
            message_type: MessageType::Narration,
            ..MessageEntry::default()
        },
        MessageEntry {
            text: "I step inside.".to_string(),
            message_type: MessageType::Input,
            ..MessageEntry::default()
        },
        MessageEntry {
            text: "[System] note".to_string(),
            message_type: MessageType::System,
            ..MessageEntry::default()
        },
    ];
    let npcs = vec![TestNpc::named("npc_1", "Carla")];
    let persona = TestPersona::named("Julian");

    let context = OptionsPromptContext {
        room,
        npcs_in_area: &npcs,
        recent_history: &history,
        player_name: &persona.sheet.name,
        options_prompt_override: None,
        option_count: 3,
    };
    let (_system, user) = OptionsPromptBuilder::new(context).build();

    assert!(user.contains("<CurrentRoom>"));
    assert!(user.contains("<Name>"));
    assert!(user.contains("<NpcsInArea>"));
    assert!(user.contains("Carla"));
    assert!(user.contains("<RecentHistory>"));
    assert!(user.contains("sender=\"Narrator\""));
    assert!(user.contains("sender=\"Julian\""));
    assert!(user.contains("sender=\"System\""));
    assert!(user.contains("exactly 3 options"));
}

#[test]
fn user_prompt_omits_npc_section_when_area_is_empty() {
    let map = TestMap::single_room("start");
    let room = map.get_room_by_id("start").expect("room exists");

    let context = OptionsPromptContext {
        room,
        npcs_in_area: &[],
        recent_history: &[],
        player_name: "Julian",
        options_prompt_override: None,
        option_count: 3,
    };
    let (_system, user) = OptionsPromptBuilder::new(context).build();

    assert!(!user.contains("<NpcsInArea>"));
}
