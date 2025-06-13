//! Example player implementation, which plays back a [`Sequence`]
//! via the [`phonic`](https://crates.io/crates/phonic) crate.

use std::{
    collections::HashMap,
    sync::{
        atomic::{AtomicUsize, Ordering},
        Arc, RwLock,
    },
    time::Duration,
};

use crossbeam_channel::Sender;

use phonic::{
    utils::speed_from_note, DefaultOutputDevice, Error, FilePlaybackOptions, OutputDevice,
    PlaybackId, PlaybackStatusContext, PlaybackStatusEvent, Player as PhonicPlayer,
    PreloadedFileSource,
};

use crate::{
    time::{SampleTimeBase, SampleTimeDisplay},
    BeatTimeBase, Event, InstrumentId, Note, PatternEvent, SampleTime, Sequence,
};

// -------------------------------------------------------------------------------------------------

/// Preload time of the player's `run_until` function. Should be big enough to ensure that events
/// are scheduled ahead of playback time, but small enough to avoid too much latency.
/// NB: real audio/event latency is twice the amount of the preload!
#[cfg(debug_assertions)]
const PLAYBACK_PRELOAD_SECONDS: f64 = 1.0;
#[cfg(not(debug_assertions))]
const PLAYBACK_PRELOAD_SECONDS: f64 = 0.5;

// -------------------------------------------------------------------------------------------------

/// Preloads a set of sample files and stores them as [`PreloadedFileSource`] for later use.
///
/// When files are accessed, the already cached file sources are cloned, which avoids loading
/// and decoding the files again while playback. Cloned [`PreloadedFileSource`] are using a
/// shared Buffer, so cloning is very cheap.
#[derive(Default)]
pub struct SamplePool {
    pool: RwLock<HashMap<InstrumentId, PreloadedFileSource>>,
}

impl SamplePool {
    /// Create a new empty sample pool.
    pub fn new() -> Self {
        Self {
            pool: RwLock::new(HashMap::new()),
        }
    }

    /// Fetch a clone of a preloaded sample with the given playback options.
    ///
    /// ### Errors
    /// Returns an error if the instrument id is unknown.
    ///
    /// ### Panics
    /// Panics if the sample pool can not be accessed
    pub fn get_sample(
        &self,
        id: InstrumentId,
        playback_options: FilePlaybackOptions,
        playback_sample_rate: u32,
    ) -> Result<PreloadedFileSource, Error> {
        let pool = self.pool.read().expect("Failed to access sample pool");
        if let Some(sample) = pool.get(&id) {
            sample.clone(playback_options, playback_sample_rate)
        } else {
            Err(Error::MediaFileNotFound)
        }
    }

    /// Loads a sample file as [`PreloadedFileSource`] and return its unique id.
    /// A copy of this sample can then later on be fetched with `get_sample` with the returned id.
    ///
    /// ### Errors
    /// Returns an error if the sample file could not be loaded.
    ///
    /// ### Panics
    /// Panics if the sample pool can not be accessed
    pub fn load_sample(&self, file_path: &str) -> Result<InstrumentId, Error> {
        let sample =
            PreloadedFileSource::new(file_path, None, FilePlaybackOptions::default(), 44100)?;
        let id = Self::unique_id();
        let mut pool = self.pool.write().expect("Failed to access sample pool");
        pool.insert(id, sample);
        Ok(id)
    }

    /// Generate a new unique instrument id.
    fn unique_id() -> InstrumentId {
        static ID: AtomicUsize = AtomicUsize::new(0);
        InstrumentId::from(ID.fetch_add(1, Ordering::Relaxed))
    }
}

// -------------------------------------------------------------------------------------------------

/// Behaviour when playing a new note on the same voice channel.
#[derive(Copy, Clone, Debug, Default, PartialEq)]
pub enum NewNoteAction {
    /// Continue playing the old note and start a new one.
    #[default]
    Continue,
    /// Stop the playing note before starting a new one.
    Stop,
    /// Stop the playing note before with the given fade-out duration
    Off(Option<Duration>),
}

// -------------------------------------------------------------------------------------------------

