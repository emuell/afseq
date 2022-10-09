//! Example player implementation, which plays back a `Phrase` via the `afplay` crate.
use std::{cell::RefCell, collections::HashMap, rc::Rc};

use afplay::{
    source::file::preloaded::PreloadedFileSource, utils::speed_from_note, AudioFilePlayer,
    AudioOutput, DefaultAudioOutput, FilePlaybackOptions,
};

use crate::{
    event::{unique_instrument_id, InstrumentId},
    time::TimeBase,
    Event, Phrase, SampleTime,
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

    /// Fetch a clone of a preloaded sample.
    pub fn get_sample(&self, id: InstrumentId) -> Option<PreloadedFileSource> {
        self.pool.get(&id).cloned()
    }

    /// Load a sample file into a PreloadedFileSource and return its id.
    /// A copy of this sample can then later on be fetched with `get_sample` with the returned id.  
    pub fn load_sample(
        &mut self,
        file_path: &str,
    ) -> Result<InstrumentId, Box<dyn std::error::Error>> {
        let sample = PreloadedFileSource::new(file_path, None, FilePlaybackOptions::default())?;
        let id = unique_instrument_id();
        self.pool.insert(id, sample);
        Ok(id)
    }
}

// -------------------------------------------------------------------------------------------------

/// An simple example player implementation, which plays back a `Phrase` via the `afplay` crate
/// using the default audio output device using plain samples loaded from a file as instruments.
pub struct SamplePlayer {
    player: AudioFilePlayer,
    sample_pool: Rc<RefCell<SamplePool>>,
}

impl SamplePlayer {
    pub fn new(sample_pool: Rc<RefCell<SamplePool>>) -> Result<Self, Box<dyn std::error::Error>> {
        // create player
        let audio_output = DefaultAudioOutput::open()?;
        let player = AudioFilePlayer::new(audio_output.sink(), None);
        Ok(Self {
            player,
            sample_pool,
        })
    }

    /// The actual audio output sample rate.
    pub fn sample_rate(&self) -> u32 {
        self.player.output_sample_rate()
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
        // stop whatever is playing in case we're restarting
        self.player
            .stop_all_playing_sources()
            .expect("failed to stop all playing samples");

        // run until stop_fn signals to stop
        let start_offset = self.player.output_sample_frame_position();
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
                    start_offset,
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
        phrase.run_until_time(sample_time, |_rhythm_index, sample_time, event| {
            // print
            println!(
                "{:08} -> {}",
                sample_time,
                match event {
                    Some(event) => format!("{:?}", event),
                    None => "---".to_string(),
                }
            );
            // play
            if let Some(Event::NoteEvents(notes)) = event {
                for (_voice_index, note) in notes.iter().enumerate() {
                    if let Some(note) = note {
                        // TODO: stop playing note at (rhythm_index, voice_index) column
                        if note.note.is_note_on() {
                            if let Some(instrument) = note.instrument {
                                if let Some(mut sample) =
                                    self.sample_pool.borrow().get_sample(instrument)
                                {
                                    sample.set_volume(note.velocity);
                                    self.player
                                        .play_file_source(
                                            sample,
                                            speed_from_note(note.note as u8),
                                            Some(start_offset + sample_time),
                                        )
                                        .unwrap();
                                }
                            }
                        }
                    }
                }
            }
        });
    }
}
