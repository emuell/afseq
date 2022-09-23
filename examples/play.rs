use std::collections::HashMap;

use afplay::{
    source::file::preloaded::PreloadedFileSource, utils::speed_from_note, AudioFilePlayer,
    AudioOutput, DefaultAudioOutput, FilePlaybackOptions,
};

use afseq::prelude::*;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    // create player
    let audio_output = DefaultAudioOutput::open()?;
    let mut player = AudioFilePlayer::new(audio_output.sink(), None);

    // tools
    let sample_rate = player.output_sample_rate();
    let samples_to_seconds = |samples: SampleTime| samples as f64 / sample_rate as f64;
    let seconds_to_samples = |seconds: f64| (seconds as f64 * sample_rate as f64) as SampleTime;

    // preload all samples
    let load_file = |file_name| {
        PreloadedFileSource::new(file_name, None, FilePlaybackOptions::default()).unwrap()
    };

    const KICK: InstrumentId = 0;
    const SNARE: InstrumentId = 1;
    const HIHAT: InstrumentId = 2;
    let sample_pool: HashMap<InstrumentId, PreloadedFileSource> = HashMap::from([
        (KICK, load_file("assets/kick.wav")),
        (SNARE, load_file("assets/snare.wav")),
        (HIHAT, load_file("assets/hat.wav")),
    ]);

    // generate a simple drum sequence
    let time_base = BeatTimeBase {
        beats_per_min: 130.0,
        beats_per_bar: 4,
        samples_per_sec: sample_rate,
    };

    let kick_pattern = time_base.every_nth_sixteenth_with_pattern(
        1,
        [
            1, 0, 0, 0, /**/ 0, 0, 1, 0, /**/ 0, 0, 1, 0, /**/ 0, 0, 0, 0, //
            1, 0, 0, 0, /**/ 0, 0, 1, 0, /**/ 0, 0, 1, 0, /**/ 0, 0, 0, 0, //
            1, 0, 0, 0, /**/ 0, 0, 1, 0, /**/ 0, 0, 1, 0, /**/ 0, 0, 0, 0, //
            1, 0, 0, 0, /**/ 0, 0, 1, 0, /**/ 0, 0, 1, 0, /**/ 0, 1, 0, 0, //
        ],
        new_note_event(KICK, 60, 1.0),
    );
    let snare_pattern = time_base.every_nth_beat_with_offset(2, 1, new_note_event(SNARE, 60, 1.0));
    let hihat_pattern = time_base.every_nth_sixteenth_with_offset(
        2,
        0,
        new_note_event(HIHAT, 60, 1.0).map_notes({
            let mut step = 0;
            move |note| {
                let mut note = note;
                note.velocity = 1.0 / (step + 1) as f32;
                step += 1;
                if step >= 3 {
                    step = 0;
                }
                note
            }
        }),
    );
    let hihat_pattern2 = time_base.every_nth_sixteenth_with_offset(
        2,
        1,
        new_note_event(HIHAT, 60, 1.0).map_notes({
            let mut vel_step = 0;
            let mut note_step = 0;
            move |note| {
                let mut note = note;
                note.velocity = 1.0 / (vel_step + 1) as f32 * 0.5;
                vel_step += 1;
                if vel_step >= 3 {
                    vel_step = 0;
                }
                note.note = 60 + 32 - note_step;
                note_step += 1;
                if note_step >= 32 {
                    note_step = 0;
                }
                note
            }
        }),
    );

    let hihat_rhythm = Phrase::new(vec![Box::new(hihat_pattern), Box::new(hihat_pattern2)]);

    let mut phrase = Phrase::new(vec![
        Box::new(kick_pattern),
        Box::new(snare_pattern),
        Box::new(hihat_rhythm),
    ]);

    // emit notes and feed them into the player
    let print_event = |sample_time: SampleTime, event: &Option<Event>| {
        println!(
            "{:.1} ({:08}) -> {}",
            sample_time as f64 / time_base.samples_per_beat(),
            sample_time,
            match event {
                Some(event) => {
                    format!("{:?}", event)
                }
                None => "---".to_string(),
            }
        );
    };

    // delay initial playback a bit until we're emitting: the player is already running
    let playback_delay_in_samples = player.output_sample_frame_position() + seconds_to_samples(0.5);

    let play_event = |player: &mut AudioFilePlayer,
                      sample_time: SampleTime,
                      event: &Option<Event>| {
        // play
        if let Event::NoteEvents(notes) = event.as_ref().unwrap_or(&Event::NoteEvents(Vec::new())) {
            for note in notes {
                if let Some(instrument) = note.instrument {
                    if let Some(sample) = sample_pool.get(&instrument) {
                        let mut new_source = sample.clone();
                        new_source.set_volume(note.velocity);
                        player
                            .play_file_source(
                                new_source,
                                speed_from_note(note.note),
                                Some(sample_time as u64 + playback_delay_in_samples),
                            )
                            .unwrap();
                    }
                }
            }
        }
    };

    let mut emitted_sample_time: u64 = 0;
    loop {
        const PRELOAD_SECONDS: f64 = 2.0;

        let seconds_emitted = samples_to_seconds(emitted_sample_time);
        let seconds_played = samples_to_seconds(player.output_sample_frame_position());
        let seconds_to_emit = seconds_played - seconds_emitted + PRELOAD_SECONDS;

        if seconds_to_emit > 1.0 {
            emitted_sample_time += seconds_to_samples(seconds_to_emit);
            phrase.run_until_time(emitted_sample_time, |sample_time, event| {
                print_event(sample_time, event);
                play_event(&mut player, sample_time, event);
            });
        } else {
            let sleep_amount = 1.0 - seconds_to_emit;
            std::thread::sleep(std::time::Duration::from_secs_f64(sleep_amount));
        }
    }
}