/// Context, passed along serialized when triggering new notes from the sample player.   
#[derive(Clone)]
pub struct SamplePlaybackContext {
    pub pattern_index: Option<usize>,
    pub voice_index: Option<usize>,
}

impl SamplePlaybackContext {
    pub fn from_event(context: Option<PlaybackStatusContext>) -> Self {
        if let Some(context) = context {
            if let Some(context) = context.downcast_ref::<SamplePlaybackContext>() {
                return context.clone();
            }
        }
        SamplePlaybackContext {
            pattern_index: None,
            voice_index: None,
        }
    }
}

// -------------------------------------------------------------------------------------------------

/// An simple example player implementation, which plays back a `Sequence` via the `phonic` crate
/// using the default audio output device using plain samples loaded from a file as instruments.
///
/// Works on an existing sample pool, which can be used outside of the player as well.
pub struct SamplePlayer {
    player: PhonicPlayer,
    sample_pool: Arc<RwLock<SamplePool>>,
    playing_notes: Vec<HashMap<usize, (PlaybackId, Note)>>,
    new_note_action: NewNoteAction,
    sample_root_note: Note,
    playback_pos_emit_rate: Duration,
    show_events: bool,
    playback_sample_time: SampleTime,
    emitted_sample_time: SampleTime,
}

impl SamplePlayer {
    /// Create a new sample player.
    ///
    /// # Errors
    ///
    /// returns an error if the player could not be created.
    pub fn new(
        sample_pool: Arc<RwLock<SamplePool>>,
        playback_status_sender: Option<Sender<PlaybackStatusEvent>>,
    ) -> Result<Self, Box<dyn std::error::Error>> {
        // create player
        let audio_output = DefaultOutputDevice::open()?;
        let player = PhonicPlayer::new(audio_output.sink(), playback_status_sender);
        let playing_notes = Vec::new();
        let new_note_action = NewNoteAction::default();
        let sample_root_note = Note::C5;
        let playback_pos_emit_rate = Duration::from_secs(1);
        let show_events = false;
        let playback_sample_time = player.output_sample_frame_position();
        let emitted_sample_time = 0;
        Ok(Self {
            player,
            sample_pool,
            playing_notes,
            new_note_action,
            sample_root_note,
            playback_pos_emit_rate,
            show_events,
            playback_sample_time,
            emitted_sample_time,
        })
    }

    /// Access to our file player.
    pub fn file_player(&self) -> &PhonicPlayer {
        &self.player
    }
    pub fn file_player_mut(&mut self) -> &mut PhonicPlayer {
        &mut self.player
    }

    /// true when events are dumped to stdout while playing them.
    pub fn show_events(&self) -> bool {
        self.show_events
    }
    /// by default false: set to true to dump events to stdout while playing them.
    pub fn set_show_events(&mut self, show: bool) {
        self.show_events = show;
    }

    /// playback pos emit rate of triggered files. by default one second.
    pub fn playback_pos_emit_rate(&self) -> Duration {
        self.playback_pos_emit_rate
    }
    pub fn set_playback_pos_emit_rate(&mut self, emit_rate: Duration) {
        self.playback_pos_emit_rate = emit_rate;
    }

    /// get current new note action behaviour.
    pub fn new_note_action(&self) -> NewNoteAction {
        self.new_note_action
    }
    // set a new new note action behaviour.
    pub fn set_new_note_action(&mut self, action: NewNoteAction) {
        self.new_note_action = action;
    }

    /// get root note used when converting event note values to sample playback speed.
    pub fn sample_root_note(&self) -> Note {
        self.sample_root_note
    }
    // set a new global root note.
    pub fn set_sample_root_note(&mut self, root_note: Note) {
        self.sample_root_note = root_note;
    }

    /// Stop all currently playing back sources.
    pub fn stop_all_sources(&mut self) {
        let _ = self.player.stop_all_sources();
        for notes in &mut self.playing_notes {
            notes.clear();
        }
    }

    /// Stop all currently playing back sources in the given pattern slot index.
    pub fn stop_sources_in_pattern_slot(&mut self, pattern_index: usize) {
        for (playback_id, _) in self.playing_notes[pattern_index].values() {
            let _ = self.player.stop_source(*playback_id);
        }
        self.playing_notes[pattern_index].clear();
    }

