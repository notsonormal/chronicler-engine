use std::sync::{Arc, RwLock};

use axum::Router;
use tokio_util::sync::CancellationToken;

use crate::application::application_service::DefaultApplicationService;
use crate::application::game_service::GameService;
use crate::model::character::{NpcCard, PlayerCard};
use crate::model::map::{MapDef, Overworld, Region, Room};
use crate::model::settings::AppSettings;
use crate::model::state::{
    GameState, GenerationPhase, GenerationStatus, MessageType, StoredTriggerContext,
};
use crate::model::state_snapshot::GameStateSnapshot;
use crate::model::world::WorldCard;
use crate::server::{AppState, build_router};
use crate::storage::Storage;

pub struct TestAppBuilder {
    world: WorldCard,
    player: PlayerCard,
    map: MapDef,
    npcs: Vec<NpcCard>,
    room_npcs: Vec<String>,
    logs: Vec<(String, Option<String>, MessageType)>,
    last_trigger: Option<StoredTriggerContext>,
    generation_status: Option<GenerationStatus>,
    generation_phase: Option<GenerationPhase>,
    settings: AppSettings,
    storage: Option<Arc<Storage>>,
    game_service: Option<Arc<GameService>>,
    is_generating: bool,
}

impl TestAppBuilder {
    pub fn default_test() -> Self {
        let world = WorldCard {
            name: "Test World".to_string(),
            description: "A test world".to_string(),
            global_rules: vec![],
            starting_room_id: "room_1".to_string(),
            scenarios: vec![crate::model::scenario::StartingScenario {
                id: "test_intro".to_string(),
                name: "Test World Introduction".to_string(),
                description: "A simple test scenario for validation".to_string(),
                starting_room_id: "room_1".to_string(),
                text: "Welcome to the Test World, {{user}}! You find yourself in a cozy room with wooden beams and a warm fire. The smell of fresh bread fills the air. A friendly innkeeper behind the bar glances your way and smiles."
                    .to_string(),
                npcs: vec!["npc_1".to_string()],
            }],
            default_room_image: None,
        };

        let test_room = Room {
            id: "room_1".to_string(),
            name: "Test Room".to_string(),
            description: "A test room for component tests.".to_string(),
            image_path: Some("data/images/test_room.png".to_string()),
            exits: std::collections::HashMap::new(),
            items: vec![],
            navigation_description: None,
        };

        let map = MapDef {
            overworld: Overworld {
                id: "test_overworld".to_string(),
                name: "Test Overworld".to_string(),
                regions: vec![Region {
                    id: "region_1".to_string(),
                    name: "Test Region".to_string(),
                    rooms: vec![test_room],
                }],
            },
        };

        let player = PlayerCard {
            sheet: crate::model::character::CharacterSheet {
                name: "Test Player".to_string(),
                description: "A test player".to_string(),
                personality: "Brave".to_string(),
                scenario: "Test scenario.".to_string(),
                example_dialogue: "Hello!".to_string(),
                summary: None,
                profile_image: None,
                headshot_image: None,
            },
            inventory: vec![],
        };

        let npcs = vec![NpcCard {
            id: "npc_1".to_string(),
            sheet: crate::model::character::CharacterSheet {
                name: "Test NPC".to_string(),
                description: "A test NPC".to_string(),
                personality: "Friendly".to_string(),
                scenario: "Test scenario.".to_string(),
                example_dialogue: "Hello there!".to_string(),
                summary: None,
                profile_image: Some("data/images/npc.png".to_string()),
                headshot_image: Some("data/images/npc_headshot.png".to_string()),
            },
            inventory: vec![],
            triggers: vec![],
            relationships: vec![],
        }];

        Self::with_world_map(world, player, map)
            .npcs(npcs)
            .room_npc("npc_1")
    }

    pub fn new(world: WorldCard, player: PlayerCard) -> Self {
        let map = MapDef {
            overworld: Overworld {
                id: "test_overworld".to_string(),
                name: "Test Overworld".to_string(),
                regions: vec![Region {
                    id: "test_region".to_string(),
                    name: "Test Region".to_string(),
                    rooms: vec![Room {
                        id: world.starting_room_id.clone(),
                        name: format!("Room {}", world.starting_room_id),
                        description: "A plain test room.".to_string(),
                        exits: std::collections::HashMap::new(),
                        items: vec![],
                        image_path: None,
                        navigation_description: None,
                    }],
                }],
            },
        };

        Self::with_world_map(world, player, map)
    }

