use crate::error::EngineError;
use crate::model::message::{Message, Swipe};

pub trait MessageStorage: Send + Sync {
    fn set_game_id(&self, game_id: u64);
    fn current_game_id(&self) -> u64;
    fn insert_message(&self, msg: &mut Message) -> Result<(), EngineError>;
    fn update_message(&self, id: u64, text: &str) -> Result<(), EngineError>;
    fn delete_message(&self, id: u64) -> Result<(), EngineError>;
    fn load_messages(&self) -> Result<Vec<Message>, EngineError>;

    /// Soft-delete a message (used by retry so it can be restored on failure).
    fn soft_delete_message(&self, id: u64) -> Result<(), EngineError>;
    fn restore_soft_deleted(&self, ids: &[u64]) -> Result<(), EngineError>;
    /// Hard-delete soft-deleted messages (CASCADE removes their swipes).
    fn purge_soft_deleted(&self, ids: &[u64]) -> Result<(), EngineError>;
    fn insert_swipe(&self, message_id: u64, swipe: &Swipe, index: usize)
    -> Result<(), EngineError>;
    fn update_active_swipe(&self, message_id: u64, index: usize) -> Result<(), EngineError>;
    fn shift_swipe_indices(&self, message_id: u64, offset: usize) -> Result<(), EngineError>;

    fn migrate_swipes(
        &self,
        message_id: u64,
        pending_swipes: &[Swipe],
        new_active_index: usize,
        to_delete: &[u64],
    ) -> Result<(), EngineError>;
}
