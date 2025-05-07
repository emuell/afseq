# Welcome

... to the afseq scripting guide! ***afseq***, also known as **nerdo-rhythm**, is an experimental imperative music sequence generator engine.

It allows creating music sequences programmatically using either plain Rust as a library (*static, precompiled*) or Lua as a scripting engine (*dynamic, runtime interpreted*) for [live music coding](https://github.com/pjagielski/awesome-live-coding-music).

This book only covers the Lua API for live music coding. For instructions and examples on creating rhythms in plain Rust, see the [afseq crate docs](https://github.com/emuell/afseq).


## Key Features

`Programmatic`
: Build musical patterns using Lua scripts or Rust code.<br>
`Modular`
: Combine and reuse rhythm components dynamically.<br>
`Flexible`
: Create anything from simple beats to complex nested rhythms.<br>
`Generative`
: Make evolving compositions that change over time.<br>
`Dynamic`
: Modify patterns live, during playback.<br>
`External Control`
: Connect with MIDI/OSC controllers for hands-on parameter tweaking.<br>
`Template Rhythms`
: Create user configurable rhythm patterns.<br>
`Tidal Cycles`
: Use familiar Tidal Cycles mini-notation for rapid pattern creation.<br>


## Installation

afseq is a Rust *library* that deals with raw musical event generation only. It does not generate any audio. You must use an application with built-in support for afseq to use it. 

If you are familiar with Rust, you can also use `play-script.rs` from the [examples](https://github.com/emuell/afseq/tree/master/examples) in the git repository to test out afseq scripts using a basic sample player.


## Getting Started

[Quickstart](./quickstart.md)
: Start with a quick introduction through practical examples.<br>
[Guide](./guide/)
: Get into details with a more comprehensive documentation.<br>
[Examples](./examples/)
: More advanced examples, created and explained step by step.<br>
[Reference](./API/)
: Read raw technical specifications of the Lua API.


