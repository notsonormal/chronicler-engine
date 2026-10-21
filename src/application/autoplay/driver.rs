//! [DOC: docs/diataxis/reference/game_flow.md]
//! The Autoplay driver: one blocking task that runs the Jobs of one run, one after the other.

use std::sync::Arc;

use crate::application::generation::gate::{GameWriter, Refusal, RunEnded, RunId, RunRefusal};
use crate::application::pipeline::job::{Job, JobDeps, JobEnd};
use crate::domain::model::game::GameId;

/// The budget of a start that names none.
pub const DEFAULT_BUDGET: u32 = 10;

/// Why a run ended. The story holds no run marker, so the log line of the end is
/// the only record of it.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum RunEnd {
    /// The last cycle of the budget ended.
    BudgetReached,
    /// A Job ended with a failure line, and no later accept clears it.
    JobFailed,
    /// The accept of a Job refused its request. A refusal that has a failure line
    /// wrote it.
    Refused,
    /// A stop, a reset or a delete removed the run hold.
    Stopped,
}

/// The driver of one run. It runs Jobs and counts their cycles in the run hold;
/// each Job writes its own records and failure line.
pub struct AutoplayRun {
    writer: GameWriter,
    deps: JobDeps,
    game: GameId,
    run: RunId,
}

impl AutoplayRun {
    /// Makes the run hold and spawns the driver, then returns at once, as a Job
    /// start does. The driver's task joins the watch of `JobTasks`, so one wait
    /// covers the whole run.
    pub fn start(
        writer: &GameWriter,
        deps: JobDeps,
        game: GameId,
        budget: u32,
    ) -> Result<(), Refusal> {
        let run = writer.start_run(game, budget)?;
        tracing::info!(game = %game, run = ?run, budget, "autoplay run started");
        let tasks = Arc::clone(&deps.tasks);
        let driver = Self {
            writer: writer.clone(),
            deps,
            game,
            run,
        };
        tasks.watch(tokio::task::spawn_blocking(move || driver.drive()));
        Ok(())
    }

    fn drive(&self) {
        let end = self.run_jobs();
        tracing::info!(game = %self.game, run = ?self.run, end = ?end, "autoplay run ended");
    }

    /// Each pass runs the Job the run's rule picks from the end of the story, and
    /// starts no Job before the one before it ended.
    fn run_jobs(&self) -> RunEnd {
        loop {
            let job = match Job::run_for_run(&self.writer, &self.deps, self.game, self.run) {
                Ok(job) => job,
                Err(RunRefusal::Ended) => return RunEnd::Stopped,
                Err(RunRefusal::Refused(refusal)) => {
                    tracing::warn!(game = %self.game, run = ?self.run, "the run's Job was refused: {refusal:?}");
                    return RunEnd::Refused;
                }
            };
            match job.end {
                JobEnd::Done => {}
                JobEnd::Failed => return RunEnd::JobFailed,
                JobEnd::EntryMoved => return RunEnd::Stopped,
            }
            if !job.request.is_continue() {
                continue;
            }
            match self.writer.finish_cycle(self.game, self.run) {
                Err(RunEnded) => return RunEnd::Stopped,
                Ok(cycle) => {
                    tracing::info!(
                        game = %self.game,
                        run = ?self.run,
                        cycle = cycle.cycle,
                        budget = cycle.budget,
                        "autoplay cycle ended"
                    );
                    if cycle.budget_reached() {
                        return RunEnd::BudgetReached;
                    }
                }
            }
        }
    }
}

/// A panic in a Job must not leave the Game held by a run that has no driver.
/// `end_run` leaves a run hold of another run, or none, as it is.
impl Drop for AutoplayRun {
    fn drop(&mut self) {
        self.writer.end_run(self.game, self.run);
    }
}
