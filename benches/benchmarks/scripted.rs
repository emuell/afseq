use criterion::{black_box, criterion_group, Criterion};

use afseq::prelude::*;

// ---------------------------------------------------------------------------------------------

fn create_phrase() -> Phrase {
    let beat_time = BeatTimeBase {
        samples_per_sec: 44100,
        beats_per_min: 130.0,
        beats_per_bar: 4,
    };

    let kick_rhythm = new_rhythm_from_string(
        beat_time,
        None,
        r#"
          return rhythm {
            unit = "1/1", 
            emit = cycle("bd [~ bd] ~ ~ bd [~ bd] _ ~ bd [~ bd] ~ ~ bd [~ bd] [_ bd2] [~ bd _ ~]"):map({
                ["bd"] = "c4",
                ["bd2"] = "c4 v0.5",
            })
          }
        "#,
        "kick rhythm.lua",
    )
    .unwrap();

    let snare_rhythm = new_rhythm_from_string(
        beat_time,
        None,
        r#"
          return rhythm {
            unit = "beats", 
            pattern = { 0, 1 },
            emit = "c5"
          }
        "#,
        "snare rhythm.lua",
    )
    .unwrap();

    let bass_rhythm = new_rhythm_from_string(
        beat_time,
        None,
        r#"
          local scale = scale("c5", "natural minor")
          return rhythm {
          unit = "1/8",
          pattern = pattern.from({ 1, 0.5, 1, 0 }, { 0, 1, 0, 0 }, { 1, 0, 1, 0 }, { 0, 1, 0, 1 }),
          gate = function (context)
              return context.pulse_value == 1.0
          end,
          emit = pattern.from(1, 3, 4, 1, 3, 4, -7):map(function(value)
            if value < 0 then
              return { key = scale.notes[-value] - 12, volume = 0.7 }
            else
              return { key = scale.notes[value], volume = 0.7 }
            end
          end
          )
        }
        "#,
        "bass rhythm.lua",
    )
    .unwrap();

    let fx_rhythm = new_rhythm_from_string(
        beat_time,
        None,
        r#"
          return rhythm {
            unit = "seconds",
            resolution = 8,
            offset = 8,
            emit = {
              note("c_4", "---", "---"):with_volume(0.2),
              note("---", "c_5", "---"):with_volume(0.25),
              note("---", "---", "f_5"):with_volume(0.2)
            }
          }
        "#,
        "fx rhythm.lua",
    )
    .unwrap();

    let chord_rhythm = new_rhythm_from_string(
        beat_time,
        None,
        r#"
          return rhythm {
            unit = "bars", 
            resolution = 4,
            emit = sequence( 
              note("c4", "d#4", "g4"),
              note("c4", "d#4", "f4"),
              note("c4", "d#4", "g4"),
              note("c4", "d#4", "a#4")
            ):with_volume(0.3)
          }
        "#,
        "chord rhythm.lua",
    )
    .unwrap();

    Phrase::new(
        beat_time,
        vec![
            RhythmSlot::from(kick_rhythm),
            RhythmSlot::from(snare_rhythm),
            RhythmSlot::from(bass_rhythm),
            RhythmSlot::from(fx_rhythm),
            RhythmSlot::from(chord_rhythm),
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
            black_box(phrase.clone());
        })
    });
    group.finish();
}

pub fn run(c: &mut Criterion) {
    let event_count = 5000;
    let mut group = c.benchmark_group("Scripted Phrase");
    group.measurement_time(std::time::Duration::from_secs(10));
    let phrase = create_phrase();
    group.bench_function("Run", |b| {
        b.iter(|| {
            let sample_time = SampleTime::MAX;
            let mut phrase = phrase.clone();
            let mut num_events = 0;
            while let Some(event) = phrase.run_until_time(sample_time) {
                black_box(event);
                num_events += 1;
                if num_events >= event_count {
                    break;
                }
            }
        })
    });
    group.finish();
}

// ---------------------------------------------------------------------------------------------

criterion_group! {
    name = scripted;
    config = Criterion::default().sample_size(50);
    targets = create, clone, run
}
