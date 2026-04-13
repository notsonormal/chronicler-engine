use std::sync::Arc;
use tokio::sync::broadcast;

#[derive(Clone)]
pub struct Hub {
    tx: Arc<broadcast::Sender<String>>,
}

impl Hub {
    pub fn new() -> Self {
        let (tx, _) = broadcast::channel(100);
        Self { tx: Arc::new(tx) }
    }

    pub fn broadcast(&self, message: String) {
        let _ = self.tx.send(message);
    }

    pub fn subscribe(&self) -> broadcast::Receiver<String> {
        self.tx.subscribe()
    }
}

impl Default for Hub {
    fn default() -> Self {
        Self::new()
    }
}
