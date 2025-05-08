# Guide

A `Rhythm` is the main building block in afseq. It lets you define when and what to play.

afseq consumes [Lua script](https://www.lua.org/) files that define rhythms as specified in the [API documentation](../API/).

## Components

- [TimeBase](./timebase.md) defines the **time unit** of a rhythm.
- [Pattern](./pattern.md) → [Gate](./gate.md) → [Emitter](./emitter.md) perform the basic **event generation** in 3 stages.
- [Parameters](./parameters.md) change behavior of components during runtime.

All content in rhythms can be either **static** or **dynamic**:

- **Static** content is defined as a Lua table of events. There are various helpers included in the API, such as [note scales, chords](./notes&scales.md), and [pattern creation tools](../API/pattern.md).

- **Dynamic** content is generated on the fly by Lua functions while the rhythm runs. [Generators](../extras/generators.md) are functions with local state, which can e.g. be used to apply replicable [randomization](../extras/randomization.md).

**Cycle** emitters use the Tidal [Cycles](./cycles.md) mini-notation to create patterns in a flexible, condensed form.

## Examples

See [Quickstart](../quickstart.md) for a set of simple examples to start and to play around with. 

The [Examples](../examples/README.md) section contains more advanced and guided examples. 
