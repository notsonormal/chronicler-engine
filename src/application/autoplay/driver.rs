//! [DOC: docs/diataxis/reference/game_flow.md]
//! The Autoplay loop: one blocking task that runs the Jobs of one run, one after the other.

use std::sync::Arc;

use crate::application::autoplay::budget::Budget;
use crate::application::generation::gate::{
    AutoplayAccept, AutoplayProgress, AutoplayRunId, Decision, GameWriter, Refusal,
};
use crate::application::pipeline::job::{Job, JobDeps, JobEnd};
use crate::domain::model::game::GameId;
use crate::domain::model::generation::{JobRequest, StoryEnd};

/// Why a run ended. The story holds no run marker, so the log line of the end is
/// the only record of it.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum RunEnd {
    /// The last cycle of the budget ended.
    BudgetReached,
    /// A Job ended with a failure line, and no later accept clears it.
    JobFailed,
    /// The accept of a Job refused its request, or could not read the story. A
    /// refusal that has a failure line wrote it.
    Refused,
    /// A stop, a reset or a delete removed the holder.
    Stopped,
}

/// Owns the budget, the cycles and the cycle rule; the gate stores the progress
/// with the holder, and each Job writes its own records and failure line.
pub struct AutoplayRun {
    writer: GameWriter,
    deps: JobDeps,
    game: GameId,
    run: AutoplayRunId,
    budget: Budget,
}

impl AutoplayRun {
    /// The loop's task joins the watch of `JobTasks`, so one wait covers the whole
    /// run.
    pub fn start(
        writer: &GameWriter,
        deps: JobDeps,
        game: GameId,
        budget: Budget,
    ) -> Result<(), Refusal> {
        let run = writer.reserve_for_autoplay(
            game,
            AutoplayProgress {
                done: 0,
                budget: budget.cycles(),
            },
        )?;
        tracing::info!(game = %game, run = ?run, budget = budget.cycles(), "autoplay run started");
        let tasks = Arc::clone(&deps.tasks);
        let driver = Self {
            writer: writer.clone(),
            deps,
            game,
            run,
            budget,
        };
        tasks.watch(tokio::task::spawn_blocking(move || driver.drive()));
        Ok(())
    }

    fn drive(&self) {
        let end = self.run_jobs();
        tracing::info!(game = %self.game, run = ?self.run, end = ?end, "autoplay run ended");
    }

    fn run_jobs(&self) -> RunEnd {
        let mut done = 0;
        let mut after_job = false;
        loop {
            self.deps.pause.point(self.game);
            let budget = self.budget;
            let accepted = self
                .writer
                .accept_for_autoplay(self.game, self.run, |story| {
                    Self::decide(story, done, after_job, budget)
                });
            let accepted_job = match accepted {
                AutoplayAccept::Ended => return RunEnd::Stopped,
                AutoplayAccept::Refused(refusal) => {
                    tracing::warn!(game = %self.game, run = ?self.run, "the run's Job was refused: {refusal:?}");
                    return RunEnd::Refused;
                }
                AutoplayAccept::Finished(progress) => {
                    self.log_cycles(done, progress);
                    return RunEnd::BudgetReached;
                }
                AutoplayAccept::Accepted(accepted, progress) => {
                    self.log_cycles(done, progress);
                    done = progress.done;
                    accepted
                }
            };
            match Job::run_accepted(accepted_job, &self.deps) {
                JobEnd::Done => after_job = true,
                JobEnd::Failed => return RunEnd::JobFailed,
                JobEnd::EntryMoved => return RunEnd::Stopped,
            }
        }
    }

    /// The gate runs it under its lock, so it takes no `self`: it reads only the end
    /// of the story and the loop's own copies.
    fn decide(story: &StoryEnd, done: u32, after_job: bool, budget: Budget) -> Decision {
        let done = done + u32::from(after_job && story.ends_a_cycle());
        let progress = AutoplayProgress {
            done,
            budget: budget.cycles(),
        };
        if done >= progress.budget {
            Decision::Finish(progress)
        } else {
            Decision::Next(progress, JobRequest::next_in_autoplay(story.tail.as_ref()))
        }
    }

    fn log_cycles(&self, before: u32, progress: AutoplayProgress) {
        if progress.done > before {
            tracing::info!(
                game = %self.game,
                run = ?self.run,
                cycle = progress.done,
                budget = progress.budget,
                "autoplay cycle ended"
            );
        }
    }
}

/// A panic in a Job must not leave the Game held by a run that has no loop.
impl Drop for AutoplayRun {
    fn drop(&mut self) {
        self.writer.end_autoplay(self.game, self.run);
    }
}
