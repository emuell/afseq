#![allow(clippy::missing_safety_doc)]

use std::{
    cell::RefCell,
    ffi, fs,
    rc::Rc,
    sync::{Arc, RwLock},
    time::Duration,
};

use pattrns::prelude::*;

extern "C" {
    fn emscripten_cancel_animation_frame(requestAnimationFrameId: ffi::c_long);
    fn emscripten_request_animation_frame_loop(
        func: unsafe extern "C" fn(ffi::c_double, *mut ffi::c_void) -> ffi::c_int,
        user_data: *mut ffi::c_void,
    ) -> ffi::c_long;
}

// -------------------------------------------------------------------------------------------------

// We're called from single thread in JS only, thus we can avoid using Mutex or other RWLocks
// which would require using atomics in the WASM.
thread_local!( //
    static PLAYGROUND: RefCell<Option<Playground>> = const { RefCell::new(None) }
);

/// Helper function to safely access the thread-local Playground state immutably.
fn with_playground<F, R>(f: F) -> R
where
    F: FnOnce(&Playground) -> R,
    R: Default,
{
    PLAYGROUND.with_borrow(|player| {
        if let Some(playground) = player {
            f(playground)
        } else {
            R::default()
        }
    })
}

/// Helper function to safely access the thread-local Playground state mutably.
fn with_playground_mut<F, R>(f: F) -> R
where
    F: FnOnce(&mut Playground) -> R,
    R: Default,
{
    PLAYGROUND.with_borrow_mut(|player| {
        if let Some(playground) = player {
            f(playground)
        } else {
            R::default()
        }
    })
}

// -------------------------------------------------------------------------------------------------

/// Single sample asset, passed as JSON to the frontend.
#[derive(serde::Serialize)]
struct SampleEntry {
    name: String,
    id: usize,
}

/// Single example script content section, passed as JSON to the frontend.
#[derive(serde::Serialize)]
struct ScriptSection {
    name: String,
    entries: Vec<ScriptEntry>,
}

/// Single example script asset, passed as JSON to the frontend.
#[derive(serde::Serialize)]
struct ScriptEntry {
    name: String,
    content: String,
}

/// Single pattern triggered by a MIDI note
#[derive(Clone)]
struct PlayingNote {
    note: u8,
    velocity: u8,
    sample_offset: SampleTime,
}

/// The backend's global app state.
struct Playground {
    playing: bool,
    player: SamplePlayer,
    samples: Vec<SampleEntry>,
    sequence: Option<Sequence>,
    time_base: BeatTimeBase,
    time_base_changed: bool,
    instrument_id: Option<usize>,
    script_content: String,
    script_changed: bool,
    script_runtime_error: String,
    playing_notes: Vec<PlayingNote>,
    output_start_sample_time: u64,
    emitted_sample_time: u64,
    run_frame_id: ffi::c_long,
}

impl Playground {
    // Event scheduler read-ahead time (latency)
    const PLAYBACK_PRELOAD_SECONDS: f64 = if cfg!(debug_assertions) { 1.0 } else { 0.25 };
    // Max expected MIDI notes
    const NUM_MIDI_NOTES: usize = 127;
    // Path to our assets folder. see build.rs.
    const ASSETS_PATH: &str = "/assets";

    /// Creates a new Playground instance with initialized state.
    /// Returns an error if initialization fails at any step.
    pub fn new() -> Result<Self, Box<dyn std::error::Error>> {
        // load samples
        println!("Loading sample files...");
        let mut samples = Vec::new();
        let sample_pool = SamplePool::new();
        for dir_entry in fs::read_dir(format!("{}/samples", Self::ASSETS_PATH))?.flatten() {
            let path = dir_entry.path();
            if let Some(extension) = path.extension().map(|e| e.to_string_lossy()) {
                if matches!(extension.as_bytes(), b"mp3" | b"wav" | b"flac") {
                    let id = usize::from(sample_pool.load_sample(&path)?);
                    let name = path.file_stem().unwrap().to_string_lossy().to_string();
                    println!("Added sample '{}' with id {}", name, id);
                    samples.push(SampleEntry { id, name });
                }
            }
        }

        // create and configure sample player
        println!("Creating audio player...");
        let playing = false;
        let mut player = SamplePlayer::new(Arc::new(RwLock::new(sample_pool)), None)?;
        player.set_sample_root_note(Note::C4);
        player.set_new_note_action(NewNoteAction::Off(Some(Duration::from_millis(200))));

        // create sequence
        let sequence = None;

        // time base
        let time_base = BeatTimeBase {
            beats_per_min: 120.0,
            beats_per_bar: 4,
            samples_per_sec: player.file_player().output_sample_rate(),
        };
        let time_base_changed = false;

        // script content
        let script_content = "return pattern { }".to_string();
        let script_changed = true;
        let script_runtime_error = String::new();

        // MIDI note playback
        let playing_notes = Vec::new();

        // default instrument
        let instrument_id = samples.first().map(|e| e.id);

        // playback time
        let output_start_sample_time = player.file_player().output_sample_frame_position();
        let emitted_sample_time = 0;

        // install emscripten frame timer
        let run_frame_id = unsafe {
            println!("Start running...");
            emscripten_request_animation_frame_loop(Self::run_frame, std::ptr::null_mut())
        };

        Ok(Self {
            player,
            playing,
            samples,
            sequence,
            time_base,
            time_base_changed,
            script_content,
            script_changed,
            script_runtime_error,
            playing_notes,
            instrument_id,
            output_start_sample_time,
            emitted_sample_time,
            run_frame_id,
        })
    }

