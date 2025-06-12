use std::hint::black_box;

use criterion::{criterion_group, Criterion};

use afseq::prelude::*;

// ---------------------------------------------------------------------------------------------

fn create_phrase() -> Phrase {
    let beat_time = BeatTimeBase {
        samples_per_sec: 44100,
        beats_per_min: 130.0,
        beats_per_bar: 4,
    };

    let kick_pattern = new_pattern_from_string(
        beat_time,
        None,
        r#"
          return pattern {
            unit = "1/1", 
            pulse = function(context) 
              return 1 
            end,
            event = cycle("bd? [~ bd] ~ ~ bd [~ bd] _ ~ bd? [~ bd] ~ ~ bd [~ bd] [_ bd2] [~ bd _ ~]"):map({
                ["bd"] = "c4",
                ["bd2"] = "c4 v0.5",
            })
          }
        "#,
        "kick pattern.lua",
    )
    .unwrap();

    let snare_pattern = new_pattern_from_string(
        beat_time,
        None,
        r#"
          return pattern {
            unit = "beats", 
            pulse = { 0, 1 },
            event = "c5"
          }
        "#,
        "snare pattern.lua",
    )
    .unwrap();

    let bass_pattern = new_pattern_from_string(
        beat_time,
        None,
        r#"
          local scale = scale("c5", "natural minor")
          return pattern {
          unit = "1/8",
          pulse = pulse.from({ 1, 0.5, 1, 0 }, { 0, 1, 0, 0 }, { 1, 0, 1, 0 }, { 0, 1, 0, 1 }),
          gate = function (context)
              return context.pulse_value == 1.0
          end,
          event = pulse.from(1, 3, 4, 1, 3, 4, -7):map(function(index, value)
            if value < 0 then
              return { key = scale.notes[-value] - 12, volume = 0.7 }
            else
              return { key = scale.notes[value], volume = 0.7 }
            end
          end
          )
        }
        "#,
        "bass pattern.lua",
    )
    .unwrap();

    let fx_pattern = new_pattern_from_string(
        beat_time,
        None,
        r#"
          return pattern {
            unit = "seconds",
            resolution = 8,
            offset = 8,
            event = {
              note("c_4", "---", "---"):volume(0.2),
              note("---", "c_5", "---"):volume(0.25),
              note("---", "---", "f_5"):volume(0.2)
            }
          }
        "#,
        "fx pattern.lua",
    )
    .unwrap();

    let chord_pattern = new_pattern_from_string(
        beat_time,
        None,
        r#"
          local CMIN = scale("c4", "minor")
          local CHORDS = {
            note(CMIN:chord("i", 3)),
            note(CMIN:chord("i", 3)):transpose({0, 0, -2}),
            note(CMIN:chord("i", 3)),
            note(CMIN:chord("i", 4)):transpose({0, 0, 3, -12})
          }
          return pattern {
            unit = "bars", 
            resolution = 4,
            event = function(context)
              return CHORDS[math.imod(context.step, #CHORDS)] 
            end
          }
        "#,
        "chord pattern.lua",
    )
    .unwrap();

    Phrase::new(
        beat_time,
        vec![
            PatternSlot::from(kick_pattern),
            PatternSlot::from(snare_pattern),
            PatternSlot::from(bass_pattern),
            PatternSlot::from(fx_pattern),
            PatternSlot::from(chord_pattern),
        ],
        BeatTimeStep::Bar(8.0),
    )
}

// ---------------------------------------------------------------------------------------------

pub fn create(c: &mut Criterion) {
    let mut group = c.benchmark_group("Scripted Phrase");
    group.bench_function("Create", |b| {
        b.iter(|| {
            black_box(create_phrase());
        })
    });
    group.finish();
}

pub fn clone(c: &mut Criterion) {
    let mut group = c.benchmark_group("Scripted Phrase");
    let phrase = create_phrase();
    group.bench_function("Clone", |b| {
        b.iter(|| {
            let mut phrase = black_box(phrase.clone());
            phrase.reset();
        })
    });
    group.finish();
}

pub fn run(c: &mut Criterion) {
    let event_count = 2500;
    let mut group = c.benchmark_group("Scripted Phrase");
    group.measurement_time(std::time::Duration::from_secs(10));
    let phrase = create_phrase();
    group.bench_function("Run", |b| {
        b.iter_batched(
            || {
                let mut phrase = phrase.clone();
                phrase.reset();
                phrase
            },
            |mut phrase| {
                let sample_time = SampleTime::MAX;
                let mut num_events = 0;
                while let Some(event) = phrase.run_until_time(sample_time) {
                    black_box(event);
                    num_events += 1;
                    if num_events >= event_count {
                        break;
                    }
                }
            },
            criterion::BatchSize::SmallInput,
        )
    });
    group.finish();
}

pub fn seek(c: &mut Criterion) {
    let mut group = c.benchmark_group("Scripted Phrase");
    let phrase = create_phrase();
    let samples_per_sec = phrase.time_base().samples_per_sec as SampleTime;
    let seek_step = 10;
    let seek_time = 60 * 60;
    group.bench_function("Seek", |b| {
        b.iter_batched(
            || {
                let mut phrase = phrase.clone();
                phrase.reset();
                phrase
            },
            |mut phrase| {
                let mut sample_time = samples_per_sec;
                while sample_time < seek_time * samples_per_sec {
                    phrase.advance_until_time(sample_time);
                    sample_time += seek_step * samples_per_sec;
                }
            },
            criterion::BatchSize::SmallInput,
        )
    });
    group.finish();
}

// ---------------------------------------------------------------------------------------------

criterion_group! {
    name = scripted;
    config = Criterion::default();
    targets = create, clone, run, seek
}
