//! Example player implementation, which plays back a `Phrase` via the `afplay` crate.
use crossbeam_channel::Sender;
use std::collections::HashMap;

use afplay::{
    source::file::preloaded::PreloadedFileSource, utils::speed_from_note, AudioFilePlaybackId,
    AudioFilePlaybackStatusEvent, AudioFilePlayer, AudioOutput, DefaultAudioOutput, Error,
    FilePlaybackOptions,
};

use crate::{
    event::{unique_instrument_id, InstrumentId},
    time::TimeBase,
    Event, Note, Phrase, SampleTime,
};

// -------------------------------------------------------------------------------------------------

/// Preloads a set of sample files and stores them as
/// [`afplay::PreloadedFileSource`](afplay::source::file::preloaded::PreloadedFileSource) for later
/// use.
///
/// When files are accessed, the stored file sources are cloned, which avoids loading and decoding
/// the files again. Cloned FileSources are using a shared Buffer, so cloning is very cheap.
#[derive(Default)]
pub struct SamplePool {
    pool: HashMap<InstrumentId, PreloadedFileSource>,
}

impl SamplePool {
    /// Create a new pool
    pub fn new() -> Self {
        Self {
            pool: HashMap::new(),
        }
    }

    /// Fetch a clone of a preloaded sample with the given plaback options.
    pub fn get_sample(
        &self,
        id: InstrumentId,
        playback_options: FilePlaybackOptions,
        playback_sample_rate: u32,
    ) -> Option<PreloadedFileSource> {
        self.pool.get(&id).map(|sample| {
            sample
                .clone(playback_options, playback_sample_rate)
                .expect("Failed to clone sample file")
        })
    }

