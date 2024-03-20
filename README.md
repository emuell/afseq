# afseq

## Overview

**afseq**, aka **NerdoRhythm**, is an experimental, dynamic, imperative and functional music sequence generator for Rust and Lua. It allows you to create musical sequences either in plain *Rust* (static, precompiled) or in *Lua* (dynamic, real-time). 

afseq is part of the [afplay](https://github.com/emuell/afplay) crates.

This part of the afplay crates deals **only** with raw musical event generation. See the `examples` folder for how to combine a simple playback engine using `afplay` with `afseq` to create a simple sequencer playback engine. 

## Demo applications

See `examples/play.rs` for an example using rust only: it defines and plays a little music thing. The content can only be changed at compile time.

See `examples/play-script.rs` for an example using the Lua API: it also defines and plays a little music thing, but its contents can be added/removed and changed on the fly to do music live coding.  
    
## Components

A musical event sequence in afseq is divided into 3 separate stages:

- **Pattern**: dynamic pulse train generator to define the basic rhythm.
- **Gate**: optional pulse train filter between pattern and emitter. 
- **Emitter**: dynamic note or parameter value generator triggered by the pattern.

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
   - or emit pulse values as parameter change value.
```

### TimeBase

The TimeBase represents the unit of time for the rhythm, either in musical beats or wall-clock time (seconds, ms). It defines the unit and duration of a step in the sequence.

The default time unit of rhythm is one beat. 

### Pattern

A Pattern is a sequence of pulses that defines the basic rhythm. It consists of a list of pulses with possible subdivisions, an optional number of repeats and an optional time offset. A pattern can generate pulses using a specific algorithm, such as a Euclidean rhythm or using a fixed, predefined pattern or using a dynamic generator - a function.

The default pattern of a rhythm is a never ending pulse train of 1's.

### Gate

A Gate is a filter that determines whether or not an event should be emitted based on a pulse value. The gate can be used to filter out pulse events or to add randomness to the rhythm. A gate can be a predefined gate from the library or a dynamic filter - a function.

The default gate in a rhythm is a probability gate, which passes 1 and 0 events as they are, and values in between (0 - 1) with a probability. 

### Emitter

An Emitter is an iterator that generates events for each pulse value. It can be made up of a fixed list of events, or it can be a dynamic generator - a function. 

The default emitter spits out middle C note values for each pulse.    

## Examples

### Rust 

```rust
TODO
```

### Lua 

```lua
TODO
```

## License

afseq is distributed under the terms of both the MIT license and the Apache License (Version 2.0).

* Apache License, Version 2.0, ([LICENSE-APACHE](LICENSE-APACHE) or http://www.apache.org/licenses/LICENSE-2.0)
* MIT license ([LICENSE-MIT](LICENSE-MIT) or http://opensource.org/licenses/MIT)