    // read examples from the file system into a vector of ExampleScriptEntry
    fn example_scripts() -> Result<Vec<ScriptEntry>, Box<dyn std::error::Error>> {
        let mut example_entries = Vec::new();
        let example_paths = fs::read_dir(format!("{}/examples", Self::ASSETS_PATH))?;
        for example in example_paths.flatten() {
            let path = example.path();
            let mut name = path
                .file_stem()
                .unwrap_or_default()
                .to_string_lossy()
                .to_string();
            name = name
                .trim_start_matches(|c: char| c.is_ascii_digit() || c == ' ')
                .to_string();
            let content = fs::read_to_string(&path)?;
            example_entries.push(ScriptEntry { name, content });
        }
        Ok(example_entries)
    }

    // read quickstart examples from the file system into a vector of ExampleScriptSection
    fn quickstart_scripts() -> Result<Vec<ScriptSection>, Box<dyn std::error::Error>> {
        let mut quickstart_scripts = Vec::new();
        let section_paths = fs::read_dir(format!("{}/quickstart", Self::ASSETS_PATH))?;
        for section_path in section_paths.flatten() {
            if section_path.metadata()?.is_dir() {
                let mut section_name = section_path.file_name().to_string_lossy().to_string();
                section_name = section_name
                    .trim_start_matches(|c: char| c.is_ascii_digit() || c == ' ')
                    .to_string();
                let mut section_entries = Vec::new();
                let script_paths = fs::read_dir(section_path.path())?;
                for script_path in script_paths {
                    let script_path = script_path?;
                    if script_path.metadata()?.is_file() {
                        let mut script_name = script_path
                            .path()
                            .file_stem()
                            .unwrap_or_default()
                            .to_string_lossy()
                            .to_string();
                        script_name = script_name
                            .trim_start_matches(|c: char| c.is_ascii_digit() || c == ' ')
                            .to_string();
                        let script_content = fs::read_to_string(script_path.path())?;
                        section_entries.push(ScriptEntry {
                            name: script_name.to_string(),
                            content: script_content,
                        });
                    }
                }
                if !section_entries.is_empty() {
                    quickstart_scripts.push(ScriptSection {
                        name: section_name.to_string(),
                        entries: section_entries,
                    });
                }
            }
        }
        Ok(quickstart_scripts)
    }

    /// Starts playback of the current sequence.
    fn start_playing(&mut self) {
        if !self.playing {
            // reset play head
            let preload_offset = self
                .time_base
                .seconds_to_samples(Self::PLAYBACK_PRELOAD_SECONDS);
            self.output_start_sample_time =
                self.player.file_player().output_sample_frame_position() + preload_offset;
            self.emitted_sample_time = 0;
            // update sequence and play
            self.script_changed = true;
            self.playing = true;
        }
    }

    /// Stops all currently playing audio sources and resets the sequence.
    fn stop_playing(&mut self) {
        let _ = self.player.file_player_mut().stop_all_sources();
        if let Some(sequence) = self.sequence.as_mut() {
            sequence.reset();
        }
        self.playing = false;
    }

    /// Stops all currently playing audio sources.
    fn stop_playing_notes(&mut self) {
        let _ = self.player.file_player_mut().stop_all_sources();
    }

    /// Set global playback volume.
    fn set_volume(&mut self, volume: f32) {
        self.player.set_global_volume(volume);
    }