    /// Load a sample file into a PreloadedFileSource and return its id.
    /// A copy of this sample can then later on be fetched with `get_sample` with the returned id.  
    pub fn load_sample(&mut self, file_path: &str) -> Result<InstrumentId, Error> {
        let sample =
            PreloadedFileSource::new(file_path, None, FilePlaybackOptions::default(), 44100)?;
        let id = unique_instrument_id();
        self.pool.insert(id, sample);
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

/// An simple example player implementation, which plays back a `Phrase` via the `afplay` crate
/// using the default audio output device using plain samples loaded from a file as instruments.
pub struct SamplePlayer {
    player: AudioFilePlayer,
    sample_pool: SamplePool,
    playing_notes: Vec<HashMap<usize, (AudioFilePlaybackId, Note)>>,
    new_note_action: NewNoteAction,
    show_events: bool,
}

impl SamplePlayer {
    pub fn new(
        playback_status_sender: Option<Sender<AudioFilePlaybackStatusEvent>>,
    ) -> Result<Self, Box<dyn std::error::Error>> {
        // create player
        let audio_output = DefaultAudioOutput::open()?;
        let player = AudioFilePlayer::new(audio_output.sink(), playback_status_sender);
        let sample_pool = SamplePool::new();
        let playing_notes = Vec::new();
        let new_note_action = NewNoteAction::Continue;
        let show_events = false;
        Ok(Self {
            player,
            sample_pool,
            playing_notes,
            new_note_action,
            show_events,
        })
    }

    /// Access to our file player.
    pub fn file_player(&mut self) -> &mut AudioFilePlayer {
        &mut self.player
    }

    /// Access to our sampel pool.
    pub fn sample_pool(&mut self) -> &mut SamplePool {
        &mut self.sample_pool
    }

    /// true when events are dumped to stdout while playing them.
    pub fn show_events(&self) -> bool {
        self.show_events
    }
    /// by default false: set to true to dump events to stdout while playing them.
    pub fn set_show_events(&mut self, show: bool) {
        self.show_events = show;
    }

    /// get current new note action behaviour.
    pub fn new_note_action(&self) -> NewNoteAction {
        self.new_note_action
    }
    // set a new new note action behaviour.
    pub fn set_new_note_action(&mut self, action: NewNoteAction) {
        self.new_note_action = action
    }

    /// Run/play the given phrase until it stops.
    pub fn run(&mut self, phrase: &mut Phrase, time_base: &dyn TimeBase) {
        let dont_stop = || false;
        self.run_until(phrase, time_base, dont_stop);
    }

    /// Run the given phrase until it stops or the passed stop condition function returns true.
    pub fn run_until<StopFn: Fn() -> bool>(
        &mut self,
        phrase: &mut Phrase,
        time_base: &dyn TimeBase,
        stop_fn: StopFn,
    ) {
        // rebuild playing notes vec
        self.playing_notes.clear();
        self.playing_notes
            .resize(phrase.rhythms().len(), HashMap::new());

        // stop whatever is playing in case we're restarting
        self.player
            .stop_all_sources()
            .expect("failed to stop all playing samples");
        // start playing at the player's current time with a little delay to avoid clicks
        let start_delay = self.player.output_sample_rate() as u64 / 8;
        let start_offset = self.player.output_sample_frame_position();
        // run PRELOAD_SECONDS ahead of the player's time until stop_fn signals us to stop
        let mut emitted_sample_time: u64 = 0;
        loop {
            const PRELOAD_SECONDS: f64 = 2.0;
            let seconds_emitted = time_base.samples_to_seconds(emitted_sample_time);
            let seconds_played = time_base
                .samples_to_seconds(self.player.output_sample_frame_position() - start_offset);
            let seconds_to_emit = seconds_played - seconds_emitted + PRELOAD_SECONDS;

            if seconds_to_emit > 1.0 {
                let samples_to_emit = time_base.seconds_to_samples(seconds_to_emit);
                self.run_phrase_until_time(
                    phrase,
                    start_offset + start_delay,
                    emitted_sample_time + samples_to_emit,
                );
                emitted_sample_time += samples_to_emit;
            } else {
                if stop_fn() {
                    break;
                }
                let sleep_amount = (1.0 - seconds_to_emit).max(0.0);
                std::thread::sleep(std::time::Duration::from_secs_f64(sleep_amount));
            }
        }
    }

    fn run_phrase_until_time(
        &mut self,
        phrase: &mut Phrase,
        start_offset: SampleTime,
        sample_time: SampleTime,
    ) {
        phrase.run_until_time(sample_time, |rhythm_index, sample_time, event| {
            // print
            if self.show_events {
                println!(
                    "{:08} -> {}",
                    sample_time,
                    match event {
                        Some(event) => format!("{:?}", event),
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
                        if let Some((playback_id, _)) = playing_notes_in_rhythm.get(&voice_index) {
                            if self.new_note_action == NewNoteAction::Stop
                                || note_event.note.is_note_off()
                            {
                                self.player
                                    .stop_source_at_sample_time(
                                        *playback_id,
                                        start_offset + sample_time,
                                    )
                                    .unwrap();
                                playing_notes_in_rhythm.remove(&voice_index);
                            }
                        }
                        // start a new sample - when this is a note off, we already stopped it above
                        if note_event.note.is_note_on() {
                            if let Some(instrument) = note_event.instrument {
                                let playback_options = FilePlaybackOptions::default()
                                    .speed(speed_from_note(note_event.note as u8));
                                let playback_sample_rate = self.player.output_sample_rate();
                                if let Some(mut sample) = self.sample_pool.get_sample(
                                    instrument,
                                    playback_options,
                                    playback_sample_rate,
                                ) {
                                    sample.set_volume(note_event.velocity);
                                    let playback_id = self
                                        .player
                                        .play_file_source(sample, Some(start_offset + sample_time))
                                        .unwrap();
                                    playing_notes_in_rhythm
                                        .insert(voice_index, (playback_id, note_event.note));
                                }
                            }
                        }
                    }
                }
            }
        });
    }
}
