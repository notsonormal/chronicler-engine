//! Test data bundle builder for integration tests.
#![allow(clippy::expect_used)]

use std::sync::Arc;

use crate::adapters::driven::storage::Storage;
use crate::domain::model::character::{NpcCard, PersonaCard};
use crate::domain::model::map::{MapDef, Overworld, Region, Room};
use crate::domain::model::world::WorldCard;

/// Builder for a [`TestData`] bundle. Defaults to canonical Test World + Test
/// Map + Test Player + `npc_1`. Override via `.world()`, `.map()`,
/// `.persona()`, `.npc()`, `.npcs()`, `.room_npc()`.
pub struct TestDataBuilder {
    world: WorldCard,
    map: MapDef,
    persona: PersonaCard,
    npcs: Vec<NpcCard>,
    room_npcs: Vec<String>,
}

impl TestDataBuilder {
    pub fn default_test() -> Self {
        let world = WorldCard {
            key: "test".to_string(),
            name: "Test World".to_string(),
            description: "A test world".to_string(),
            scenarios: vec![crate::domain::model::scenario::StartingScenario {
                id: "test_intro".to_string(),
                name: "Test World Introduction".to_string(),
                description: "A simple test scenario for validation".to_string(),
                starting_room_id: "room_1".to_string(),
                text: "Welcome to the Test World, {{user}}! You find yourself in a cozy room with wooden beams and a warm fire. The smell of fresh bread fills the air. A friendly innkeeper behind the bar glances your way and smiles."
                    .to_string(),
                npcs: vec!["npc_1".to_string()],
            }],
            ..Default::default()
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

        let persona = PersonaCard {
            key: "test_player".to_string(),
            sheet: crate::domain::model::character::CharacterSheet {
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
            sheet: crate::domain::model::character::CharacterSheet {
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

        Self {
            world,
            map,
            persona,
            npcs,
            room_npcs: vec!["npc_1".to_string()],
        }
    }

    pub fn world(mut self, world: WorldCard) -> Self {
        self.world = world;
        self
    }

    pub fn map(mut self, map: MapDef) -> Self {
        self.map = map;
        self
    }

    pub fn persona(mut self, persona: PersonaCard) -> Self {
        self.persona = persona;
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

    pub fn build(self) -> TestData {
        TestData {
            world: Arc::new(self.world),
            map: Arc::new(self.map),
            persona: Arc::new(self.persona),
            npcs: self.npcs,
            room_npcs: self.room_npcs,
        }
    }
}

/// Immutable world-data bundle for an integration test. Pairs with
/// [`TestAppBuilder`] for app-wiring. Use [`TestData::seed_into`] to persist
/// into a storage instance.
#[derive(Clone)]
pub struct TestData {
    pub world: Arc<WorldCard>,
    pub map: Arc<MapDef>,
    pub persona: Arc<PersonaCard>,
    pub npcs: Vec<NpcCard>,
    pub room_npcs: Vec<String>,
}

impl TestData {
    /// Persist world, map, persona, npcs, and a default game into `storage`.
    /// Returns the seeded world id. Does not save snapshots or persist
    /// messages — those remain the responsibility of `TestAppBuilder`.
    pub fn seed_into(&self, storage: &Storage) -> i64 {
        let world_id = storage
            .seed_world(&self.world, &self.map)
            .expect("test setup: seed world");
        storage
            .seed_persona(&self.persona.key, &self.persona)
            .expect("test setup: seed persona");
        for npc in &self.npcs {
            storage
                .seed_character(world_id, npc)
                .expect("test setup: seed character");
        }
        let game_id = storage
            .create_game(
                &self.world.name,
                &self.world.key,
                &self.persona.key,
                &self.persona.sheet.name,
                "Test Game",
            )
            .expect("test setup: create game");
        storage.set_game_id(game_id);
        world_id
    }

    pub fn find_npc(&self, id: &str) -> Option<&NpcCard> {
        self.npcs.iter().find(|n| n.id == id)
    }

    pub fn world_key(&self) -> String {
        self.world.key.clone()
    }

    pub fn player_name(&self) -> String {
        self.persona.sheet.name.clone()
    }
}
