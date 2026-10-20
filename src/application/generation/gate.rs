//! [DOC: docs/diataxis/reference/game_flow.md]
//! GameWriter and JobWriter — the writer rule and the guard: the one registry of job entries, and the save units a Job writes through.

use std::collections::HashMap;
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::{Arc, Mutex};

use crate::adapters::driven::storage::{Storage, Tx, TxToken};
use crate::domain::model::agent::Kind;
use crate::domain::model::character::NpcCard;
use crate::domain::model::dock::{DockRecord, Notice};
use crate::domain::model::game::{GameId, NewGame};
use crate::domain::model::generation::{GenerationOrder, Place};
use crate::domain::model::message::{Maker, Message, StoredInputs, Swipe, SwipeAt};
use crate::domain::model::state::generation_status::{GenerationFailure, GenerationFailureKind, Step};
use crate::domain::model::state::message_types::{MessageKind, MessageType};
use crate::domain::model::state::swipe_state::SwipeState;
use crate::domain::model::state::trigger_context::{EventRecord, TriggerRef};
use crate::domain::model::tail::Tail;
use crate::domain::model::utils::generation_record;
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

#[derive(Debug)]
pub enum Stopped {
    /// The Game's job entry no longer holds this Job's id: a stop or a reset
    /// ended the Job, so its late writes land nowhere.
    EntryMoved,
    /// The unit failed in storage: the Job writes the failure line and ends.
    SaveFailed(EngineError),
}

