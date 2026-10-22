//! [DOC: docs/diataxis/reference/game_flow.md]
//! The watch on the tasks a Job spawns, so a tier-1 test waits for a Job to end

use std::sync::Mutex;

/// One instance per app: every Job's task joins one watch, so one wait covers them
/// all. It is built into production because the `testing` feature is a default one.
#[derive(Default)]
pub struct JobTasks {
    handles: Mutex<Vec<tokio::task::JoinHandle<()>>>,
}

impl JobTasks {
    pub fn watch(&self, handle: tokio::task::JoinHandle<()>) {
        let mut handles = self.handles.lock().unwrap_or_else(|e| e.into_inner());
        handles.retain(|handle| !handle.is_finished());
        handles.push(handle);
    }

    pub async fn wait(&self, timeout: std::time::Duration) -> bool {
        tokio::time::timeout(timeout, self.wait_for_each())
            .await
            .is_ok()
    }

    pub fn running(&self) -> usize {
        let handles = self.handles.lock().unwrap_or_else(|e| e.into_inner());
        handles
            .iter()
            .filter(|handle| !handle.is_finished())
            .count()
    }

    async fn wait_for_each(&self) {
        loop {
            let pending: Vec<tokio::task::JoinHandle<()>> = {
                let mut handles = self.handles.lock().unwrap_or_else(|e| e.into_inner());
                handles.drain(..).collect()
            };
            if pending.is_empty() {
                return;
            }
            for handle in pending {
                // A Job that panicked failed its own test already; this only orders
                // the test after the task.
                if let Err(join_error) = handle.await {
                    tracing::debug!("a Job task ended without a result: {join_error}");
                }
            }
        }
    }
}