    pub fn default_app() -> Router {
        Self::default_test().build()
    }

    fn with_world_map(world: WorldCard, player: PlayerCard, map: MapDef) -> Self {
        Self {
            world,
            player,
            map,
            npcs: vec![],
            room_npcs: vec![],
            logs: vec![],
            last_trigger: None,
            generation_status: None,
            generation_phase: None,
            settings: AppSettings::default(),
            storage: None,
            game_service: None,
            is_generating: false,
        }
    }

    pub fn map(mut self, map: MapDef) -> Self {
        self.map = map;
        self
    }

    pub fn npc(mut self, npc: NpcCard) -> Self {
        self.npcs.push(npc);
        self
    }

    pub fn npcs(mut self, npcs: Vec<NpcCard>) -> Self {
        self.npcs = npcs;
        self
    }

    pub fn room_npc(mut self, npc_id: &str) -> Self {
        self.room_npcs.push(npc_id.to_string());
        self
    }

    pub fn last_trigger(mut self, trigger: StoredTriggerContext) -> Self {
        self.last_trigger = Some(trigger);
        self
    }

    pub fn log(mut self, text: &str, speaker: Option<&str>, log_type: MessageType) -> Self {
        self.logs
            .push((text.to_string(), speaker.map(|s| s.to_string()), log_type));
        self
    }

    pub fn generation_status(mut self, status: GenerationStatus, phase: GenerationPhase) -> Self {
        self.generation_status = Some(status);
        self.generation_phase = Some(phase);
        self
    }

    pub fn settings(mut self, settings: AppSettings) -> Self {
        self.settings = settings;
        self
    }

    pub fn storage(mut self, storage: Arc<Storage>) -> Self {
        self.storage = Some(storage);
        self
    }

    pub fn game_service(mut self, service: Arc<GameService>) -> Self {
        self.game_service = Some(service);
        self
    }

    pub fn is_generating(mut self, value: bool) -> Self {
        self.is_generating = value;
        self
    }

    pub fn build(self) -> Router {
        let world = Arc::new(self.world);
        let map = Arc::new(self.map);
        let player = Arc::new(self.player);
        let starting_room = map
            .overworld
            .regions
            .first()
            .and_then(|r| r.rooms.first())
            .map(|r| r.id.clone())
            .unwrap_or_else(|| "room_1".to_string());

        let mut state = GameState::new(
            Arc::clone(&world),
            Arc::clone(&map),
            Arc::clone(&player),
            self.npcs,
            starting_room,
        );

        for npc_id in &self.room_npcs {
            if let Some(npc) = state.npcs.get(npc_id).cloned() {
                state.scene.npcs_in_area.push(npc);
            }
        }

        if let Some(trigger) = self.last_trigger {
            state.narrative.last_trigger = Some(trigger);
        }

        if let Some(status) = self.generation_status {
            state.narrative.input_buffer.status = status;
        }
        if let Some(phase) = self.generation_phase {
            state.narrative.input_buffer.phase = phase;
        }

        for (text, sender, log_type) in self.logs {
            state.add_message(text, sender, log_type);
        }

        let storage = self
            .storage
            .unwrap_or_else(|| Arc::new(Storage::new_in_memory()));

        let snapshot = GameStateSnapshot::from_game_state(&state);
        let _ = storage.save_snapshot(&snapshot);
        for msg in state.narrative.history.iter().cloned().collect::<Vec<_>>() {
            let _ = storage.insert_message(&msg);
        }

        let settings_arc = Arc::new(RwLock::new(self.settings));
        let preset_storage = Arc::new(Storage::new_in_memory());
        let game_service: Arc<GameService> = self.game_service.unwrap_or_else(|| {
            Arc::new(GameService::with_storage(
                Some(Arc::clone(&storage)),
                Some(Arc::clone(&preset_storage)),
                Arc::clone(&settings_arc),
            ))
        });
        let app_state = AppState {
            storage,
            preset_storage,
            world,
            map,
            player,
            npcs: Arc::new(state.npcs.clone()),
            game_service: Arc::clone(&game_service),
            application_service: Arc::new(DefaultApplicationService::new(game_service)),
            settings: settings_arc,
            cancel_token: Arc::new(RwLock::new(CancellationToken::new())),
            is_generating: Arc::new(std::sync::atomic::AtomicBool::new(self.is_generating)),
        };
        build_router(app_state)
    }
}
