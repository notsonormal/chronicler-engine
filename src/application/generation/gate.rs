//! [DOC: docs/diataxis/reference/game_flow.md]
//! GameWriter and JobWriter — the writer rule and the guard: the one registry of the busy state, and the transactions a Job writes through.

use std::collections::{HashMap, HashSet};
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::{Arc, Mutex};

use crate::adapters::driven::storage::{Storage, Tx, TxToken};
use crate::application::errors::ApplicationError;
use crate::domain::model::character::CharacterCard;
use crate::domain::model::dock::{DockRecord, OptionsFailure};
use crate::domain::model::game::{ConfigChange, GameId, NewGame};
use crate::domain::model::agent::Kind;
use crate::domain::model::change::{EngineKind, GameKind};
use crate::domain::model::generation::{
    GenerationOrder, JobPlan, JobRequest, NewSwipe, PlanError, Place, StoryEnd,
};
use crate::domain::model::message::{Maker, Message, Swipe, SwipeAt};
use crate::domain::model::state::generation_status::{GenerationFailure, Record, Step};
use crate::domain::model::state::message_types::{MessageKind, MessageType};
use crate::domain::model::state::swipe_state::SwipeState;
use crate::domain::model::tail::Tail;
use crate::error::EngineError;

#[derive(Debug, Clone, PartialEq)]
pub enum NotAccepted {
    /// The Game is busy: `409` "Still thinking...".
    StillThinking,
    /// The Message that the request names is no longer the last one: `409`.
    LastMessageChanged,
    /// The request itself is wrong: `400`, with the reason line.
    BadRequest(String),
    /// The request could not run: `500`, carrying the failure's own line. The
    /// accept wrote the Game's failure line from the same failure, so the answer
    /// and the status name the same thing. The raw text stays in the disclosure.
    Failed(GenerationFailure),
}

impl NotAccepted {
    /// The line the user reads for the request that was not accepted, also when it surfaces as an error
    /// outside the HTTP answer.
    pub fn line(&self) -> String {
        match self {
            Self::StillThinking => "Still thinking...".to_string(),
            Self::LastMessageChanged => {
                "The page showed a Message that is no longer the last one.".to_string()
            }
            Self::BadRequest(line) => line.clone(),
            Self::Failed(failure) => failure.line(),
        }
    }
}

/// Why a step or a transaction of a Job did not finish.
#[derive(Debug)]
pub enum JobFailure {
    /// The Game's Job entry no longer holds this Job's id: a stop or a reset
    /// ended the Job, so its late writes land nowhere.
    EntryMoved,
    /// The Job writes this line and ends.
    Line(GenerationFailure),
}

impl From<GenerationFailure> for JobFailure {
    fn from(failure: GenerationFailure) -> Self {
        Self::Line(failure)
    }
}

