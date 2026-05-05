use crate::narrative::quantifier::types::QuantifierPromptContext;

/// [DOC: docs/reference/quantifier_prompt.md]
pub struct QuantifierPromptBuilder<'a> {
    context: QuantifierPromptContext<'a>,
}

impl<'a> QuantifierPromptBuilder<'a> {
    pub fn new(context: QuantifierPromptContext<'a>) -> Self {
        Self { context }
    }

    pub fn build(&self) -> (String, String) {
        (self.build_system_prompt(), self.build_user_prompt())
    }

    fn build_system_prompt(&self) -> String {
        let mut prompt = String::from(
            r#"You are a scene quantifier for a text adventure game.
Your task is to determine which NPCs are present in the current room
and whether the player actually moved to a new location.

Respond ONLY with a JSON object in this exact format:
{"npcs_in_room": ["id1", "id2"], "movement": {"type": "entering|in|leaving", "destination": "room_id"}}

How to determine movement:
1. Read <CurrentRoom> — this is where the player is right now.
2. Read <LatestNarration> — this is what just happened.
3. Ask: does the narration describe the player being in a different place than <CurrentRoom>?
   - If YES → movement occurred. Set type to "entering" and destination to the new room.
   - If NO → no movement. Set type to null.
   - If unclear → assume no movement. Set type to null.

Rules:
- Only include NPCs that would logically be in the room based on context.
- NPCs from the previous room may have followed the player.
- Use the exact NPC IDs provided in the AvailableNpcIds list.
- If the player is blocked, stopped, prevented, or fails to move in <LatestNarration>, they have NOT moved.
- An NPC interposing, blocking a path, or saying "you can't go" means the player remains.
- If no NPCs are present, return an empty array: {"npcs_in_room": []}
- If no movement detected, set type to null: {"movement": {"type": null}}

Examples:
- Narration: "You walk through the door into the kitchen." (CurrentRoom was hallway) → {"movement": {"type": "entering", "destination": "kitchen"}}
- Narration: "The guard blocks your path. 'Halt!' he shouts." (CurrentRoom was courtyard) → {"movement": {"type": null}}
- Narration: "She swiftly interposes herself between you and the gate." (CurrentRoom was garden) → {"movement": {"type": null}}
- Narration: "The foyer felt claustrophobic. Carla stood in the doorway." (CurrentRoom was Front Gates) → {"movement": {"type": "entering", "destination": "entrance_hall"}}
- Narration: "You examine the ancient vase carefully." (CurrentRoom was library) → {"movement": {"type": null}}

<AvailableNpcIds>
"#,
        );

        for npc in self.context.all_known_npcs {
            prompt.push_str(&format!(
                "  <Npc id=\"{}\" name=\"{}\"/>\n",
                npc.id, npc.sheet.name
            ));
        }

        prompt.push_str("</AvailableNpcIds>\n\n<AvailableRooms>\n");

        for room in self.context.all_rooms {
            prompt.push_str(&format!(
                "  <Room id=\"{}\" name=\"{}\"/>\n",
                room.id, room.name
            ));
        }

        prompt.push_str("</AvailableRooms>\n");

        prompt
    }

    fn build_user_prompt(&self) -> String {
        let mut prompt = String::new();

        prompt.push_str("<CurrentRoom>\n");
        prompt.push_str(&format!("  <Name>{}</Name>\n", self.context.room.name));
        prompt.push_str(&format!(
            "  <Description>{}</Description>\n",
            self.context.room.description
        ));

        // Include navigation hint to improve movement detection
        if let Some(nav_desc) = &self.context.room.navigation_description {
            prompt.push_str(&format!("  <Navigation>{nav_desc}</Navigation>\n"));
        }

        prompt.push_str("</CurrentRoom>\n\n");

        if !self.context.previous_room_npcs.is_empty() {
            prompt.push_str("<PreviousRoomNpcs>\n");
            for npc in self.context.previous_room_npcs {
                prompt.push_str(&format!(
                    "  <Npc id=\"{}\" name=\"{}\">{}</Npc>\n",
                    npc.id, npc.sheet.name, npc.sheet.description
                ));
            }
            prompt.push_str("</PreviousRoomNpcs>\n\n");
        }

        if !self.context.room.npcs.is_empty() {
            prompt.push_str("<RoomConfiguredNpcs>\n");
            prompt.push_str("  ");
            prompt.push_str(&self.context.room.npcs.join(", "));
            prompt.push_str("\n</RoomConfiguredNpcs>\n\n");
        }

        if !self.context.recent_history.is_empty() {
            prompt.push_str("<RecentHistory>\n");
            for entry in self.context.recent_history {
                let sender = entry.sender.as_deref().unwrap_or("Narrator");
                prompt.push_str(&format!(
                    "  <Entry sender=\"{}\">{}</Entry>\n",
                    sender, entry.text
                ));
            }
            prompt.push_str("</RecentHistory>\n\n");
        }

        prompt.push_str(&format!(
            "<LatestNarration>\n  {}: {}\n</LatestNarration>\n\n",
            self.context.player_name, self.context.player_action
        ));

        prompt.push_str(
            r#"Based on the context above, determine:
- Which NPCs are present in the current room
- Whether the player actually entered, left, or remained in place

IMPORTANT: Base your decision ONLY on what happens in <LatestNarration>, not on what the player attempted in <RecentHistory>.

Respond ONLY with the JSON format specified in the system instructions.
"#,
        );

        prompt
    }
}