    /// Handle incoming MIDI note on event
    fn handle_midi_note_on(&mut self, note: u8, velocity: u8) {
        assert!(note as usize <= Self::NUM_MIDI_NOTES);
        if self.playing_notes.is_empty() || self.pattern_slot(note as usize).is_none() {
            // reset play head
            self.output_start_sample_time =
                self.player.file_player().output_sample_frame_position();
            self.emitted_sample_time = 0;
            // memorize playing note
            let new_note = PlayingNote {
                note,
                velocity,
                sample_offset: 0,
            };
            self.playing_notes.push(new_note);
            // start MIDI playback in `run`
            self.script_changed = true;
        } else {
            // memorize playing note
            let new_note = PlayingNote {
                note,
                velocity,
                sample_offset: self.emitted_sample_time,
            };
            self.playing_notes.push(new_note.clone());
            // add a new pattern for the new note
            let new_pattern = self.new_pattern(Some(new_note));
            let pattern_slot = self
                .pattern_slot(note as usize)
                .expect("Missing MIDI pattern slot");
            *pattern_slot = PatternSlot::Pattern(new_pattern);
        }
    }

    /// Handle incoming MIDI note off event
    fn handle_midi_note_off(&mut self, note: u8) {
        assert!(note as usize <= Self::NUM_MIDI_NOTES);
        // ony handle off events when we got an on event
        if let Some((playing_notes_index, _)) = self
            .playing_notes
            .iter()
            .enumerate()
            .find(|(_, n)| n.note == note)
        {
            // remove playing note
            self.playing_notes.remove(playing_notes_index);
            // remove the pattern slot from sequence's phrase
            if let Some(pattern_slot) = self.pattern_slot(note as usize) {
                *pattern_slot = PatternSlot::Stop;
                // stop pending from the note
                self.player.stop_sources_in_pattern_slot(note as usize);
            }
            // restore default playback in `run` with the last note removed
            if self.playing_notes.is_empty() {
                self.script_changed = true;
            }
        }
    }

    /// Updates the tempo (beats per minute) of playback.
    fn set_bpm(&mut self, bpm: f32) {
        self.time_base.beats_per_min = bpm;
        self.time_base_changed = true;
    }

    /// Sets the default instrument ID for playback.
    fn set_instrument(&mut self, id: i32) {
        self.instrument_id = Some(id as usize);
        self.script_changed = true;
    }

    /// Updates the script content and marks it as changed to trigger recompilation.
    fn update_script_content(&mut self, content: String) {
        self.script_content = content;
        self.script_changed = true;
    }

    /// Emscripten animation frame callback that drives the audio playback.
    /// Returns 1 to continue running or 0 to stop if Playground is not available.
    extern "C" fn run_frame(_time: ffi::c_double, _user_data: *mut ffi::c_void) -> ffi::c_int {
        PLAYGROUND.with_borrow_mut(|player| {
            if let Some(playground) = player {
                playground.run();
                1 // continue
            } else {
                0 // stop
            }
        })
    }

    /// Main playback loop: Handles player state updates and runs the player
    fn run(&mut self) {
        // apply script content changes
        if self.script_changed || self.sequence.is_none() {
            // clear errors
            pattrns::bindings::clear_lua_callback_errors();
            self.script_runtime_error = String::new();
            // build note pattern slots: one pattern for each live played note
            let pattern_slots = if !self.playing_notes.is_empty() {
                let mut slots = vec![PatternSlot::Stop; Self::NUM_MIDI_NOTES];
                for playing_note in &self.playing_notes.clone() {
                    slots[playing_note.note as usize] =
                        PatternSlot::Pattern(self.new_pattern(Some(playing_note.clone())));
                }
                slots
            } else {
                // build a single regular pattern slot
                [PatternSlot::Pattern(self.new_pattern(None))].to_vec()
            };
            // replace sequence
            let mut sequence = Sequence::new(
                self.time_base,
                vec![Phrase::new(
                    self.time_base,
                    pattern_slots,
                    BeatTimeStep::Bar(4.0),
                )],
            );
            self.player
                .prepare_run_until_time(&mut sequence, self.emitted_sample_time);
            self.sequence.replace(sequence);
            // reset all update flags: we're fully up to date now.
            self.script_changed = false;
            self.time_base_changed = false;
        }

        // apply time base changes
        if self.time_base_changed {
            self.time_base_changed = false;
            self.sequence
                .as_mut()
                .unwrap()
                .set_time_base(&self.time_base);
        }

        // check if audio output has been suspended by the browser
        let suspended = self.player.file_player().output_suspended();

        // run the player, when playing and audio output is not suspended
        if (self.playing || !self.playing_notes.is_empty()) && !suspended {
            // calculate emitted and playback time differences
            let time_base = self.time_base;
            let output_sample_time = self.player.file_player().output_sample_frame_position();
            let samples_played = // can be be negative, because we start with a preload offset 
                (output_sample_time as i64 - self.output_start_sample_time as i64).max(0) as u64;
            let seconds_played = time_base.samples_to_seconds(samples_played);
            let seconds_emitted = time_base.samples_to_seconds(self.emitted_sample_time);
            // run sequence ahead of player up to PLAYBACK_PRELOAD_SECONDS seconds
            let seconds_to_emit =
                (seconds_played - seconds_emitted + Self::PLAYBACK_PRELOAD_SECONDS).max(0.0);
            let samples_to_emit = time_base.seconds_to_samples(seconds_to_emit);
            if seconds_to_emit > 4.0 * Self::PLAYBACK_PRELOAD_SECONDS {
                // we lost too much time: maybe because the browser suspended the run loop
                self.player.advance_until_time(
                    self.sequence.as_mut().unwrap(),
                    self.emitted_sample_time + samples_to_emit,
                );
            } else if samples_to_emit > 0 {
                // continue running player to generate events in real-time
                self.player.run_until_time(
                    self.sequence.as_mut().unwrap(),
                    self.output_start_sample_time,
                    self.emitted_sample_time + samples_to_emit,
                );
                // handle runtime errors
                if let Some(err) = pattrns::bindings::has_lua_callback_errors() {
                    let err = err.to_string();
                    if err != self.script_runtime_error {
                        self.script_runtime_error = err;
                    }
                }
            }
            self.emitted_sample_time += samples_to_emit;
        }
    }

