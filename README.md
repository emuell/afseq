# afseq

## Overview

**afseq**, aka **NerdoRhythm**, is an experimental, dynamic, imperative and functional music sequence generator for Rust and Lua. 

It allows you to create music sequences either in plain Rust (static, precompiled) or in Lua (dynamic, real-time). In addition to a custom imperative event generator via the [rhythm config](./types/nerdo/library/rhythm.lua), it also supports creating events using the [Tidal Cycles mini-notation](https://tidalcycles.org/docs/reference/mini_notation/) via the [cycle function](./types/nerdo/library/cycle.lua).

afseq is part of the [afplay](https://github.com/emuell/afplay) crates.

This part of the afplay crates deals **only** with raw musical event generation. It does not generate audio. See the `examples` folder for how to combine a simple playback engine using `afplay` with `afseq` to create a simple sequencer playback engine. 

## Demo applications

See `examples/play.rs` for an example using rust only: it defines and plays a little music thing. The content can only be changed at compile time.

See `examples/play-script.rs` for an example using the Lua API: it also defines and plays a little music thing, but its contents can be added/removed and changed on the fly to do music live coding.  
    
## Components

A Rhythm is composed of 3 units in afseq:

- **Pattern**: dynamic pulse train generator to define the rhythmical pattern.
- **Gate**: optional pulse train filter between pattern and emitter. 
- **Emitter**: dynamic note or parameter value generator which gets triggered by the pattern.

By separating the *rhythmic* from the *tonal* (or parameter value) part of a musical sequence, each part of the sequence can be freely modified, composed and (re)combined. 

*We're basically treating music in two dimensions here: the rhythmic part as one dimension, and the tonal part as another.* 

However, it's also possible to use just the emitter part of afseq, writing both parts in one *dimension* only. This can be done by using a simple never ending 1-valued train pulse as the input pattern, which defines the time grid for the emitter.

```
+++ Rhythm +++

~ Timebase ~ 
Define basic time unit and step length of a pulse.
  e.g. 1 second or a quarter note or bar...

┌------------┐
│  Pattern   |
└------------┘
Define basic rhythmical pattern as pulse train.
  e.g. `[0, 0.5, 0, 1, [1, 1, 1]]` where the inner [] is a subdivision that 
  "crams" a subset of pulses into the duration of a single pulse.
      ↓
┌------------┐
│    Gate    |
└------------┘
Passes or suppresses pattern pulses.
  e.g. Probability Gate: pass 1s directly, skip 0s, values in range (0 - 1) 
  are passed with the pulse value as probability.
      ↓
┌-------------┐
│   Emitter   |
└-------------┘
Generate events in a rhythmical pattern.
  e.g. constantly trigger C-4, a single note for each pulse.
   - or emit a single note sequence of notes for each pulse -> an arpeggio.
   - or emit a sequence of chords -> a chord progression.
   - or emit a todal cycle. 
```

### TimeBase

The TimeBase represents the unit of time for the rhythm, either in musical beats or wall-clock time (seconds, ms). It defines the unit and duration of a step in the sequence.

The default time unit of rhythm is one beat. 

### Pattern

A Pattern is a sequence of pulses that defines the musical sequence's rhythm. It consists of a list of pulses with possible subdivisions, an optional number of repeats and an optional time offset. A pattern can generate pulses using a specific algorithm, such as a Euclidean rhythm or using a fixed, predefined pattern, or by using a dynamic generator - a function.

The default pattern of a rhythm is a never ending pulse train of 1's.

### Gate

A Gate is a filter that determines whether or not an event should be emitted based on a pulse value. The gate can be used to filter out pulse events or to add randomness to the rhythm. A gate can be a predefined gate from the library or a dynamic filter - a function.

The default gate in a rhythm is a threshold gate, which passes all pulse values > 0. 

### Emitter

An Emitter is an iterator that generates events for each pulse value. It can be made up of a fixed list of events, tidal cycles, or it can be a dynamic generator - a function. 

The default emitter spits out middle C note values for each pulse.    

## Examples

### Rust 

The rust API uses Fluent interfaces to build rhythms.

```rust
use afseq::prelude::*;

fn main() -> Result<(), Box<dyn std::error::Error>> {

    // events can tagged with ids to e.g. trigger different instruments
    let KICK = InstrumentId::from(1);
    let SNARE = InstrumentId::from(2);
    let SYNTH = InstrumentId::from(2);

    // define a time base for the rhythm
    let beat_time = BeatTimeBase {
        beats_per_min: 130.0,
        beats_per_bar: 4,
        samples_per_sec: 44100,
    };

    // create a kick pattern in a beat time grid
    let kick_rhythm = beat_time
        .every_nth_beat(1.0)
        .with_instrument(KICK)
        .with_pattern(
            vec![
                Pulse::from(1.0), // Bar 1
                Pulse::from(vec![0.0, 1.0]), // divide beat into two 1/8th
                Pulse::from(0.0),
                Pulse::from(0.0),
                Pulse::from(1.0), // Bar 2
                Pulse::from(vec![0.0, 1.0]),
                Pulse::from(0.0),
                Pulse::from(0.0),
                Pulse::from(1.0), // Bar 3
                Pulse::from(vec![0.0, 1.0]),
                Pulse::from(0.0),
                Pulse::from(0.0),
                Pulse::from(1.0), // Bar 4
                Pulse::from(vec![0.0, 1.0]),
                Pulse::from(vec![0.0, 1.0]),
                Pulse::from(vec![0.0, 1.0, 0.0, 0.0]),
            ]
            .to_pattern(),
        )
        .trigger(new_note_event("C_5"));

    // trigger a snare every two beats with an offset of a beat
    let snare_rhythm = beat_time
        .every_nth_beat(2.0)
        .with_offset(BeatTimeStep::Beats(1.0))
        .with_instrument(SNARE)
        .trigger(new_note_event("C_5"));

    // trigger chords every 4 bars
    let chord_rhythm = beat_time
        .every_nth_bar(4.0)
        .with_instrument(SYNTH)
        .trigger(new_polyphonic_note_sequence_event(vec![
              vec![
                  new_note(("C 4", None, 0.3)),
                  new_note(("D#4", None, 0.3)),
                  new_note(("G 4", None, 0.3)),
              ],
              vec![
                  new_note(("C 4", None, 0.3)),
                  new_note(("D#4", None, 0.3)),
                  new_note(("F 4", None, 0.3)),
              ],
              vec![
                  new_note(("C 4", None, 0.3)),
                  new_note(("D#4", None, 0.3)),
                  new_note(("G 4", None, 0.3)),
              ],
              vec![
                  new_note(("C 4", None, 0.3)),
                  new_note(("D#4", None, 0.3)),
                  new_note(("A#4", None, 0.3)),
              ],
          ]),
    );

    // combine patterns into a phrase to play them together
    let phrase = Phrase::new(
        beat_time,
        vec![
            RhythmSlot::from(kick_rhythm),
            RhythmSlot::from(snare_rhythm),
            RhythmSlot::from(chord_rhythm),
        ],
        BeatTimeStep::Bar(8.0),
    );

    // print first 100 events
    for (_rhythm_index, event) in phrase.into_iter().take(100) {
        println!("Event: {:?}", event);
    }

    Ok(())
}
```

### Lua in Rust 

The Lua API uses configuration tables.

```rust
use afseq::prelude::*;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    // events can tagged with ids to e.g. trigger different instruments
    let KICK = InstrumentId::from(1);
    let SNARE = InstrumentId::from(2);
    let SYNTH = InstrumentId::from(2);

    // define a time base for the rhythm
    let beat_time = BeatTimeBase {
        beats_per_min: 130.0,
        beats_per_bar: 4,
        samples_per_sec: 44100,
    };

    // create a kick pattern in a beat time grid
    let kick_rhythm = new_rhythm_from_string(
        beat_time,
        Some(KICK),
        r#"
          return rhythm {
            unit = "1/4", 
            pattern = { 
              1, { 0, 1 }, 0, 0, 
              1, { 0, 1 }, 0, 0, 
              1, { 0, 1 }, 0, 0, 
              1, { 0, 1 }, { 0, 1 }, { 0, 1, 0, 0 }
            },
            emit = "c5"
          }
        "#,
        "kick rhythm.lua",
    )?;

    // trigger a snare every two beats with an offset of a beat
    let snare_rhythm = new_rhythm_from_string(
        beat_time,
        Some(SNARE),
        r#"
          return rhythm {
            unit = "beats", 
            pattern = { 0, 1 },
            emit = "c5"
          }
        "#,
        "snare rhythm.lua",
    )?;

    // trigger chords every 4 bars
    let chord_rhythm = new_rhythm_from_string(
        beat_time,
        Some(SYNTH),
        r#"
          return rhythm {
            unit = "bars", 
            resolution = 4,
            emit = sequence( 
              note("c4", "d#4", "g4"),
              note("c4", "d#4", "f4"),
              note("c4", "d#4", "g4"),
              note("c4", "d#4", "a#4")
            ):volume(0.3)
          }
        "#,
        "chord rhythm.lua",
    )?;
  
    // combine patterns into a phrase to play them together
    let phrase = Phrase::new(
        beat_time,
        vec![
            RhythmSlot::from(kick_rhythm),
            RhythmSlot::from(snare_rhythm),
            RhythmSlot::from(chord_rhythm),
        ],
        BeatTimeStep::Bar(8.0),
    );

    // print first 100 events
    for (_rhythm_index, event) in phrase.into_iter().take(100) {
        println!("Event: {:?}", event);
    }

    Ok(())
}
```

### Lua 

The Lua API also contains various tools to ease creating patterns.

```lua
--trigger notes in an euclidean tripplet pattern
return rhythm {
  unit = "1/8",
  resolution = 3/2,
  pattern = pattern.euclidean(6, 16, 2),
  emit = { "c3", "c4 v0.5", "d3", "e4", "f4", "c2" }
}

--trigger notes in a seeded, random subdivision pattern
math.randomseed(23498)
return rhythm {
  unit = "1/8",
  pattern = { 1, { 0, 1 }, 0, 0.3, 0.2, 1, { 0.5, 0.1, 1 }, 0.5 },
  emit = { "c4" },
}
```

... and tools to ease working with chords and scales: 

```lua
-- trigger a chord sequence every 4 bars after 4 bars
return rhythm {
  unit = "bars",
  resolution = 4,
  offset = 1,
  emit = sequence("c4'm", note("g3'm7"):transposed({0, 12, 0, 0}))
}

-- trigger chord arpeggios from a tritone scale in euclidean patterns
local tritone = scale("c5", "tritone")
return rhythm {
  unit = "1/8",
  emit = pattern.from(tritone:chord(1, 4)):euclidean(6) +
    pattern.from(tritone:chord(5, 4)):euclidean(6)
}
```

Patterns and emitters can be Lua functions to create dynamic contents:

```lua
-- probability pattern with humanized notes
return rhythm {
  unit = "1/8",
  pattern = {0, 1, {1, 0.8}, 0, 1, {1, 0.8, 0.5}},
  emit = function (context)
    if context.pulse_time <= 0.5 then
      return note("c5 v0.5"):with_delay(math.random() * 0.05)
    else
      return "c5"
    end
  end
}

-- trigger different chords depending on the generator step count
return rhythm {
  unit = "1/1",
  resolution = 2/3,
  emit = function (context)
    local step = math.floor((context.step - 1) / 10)
    if step % 4 == 0 then 
      return note("c4'm", "c2", 'off') 
    else
      return note("c4'm7", "c2", (step % 3 == 0) and "f4" or "g4") 
    end
  end
}

-- notes can be generated using the Tidal Cycles mini-notation as well
-- each pattern pulse then triggers a cycle iteration
-- see https://tidalcycles.org/docs/reference/mini_notation/
return rhythm {
  unit = "1/1",
  emit = cycle("<c4 e4 g4> <e4 g4> <g4 [a4|c4] d5> <b4 [f#4|e5]>") 
}
```

See [example scripts](./examples/assets/) folder and [Lua API definitions](./types/) for more info and examples.


## Acknowledgments

Thanks to **[unlessgames](https://github.com/unlessgames)** for adding Tidal Cycles mini-notation to afseq.


## License

afseq is distributed under the terms of the [GNU Affero General Public License V3](https://www.gnu.org/licenses/agpl-3.0.html)
