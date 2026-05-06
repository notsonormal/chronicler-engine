use crate::model::map::{Direction, MapDef};

#[test]
fn test_map_serde() {
    let json = r#"{
        "overworld": {
            "id": "ow_1",
            "name": "World",
            "regions": [
                {
                    "id": "reg_1",
                    "name": "Start",
                    "rooms": [
                        {
                            "id": "room_1",
                            "name": "Tavern",
                            "description": "A tavern.",
                            "exits": {
                                "north": "room_2"
                            },
                            "image_path": "data/images/tavern.png"
                        }
                    ]
                }
            ]
        }
    }"#;

    let map: MapDef = serde_json::from_str(json).unwrap();
    assert_eq!(map.overworld.id, "ow_1");
    assert_eq!(
        map.overworld.regions[0].rooms[0]
            .exits
            .get(&Direction::North)
            .unwrap(),
        "room_2"
    );
    assert_eq!(
        map.overworld.regions[0].rooms[0].image_path,
        Some("data/images/tavern.png".to_string())
    );
}