    /// Access an existing pattern slot
    fn pattern_slot(&mut self, pattern_index: usize) -> Option<&mut PatternSlot> {
        if let Some(sequence) = &mut self.sequence {
            let phrase = sequence
                .phrases_mut()
                .first_mut()
                .expect("Failed to access phrase");
            phrase.pattern_slots_mut().get_mut(pattern_index)
        } else {
            None
        }
    }

    /// Create a new pattern from the currently set script content
    fn new_pattern(&mut self, midi_note: Option<PlayingNote>) -> Rc<RefCell<dyn Pattern>> {
        // create a new pattern from our script
        let pattern = new_pattern_from_string(
            self.time_base,
            self.instrument_id.map(InstrumentId::from),
            &self.script_content,
            "[script]",
        )
        .unwrap_or_else(|err| {
            self.script_runtime_error = err.to_string();
            // create an empty fallback pattern on errors
            Rc::new(RefCell::new(BeatTimePattern::new(
                self.time_base,
                BeatTimeStep::Beats(1.0),
            )))
        });
        // and apply sample offset and event transform
        pattern
            .borrow_mut()
            .set_sample_offset(midi_note.as_ref().map(|n| n.sample_offset).unwrap_or(0));
        pattern
            .borrow_mut()
            .set_event_transform(self.new_pattern_event_transform(midi_note));
        pattern
    }

    /// Create a note event transform function which applies instrument and
    /// note_transpose transforms, when set.
    fn new_pattern_event_transform(
        &self,
        midi_note: Option<PlayingNote>,
    ) -> Option<EventTransform> {
        let transforms: Vec<_> = [
            // Instrument transform
            self.instrument_id.map(InstrumentId::from).map(|id| {
                Box::new(move |note: &mut NoteEvent| {
                    if note.instrument.is_none() {
                        note.instrument = Some(id)
                    }
                }) as Box<dyn Fn(&mut NoteEvent)>
            }),
            // Note transform
            midi_note.map(|note| {
                let offset = note.note as i32 - 48;
                let volume = note.velocity as f32 / 127.0;
                Box::new(move |note_event: &mut NoteEvent| {
                    note_event.note = note_event.note.transposed(offset);
                    note_event.volume *= volume;
                }) as Box<dyn Fn(&mut NoteEvent)>
            }),
        ]
        .into_iter()
        .flatten()
        .collect();

        if !transforms.is_empty() {
            Some(Rc::new(move |event: &mut Event| {
                if let Event::NoteEvents(note_events) = event {
                    note_events.iter_mut().flatten().for_each(|note_event| {
                        transforms
                            .iter()
                            .for_each(|transform| transform(note_event))
                    });
                }
            }))
        } else {
            None
        }
    }
}

impl Drop for Playground {
    /// Cleanup on Playground destruction.
    /// Stops the animation frame loop to prevent callbacks after destruction.
    fn drop(&mut self) {
        println!("Stopping run loop...");
        unsafe {
            emscripten_cancel_animation_frame(self.run_frame_id);
        }
    }
}

// -------------------------------------------------------------------------------------------------

