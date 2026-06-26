//! [DOC: docs/system/storage.md]
//! SQLite database connection pool and migrations

use std::sync::{Arc, Mutex};

use rusqlite::Connection;

#[derive(Clone)]
pub struct DbPool {
    conn: Arc<Mutex<Connection>>,
}

impl DbPool {
    pub fn new(path: &str) -> Result<Self, crate::error::EngineError> {
        let conn = Connection::open(path)
            .map_err(|e| crate::error::EngineError::Config(format!("Failed to open DB: {e}")))?;
        run_migrations(&conn)?;
        conn.pragma_update(None, "foreign_keys", true)
            .map_err(|e| {
                crate::error::EngineError::Config(format!("Failed to enable foreign keys: {e}"))
            })?;
        Ok(Self {
            conn: Arc::new(Mutex::new(conn)),
        })
    }

    pub fn conn(&self) -> std::sync::MutexGuard<Connection> {
        match self.conn.lock() {
            Ok(guard) => guard,
            Err(poisoned) => poisoned.into_inner(),
        }
    }

    /// Insert a new games row and return the new rowid.
    /// Single source of truth for the `games` INSERT column list.
    pub fn insert_game(
        &self,
        world_name: &str,
        world_key: &str,
        persona_key: &str,
        persona_name: &str,
        name: &str,
    ) -> Result<u64, crate::error::EngineError> {
        let conn = self.conn();
        let now = chrono::Utc::now().to_rfc3339();
        conn.execute(
            "INSERT INTO games (world_name, world_key, persona_key, persona_name, name, created_at, updated_at) \
             VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?6)",
            rusqlite::params![world_name, world_key, persona_key, persona_name, name, &now],
        )
        .map_err(|e| crate::error::EngineError::Config(format!("Failed to create game: {e}")))?;
        Ok(conn.last_insert_rowid() as u64)
    }
}

