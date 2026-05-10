//! [DOC: docs/reference/testing.md]

use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::{Arc, Mutex, MutexGuard};

use chronicler_engine::model::character::{CharacterSheet, NpcCard, PlayerCard};
use chronicler_engine::model::map::MapDef;
use chronicler_engine::model::state::GameState;
use chronicler_engine::model::world::WorldCard;

#[path = "components/connections.rs"]
mod connections;
#[path = "components/css.rs"]
mod css;
#[path = "components/debug.rs"]
mod debug;
#[path = "components/fragment.rs"]
mod fragment;
#[path = "components/settings.rs"]
mod settings;
#[path = "components/template.rs"]
mod template;
#[path = "components/text_check.rs"]
mod text_check;
#[path = "components/world.rs"]
mod world;

pub fn create_test_state() -> GameState {
    use chronicler_engine::model::map::Room;

    let world = Arc::new(WorldCard {
        name: "Test World".into(),
        description: "A test world".into(),
        global_rules: vec![],
        default_room_image: None,
    });

    let test_room = Room {
        id: "room_1".into(),
        name: "Test Room".into(),
        description: "A test room for component tests.".into(),
        image_path: Some("data/images/test_room.png".into()),
        exits: std::collections::HashMap::new(),
        items: vec![],
        npcs: vec![],
        navigation_description: None,
    };

    let map = Arc::new(MapDef {
        overworld: chronicler_engine::model::map::Overworld {
            id: "test_overworld".into(),
            name: "Test Overworld".into(),
            regions: vec![chronicler_engine::model::map::Region {
                id: "region_1".into(),
                name: "Test Region".into(),
                rooms: vec![test_room],
            }],
        },
    });

    let player = Arc::new(PlayerCard {
        sheet: CharacterSheet {
            name: "Test Player".into(),
            description: "A test player".into(),
            personality: "Brave".into(),
            scenario: "Test scenario".into(),
            example_dialogue: "Hello!".into(),
            summary: None,
            profile_image: None,
            headshot_image: None,
        },
        inventory: vec![],
    });

    let npcs = vec![NpcCard {
        id: "npc_1".into(),
        sheet: CharacterSheet {
            name: "Test NPC".into(),
            description: "A test NPC".into(),
            personality: "Friendly".into(),
            scenario: "Test scenario".into(),
            example_dialogue: "Hello there!".into(),
            summary: None,
            profile_image: Some("data/images/npc.png".into()),
            headshot_image: Some("data/images/npc_headshot.png".into()),
        },
        inventory: vec![],
        triggers: vec![],
    }];

    GameState::new(world, map, player, npcs, "room_1".to_string())
}

static SETTINGS_TEST_LOCK: Mutex<()> = Mutex::new(());
static SETTINGS_TEST_COUNTER: AtomicU64 = AtomicU64::new(0);

pub struct TempSettingsGuard {
    _lock: MutexGuard<'static, ()>,
    temp_path: std::path::PathBuf,
}

impl Default for TempSettingsGuard {
    fn default() -> Self {
        Self::new()
    }
}

impl TempSettingsGuard {
    pub fn new() -> Self {
        let lock = SETTINGS_TEST_LOCK.lock().unwrap_or_else(|e| e.into_inner());
        let counter = SETTINGS_TEST_COUNTER.fetch_add(1, Ordering::SeqCst);
        let temp_path = std::env::temp_dir().join(format!(
            "chronicler_test_settings_{}_{}.json",
            std::process::id(),
            counter
        ));
        unsafe { std::env::set_var("CHRONICLER_SETTINGS_PATH", &temp_path) };
        Self {
            _lock: lock,
            temp_path,
        }
    }
}

impl Drop for TempSettingsGuard {
    fn drop(&mut self) {
        unsafe { std::env::remove_var("CHRONICLER_SETTINGS_PATH") };
        let _ = std::fs::remove_file(&self.temp_path);
    }
}
