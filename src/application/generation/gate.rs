//! [DOC: docs/diataxis/reference/game_flow.md]
//! GameWriter and JobWriter — the writer rule and the guard: the one registry of job entries, and the save units a Job writes through.

use std::collections::HashMap;
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::{Arc, Mutex};

use crate::adapters::driven::storage::{Storage, Tx, TxToken};
use crate::domain::model::character::NpcCard;
use crate::domain::model::dock::{DockRecord, Notice};
use crate::domain::model::game::{GameId, NewGame};
use crate::domain::model::agent::Kind;
use crate::domain::model::generation::{
    GenerationOrder, JobPlan, JobRequest, NewSwipe, PlanRefusal, Place, StoryEnd,
};
use crate::domain::model::message::{Maker, Message, Swipe, SwipeAt};
use crate::domain::model::state::generation_status::{GenerationFailure, Record, Step};
use crate::domain::model::state::message_types::{MessageKind, MessageType};
use crate::domain::model::state::swipe_state::SwipeState;
use crate::domain::model::tail::Tail;
use crate::error::EngineError;

#[derive(Debug, Clone, PartialEq)]
pub enum Refusal {
    /// The Game is busy: `200` "Still thinking...".
    StillThinking,
    /// The request itself is wrong: `400`, with the reason line.
    BadRequest(String),
    /// The request could not run: `500`, carrying the failure's own line. The
    /// accept wrote the Game's failure line from the same failure, so the answer
    /// and the status name the same thing. The raw text stays in the disclosure.
    Failed(GenerationFailure),
}

/// Why a step or a save unit of a Job did not finish.
#[derive(Debug)]
pub enum JobFailure {
    /// The Game's job entry no longer holds this Job's id: a stop or a reset
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
    /// The run wrote no set: the dock shows a notice with the reason, and the raw
    /// text behind its disclosure.
    Unavailable { reason: String, raw: String },
}

/// No state: `save_state_update` consumes it, so a Swipe's state is written at
/// most twice, the pre-analysis state and then the analysed one. A user regen
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

/// The busy record in memory: the Job's own methods are its only writers.
struct JobEntry {
    /// The Job that owns the entry; a save unit whose id differs is refused.
    id: u64,
    step: Step,
}

/// The one registry of job entries: a private `Mutex` behind a type whose only
/// method is `hold`, so no caller takes the lock without a unit opener in hand.
struct JobRegistry {
    entries: Mutex<HashMap<GameId, JobEntry>>,
}

impl JobRegistry {
    fn new() -> Self {
        Self {
            entries: Mutex::new(HashMap::new()),
        }
    }

    /// A writer's check and its units happen in one critical section, and the lock
    /// order stays the guard, then the backend mutex, then the connection mutex.
    fn hold<T>(
        &self,
        storage: &Arc<Storage>,
        f: impl FnOnce(&mut HashMap<GameId, JobEntry>, &Units) -> T,
    ) -> T {
        let units = Units {
            storage: Arc::clone(storage),
            token: TxToken::new(),
        };
        let mut entries = self
            .entries
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner());
        f(&mut entries, &units)
    }
}

/// The unit opener of one hold. Only `JobRegistry::hold` makes one, so every unit
/// of record writes opens inside a hold; a reset opens two in the same hold.
struct Units {
    storage: Arc<Storage>,
    token: TxToken,
}