impl JobFailure {
    /// The failure of the generation of `kind`: an event generation reports the
    /// event line, wherever it failed.
    pub fn of_generation(self, kind: Kind) -> Self {
        match self {
            Self::Line(line) => Self::Line(line.of_generation(kind)),
            Self::EntryMoved => Self::EntryMoved,
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub enum OptionsResult {
    /// The set the dock shows as pickable options.
    Offered(Vec<String>),
    /// The run wrote no set: the dock shows an Options failure with the reason, and the raw
    /// text behind its disclosure.
    Unavailable { reason: String, raw: String },
}

/// No state: `save_state_update` consumes it, so a Swipe's state is written at
/// most twice, the pre-analysis state and then the analysed one. The Retry of a typed Input
/// drops it.
#[derive(Debug, PartialEq, Eq)]
pub struct SavedText {
    at: SwipeAt,
}

impl SavedText {
    pub fn at(&self) -> SwipeAt {
        self.at
    }
}

/// The record of a Job that runs: the Job's own methods are its only writers.
struct JobEntry {
    /// The Job that owns the entry; a transaction whose id differs is refused.
    id: u64,
    step: Step,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct AutoplayRunId(u64);

/// Only the Autoplay loop writes it; the gate stores it with the holder and
/// applies no rule to it.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct AutoplayProgress {
    pub done: u32,
    pub budget: u32,
}

impl AutoplayProgress {
    pub fn cycle(&self) -> u32 {
        self.done + 1
    }
}

/// What holds a busy Game. Memory only, like the Job entry.
enum Holder {
    /// A plain Job runs.
    Job(JobEntry),
    /// An Autoplay run holds the Game from its start to its end, also between two
    /// of its Jobs. `job` is the Job entry of the run's running Job.
    Autoplay {
        run: AutoplayRunId,
        progress: AutoplayProgress,
        job: Option<JobEntry>,
    },
}

impl Holder {
    fn job(&self) -> Option<&JobEntry> {
        match self {
            Self::Job(entry) => Some(entry),
            Self::Autoplay { job, .. } => job.as_ref(),
        }
    }

    fn job_mut(&mut self) -> Option<&mut JobEntry> {
        match self {
            Self::Job(entry) => Some(entry),
            Self::Autoplay { job, .. } => job.as_mut(),
        }
    }

    fn is_autoplay_run(&self, id: AutoplayRunId) -> bool {
        matches!(self, Self::Autoplay { run, .. } if *run == id)
    }

    fn busy(&self) -> Busy {
        match self {
            Self::Job(entry) => Busy::Job(entry.step.clone()),
            Self::Autoplay { progress, job, .. } => Busy::Autoplay {
                progress: *progress,
                step: job.as_ref().map(|entry| entry.step.clone()),
            },
        }
    }
}

/// Absent means idle.
type Holders = HashMap<GameId, Holder>;

/// The one registry of holders: a private `Mutex` behind a type whose only method is
/// `hold`, so no caller takes the lock without a transaction opener in hand.
struct BusyRegistry {
    busy: Mutex<Holders>,
}

impl BusyRegistry {
    fn new() -> Self {
        Self {
            busy: Mutex::new(HashMap::new()),
        }
    }

    /// A writer's check and its transactions happen in one critical section, and the lock
    /// order stays the guard, then the backend mutex, then the connection mutex.
    ///
    /// The busy state is memory only, so this is the one place that sees it change. The
    /// `Busy` of each Game before and after `f` decide the raise, and a step that keeps
    /// its value raises nothing.
    fn hold<T>(&self, storage: &Arc<Storage>, f: impl FnOnce(&mut Holders, &TxOpener) -> T) -> T {
        let opener = TxOpener {
            storage: Arc::clone(storage),
            token: TxToken::new(),
        };
        let mut busy = self
            .busy
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner());
        let before = Self::busy_of_each(&busy);
        let result = f(&mut busy, &opener);
        let after = Self::busy_of_each(&busy);
        Self::raise_busy_changes(storage, &before, &after);
        result
    }

    fn busy_of_each(holders: &Holders) -> HashMap<GameId, Busy> {
        holders
            .iter()
            .map(|(game, holder)| (*game, holder.busy()))
            .collect()
    }

    /// The busy state of each Game whose `Busy` changed, and the Games list when the set
    /// of Games that a run holds changed: the list marks those Games.
    fn raise_busy_changes(
        storage: &Storage,
        before: &HashMap<GameId, Busy>,
        after: &HashMap<GameId, Busy>,
    ) {
        let changes = storage.changes();
        for game in before.keys().chain(after.keys()).collect::<HashSet<_>>() {
            if before.get(game) != after.get(game) {
                changes.raise_game(*game, &[GameKind::BusyState]);
            }
        }
        let run_games = |held: &HashMap<GameId, Busy>| -> HashSet<GameId> {
            held.iter()
                .filter(|(_, busy)| matches!(busy, Busy::Autoplay { .. }))
                .map(|(game, _)| *game)
                .collect()
        };
        if run_games(before) != run_games(after) {
            changes.raise_engine(&[EngineKind::Games]);
        }
    }
}

/// The transaction opener of one hold. Only `BusyRegistry::hold` makes one, so every
/// transaction of record writes opens inside a hold; a reset opens two in the same hold.
struct TxOpener {
    storage: Arc<Storage>,
    token: TxToken,
}

impl TxOpener {
    fn open<T>(
        &self,
        game: GameId,
        f: impl FnOnce(&mut Tx<'_>) -> Result<T, EngineError>,
    ) -> Result<T, EngineError> {
        self.storage.transaction(game, self.token.clone(), f)
    }

    /// The create transaction: the Game row goes first, and its id reaches the writes
    /// that follow it in the same transaction.
    fn open_new_game<T>(
        &self,
        new: &NewGame,
        f: impl FnOnce(GameId, &mut Tx<'_>) -> Result<T, EngineError>,
    ) -> Result<T, EngineError> {
        self.storage
            .transaction_new_game(new, self.token.clone(), f)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub enum Busy {
    /// A plain Job runs.
    Job(Step),
    /// An Autoplay run holds the Game. `step` is the step of the run's Job, and
    /// `None` between two Jobs of the run.
    Autoplay {
        progress: AutoplayProgress,
        step: Option<Step>,
    },
}

impl Busy {
    pub fn step(&self) -> Option<&Step> {
        match self {
            Self::Job(step) => Some(step),
            Self::Autoplay { step, .. } => step.as_ref(),
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub enum Decision {
    /// The Autoplay run goes on: the gate stores the progress and accepts this request.
    Next(AutoplayProgress, JobRequest),
    /// The Autoplay run is over: the gate removes the holder.
    Finish(AutoplayProgress),
}

pub enum AutoplayAccept {
    /// The Job is accepted, and the progress is stored with the holder.
    Accepted(AcceptedJob, AutoplayProgress),
    /// The loop decided to finish: the holder is gone.
    Finished(AutoplayProgress),
    /// The holder of the Game is gone or holds another run's id: a stop, a reset or
    /// a delete ended the run. Nothing is written.
    Ended,
    /// The accept did not accept and wrote the Game's failure line if the outcome is `Failed`.
    /// A plan error or a failed story read removed the holder; the run's own
    /// Job in flight (unreachable) leaves it for `end_autoplay`.
    NotAccepted(NotAccepted),
}

pub struct AcceptedJob {
    pub writer: JobWriter,
    pub plan: JobPlan,
}

/// The only code that opens a transaction of record writes, and the owner of the busy
/// state. One registry serves all Games, and a Job keeps the Game it started on.
#[derive(Clone)]
pub struct GameWriter {
    storage: Arc<Storage>,
    busy: Arc<BusyRegistry>,
    next_job_id: Arc<AtomicU64>,
    next_autoplay_run_id: Arc<AtomicU64>,
}

impl GameWriter {
    pub fn new(storage: Arc<Storage>) -> Self {
        Self {
            storage,
            busy: Arc::new(BusyRegistry::new()),
            next_job_id: Arc::new(AtomicU64::new(0)),
            next_autoplay_run_id: Arc::new(AtomicU64::new(0)),
        }
    }

    fn next_job_id(&self) -> u64 {
        self.next_job_id
            .fetch_add(1, Ordering::SeqCst)
            .wrapping_add(1)
    }

    fn hold<T>(&self, f: impl FnOnce(&mut Holders, &TxOpener) -> T) -> T {
        self.busy.hold(&self.storage, f)
    }

    fn next_autoplay_run_id(&self) -> AutoplayRunId {
        AutoplayRunId(
            self.next_autoplay_run_id
                .fetch_add(1, Ordering::SeqCst)
                .wrapping_add(1),
        )
    }

    /// The busy rule: a Game with a holder answers "Still thinking..." before any
    /// other check.
    fn require_idle(holders: &Holders, game: GameId) -> Result<(), NotAccepted> {
        if holders.contains_key(&game) {
            return Err(NotAccepted::StillThinking);
        }
        Ok(())
    }

    pub fn accept(&self, game: GameId, request: JobRequest) -> Result<AcceptedJob, NotAccepted> {
        self.hold(|holders, opener| {
            Self::require_idle(holders, game)?;
            let job_id = self.next_job_id();
            let story = self.story_end_or_not_accepted(game, opener)?;
            let (plan, step) = self.plan_and_start(opener, game, story, &request)?;
            holders.insert(game, Holder::Job(JobEntry { id: job_id, step }));
            Ok(self.accepted(game, job_id, plan))
        })
    }

    /// The Autoplay run's own id passes the holder, and only between two of its Jobs.
    /// `decide` runs under the registry hold, on
    /// the end of the story that the accept reads, and calls no `GameWriter`.
    pub fn accept_for_autoplay(
        &self,
        game: GameId,
        run: AutoplayRunId,
        decide: impl FnOnce(&StoryEnd) -> Decision,
    ) -> AutoplayAccept {
        self.hold(|holders, opener| {
            let between_jobs = holders
                .get(&game)
                .filter(|holder| holder.is_autoplay_run(run))
                .map(|holder| holder.job().is_none());
            match between_jobs {
                None => return AutoplayAccept::Ended,
                // The loop accepts one Job at a time, so its own Job in flight is
                // unreachable; the Game is busy, and nothing is written.
                Some(false) => return AutoplayAccept::NotAccepted(NotAccepted::StillThinking),
                Some(true) => {}
            }
            let accepted = self.accept_in_autoplay_hold(holders, opener, game, run, decide);
            if matches!(accepted, AutoplayAccept::NotAccepted(_)) {
                holders.remove(&game);
            }
            accepted
        })
    }

    fn accept_in_autoplay_hold(
        &self,
        holders: &mut Holders,
        opener: &TxOpener,
        game: GameId,
        run: AutoplayRunId,
        decide: impl FnOnce(&StoryEnd) -> Decision,
    ) -> AutoplayAccept {
        let job_id = self.next_job_id();
        let story = match self.story_end_or_not_accepted(game, opener) {
            Ok(story) => story,
            Err(not_accepted) => return AutoplayAccept::NotAccepted(not_accepted),
        };
        let (progress, request) = match decide(&story) {
            Decision::Finish(progress) => {
                holders.remove(&game);
                return AutoplayAccept::Finished(progress);
            }
            Decision::Next(progress, request) => (progress, request),
        };
        let (plan, step) = match self.plan_and_start(opener, game, story, &request) {
            Ok(started) => started,
            Err(not_accepted) => return AutoplayAccept::NotAccepted(not_accepted),
        };
        holders.insert(
            game,
            Holder::Autoplay {
                run,
                progress,
                job: Some(JobEntry { id: job_id, step }),
            },
        );
        AutoplayAccept::Accepted(self.accepted(game, job_id, plan), progress)
    }

    /// The plan of the request, and the transactions that make it a started Job. The transaction
    /// lands before the caller makes the entry: a refused accept makes no entry, so
    /// a Game is never busy for a Job that never started.
    fn plan_and_start(
        &self,
        opener: &TxOpener,
        game: GameId,
        story: StoryEnd,
        request: &JobRequest,
    ) -> Result<(JobPlan, Step), NotAccepted> {
        let plan = JobPlan::for_request(request, story).map_err(|plan_error| match plan_error {
            PlanError::BadRequest(line) => NotAccepted::BadRequest(line),
            PlanError::Missing(record) => self.missing(game, record, opener),
            PlanError::LastMessageChanged => NotAccepted::LastMessageChanged,
        })?;
        self.require_records(game, &plan, opener)?;
        let step = plan.first_step();
        self.accept_in_tx(game, opener)?;
        Ok((plan, step))
    }

    fn accepted(&self, game: GameId, job_id: u64, plan: JobPlan) -> AcceptedJob {
        AcceptedJob {
            writer: JobWriter {
                writer: self.clone(),
                game,
                id: job_id,
            },
            plan,
        }
    }

    /// The budget rule is the loop's own, so the gate stores the budget as it comes.
    pub fn reserve_for_autoplay(
        &self,
        game: GameId,
        progress: AutoplayProgress,
    ) -> Result<AutoplayRunId, NotAccepted> {
        let run = self.next_autoplay_run_id();
        self.hold(|holders, _opener| {
            Self::require_idle(holders, game)?;
            holders.insert(
                game,
                Holder::Autoplay {
                    run,
                    progress,
                    job: None,
                },
            );
            Ok(run)
        })
    }

    /// A holder of another run, or none, stays as it is.
    pub fn end_autoplay(&self, game: GameId, run: AutoplayRunId) {
        self.hold(|holders, _opener| {
            if holders
                .get(&game)
                .is_some_and(|holder| holder.is_autoplay_run(run))
            {
                holders.remove(&game);
            }
        });
    }

    /// The write of the config columns `change` names, refused while an Autoplay run
    /// holds the Game. The check and the transaction run in one registry hold, so a run
    /// cannot start between them. The caller read and validated `change` before.
    pub fn save_config(&self, game: GameId, change: &ConfigChange) -> Result<(), ApplicationError> {
        self.hold(|holders, opener| {
            if matches!(holders.get(&game), Some(Holder::Autoplay { .. })) {
                return Err(ApplicationError::NotAccepted(NotAccepted::StillThinking));
            }
            opener.open(game, |tx| tx.set_game_config(change))?;
            Ok(())
        })
    }

    pub fn busy(&self, game: GameId) -> Option<Busy> {
        self.hold(|holders, _opener| holders.get(&game).map(Holder::busy))
    }

    pub fn stop(&self, game: GameId) -> Result<(), EngineError> {
        self.hold(|holders, opener| {
            // The read goes first: a failed read leaves the Job running, and the stop
            // answers that it failed.
            let record = DockRecord {
                offered: self.dock_of(game)?.offered,
                failure: None,
            };
            holders.remove(&game);
            opener.open(game, |tx| Self::clear_failure_and_put_dock(tx, &record))
        })
    }

    pub fn switch_swipe(
        &self,
        game: GameId,
        message: u64,
        index: usize,
    ) -> Result<(), NotAccepted> {
        self.hold(|holders, opener| {
            Self::require_idle(holders, game)?;
            let tail = self.tail_or_not_accepted(game, opener)?;
            let last =
                tail.ok_or_else(|| NotAccepted::BadRequest("No message to switch".to_string()))?;
            if last.last.id != message {
                return Err(NotAccepted::BadRequest(
                    "Only the last message can be swiped".to_string(),
                ));
            }
            if index >= last.last.swipes.len() {
                return Err(NotAccepted::BadRequest(
                    "Swipe index out of bounds".to_string(),
                ));
            }
            let at = SwipeAt {
                message,
                swipe: index,
            };
            opener
                .open(game, |tx| {
                    tx.set_active_swipe(at)?;
                    Self::clear_failure_and_put_dock(tx, &DockRecord::default())
                })
                .map_err(|error| self.save_failed(game, &error, opener))
        })
    }

    pub fn edit(&self, game: GameId, message: u64, text: &str) -> Result<(), NotAccepted> {
        self.hold(|holders, opener| {
            Self::require_idle(holders, game)?;
            let at = match self.storage.stories().active_swipe(game, message) {
                Ok(Some(at)) => at,
                Ok(None) => return Err(NotAccepted::BadRequest("Message not found".to_string())),
                Err(error) => return Err(self.unreadable(game, Record::Message, &error, opener)),
            };
            opener
                .open(game, |tx| tx.set_swipe_text(at, text))
                .map_err(|error| self.save_failed(game, &error, opener))
        })
    }

    pub fn delete_last(&self, game: GameId, named: u64) -> Result<(), NotAccepted> {
        self.hold(|holders, opener| {
            Self::require_idle(holders, game)?;
            let tail = self.tail_or_not_accepted(game, opener)?;
            let last =
                tail.ok_or_else(|| NotAccepted::BadRequest("No message to delete".to_string()))?;
            if last.last.id != named {
                return Err(NotAccepted::LastMessageChanged);
            }
            if !last.can_delete() {
                return Err(NotAccepted::BadRequest(
                    "The story keeps its last message".to_string(),
                ));
            }
            opener
                .open(game, |tx| {
                    tx.delete_message(last.last.id)?;
                    Self::clear_failure_and_put_dock(tx, &DockRecord::default())
                })
                .map_err(|error| self.save_failed(game, &error, opener))
        })
    }

    pub fn create(&self, new: &NewGame) -> Result<GameId, EngineError> {
        let opening = self.opening_message(new)?;
        self.hold(|_holders, opener| self.create_in_tx(new, &opening, opener))
    }

    /// Both transactions run in one hold, and the old Game's holder ends under the same
    /// guard, so its next transaction or step is refused by the id
    /// check; a running model call ends on its own. The create transaction runs first: a
    /// failure there leaves the old Game, its entry and the pointer as they were.
    pub fn replace(&self, old: GameId, new: &NewGame) -> Result<GameId, EngineError> {
        let opening = self.opening_message(new)?;
        self.hold(|holders, opener| {
            let new_id = self.create_in_tx(new, &opening, opener)?;
            holders.remove(&old);
            opener.open(old, |tx| tx.delete_game())?;
            Ok(new_id)
        })
        .inspect(|_| self.storage.changes().forget_game(old))
    }

    /// One hold: the Game's holder ends under the same guard as its rows, so the
    /// Job's next transaction or step is refused by the id check before its records go.
    pub fn delete(&self, game: GameId) -> Result<(), EngineError> {
        self.hold(|holders, opener| {
            holders.remove(&game);
            opener.open(game, |tx| tx.delete_game())
        })
        .inspect(|()| self.storage.changes().forget_game(game))
    }

    fn create_in_tx(
        &self,
        new: &NewGame,
        opening: &Message,
        opener: &TxOpener,
    ) -> Result<GameId, EngineError> {
        opener.open_new_game(new, |game_id, tx| {
            tx.put_dock(&DockRecord::default())?;
            let message_id = tx.insert_message(opening)?;
            tx.insert_swipe(
                SwipeAt {
                    message: message_id,
                    swipe: 0,
                },
                &opening.swipes[0],
            )?;
            Ok(game_id)
        })
    }

    fn dock_of(&self, game: GameId) -> Result<DockRecord, EngineError> {
        Ok(self.storage.docks().get(game)?.unwrap_or_default())
    }

    fn tail_or_not_accepted(
        &self,
        game: GameId,
        opener: &TxOpener,
    ) -> Result<Option<Tail>, NotAccepted> {
        self.storage
            .stories()
            .tail(game)
            .map_err(|error| self.unreadable(game, Record::Story, &error, opener))
    }

    /// A record the request needs that the engine has no row for: the failure line
    /// names it.
    fn missing(&self, game: GameId, record: Record, opener: &TxOpener) -> NotAccepted {
        self.not_accepted_with_failure(game, GenerationFailure::missing(record), opener)
    }

    /// A record the request needs that the engine cannot read: the line names it,
    /// and the disclosure holds the error behind it.
    fn unreadable(
        &self,
        game: GameId,
        record: Record,
        error: &EngineError,
        opener: &TxOpener,
    ) -> NotAccepted {
        self.not_accepted_with_failure(game, GenerationFailure::unreadable(record, error), opener)
    }

    /// The write the request asked for failed: the line is the design's save line,
    /// and the disclosure holds the error behind it.
    fn save_failed(&self, game: GameId, error: &EngineError, opener: &TxOpener) -> NotAccepted {
        self.not_accepted_with_failure(game, GenerationFailure::save_failed(error), opener)
    }

    /// The one not-accepted path of a request that cannot run: the Game's failure line
    /// and the answer come from the same failure.
    fn not_accepted_with_failure(
        &self,
        game: GameId,
        failure: GenerationFailure,
        opener: &TxOpener,
    ) -> NotAccepted {
        if let Err(error) = opener.open(game, |tx| tx.set_game_failure(Some(&failure))) {
            tracing::error!("accept: writing the failure line failed: {error}");
        }
        NotAccepted::Failed(failure)
    }

    /// The Game row, the Persona and the start state that the plan names: the
    /// records a Job reads, checked before the Game turns busy.
    fn require_records(
        &self,
        game: GameId,
        plan: &JobPlan,
        opener: &TxOpener,
    ) -> Result<(), NotAccepted> {
        let row = match self.storage.get_game(game.0) {
            Ok(Some(row)) => row,
            Ok(None) => return Err(self.missing(game, Record::Game, opener)),
            Err(error) => return Err(self.unreadable(game, Record::Game, &error, opener)),
        };
        if let Err(error) = self.storage.require_persona(&row.persona_key) {
            return Err(self.unreadable(game, Record::Persona, &error, opener));
        }

        let Some(order) = &plan.first else {
            return Ok(());
        };
        match self.storage.stories().swipe_state(game, order.start) {
            Ok(Some(_)) => Ok(()),
            Ok(None) => Err(self.missing(game, Record::StartState, opener)),
            Err(error) => Err(self.unreadable(game, Record::StartState, &error, opener)),
        }
    }

    fn story_end_or_not_accepted(
        &self,
        game: GameId,
        opener: &TxOpener,
    ) -> Result<StoryEnd, NotAccepted> {
        let tail = self.tail_or_not_accepted(game, opener)?;
        let has_text = self.story_has_text(game, opener)?;
        Ok(StoryEnd { tail, has_text })
    }

    /// The end of a request that changes the story under a Game with no running
    /// Job: the failure line goes, and the dock holds `record`.
    fn clear_failure_and_put_dock(tx: &mut Tx<'_>, record: &DockRecord) -> Result<(), EngineError> {
        tx.set_game_failure(None)?;
        tx.put_dock(record)
    }

    fn accept_in_tx(&self, game: GameId, opener: &TxOpener) -> Result<(), NotAccepted> {
        opener
            .open(game, |tx| tx.set_game_failure(None))
            .map_err(|error| self.save_failed(game, &error, opener))
    }

    /// `/options` needs a story with text to work from, which is a question about
    /// every Message.
    fn story_has_text(&self, game: GameId, opener: &TxOpener) -> Result<bool, NotAccepted> {
        self.storage
            .stories()
            .has_text(game)
            .map_err(|error| self.unreadable(game, Record::Story, &error, opener))
    }

    /// The World and the Persona are read before the transaction opens, because the transaction
    /// holds the backend mutex.
    fn opening_message(&self, new: &NewGame) -> Result<Message, EngineError> {
        let world_with_map = self.storage.require_world(&new.world_key)?;
        let persona = self.storage.require_persona(&new.persona_key)?;
        let characters: HashMap<String, CharacterCard> = self
            .storage
            .list_characters(world_with_map.world_id)?
            .into_iter()
            .map(|card| (card.id.clone(), card))
            .collect();
        let world = &world_with_map.world_card;
        let state = SwipeState::starting(
            world.default_scenario(),
            &characters,
            &world.starting_room_id(),
        );
        Ok(crate::domain::model::utils::opening::opening_message(
            world,
            &persona.sheet.name,
            state,
        ))
    }
}

/// The only write handle of a running Job: only an accept makes one, and every
/// method checks the job id under the guard, so a late save of an ended Job lands
/// nowhere.
pub struct JobWriter {
    writer: GameWriter,
    game: GameId,
    id: u64,
}

impl JobWriter {
    pub fn game(&self) -> GameId {
        self.game
    }

    pub fn job_id(&self) -> u64 {
        self.id
    }

    /// One hold for this Job: the Job entry of the Game's holder must still hold
    /// this Job's id, and the transactions of `f` open under the same lock.
    fn hold<T>(
        &self,
        f: impl FnOnce(&mut JobEntry, &TxOpener) -> Result<T, JobFailure>,
    ) -> Result<T, JobFailure> {
        let (game, id) = (self.game, self.id);
        self.writer.hold(
            |holders, opener| match holders.get_mut(&game).and_then(Holder::job_mut) {
                Some(entry) if entry.id == id => f(entry, opener),
                _ => Err(JobFailure::EntryMoved),
            },
        )
    }

    fn in_tx<T>(
        opener: &TxOpener,
        game: GameId,
        f: impl FnOnce(&mut Tx<'_>) -> Result<T, EngineError>,
    ) -> Result<T, JobFailure> {
        opener
            .open(game, f)
            .map_err(|error| GenerationFailure::save_failed(&error).into())
    }

    pub fn save_input(&self, text: &str, start: &SwipeState) -> Result<SwipeAt, JobFailure> {
        self.hold(|_entry, opener| {
            let swipe = Swipe::new(text, Maker::Player, start.clone());
            let message = Message::new(MessageType::Input, MessageKind::TypedInput, swipe.clone());

            Self::in_tx(opener, self.game, |tx| {
                let message_id = tx.insert_message(&message)?;
                let at = SwipeAt {
                    message: message_id,
                    swipe: 0,
                };
                tx.insert_swipe(at, &swipe)?;
                Ok(at)
            })
        })
    }

    pub fn save_text(
        &self,
        order: &GenerationOrder,
        text: String,
        start: &SwipeState,
        tail: &Tail,
    ) -> Result<SavedText, JobFailure> {
        self.hold(|_entry, opener| {
            let empties_the_dock = order.kind().saves_a_narration();
            let NewSwipe {
                message_type,
                kind,
                swipe,
            } = order.write(text, start);
            let at = match order.place {
                Place::NewMessage => {
                    let message = Message::new(message_type, kind, swipe.clone());
                    self.write_new_message(&message, &swipe, empties_the_dock, opener)?
                }
                Place::NewSwipeOnLast => {
                    self.write_new_swipe(tail, &swipe, empties_the_dock, opener)?
                }
            };
            Ok(SavedText { at })
        })
    }

    /// Takes `saved` by value, so one Swipe's state is written at most twice.
    pub fn save_state_update(&self, saved: SavedText, state: SwipeState) -> Result<(), JobFailure> {
        self.hold(|_entry, opener| {
            opener
                .open(self.game, |tx| tx.set_swipe_state(saved.at, &state))
                .map_err(|error| GenerationFailure::contradiction(error.to_string()).into())
        })
    }

    pub fn save_failure(&self, failure: GenerationFailure) -> Result<(), JobFailure> {
        self.hold(|_entry, opener| {
            Self::in_tx(opener, self.game, |tx| tx.set_game_failure(Some(&failure)))
        })
    }

    /// A failed run writes the Options failure only: the set the dock holds stays, and after a
    /// Narration the text save already emptied it.
    pub fn save_options(&self, result: OptionsResult) -> Result<(), JobFailure> {
        self.hold(|_entry, opener| match result {
            OptionsResult::Offered(offered) => Self::in_tx(opener, self.game, |tx| {
                tx.put_dock(&DockRecord {
                    offered,
                    failure: None,
                })
            }),
            OptionsResult::Unavailable { reason, raw } => Self::in_tx(opener, self.game, |tx| {
                tx.put_options_failure(&OptionsFailure { reason, raw })
            }),
        })
    }

    /// The step name of the status line: memory only, and its only writer.
    pub fn step(&self, step: Step) -> Result<(), JobFailure> {
        self.hold(|entry, _opener| {
            entry.step = step;
            Ok(())
        })
    }

    fn write_new_message(
        &self,
        message: &Message,
        swipe: &Swipe,
        empties_the_dock: bool,
        opener: &TxOpener,
    ) -> Result<SwipeAt, JobFailure> {
        Self::in_tx(opener, self.game, |tx| {
            let message_id = tx.insert_message(message)?;
            let at = SwipeAt {
                message: message_id,
                swipe: 0,
            };
            tx.insert_swipe(at, swipe)?;
            if empties_the_dock {
                tx.put_dock(&DockRecord::default())?;
            }
            Ok(at)
        })
    }

    fn write_new_swipe(
        &self,
        tail: &Tail,
        swipe: &Swipe,
        empties_the_dock: bool,
        opener: &TxOpener,
    ) -> Result<SwipeAt, JobFailure> {
        let at = SwipeAt {
            message: tail.last.id,
            swipe: tail.last.swipes.len(),
        };
        Self::in_tx(opener, self.game, |tx| {
            tx.insert_swipe(at, swipe)?;
            tx.set_active_swipe(at)?;
            if empties_the_dock {
                tx.put_dock(&DockRecord::default())?;
            }
            Ok(())
        })?;
        Ok(at)
    }
}

impl Drop for JobWriter {
    /// The entry goes only while it still holds this Job's id: a stop or a reset
    /// already freed it. A plain Job's holder goes with its entry; an Autoplay
    /// run's holder stays, without a Job.
    fn drop(&mut self) {
        let (game, id) = (self.game, self.id);
        self.writer.hold(|holders, _opener| {
            let ours = holders
                .get(&game)
                .and_then(Holder::job)
                .is_some_and(|entry| entry.id == id);
            if !ours {
                return;
            }
            match holders.get_mut(&game) {
                Some(Holder::Autoplay { job, .. }) => *job = None,
                _ => {
                    holders.remove(&game);
                }
            }
        });
    }
}