    /// Run/play the given sequence until it stops.
    pub fn run(
        &mut self,
        sequence: &mut Sequence,
        time_base: &dyn SampleTimeBase,
        reset_playback_pos: bool,
    ) {
        let dont_stop = || false;
        self.run_until(sequence, time_base, reset_playback_pos, dont_stop);
    }

    /// Run the given sequence until it stops or the passed stop condition function returns true.
    pub fn run_until<StopFn: Fn() -> bool>(
        &mut self,
        sequence: &mut Sequence,
        time_base: &dyn SampleTimeBase,
        reset_playback_pos: bool,
        stop_fn: StopFn,
    ) {
        // reset time counters when starting the first time or when explicitly requested, else continue
        // playing from our previous time to avoid interrupting playback streams
        if reset_playback_pos || self.emitted_sample_time == 0 {
            self.reset_playback_position(sequence);
            log::debug!(target: "Player", "Resetting playback pos");
        } else {
            self.prepare_run_until_time(sequence, self.emitted_sample_time);
            log::debug!(target: "Player",
                "Advance sequence to time {:.2}",
                time_base.samples_to_seconds(self.emitted_sample_time)
            );
        }
        while !stop_fn() {
            // calculate emitted and playback time differences
            let seconds_emitted = time_base.samples_to_seconds(self.emitted_sample_time);
            let seconds_played = time_base.samples_to_seconds(
                self.player.output_sample_frame_position() - self.playback_sample_time,
            );
            let seconds_to_emit = seconds_played - seconds_emitted + PLAYBACK_PRELOAD_SECONDS * 2.0;
            // run sequence ahead of player up to PRELOAD_SECONDS
            if seconds_to_emit >= PLAYBACK_PRELOAD_SECONDS || self.emitted_sample_time == 0 {
                log::debug!(target: "Player",
                    "Seconds emitted {:.2}s - Seconds played {:.2}s: Emitting {:.2}s",
                    seconds_emitted,
                    seconds_played,
                    seconds_to_emit
                );
                let samples_to_emit = time_base.seconds_to_samples(seconds_to_emit);
                self.run_until_time(
                    sequence,
                    self.playback_sample_time,
                    self.emitted_sample_time + samples_to_emit,
                );
                self.emitted_sample_time += samples_to_emit;
            } else {
                // wait until next events are due, but check stop_fn at least every...
                const MAX_SLEEP_TIME: f64 = 0.1;
                let time_until_next_emit_batch =
                    (PLAYBACK_PRELOAD_SECONDS - seconds_to_emit).max(0.0);
                let mut time_slept = 0.0;
                while time_slept < time_until_next_emit_batch && !stop_fn() {
                    let sleep_amount = time_until_next_emit_batch.min(MAX_SLEEP_TIME);
                    std::thread::sleep(std::time::Duration::from_secs_f64(sleep_amount));
                    // log::debug!(target: "Player", "Slept {} seconds", sleep_amount);
                    time_slept += sleep_amount;
                }
            }
        }
    }

    /// Initialize the given sequence for playback with `run_until_time`.
    /// This seeks the sequence to the given position and keeps track of internal playback state.
    pub fn prepare_run_until_time(&mut self, sequence: &mut Sequence, sample_time: u64) {
        // match playing notes state to the passed patterns
        self.playing_notes
            .resize(sequence.phrase_pattern_slot_count(), HashMap::new());
        // move new phase to our previously played time
        sequence.advance_until_time(sample_time);
    }

    /// Manually seek the given sequence to the  given time offset and actual position.
    pub fn advance_until_time(&mut self, sequence: &mut Sequence, time: SampleTime) {
        self.stop_all_sources();
        sequence.advance_until_time(time);
    }

    /// Manually run the given sequence with the given time offset and actual position.
    /// When exchanging the seuquence, call `prepare_run_until_time` before calling `run_until_time`.
    pub fn run_until_time(
        &mut self,
        sequence: &mut Sequence,
        time_offset: SampleTime,
        time: SampleTime,
    ) {
        let time_base = *sequence.time_base();
        sequence.consume_events_until_time(time, &mut |pattern_index, pattern_event| {
            self.handle_pattern_event(pattern_index, pattern_event, time_base, time_offset);
        });
    }

