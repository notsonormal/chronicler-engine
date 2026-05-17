use crate::error::EngineError;
use crate::model::message::Message;

pub trait MessageStorage: Send + Sync {
    fn insert_message(&self, msg: &mut Message) -> Result<(), EngineError>;
    fn update_message(&self, id: u64, text: &str) -> Result<(), EngineError>;
    fn delete_message(&self, id: u64) -> Result<(), EngineError>;
    fn load_messages(&self) -> Result<Vec<Message>, EngineError>;
}
