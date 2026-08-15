//! Test-only `Storage` extension trait for seeding deterministic test worlds.

#![allow(dead_code)]

use chronicler_engine::adapters::driven::storage::Storage;

pub trait TestWorldFixture {
    fn seed_test_world_fixture(&self);
    fn seed_test_world_with_scenario_fixture(&self);
}

impl TestWorldFixture for Storage {
    fn seed_test_world_fixture(&self) {
        use chronicler_engine::test_support::{TestMap, TestPersona, TestWorld};
        let world = TestWorld::minimal();
        let map = TestMap::single_room("start");
        self.seed_world(&world, &map).expect("seed world");
        let player = TestPersona::standard();
        self.seed_persona(&player.key, &player)
            .expect("seed persona");
    }

    fn seed_test_world_with_scenario_fixture(&self) {
        use chronicler_engine::test_support::{create_test_map, create_test_world_with_scenario};
        let world = create_test_world_with_scenario();
        let map = create_test_map();
        self.seed_world(&world, &map)
            .expect("seed world with scenario");
        let player = chronicler_engine::test_support::TestPersona::standard();
        self.seed_persona(&player.key, &player)
            .expect("seed persona");
    }
}