impl Units {
    fn open<T>(
        &self,
        game: GameId,
        f: impl FnOnce(&mut Tx<'_>) -> Result<T, EngineError>,
    ) -> Result<T, EngineError> {
        self.storage.transaction(game, self.token.clone(), f)
    }

    /// The create unit: the Game row goes first, and its id reaches the writes
    /// that follow it in the same unit.
    fn open_new_game<T>(
        &self,
        new: &NewGame,
        f: impl FnOnce(GameId, &mut Tx<'_>) -> Result<T, EngineError>,
    ) -> Result<T, EngineError> {
        self.storage
            .transaction_new_game(new, self.token.clone(), f)
    }
}

/// The only code that opens a unit of record writes, and the owner of the job
/// entries. One registry serves all Games, and a Job keeps the Game it started
/// on.
#[derive(Clone)]
pub struct GameWriter {
    storage: Arc<Storage>,
    entries: Arc<JobRegistry>,
    next_job_id: Arc<AtomicU64>,
}

impl GameWriter {
    pub fn new(storage: Arc<Storage>) -> Self {
        Self {
            storage,
            entries: Arc::new(JobRegistry::new()),
            next_job_id: Arc::new(AtomicU64::new(0)),
        }
    }

    fn next_job_id(&self) -> u64 {
        self.next_job_id
            .fetch_add(1, Ordering::SeqCst)
            .wrapping_add(1)
    }

    fn hold<T>(&self, f: impl FnOnce(&mut HashMap<GameId, JobEntry>, &Units) -> T) -> T {
        self.entries.hold(&self.storage, f)
    }

    /// The busy rule: a Game with a job entry answers "Still thinking..." before
    /// any other check.
    fn require_idle(entries: &HashMap<GameId, JobEntry>, game: GameId) -> Result<(), Refusal> {
        if entries.contains_key(&game) {
            return Err(Refusal::StillThinking);
        }
        Ok(())
    }

    pub fn accept(
        &self,
        game: GameId,
        request: JobRequest,
    ) -> Result<(JobWriter, JobPlan), Refusal> {
        let job_id = self.next_job_id();
        self.hold(|entries, units| {
            Self::require_idle(entries, game)?;
            let story = self.story_end_or_refuse(game, units)?;
            let plan = JobPlan::for_request(&request, story).map_err(|refusal| match refusal {
                PlanRefusal::BadRequest(line) => Refusal::BadRequest(line),
                PlanRefusal::Missing(record) => self.missing(game, record, units),
            })?;
            self.require_records(game, &plan, units)?;
            let step = plan.first_step();
            // The unit lands before the entry: a refused accept makes no entry, so a
            // Game is never busy for a Job that never started.
            self.accept_unit(game, units)?;
            entries.insert(game, JobEntry { id: job_id, step });
            Ok((
                JobWriter {
                    writer: self.clone(),
                    game,
                    id: job_id,
                },
                plan,
            ))
        })
    }

    pub fn busy_step(&self, game: GameId) -> Option<Step> {
        self.hold(|entries, _units| entries.get(&game).map(|entry| entry.step.clone()))
    }

    pub fn stop(&self, game: GameId) -> Result<(), EngineError> {
        self.hold(|entries, units| {
            entries.remove(&game);
            let record = DockRecord {
                offered: self.dock_of(game)?.offered,
                notice: None,
            };
            units.open(game, |tx| Self::clear_failure_and_put_dock(tx, &record))
        })
    }

    pub fn switch_swipe(&self, game: GameId, message: u64, index: usize) -> Result<(), Refusal> {
        self.hold(|entries, units| {
            Self::require_idle(entries, game)?;
            let tail = self.tail_or_refuse(game, units)?;
            let last =
                tail.ok_or_else(|| Refusal::BadRequest("No message to switch".to_string()))?;
            if last.last.id != message {
                return Err(Refusal::BadRequest(
                    "Only the last message can be swiped".to_string(),
                ));
            }
            if index >= last.last.swipes.len() {
                return Err(Refusal::BadRequest("Swipe index out of bounds".to_string()));
            }
            let at = SwipeAt {
                message,
                swipe: index,
            };
            units
                .open(game, |tx| {
                    tx.set_active_swipe(at)?;
                    Self::clear_failure_and_put_dock(tx, &DockRecord::default())
                })
                .map_err(|error| self.save_failed(game, &error, units))
        })
    }

    pub fn edit(&self, game: GameId, message: u64, text: &str) -> Result<(), Refusal> {
        self.hold(|entries, units| {
            Self::require_idle(entries, game)?;
            let at = match self.storage.stories().active_swipe(game, message) {
                Ok(Some(at)) => at,
                Ok(None) => return Err(Refusal::BadRequest("Message not found".to_string())),
                Err(error) => return Err(self.unreadable(game, Record::Message, &error, units)),
            };
            units
                .open(game, |tx| tx.set_swipe_text(at, text))
                .map_err(|error| self.save_failed(game, &error, units))
        })
    }

    pub fn delete_last(&self, game: GameId) -> Result<(), Refusal> {
        self.hold(|entries, units| {
            Self::require_idle(entries, game)?;
            let tail = self.tail_or_refuse(game, units)?;
            let last =
                tail.ok_or_else(|| Refusal::BadRequest("No message to delete".to_string()))?;
            if !last.can_delete() {
                return Err(Refusal::BadRequest(
                    "The story keeps its last message".to_string(),
                ));
            }
            units
                .open(game, |tx| {
                    tx.delete_message(last.last.id)?;
                    Self::clear_failure_and_put_dock(tx, &DockRecord::default())
                })
                .map_err(|error| self.save_failed(game, &error, units))
        })
    }

    pub fn create(&self, new: &NewGame) -> Result<GameId, EngineError> {
        let opening = self.opening_message(new)?;
        self.hold(|_entries, units| self.create_unit(new, &opening, units))
    }

    /// Both units run in one hold, and the old Game's Job entry ends under the same
    /// guard, so its next save unit or step is refused by the id check; a running
    /// model call ends on its own. The create unit runs first: a failure there
    /// leaves the old Game, its entry and the pointer as they were.
    pub fn replace(&self, old: GameId, new: &NewGame) -> Result<GameId, EngineError> {
        let opening = self.opening_message(new)?;
        self.hold(|entries, units| {
            let new_id = self.create_unit(new, &opening, units)?;
            entries.remove(&old);
            units.open(old, |tx| tx.delete_game())?;
            Ok(new_id)
        })
    }

    /// One hold: the Game's job entry ends under the same guard as its rows, so the
    /// Job's next save unit or step is refused by the id check before its records go.
    pub fn delete(&self, game: GameId) -> Result<(), EngineError> {
        self.hold(|entries, units| {
            entries.remove(&game);
            units.open(game, |tx| tx.delete_game())
        })
    }

    fn create_unit(
        &self,
        new: &NewGame,
        opening: &Message,
        units: &Units,
    ) -> Result<GameId, EngineError> {
        units.open_new_game(new, |game_id, tx| {
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

    fn tail_or_refuse(&self, game: GameId, units: &Units) -> Result<Option<Tail>, Refusal> {
        self.storage
            .stories()
            .tail(game)
            .map_err(|error| self.unreadable(game, Record::Story, &error, units))
    }

    /// A record the request needs that the engine has no row for: the failure line
    /// names it.
    fn missing(&self, game: GameId, record: Record, units: &Units) -> Refusal {
        self.refuse(game, GenerationFailure::missing(record), units)
    }

    /// A record the request needs that the engine cannot read: the line names it,
    /// and the disclosure holds the error behind it.
    fn unreadable(
        &self,
        game: GameId,
        record: Record,
        error: &EngineError,
        units: &Units,
    ) -> Refusal {
        self.refuse(game, GenerationFailure::unreadable(record, error), units)
    }

    /// The write the request asked for failed: the line is the design's save line,
    /// and the disclosure holds the error behind it.
    fn save_failed(&self, game: GameId, error: &EngineError, units: &Units) -> Refusal {
        self.refuse(game, GenerationFailure::save_failed(error), units)
    }

    /// The one refusal path of a request that cannot run: the Game's failure line
    /// and the answer come from the same failure.
    fn refuse(&self, game: GameId, failure: GenerationFailure, units: &Units) -> Refusal {
        if let Err(error) = units.open(game, |tx| tx.set_game_failure(Some(&failure))) {
            tracing::error!("accept: writing the failure line failed: {error}");
        }
        Refusal::Failed(failure)
    }

    /// The Game row, the Persona and the start state that the plan names: the
    /// records a Job reads, checked before the Game turns busy.
    fn require_records(&self, game: GameId, plan: &JobPlan, units: &Units) -> Result<(), Refusal> {
        let row = match self.storage.get_game(game.0) {
            Ok(Some(row)) => row,
            Ok(None) => return Err(self.missing(game, Record::Game, units)),
            Err(error) => return Err(self.unreadable(game, Record::Game, &error, units)),
        };
        if let Err(error) = self.storage.require_persona(&row.persona_key) {
            return Err(self.unreadable(game, Record::Persona, &error, units));
        }

        let Some(order) = &plan.first else {
            return Ok(());
        };
        match self.storage.stories().swipe_state(game, order.start) {
            Ok(Some(_)) => Ok(()),
            Ok(None) => Err(self.missing(game, Record::StartState, units)),
            Err(error) => Err(self.unreadable(game, Record::StartState, &error, units)),
        }
    }

    fn story_end_or_refuse(&self, game: GameId, units: &Units) -> Result<StoryEnd, Refusal> {
        let tail = self.tail_or_refuse(game, units)?;
        let has_text = self.story_has_text(game, units)?;
        Ok(StoryEnd { tail, has_text })
    }

    /// The end of a request that changes the story under a Game with no running
    /// Job: the failure line goes, and the dock holds `record`.
    fn clear_failure_and_put_dock(tx: &mut Tx<'_>, record: &DockRecord) -> Result<(), EngineError> {
        tx.set_game_failure(None)?;
        tx.put_dock(record)
    }

    fn accept_unit(&self, game: GameId, units: &Units) -> Result<(), Refusal> {
        units
            .open(game, |tx| tx.set_game_failure(None))
            .map_err(|error| self.save_failed(game, &error, units))
    }

    /// `/options` needs a story with text to work from, which is a question about
    /// every Message.
    fn story_has_text(&self, game: GameId, units: &Units) -> Result<bool, Refusal> {
        self.storage
            .stories()
            .has_text(game)
            .map_err(|error| self.unreadable(game, Record::Story, &error, units))
    }

    /// The World and the Persona are read before the unit opens, because the unit
    /// holds the backend mutex.
    fn opening_message(&self, new: &NewGame) -> Result<Message, EngineError> {
        let world_with_map = self.storage.require_world(&new.world_key)?;
        let persona = self.storage.require_persona(&new.persona_key)?;
        let characters: HashMap<String, NpcCard> = self
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

/// The only write handle of a running Job: only `accept` makes one, and every
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

    /// One hold for this Job: the Game's entry must still hold this Job's id, and
    /// the units of `f` open under the same lock.
    fn hold<T>(
        &self,
        f: impl FnOnce(&mut JobEntry, &Units) -> Result<T, JobFailure>,
    ) -> Result<T, JobFailure> {
        let (game, id) = (self.game, self.id);
        self.writer
            .hold(|entries, units| match entries.get_mut(&game) {
                Some(entry) if entry.id == id => f(entry, units),
                _ => Err(JobFailure::EntryMoved),
            })
    }

    fn unit<T>(
        units: &Units,
        game: GameId,
        f: impl FnOnce(&mut Tx<'_>) -> Result<T, EngineError>,
    ) -> Result<T, JobFailure> {
        units
            .open(game, f)
            .map_err(|error| GenerationFailure::save_failed(&error).into())
    }

    pub fn save_input(&self, text: &str, start: SwipeAt) -> Result<SwipeAt, JobFailure> {
        self.hold(|_entry, units| {
            let state = self.state_at(start)?;
            let swipe = Swipe::new(text, Maker::Player, state);
            let message = Message::new(MessageType::Input, MessageKind::TypedInput, swipe.clone());

            Self::unit(units, self.game, |tx| {
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
    ) -> Result<SavedText, JobFailure> {
        self.hold(|_entry, units| {
            let empties_the_dock = order.kind().saves_a_narration();
            let NewSwipe {
                message_type,
                kind,
                swipe,
            } = order.write(text, start);
            let at = match order.place {
                Place::NewMessage => {
                    let message = Message::new(message_type, kind, swipe.clone());
                    self.write_new_message(&message, &swipe, empties_the_dock, units)?
                }
                Place::NewSwipeOnLast => {
                    let tail = self.tail_of()?;
                    self.write_new_swipe(&tail, &swipe, empties_the_dock, units)?
                }
            };
            Ok(SavedText { at })
        })
    }

    /// Takes `saved` by value, so one Swipe's state is written at most twice.
    pub fn save_state_update(&self, saved: SavedText, state: SwipeState) -> Result<(), JobFailure> {
        self.hold(|_entry, units| {
            units
                .open(self.game, |tx| tx.set_swipe_state(saved.at, &state))
                .map_err(|error| GenerationFailure::contradiction(error.to_string()).into())
        })
    }

    pub fn save_failure(&self, failure: GenerationFailure) -> Result<(), JobFailure> {
        self.hold(|_entry, units| {
            Self::unit(units, self.game, |tx| tx.set_game_failure(Some(&failure)))
        })
    }

    /// `after_narration`: the Narration's text save already emptied the offered set,
    /// so the notice stands alone; an on-demand run keeps the set it found.
    pub fn save_options(
        &self,
        result: OptionsResult,
        after_narration: bool,
    ) -> Result<(), JobFailure> {
        self.hold(|_entry, units| {
            let record = match (result, after_narration) {
                (OptionsResult::Offered(offered), _) => DockRecord {
                    offered,
                    notice: None,
                },
                (OptionsResult::Unavailable { reason, raw }, true) => DockRecord {
                    offered: Vec::new(),
                    notice: Some(Notice { reason, raw }),
                },
                (OptionsResult::Unavailable { reason, raw }, false) => DockRecord {
                    offered: self.dock()?.offered,
                    notice: Some(Notice { reason, raw }),
                },
            };
            Self::unit(units, self.game, |tx| tx.put_dock(&record))
        })
    }

    /// The step name of the status line: memory only, and its only writer.
    pub fn step(&self, step: Step) -> Result<(), JobFailure> {
        self.hold(|entry, _units| {
            entry.step = step;
            Ok(())
        })
    }

    fn dock(&self) -> Result<DockRecord, JobFailure> {
        match self.writer.storage.docks().get(self.game) {
            Ok(record) => Ok(record.unwrap_or_default()),
            Err(error) => Err(GenerationFailure::unreadable(Record::Dock, &error).into()),
        }
    }

    fn state_at(&self, at: SwipeAt) -> Result<SwipeState, JobFailure> {
        match self.writer.storage.stories().swipe_state(self.game, at) {
            Ok(Some(state)) => Ok(state),
            Ok(None) => Err(GenerationFailure::missing(Record::StartState).into()),
            Err(error) => Err(GenerationFailure::unreadable(Record::StartState, &error).into()),
        }
    }

    fn tail_of(&self) -> Result<Tail, JobFailure> {
        match self.writer.storage.stories().tail(self.game) {
            Ok(Some(tail)) => Ok(tail),
            Ok(None) => Err(GenerationFailure::missing(Record::Story).into()),
            Err(error) => Err(GenerationFailure::unreadable(Record::Story, &error).into()),
        }
    }

    fn write_new_message(
        &self,
        message: &Message,
        swipe: &Swipe,
        empties_the_dock: bool,
        units: &Units,
    ) -> Result<SwipeAt, JobFailure> {
        Self::unit(units, self.game, |tx| {
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
        units: &Units,
    ) -> Result<SwipeAt, JobFailure> {
        let at = SwipeAt {
            message: tail.last.id,
            swipe: tail.last.swipes.len(),
        };
        Self::unit(units, self.game, |tx| {
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
    /// already freed it.
    fn drop(&mut self) {
        let (game, id) = (self.game, self.id);
        self.writer.hold(|entries, _units| {
            if entries.get(&game).is_some_and(|entry| entry.id == id) {
                entries.remove(&game);
            }
        });
    }
}