    /// Handle a single pattern event from the sequence
    fn handle_pattern_event(
        &mut self,
        pattern_index: usize,
        pattern_event: PatternEvent,
        time_base: BeatTimeBase,
        time_offset: SampleTime,
    ) {
        // Print event if enabled
        if self.show_events {
            const SHOW_INSTRUMENTS_AND_PARAMETERS: bool = true;
            println!(
                "{}: {}",
                time_base.display(pattern_event.time),
                match &pattern_event.event {
                    Some(event) => event.to_string(SHOW_INSTRUMENTS_AND_PARAMETERS),
                    None => "---".to_string(),
                }
            );
        }

        // Process note events
        let playing_notes_in_pattern = &mut self.playing_notes[pattern_index];
        if let Some(Event::NoteEvents(notes)) = pattern_event.event {
            for (voice_index, note_event) in notes.iter().enumerate() {
                let note_event = match note_event {
                    None => continue,
                    Some(note_event) => note_event,
                };
                // Handle note off or stop action
                if note_event.note.is_note_off()
                    || (note_event.note.is_note_on()
                        && self.new_note_action != NewNoteAction::Continue)
                {
                    if let Some((playback_id, _)) = playing_notes_in_pattern.get(&voice_index) {
                        let _ = self.player.stop_source_at_sample_time(
                            *playback_id,
                            time_offset + pattern_event.time,
                        );
                        playing_notes_in_pattern.remove(&voice_index);
                    }
                }
                // Play new note
                if !note_event.note.is_note_on() {
                    continue;
                }
                if let Some(instrument) = note_event.instrument {
                    let midi_note = (note_event.note as i32 + 60 - self.sample_root_note as i32)
                        .clamp(0, 127) as u8;
                    let volume = note_event.volume.max(0.0);
                    let panning = note_event.panning.clamp(-1.0, 1.0);
                    let mut playback_options = FilePlaybackOptions::default()
                        .speed(speed_from_note(midi_note))
                        .volume(volume)
                        .panning(panning)
                        .playback_pos_emit_rate(self.playback_pos_emit_rate);
                    playback_options.fade_out_duration = match self.new_note_action {
                        NewNoteAction::Continue | NewNoteAction::Stop => {
                            Some(Duration::from_millis(100))
                        }
                        NewNoteAction::Off(duration) => duration,
                    };

                    let playback_sample_rate = self.player.output_sample_rate();
                    let sample_pool = self
                        .sample_pool
                        .read()
                        .expect("Failed to access sample pool");

                    if let Ok(sample) =
                        sample_pool.get_sample(instrument, playback_options, playback_sample_rate)
                    {
                        let sample_delay =
                            (note_event.delay * pattern_event.duration as f32) as SampleTime;
                        let start_time = Some(time_offset + pattern_event.time + sample_delay);

                        let context: Option<PlaybackStatusContext> =
                            Some(Arc::new(SamplePlaybackContext {
                                pattern_index: Some(pattern_index),
                                voice_index: Some(voice_index),
                            }));

                        let playback_id = self
                            .player
                            .play_file_source_with_context(sample, start_time, context)
                            .expect("Failed to play file source");

                        playing_notes_in_pattern
                            .insert(voice_index, (playback_id, note_event.note));
                    } else {
                        log::error!(target: "Player", "Failed to get sample with id {}", instrument);
                    }
                }
            }
        }
    }

    fn reset_playback_position(&mut self, sequence: &Sequence) {
        // rebuild playing notes vec
        self.playing_notes
            .resize(sequence.phrase_pattern_slot_count(), HashMap::new());
        // stop whatever is playing in case we're restarting
        self.player
            .stop_all_sources()
            .expect("failed to stop all playing samples");
        // fetch player's actual position and use it as start offset
        self.playback_sample_time = self.player.output_sample_frame_position();
        self.emitted_sample_time = 0;
    }
}
