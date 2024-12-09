# afseq

***afseq***, also known as **NerdoRhythm**, is an experimental imperative-style music sequence generator engine. In addition to the imperative event generator approach, it also supports the creation of musical events using [Tidal Cycle](https://tidalcycles.org/)'s mini-notation.

This allows you to programmatically create music sequences either in plain Rust (*-> static, compiled*) or in Lua (*-> dynamic, interpreted*). So it's also suitable for [live coding](https://github.com/pjagielski/awesome-live-coding-music) music. 

afseq is part of the [afplay](https://github.com/emuell/afplay) crates. This crate only deals with the *generation of raw musical events*. It does not generate audio. You must use an application with built-in support for afseq to use it. 

## Conceptional Overview

afseq creates rhythms. A rhythm is composed of 3 units:

- **Pattern**: dynamic pulse train generator to define the rhythmical pattern.
- **Gate**: optional pulse train filter between pattern and emitter.
- **Emitter**: dynamic note or parameter value generator which gets triggered by the pattern.

By separating the rhythmic from the tonal (or parameter value) part of a musical sequence, each part of the sequence can be freely modified, composed and (re)combined. We're basically treating music in two dimensions here: the rhythmic part as one dimension, and the tonal part as another

## Demo Applications

See [`examples/play.rs`](./examples/play.rs) for an example using only Rust: it defines and plays a little music thing. The content can only be changed at compile time.

See [`examples/play-script.rs`](./examples/play-script.rs) for an example using the Lua API: it also defines and plays a little music thing, but its content can be added/removed and changed on the fly to do some basic live music hacking.  

## Scripting Docs

Read the afseq [Scripting Book](https://emuell.github.io/afseq/).
It contains an introduction, guides, full Lua API documentation and a few script examples.

## Rust Docs

The Rust library uses standard Rust documentation features. The library currently published on crates.io, but you can generate the docs manually via:

```bash
cargo doc --features=scripting --open
```

## Contribute

Patches are welcome! Please fork the latest git repository and create a feature or bugfix branch.

## Acknowledgements

Thanks to **[unlessgames](https://github.com/unlessgames)** for adding the Tidal Cycles mini-notation to afseq.

## Licence

afseq is distributed under the terms of the [GNU Affero General Public License V3](https://www.gnu.org/licenses/agpl-3.0.html).
