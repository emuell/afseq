# afseq

***afseq***, also known as **nerdo-rhythm**, is an experimental imperative-style music sequence generator engine. 

It allows you to programmatically create music sequences either in plain Rust as library (*static, compiled*) or in Lua as a scripting engine (*dynamic, interpreted*). So it's also suitable for [live coding music](https://github.com/pjagielski/awesome-live-coding-music). 

In addition to its imperative event generator approach, it also supports the creation of musical events using [tidalcycle](https://tidalcycles.org/)'s mini-notation.

afseq is part of the [afplay](https://github.com/emuell/afplay) crates. This crate only deals with the *generation of raw musical events*. It does not generate audio. You must use an application with built-in support for afseq to use it.


## Conceptional Overview

afseq generates musical sequences using three distinct components, stages.

- **Pattern**: dynamic pulse train generator to define the rhythmical pattern.
- **Gate**: optional pulse train filter between pattern and emitter.
- **Emitter**: dynamic note or parameter value generator which gets triggered by the pattern.

By separating the rhythmical pattern from the tonal part of a musical sequence, each part of a sequence can be freely modified, composed and (re)combined.

## Documentation & Guides

Read the [Scripting Book](https://emuell.github.io/afseq/).
It contains an introduction, guides, full Lua API documentation and a few script examples.

The Rust backend uses standard Rust documentation features. The docs are currently not hosted online, but you can generate them locally via `cargo doc --open`.


## Demo Applications

See [`examples/play.rs`](./examples/play.rs) for an example using only Rust: it defines and plays a little music thing. The content can only be changed at compile time.

See [`examples/play-script.rs`](./examples/play-script.rs) for an example using the Lua API: it also defines and plays a little music thing, but its content can be added/removed and changed on the fly to do some basic live music hacking.  


## Acknowledgements

Thanks to [unlessgames](https://github.com/unlessgames) for adding the Tidal Cycles mini-notation to afseq.


## Contribute

Patches are welcome! Please fork the latest git repository and create a feature or bugfix branch.


## Licence

afseq is distributed under the terms of the [GNU Affero General Public License V3](https://www.gnu.org/licenses/agpl-3.0.html).
