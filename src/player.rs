//! Example player implementation, which plays back a `Sequence` via the `afplay` crate.

use std::{
    collections::HashMap,
    sync::{Arc, RwLock},
    time::Duration,
};

use crossbeam_channel::Sender;

use afplay::{
    source::file::preloaded::PreloadedFileSource, utils::speed_from_note, AudioFilePlaybackId,
    AudioFilePlaybackStatusContext, AudioFilePlaybackStatusEvent, AudioFilePlayer, AudioOutput,
    DefaultAudioOutput, Error, FilePlaybackOptions,
};

use crate::{
    event::{unique_instrument_id, InstrumentId},
    time::{SampleTimeDisplay, TimeBase},
    Event, Note, SampleTime, Sequence,
};

// -------------------------------------------------------------------------------------------------

/// Preload time of the player's `run_until` function. Should be big enough to ensure that events
/// are scheduled ahead of playback time, but small enough to avoid latency.
/// NB: real audio/event latency is twice the amount of the preload!
#[cfg(debug_assertions)]
const PLAYBACK_PRELOAD_SECONDS: f64 = 1.0;
#[cfg(not(debug_assertions))]
const PLAYBACK_PRELOAD_SECONDS: f64 = 0.5;

// -------------------------------------------------------------------------------------------------

/// Preloads a set of sample files and stores them as
/// [`PreloadedFileSource`] for later use.
///
/// When files are accessed, the stored file sources are cloned, which avoids loading and decoding
/// the files again. Cloned [`PreloadedFileSource`] are using a shared Buffer, so cloning is very cheap.
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

    /// Load a sample file into a [`PreloadedFileSource`] and return its id.
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
        let id = unique_instrument_id();
        let mut pool = self.pool.write().expect("Failed to access sample pool");
        pool.insert(id, sample);
        Ok(id)
    }
}

// -------------------------------------------------------------------------------------------------

/// Behaviour when playing a new note on the same voice channel.
#[derive(Copy, Clone, Debug, PartialEq)]
pub enum NewNoteAction {
    /// Continue playing the old note and start a new one.
    Continue,
    /// Stop the playing note before starting a new one.
    Stop,
}

// -------------------------------------------------------------------------------------------------

/// Context, passed along serialized when triggering new notes from the sample player.   
#[derive(Clone)]
pub struct SamplePlaybackContext {
    pub rhythm_index: Option<usize>,
    pub voice_index: Option<usize>,
}

impl SamplePlaybackContext {
    pub fn from_event(context: Option<AudioFilePlaybackStatusContext>) -> Self {
        if let Some(context) = context {
            if let Some(context) = context.downcast_ref::<SamplePlaybackContext>() {
                return context.clone();
            }
        }
        SamplePlaybackContext {
            rhythm_index: None,
            voice_index: None,
        }
    }
}

// -------------------------------------------------------------------------------------------------

/// An simple example player implementation, which plays back a `Sequence` via the `afplay` crate
/// using the default audio output device using plain samples loaded from a file as instruments.
///
/// Works on an existing sample pool, which can be used outside of the player as well.
pub struct SamplePlayer {
    player: AudioFilePlayer,
    sample_pool: Arc<RwLock<SamplePool>>,
    playing_notes: Vec<HashMap<usize, (AudioFilePlaybackId, Note)>>,
    new_note_action: NewNoteAction,
    playback_pos_emit_rate: Duration,
    show_events: bool,
    playback_sample_time: SampleTime,
    emitted_sample_time: SampleTime,
    emitted_beats: u32,
}

impl SamplePlayer {
    /// Create a new sample player.
    ///
    /// # Errors
    ///
    /// returns an error if the player could not be created.
    pub fn new(
        sample_pool: Arc<RwLock<SamplePool>>,
        playback_status_sender: Option<Sender<AudioFilePlaybackStatusEvent>>,
    ) -> Result<Self, Box<dyn std::error::Error>> {
        // create player
        let audio_output = DefaultAudioOutput::open()?;
        let player = AudioFilePlayer::new(audio_output.sink(), playback_status_sender);
        let playing_notes = Vec::new();
        let new_note_action = NewNoteAction::Continue;
        let playback_pos_emit_rate = Duration::from_secs(1);
        let show_events = false;
        let playback_sample_time = player.output_sample_frame_position();
        let emitted_sample_time = 0;
        let emitted_beats = 0;
        Ok(Self {
            player,
            sample_pool,
            playing_notes,
            new_note_action,
            playback_pos_emit_rate,
            show_events,
            playback_sample_time,
            emitted_sample_time,
            emitted_beats,
        })
    }