// helper function to create a new raw CString from strings which may contain inner \0 chars.
unsafe fn new_raw_cstring(str: &str) -> *mut ffi::c_char {
    if str.contains('\0') {
        ffi::CString::from_vec_unchecked(str.replace('\0', "\\0").into()).into_raw()
    } else {
        ffi::CString::from_vec_unchecked(str.into()).into_raw()
    }
}

// helper function to drop a string created with `new_raw_cstring`
unsafe fn drop_raw_cstring(chars: *const ffi::c_char) {
    if !chars.is_null() {
        drop(ffi::CString::from_raw(chars as *mut ffi::c_char))
    }
}

/// Frees a string ptr which got passed to JS after it got consumed.
#[no_mangle]
pub unsafe extern "C" fn free_cstring(ptr: *mut ffi::c_char) {
    drop_raw_cstring(ptr);
}

/// Creates global `Playground` state.
#[no_mangle]
pub extern "C" fn initialize_playground() -> *const ffi::c_char {
    // create or recreate the player instance
    println!("Creating new player instance...");
    match Playground::new() {
        Err(err) => {
            eprintln!("Failed to create player instance: {}", err);
            PLAYGROUND.replace(None);
            unsafe { new_raw_cstring(&err.to_string()) }
        }
        Ok(player) => {
            println!("Successfully created a new player instance");
            PLAYGROUND.replace(Some(player));
            std::ptr::null()
        }
    }
}

/// Destroys global `Playground` state.
#[no_mangle]
pub extern "C" fn shutdown_playground() {
    // drop the player instance
    println!("Dropping player instance...");
    PLAYGROUND.replace(None);
}

/// Start playback.
#[no_mangle]
pub extern "C" fn start_playing() {
    with_playground_mut(|playground| playground.start_playing());
}

/// Stop playback.
#[no_mangle]
pub extern "C" fn stop_playing() {
    with_playground_mut(|playground| playground.stop_playing());
}

/// Stop all playing notes.
#[no_mangle]
pub extern "C" fn stop_playing_notes() {
    with_playground_mut(|playground| playground.stop_playing_notes());
}

/// Set new global volume factor.
#[no_mangle]
pub extern "C" fn set_volume(volume: f32) {
    with_playground_mut(|playground| playground.set_volume(volume));
}

/// Handle note on event from the frontend
#[no_mangle]
pub extern "C" fn midi_note_on(note: u8, velocity: u8) {
    with_playground_mut(|playground| playground.handle_midi_note_on(note, velocity));
}

/// Handle note off event from the frontend
#[no_mangle]
pub extern "C" fn midi_note_off(note: u8) {
    with_playground_mut(|playground| playground.handle_midi_note_off(note));
}

/// Update player's BPM.
#[no_mangle]
pub extern "C" fn set_bpm(bpm: ffi::c_int) {
    with_playground_mut(|playground| playground.set_bpm(bpm as f32));
}

/// Update player's default instrument id.
#[no_mangle]
pub extern "C" fn set_instrument(id: ffi::c_int) {
    with_playground_mut(|playground| playground.set_instrument(id));
}

#[no_mangle]
pub unsafe extern "C" fn update_script(content_ptr: *const ffi::c_char) {
    let content = unsafe {
        ffi::CStr::from_ptr(content_ptr)
            .to_string_lossy()
            .into_owned()
    };
    with_playground_mut(|playground| playground.update_script_content(content));
}

/// Returns available sample names and ids as json string.
#[no_mangle]
pub unsafe extern "C" fn get_samples() -> *const ffi::c_char {
    let json = with_playground(|playground| serde_json::to_string(&playground.samples).unwrap());
    new_raw_cstring(&json)
}

/// Returns example script names and contents as json string.
#[no_mangle]
pub unsafe extern "C" fn get_example_scripts() -> *const ffi::c_char {
    let example_scripts = Playground::example_scripts().unwrap();
    new_raw_cstring(&serde_json::to_string(&example_scripts).unwrap())
}

#[no_mangle]
pub unsafe extern "C" fn get_quickstart_scripts() -> *const ffi::c_char {
    let quickstart_scripts = Playground::quickstart_scripts().unwrap();
    new_raw_cstring(&serde_json::to_string(&quickstart_scripts).unwrap())
}

/// Returns actual script runtime errors, if any
#[no_mangle]
pub unsafe extern "C" fn get_script_error() -> *const ffi::c_char {
    let string = with_playground(|playground| playground.script_runtime_error.clone());
    new_raw_cstring(&string)
}

fn main() {
    // Disabled in build.rs via `cargo::rustc-link-arg=--no-entry`
    panic!("The main function is not exported and thus should never be called");
}