impl From<EngineError> for Stopped {
    fn from(error: EngineError) -> Self {
        Self::SaveFailed(error)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub enum JobRequest {
    /// Free text the player typed. Empty text is a Continue.
    Action(String),
    /// `/guide`: a steering direction that shapes the next Narration.
    Guide(String),
    /// `/impersonate`: the player dictates their own Persona's Message.
    Impersonate(Option<String>),
    /// `/options`: the Options run alone.
    Options,
    /// A Retry of the last Message, of the kind the last Message holds.
    Retry,
    /// A Retrigger of the Trigger the last Message holds.
    Retrigger,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum OptionsRun {
    /// The run follows the Job's last Narration, when the toggle is on.
    AfterNarration,
    /// The run is the whole Job: `/options`.
    OnDemand,
}

/// The order of one request: ids and texts only.
#[derive(Debug, Clone, PartialEq)]
pub struct JobPlan {
    /// The free text to save before the first generation, when the request carries
    /// any.
    pub input: Option<String>,
    /// The first generation. `None` for `/options`.
    pub first: Option<GenerationOrder>,
    pub options: OptionsRun,
}

impl JobPlan {
    /// The step the status shows from the accept, until the Job names its own: a
    /// Retry of an event Narration starts in the event step.
    pub(crate) fn first_step(&self) -> Step {
        match &self.first {
            Some(order) if order.kind == Kind::Event => Step::GeneratingEvent,
            Some(_) => Step::Narrating,
            None => Step::Options,
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
            let tail = self.tail_or_refuse(game, units)?;
            let mut plan = self.plan_for(game, &request, tail.as_ref(), units)?;
            self.require_job_data(game, &mut plan, tail.as_ref(), units)?;
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
            units.open(game, |tx| {
                tx.set_game_failure(None)?;
                tx.put_dock(&record)
            })
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
                    tx.put_dock(&DockRecord::default())?;
                    tx.set_game_failure(None)
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
                Err(error) => return Err(self.unreadable(game, "the message", &error, units)),
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
                    tx.put_dock(&DockRecord::default())?;
                    tx.set_game_failure(None)
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
            .map_err(|error| self.unreadable(game, "the story", &error, units))
    }

    /// A record the request needs that the engine has no row for: the failure line
    /// names it.
    fn missing(&self, game: GameId, name: &str, units: &Units) -> Refusal {
        self.refuse(game, GenerationFailure::missing_record(name, None), units)
    }

    /// A record the request needs that the engine cannot read: the line names it,
    /// and the disclosure holds the error behind it.
    fn unreadable(&self, game: GameId, name: &str, error: &EngineError, units: &Units) -> Refusal {
        self.refuse(
            game,
            GenerationFailure::missing_record(name, Some(error.to_string())),
            units,
        )
    }

    /// The write the request asked for failed: the line is the design's save line,
    /// and the disclosure holds the error behind it.
    fn save_failed(&self, game: GameId, error: &EngineError, units: &Units) -> Refusal {
        self.refuse(
            game,
            GenerationFailure::new(GenerationFailureKind::SaveFailed, error.to_string()),
            units,
        )
    }

    /// The one refusal path of a request that cannot run: the Game's failure line
    /// and the answer come from the same failure.
    fn refuse(&self, game: GameId, failure: GenerationFailure, units: &Units) -> Refusal {
        if let Err(error) = units.open(game, |tx| tx.set_game_failure(Some(&failure))) {
            tracing::error!("accept: writing the failure line failed: {error}");
        }
        Refusal::Failed(failure)
    }

    fn plan_for(
        &self,
        game: GameId,
        request: &JobRequest,
        tail: Option<&Tail>,
        units: &Units,
    ) -> Result<JobPlan, Refusal> {
        match request {
            JobRequest::Options => {
                if !self.story_has_text(game, units)? {
                    return Err(Refusal::BadRequest(
                        "No scene to generate options for yet".to_string(),
                    ));
                }
                Ok(JobPlan {
                    input: None,
                    first: None,
                    options: OptionsRun::OnDemand,
                })
            }
            JobRequest::Retry => {
                let order = tail
                    .map(Tail::redo)
                    .ok_or_else(|| Refusal::BadRequest("No input to retry".to_string()))?;
                Ok(JobPlan {
                    input: None,
                    first: Some(order),
                    options: OptionsRun::AfterNarration,
                })
            }
            JobRequest::Retrigger => {
                let order = tail.and_then(Tail::retrigger).ok_or_else(|| {
                    Refusal::BadRequest("No trigger context available".to_string())
                })?;
                Ok(JobPlan {
                    input: None,
                    first: Some(order),
                    options: OptionsRun::AfterNarration,
                })
            }
            JobRequest::Action(text) => {
                let start = self.require_start(tail, game, units)?;
                let input = (!text.is_empty()).then(|| text.clone());
                Ok(self.generation_plan(Kind::Narration, StoredInputs::default(), input, start))
            }
            JobRequest::Guide(guide) => {
                let start = self.require_start(tail, game, units)?;
                let inputs = StoredInputs {
                    guide_text: Some(guide.clone()),
                    ..StoredInputs::default()
                };
                Ok(self.generation_plan(Kind::Narration, inputs, None, start))
            }
            JobRequest::Impersonate(direction) => {
                let start = self.require_start(tail, game, units)?;
                let inputs = StoredInputs {
                    impersonated: true,
                    impersonate_direction: direction.clone(),
                    ..StoredInputs::default()
                };
                Ok(self.generation_plan(Kind::Impersonate, inputs, None, start))
            }
        }
    }

    fn generation_plan(
        &self,
        kind: Kind,
        inputs: StoredInputs,
        input: Option<String>,
        start: SwipeAt,
    ) -> JobPlan {
        JobPlan {
            input,
            first: Some(GenerationOrder {
                kind,
                place: Place::NewMessage,
                start,
                inputs,
                event: None,
            }),
            options: OptionsRun::AfterNarration,
        }
    }

    fn require_start(
        &self,
        tail: Option<&Tail>,
        game: GameId,
        units: &Units,
    ) -> Result<SwipeAt, Refusal> {
        tail.map(Tail::current_at)
            .ok_or_else(|| self.missing(game, "the story", units))
    }

    fn require_job_data(
        &self,
        game: GameId,
        plan: &mut JobPlan,
        tail: Option<&Tail>,
        units: &Units,
    ) -> Result<(), Refusal> {
        let row = match self.storage.get_game(game.0) {
            Ok(Some(row)) => row,
            Ok(None) => return Err(self.missing(game, "the game", units)),
            Err(error) => return Err(self.unreadable(game, "the game", &error, units)),
        };
        if let Err(error) = self.storage.require_persona(&row.persona_key) {
            return Err(self.unreadable(game, "the persona", &error, units));
        }

        let Some(order) = &mut plan.first else {
            return Ok(());
        };
        match self.storage.stories().swipe_state(game, order.start) {
            Ok(Some(_)) => {}
            Ok(None) => return Err(self.missing(game, "the start state", units)),
            Err(error) => return Err(self.unreadable(game, "the start state", &error, units)),
        }
        if order.kind == Kind::Impersonate && order.inputs.impersonate_direction.is_none() {
            return Err(self.missing(game, "the impersonate direction", units));
        }
        if order.kind == Kind::Event && order.event.is_none() {
            order.event = Some(self.event_record(game, tail, units)?);
        }
        Ok(())
    }

    fn event_record(
        &self,
        game: GameId,
        tail: Option<&Tail>,
        units: &Units,
    ) -> Result<EventRecord, Refusal> {
        let trigger = tail
            .and_then(Tail::current_state)
            .and_then(|state| state.trigger_context.clone())
            .ok_or_else(|| self.missing(game, "the trigger context", units))?;
        let direction = self.trigger_direction(game, &trigger, units)?;
        Ok(EventRecord {
            header_name: trigger.trigger_name.clone(),
            trigger,
            direction,
        })
    }

    fn trigger_direction(
        &self,
        game: GameId,
        trigger: &TriggerRef,
        units: &Units,
    ) -> Result<String, Refusal> {
        let bundle = self
            .storage
            .world_bundle_for(game.0)
            .map_err(|error| self.unreadable(game, "the World", &error, units))?;
        bundle
            .npcs
            .get(&trigger.npc_id)
            .and_then(|npc| npc.triggers.get(trigger.trigger_idx))
            .map(|trigger| trigger.narration.narration_prompt.clone())
            .ok_or_else(|| self.missing(game, "the trigger", units))
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
            .map_err(|error| self.unreadable(game, "the story", &error, units))
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
            &world_with_map.map,
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
        f: impl FnOnce(&mut JobEntry, &Units) -> Result<T, Stopped>,
    ) -> Result<T, Stopped> {
        let (game, id) = (self.game, self.id);
        self.writer
            .hold(|entries, units| match entries.get_mut(&game) {
                Some(entry) if entry.id == id => f(entry, units),
                _ => Err(Stopped::EntryMoved),
            })
    }

    fn unit<T>(
        units: &Units,
        game: GameId,
        f: impl FnOnce(&mut Tx<'_>) -> Result<T, EngineError>,
    ) -> Result<T, Stopped> {
        units.open(game, f).map_err(Stopped::from)
    }

    pub fn save_input(&self, text: &str, start: SwipeAt) -> Result<SwipeAt, Stopped> {
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
        pre: SwipeState,
    ) -> Result<SavedText, Stopped> {
        self.hold(|_entry, units| {
            let empties_the_dock = Self::empties_the_dock(order.kind);
            let at = match order.place {
                Place::NewMessage => {
                    let swipe = Self::text_swipe(order, text, pre, None);
                    let message = Message::new(
                        generation_record::message_type(order.kind),
                        Self::message_kind_of(order)?,
                        swipe.clone(),
                    );
                    self.write_new_message(&message, &swipe, empties_the_dock, units)?
                }
                Place::NewSwipeOnLast => {
                    let tail = self.tail_of()?;
                    let room_report = match tail.count {
                        1 => self.opening_room_report(&pre.movement.current_room_id),
                        _ => None,
                    };
                    let swipe = Self::text_swipe(order, text, pre, room_report);
                    self.write_new_swipe(&tail, &swipe, empties_the_dock, units)?
                }
            };
            Ok(SavedText { at })
        })
    }

    /// Takes `saved` by value, so one Swipe's state is written at most twice.
    pub fn save_state_update(
        &self,
        saved: SavedText,
        state: SwipeState,
        room_report: Option<String>,
    ) -> Result<(), Stopped> {
        self.hold(|_entry, units| {
            Self::unit(units, self.game, |tx| {
                tx.set_swipe_state(saved.at, &state, room_report.as_deref())
            })
        })
    }

    pub fn save_failure(&self, failure: GenerationFailure) -> Result<(), Stopped> {
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
    ) -> Result<(), Stopped> {
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
    pub fn step(&self, step: Step) -> Result<(), Stopped> {
        self.hold(|entry, _units| {
            entry.step = step;
            Ok(())
        })
    }

    fn dock(&self) -> Result<DockRecord, Stopped> {
        Ok(self
            .writer
            .storage
            .docks()
            .get(self.game)
            .map_err(Stopped::from)?
            .unwrap_or_default())
    }

    fn state_at(&self, at: SwipeAt) -> Result<SwipeState, Stopped> {
        match self.writer.storage.stories().swipe_state(self.game, at) {
            Ok(Some(state)) => Ok(state),
            Ok(None) => Err(Stopped::SaveFailed(EngineError::Validation(
                "the start state is missing".to_string(),
            ))),
            Err(error) => Err(Stopped::SaveFailed(error)),
        }
    }

    fn tail_of(&self) -> Result<Tail, Stopped> {
        match self.writer.storage.stories().tail(self.game) {
            Ok(Some(tail)) => Ok(tail),
            Ok(None) => Err(Stopped::SaveFailed(EngineError::Validation(
                "the story is empty".to_string(),
            ))),
            Err(error) => Err(Stopped::SaveFailed(error)),
        }
    }

    fn opening_room_report(&self, room_id: &str) -> Option<String> {
        let world_key = self
            .writer
            .storage
            .get_game(self.game.0)
            .ok()
            .flatten()?
            .world_key;
        let world = self.writer.storage.require_world(&world_key).ok()?;
        Some(
            world
                .map
                .get_room_by_id(room_id)
                .map(|room| room.name.clone())
                .unwrap_or_else(|| room_id.to_string()),
        )
    }

    fn write_new_message(
        &self,
        message: &Message,
        swipe: &Swipe,
        empties_the_dock: bool,
        units: &Units,
    ) -> Result<SwipeAt, Stopped> {
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
    ) -> Result<SwipeAt, Stopped> {
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

    fn text_swipe(
        order: &GenerationOrder,
        text: String,
        pre: SwipeState,
        room_report: Option<String>,
    ) -> Swipe {
        Swipe {
            room_report,
            event_header: order
                .event
                .as_ref()
                .map(|record| record.header_name.clone()),
            impersonated: order.inputs.impersonated,
            guide_text: order.inputs.guide_text.clone(),
            impersonate_direction: order.inputs.impersonate_direction.clone(),
            ..Swipe::new(text, generation_record::maker(order.kind), pre)
        }
    }

    fn message_kind_of(order: &GenerationOrder) -> Result<MessageKind, Stopped> {
        generation_record::message_kind(order.kind, order.event.as_ref()).ok_or_else(|| {
            Stopped::SaveFailed(EngineError::Validation(
                "the event record is missing".to_string(),
            ))
        })
    }

    fn empties_the_dock(kind: Kind) -> bool {
        matches!(kind, Kind::Narration | Kind::Event)
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
