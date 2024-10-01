# Welcome

... to the afseq scripting guide!

Note: This guide only covers the afseq Lua scripting API. Please see the [afseq crate docs]() for how to create rhythms in plain Rust.

### What is afseq?

**afseq**, aka **NerdoRhythm**, is an novel experimental **imperative and functional music sequence generator** for Rust and Lua. It allows you to create music sequences either in plain Rust (-> static, precompiled) or in Lua (-> dynamic, real-time). 

In addition to a custom imperative event generator approach, afseq also supports creating events using the [Tidal Cycles mini-notation](https://tidalcycles.org/docs/reference/mini_notation/).

afseq only deals with raw musical event generation. It does not generate audio. See the [examples](https://github.com/emuell/afseq/tree/master/examples) in the git repository for how to combine a simple playback engine with afseq to create a simple sequencer playback engine. The [play-script.rs](https://github.com/emuell/afseq/blob/master/examples/play-script.rs) example also can be used to test out afseq scripts, using a simple sample player 

The [next chapter](../guide/README.md) will give you an overview of the overall architecture of a `rhythm`, the main building block in afseq.