    /// Access to our file player.
    pub fn file_player(&self) -> &AudioFilePlayer {
        &self.player
    }
    pub fn file_player_mut(&mut self) -> &mut AudioFilePlayer {
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

    /// Run/play the given sequence until it stops.
    pub fn run(
        &mut self,
        sequence: &mut Sequence,
        time_base: &dyn TimeBase,
        reset_playback_pos: bool,
    ) {
        let dont_stop = || false;
        self.run_until(sequence, time_base, reset_playback_pos, dont_stop);
    }

    /// Run the given sequence until it stops or the passed stop condition function returns true.
    pub fn run_until<StopFn: Fn() -> bool>(
        &mut self,
        sequence: &mut Sequence,
        time_base: &dyn TimeBase,
        reset_playback_pos: bool,
        stop_fn: StopFn,
    ) {
        // reset time counters when starting the first time or when explicitly requested, else continue
        // playing from our previous time to avoid interrupting playback streams
        if reset_playback_pos || self.emitted_sample_time == 0 {
            self.reset_playback_position(sequence);
            log::debug!(target: "Player", "Resetting playback pos");
        } else {
            // match playing notes state to the passed rhythm
            self.playing_notes
                .resize(sequence.phrase_rhythm_slot_count(), HashMap::new());
            // seek new phase to our previously played time
            sequence.skip_events_until_time(self.emitted_sample_time);
            log::debug!(target: "Player",
                "Seek sequence to time {:.2}",
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

    fn reset_playback_position(&mut self, sequence: &Sequence) {
        // rebuild playing notes vec
        self.playing_notes.clear();
        self.playing_notes
            .resize(sequence.phrase_rhythm_slot_count(), HashMap::new());
        // stop whatever is playing in case we're restarting
        self.player
            .stop_all_sources()
            .expect("failed to stop all playing samples");
        // fetch player's actual position and use it as start offset
        self.playback_sample_time = self.player.output_sample_frame_position();
        self.emitted_sample_time = 0;
        self.emitted_beats = 0;
    }

    fn run_until_time(
        &mut self,
        sequence: &mut Sequence,
        start_offset: SampleTime,
        sample_time: SampleTime,
    ) {
        let time_base = *sequence.time_base();
        sequence.consume_events_until_time(
            sample_time,
            &mut |rhythm_index, sample_time, event: Option<Event>, event_duration| {
                // print
                if self.show_events {
                    const SHOW_INSTRUMENTS_AND_PARAMETERS: bool = true;
                    println!(
                        "{}: {}",
                        time_base.display(sample_time),
                        match &event {
                            Some(event) => event.to_string(SHOW_INSTRUMENTS_AND_PARAMETERS),
                            None => "---".to_string(),
                        }
                    );
                }
                // play
                let playing_notes_in_rhythm = &mut self.playing_notes[rhythm_index];
                if let Some(Event::NoteEvents(notes)) = event {
                    for (voice_index, note_event) in notes.iter().enumerate() {
                        if let Some(note_event) = note_event {
                            // stop playing samples on this voice channel
                            if let Some((playback_id, _)) =
                                playing_notes_in_rhythm.get(&voice_index)
                            {
                                if self.new_note_action == NewNoteAction::Stop
                                    || note_event.note.is_note_off()
                                {
                                    if let Err(_err) = self.player.stop_source_at_sample_time(
                                        *playback_id,
                                        start_offset + sample_time,
                                    ) {
                                        // this is expected when the sample played to end
                                    }
                                    playing_notes_in_rhythm.remove(&voice_index);
                                }
                            }
                            // start a new sample - when this is a note off, we already stopped it above
                            if note_event.note.is_note_on() {
                                if let Some(instrument) = note_event.instrument {
                                    let playback_options = FilePlaybackOptions::default()
                                        .speed(speed_from_note(note_event.note as u8))
                                        .playback_pos_emit_rate(self.playback_pos_emit_rate);
                                    let playback_sample_rate = self.player.output_sample_rate();
                                    let sample_pool = self
                                        .sample_pool
                                        .read()
                                        .expect("Failed to access sample pool");
                                    if let Ok(mut sample) = sample_pool.get_sample(
                                        instrument,
                                        playback_options,
                                        playback_sample_rate,
                                    ) {
                                        sample.set_volume(note_event.volume);
                                        let context = Arc::new(SamplePlaybackContext {
                                            rhythm_index: Some(rhythm_index),
                                            voice_index: Some(voice_index),
                                        });
                                        let sample_delay = (note_event.delay
                                            * event_duration as f32)
                                            as SampleTime;
                                        let playback_id = self
                                            .player
                                            .play_file_source_with_context(
                                                sample,
                                                Some(start_offset + sample_time + sample_delay),
                                                Some(context),
                                            )
                                            .expect("Failed to play file source");
                                        playing_notes_in_rhythm
                                            .insert(voice_index, (playback_id, note_event.note));
                                    }
                                    else {
                                        log::error!(target: "Player", "Failed to get sample with id {}", instrument);
                                    }
                                }
                            }
                        }
                    }
                }
            },
        );
    }
}