fn run_migrations(conn: &Connection) -> Result<(), crate::error::EngineError> {
    fn column_exists(conn: &Connection, table: &str, col: &str) -> bool {
        conn.query_row(
            "SELECT COUNT(*) FROM pragma_table_info(?) WHERE name = ?",
            [table, col],
            |r| r.get::<_, i64>(0),
        )
        .unwrap_or(0)
            > 0
    }

    let version: i32 = conn
        .query_row("PRAGMA user_version", [], |row| row.get(0))
        .unwrap_or(0);

    if version < 9 {
        let exec = |sql: &str| {
            conn.execute(sql, [])
                .map_err(|e| crate::error::EngineError::Config(format!("Migration failed: {e}")))
        };

        exec(
            "CREATE TABLE IF NOT EXISTS games (
                id INTEGER PRIMARY KEY AUTOINCREMENT,
                world_name TEXT NOT NULL DEFAULT 'default',
                world_key TEXT NOT NULL DEFAULT 'default',
                name TEXT NOT NULL DEFAULT 'Unnamed',
                created_at TEXT NOT NULL,
                updated_at TEXT NOT NULL
            )",
        )?;

        exec(
            "CREATE TABLE IF NOT EXISTS game_state_snapshots (
                id INTEGER PRIMARY KEY AUTOINCREMENT,
                game_id INTEGER NOT NULL DEFAULT 1 REFERENCES games(id) ON DELETE CASCADE,
                movement TEXT NOT NULL,
                narrative TEXT NOT NULL,
                scene TEXT NOT NULL,
                npc_encounter_log TEXT NOT NULL,
                created_at TEXT NOT NULL
            )",
        )?;

        exec(
            "CREATE INDEX IF NOT EXISTS idx_snapshots_game_latest ON game_state_snapshots(game_id, created_at DESC)",
        )?;

        exec(
            "CREATE TABLE IF NOT EXISTS messages (
                id INTEGER PRIMARY KEY AUTOINCREMENT,
                game_id INTEGER NOT NULL DEFAULT 1 REFERENCES games(id) ON DELETE CASCADE,
                sender TEXT,
                message_type TEXT NOT NULL,
                timestamp TEXT NOT NULL,
                active_swipe_index INTEGER NOT NULL DEFAULT 0,
                is_deleted INTEGER NOT NULL DEFAULT 0
            )",
        )?;

        exec("CREATE INDEX IF NOT EXISTS idx_messages_game_id ON messages(game_id, id)")?;

        exec(
            "CREATE TABLE IF NOT EXISTS message_swipes (
                id INTEGER PRIMARY KEY AUTOINCREMENT,
                message_id INTEGER NOT NULL REFERENCES messages(id) ON DELETE CASCADE,
                swipe_index INTEGER NOT NULL,
                text TEXT NOT NULL,
                snapshot_id INTEGER,
                location_header TEXT,
                event_header TEXT,
                UNIQUE(message_id, swipe_index)
            )",
        )?;

        exec(
            "CREATE TABLE IF NOT EXISTS llm_messages (
                id INTEGER PRIMARY KEY AUTOINCREMENT,
                agent_name TEXT NOT NULL,
                backend_name TEXT NOT NULL,
                model_name TEXT NOT NULL,
                system_prompt TEXT NOT NULL,
                user_prompt TEXT NOT NULL,
                raw_request_json TEXT NOT NULL,
                raw_response_json TEXT NOT NULL,
                parsed_response TEXT NOT NULL,
                error_message TEXT,
                created_at TEXT NOT NULL
            )",
        )?;

        exec(
            "CREATE INDEX IF NOT EXISTS idx_llm_messages_created_at ON llm_messages(created_at DESC)",
        )?;

        exec(
            "CREATE TABLE IF NOT EXISTS prompt_presets (
                id TEXT PRIMARY KEY,
                name TEXT NOT NULL,
                preset_type TEXT NOT NULL,
                role TEXT,
                instructions TEXT,
                writing_style TEXT,
                output_format TEXT,
                is_default INTEGER NOT NULL DEFAULT 0,
                created_at TEXT NOT NULL,
                updated_at TEXT NOT NULL
            )",
        )?;

        exec("CREATE INDEX IF NOT EXISTS idx_prompt_presets_type ON prompt_presets(preset_type)")?;

        conn.pragma_update(None, "user_version", 9).map_err(|e| {
            crate::error::EngineError::Config(format!("Failed to set user_version: {e}"))
        })?;
    }

    if version < 10 {
        let exec = |sql: &str| {
            conn.execute(sql, [])
                .map_err(|e| crate::error::EngineError::Config(format!("Migration failed: {e}")))
        };

        exec(
            "CREATE TABLE IF NOT EXISTS worlds (
                id INTEGER PRIMARY KEY AUTOINCREMENT,
                key TEXT NOT NULL UNIQUE,      -- original string ID (e.g. 'redmist_estate')
                name TEXT NOT NULL,
                description TEXT NOT NULL DEFAULT '',
                global_rules TEXT NOT NULL DEFAULT '[]',  -- JSON: Vec<String>
                scenarios TEXT NOT NULL DEFAULT '[]',     -- JSON: Vec<StartingScenario>
                default_scenario_id TEXT,
                default_room_image TEXT,
                player_key TEXT NOT NULL DEFAULT '',  -- dropped in v13
                created_at TEXT NOT NULL,
                updated_at TEXT NOT NULL
            )",
        )?;

        exec(
            "CREATE TABLE IF NOT EXISTS maps (
                id INTEGER PRIMARY KEY AUTOINCREMENT,
                world_id INTEGER NOT NULL REFERENCES worlds(id) ON DELETE CASCADE,
                map_data TEXT NOT NULL,         -- JSON: full serialized MapDef
                created_at TEXT NOT NULL,
                updated_at TEXT NOT NULL
            )",
        )?;
        exec("CREATE INDEX IF NOT EXISTS idx_maps_world ON maps(world_id)")?;

        exec(
            "CREATE TABLE IF NOT EXISTS personas (
                id INTEGER PRIMARY KEY AUTOINCREMENT,
                key TEXT NOT NULL UNIQUE,      -- filename stem (e.g. 'julian')
                name TEXT NOT NULL,
                description TEXT NOT NULL DEFAULT '',
                personality TEXT NOT NULL DEFAULT '',
                scenario TEXT NOT NULL DEFAULT '',
                example_dialogue TEXT NOT NULL DEFAULT '',
                summary TEXT,
                profile_image TEXT,
                headshot_image TEXT,
                inventory TEXT NOT NULL DEFAULT '[]',  -- JSON: Vec<String>
                created_at TEXT NOT NULL,
                updated_at TEXT NOT NULL
            )",
        )?;

        exec(
            "CREATE TABLE IF NOT EXISTS characters (
                id INTEGER PRIMARY KEY AUTOINCREMENT,
                key TEXT NOT NULL,             -- from NpcCard.id (e.g. 'elena_voss')
                world_id INTEGER NOT NULL REFERENCES worlds(id) ON DELETE CASCADE,
                name TEXT NOT NULL,
                description TEXT NOT NULL DEFAULT '',
                personality TEXT NOT NULL DEFAULT '',
                scenario TEXT NOT NULL DEFAULT '',
                example_dialogue TEXT NOT NULL DEFAULT '',
                summary TEXT,
                profile_image TEXT,
                headshot_image TEXT,
                inventory TEXT NOT NULL DEFAULT '[]',     -- JSON: Vec<String>
                triggers TEXT NOT NULL DEFAULT '[]',      -- JSON: Vec<Trigger>
                relationships TEXT NOT NULL DEFAULT '[]', -- JSON: Vec<Relationship>
                created_at TEXT NOT NULL,
                updated_at TEXT NOT NULL,
                UNIQUE(key, world_id)
            )",
        )?;
        exec("CREATE INDEX IF NOT EXISTS idx_characters_world ON characters(world_id)")?;

        exec(
            "CREATE TABLE IF NOT EXISTS settings (
                id INTEGER PRIMARY KEY CHECK (id = 1),  -- singleton row
                connections TEXT NOT NULL DEFAULT '[]',  -- JSON: Vec<Connection>
                narration_connection_id TEXT NOT NULL DEFAULT 'openrouter-gpt-4o-mini',
                quantifier_connection_id TEXT NOT NULL DEFAULT 'openrouter-gpt-4o-mini',
                response_length TEXT NOT NULL DEFAULT '',
                text_check TEXT NOT NULL DEFAULT '{}',    -- JSON: TextCheckSettings
                agents TEXT NOT NULL DEFAULT '[]',        -- JSON: Vec<AgentConfig>
                active_system_prompt_preset_id TEXT NOT NULL DEFAULT 'system_default',
                active_quantifier_prompt_preset_id TEXT NOT NULL DEFAULT 'quantifier_default',
                created_at TEXT NOT NULL,
                updated_at TEXT NOT NULL
            )",
        )?;

        conn.pragma_update(None, "user_version", 11).map_err(|e| {
            crate::error::EngineError::Config(format!("Failed to set user_version: {e}"))
        })?;
    }

    if version < 12 {
        let exec = |sql: &str| {
            conn.execute(sql, [])
                .map_err(|e| crate::error::EngineError::Config(format!("Migration failed: {e}")))
        };

        if !column_exists(conn, "games", "world_key") {
            exec("ALTER TABLE games ADD COLUMN world_key TEXT NOT NULL DEFAULT ''")?;
        }
        exec(
            "UPDATE games SET world_key = COALESCE(
            (SELECT key FROM worlds WHERE worlds.name = games.world_name),
            'redmist_estate'
        ) WHERE world_key = ''",
        )?;

        conn.pragma_update(None, "user_version", 12).map_err(|e| {
            crate::error::EngineError::Config(format!("Failed to set user_version: {e}"))
        })?;
    }

    if version < 13 {
        let exec = |sql: &str| {
            conn.execute(sql, [])
                .map_err(|e| crate::error::EngineError::Config(format!("Migration failed: {e}")))
        };

        exec("ALTER TABLE games ADD COLUMN persona_key TEXT NOT NULL DEFAULT ''")?;
        exec("ALTER TABLE games ADD COLUMN persona_name TEXT NOT NULL DEFAULT ''")?;
        exec("ALTER TABLE worlds DROP COLUMN player_key")?;
        conn.pragma_update(None, "user_version", 13).map_err(|e| {
            crate::error::EngineError::Config(format!("Failed to set user_version: {e}"))
        })?;
    }

    if version < 14 {
        let exec = |sql: &str| {
            conn.execute(sql, [])
                .map_err(|e| crate::error::EngineError::Config(format!("Migration failed: {e}")))
        };
        if column_exists(conn, "worlds", "starting_room_id") {
            exec("ALTER TABLE worlds DROP COLUMN starting_room_id")?;
        }
        conn.pragma_update(None, "user_version", 14).map_err(|e| {
            crate::error::EngineError::Config(format!("Failed to set user_version: {e}"))
        })?;
    }

    Ok(())
}
