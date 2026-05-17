pub struct DbGameStateSnapshot {
    pub id: i64,
    pub game_id: i64,
    pub movement_json: String,
    pub narrative_json: String,
    pub scene_json: String,
    pub npc_encounter_log_json: String,
    pub committed: i32,
    pub created_at: String,
}
