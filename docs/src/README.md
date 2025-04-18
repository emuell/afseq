# Welcome

... to the afseq scripting guide!


## Introduction

***afseq***, also known as **nerdo-rhythm**, is an experimental imperative-style music sequence generator engine. 

It allows you to programmatically create music sequences either in plain Rust as library (*static, compiled*) or in Lua as a scripting engine (*dynamic, interpreted*). So it's also suitable for [live coding music](https://github.com/pjagielski/awesome-live-coding-music). 

In addition to its imperative event generator approach, it also supports the creation of musical events using [tidalcycle](https://tidalcycles.org/)'s mini-notation.


## Installation

afseq is a Rust *library* that deals with raw musical event generation only. It does not generate any audio. You must use an application with built-in support for afseq to use it. 

You can also use `play-script.rs` from the [examples](https://github.com/emuell/afseq/tree/master/examples) in the git repository to test out afseq scripts using a basic sample player that plays a sample from the example assets folder using the script which has the same name as the audio file. 


## Scripting

afseq uses [Lua](https://www.lua.org/) as a scripting language to dynamically generate content. 

If you're not familiar with Lua, don't worry. Lua is very easy to pick up if you have used another imperative programming language before, and fortunately there are great tutorials out there, such as [this one](https://www.lua.org/pil/1.html).


## Creating Rhythms

Ready to program some music? Then let's dive into the next chapter which will give you an overview of the overall architecture of a **rhythm**, the main building block in afseq.

---

*Note: This guide covers the afseq Lua scripting API. For instructions on creating rhythms in plain Rust, see the [afseq crate docs](https://github.com/emuell/afseq).*
