# Guide

A `Rhythm` is the main building block in afseq. It lets you define when and what to play.

afseq consumes [Lua script](https://www.lua.org/) files that define rhythms as specified in the [API documentation](../API/).

## Components

- [TimeBase](./timebase.md) defines the time unit of a rhythm.
- [Pattern](./pattern.md) -> [Gate](./gate.md) -> [Emitter](./emitter.md) do perform the basic event generation in 3 stages.
- [Parameters](./parameters.md) change behaviour of components during runtime.

By separating the rhythmical pattern from the tonal part of a musical sequence, each part of a sequence can be freely modified, composed and (re)combined.

All content in rhythms can be either static, a Lua table of events, or dynamic, a Lua function that [generates](../extras/generators.md) events on the fly. 

Dynamic functions or generators can also be controlled, automated via [input parameters](./parameters.md) to change their behaviour at runtime in response to user input (e.g. MIDI controllers, DAW parameter automation). This also allows creating more flexible rhythm templates. 

## Examples

A rhythm which emits a note every beat.

```lua
-- trigger a c4 note every beat.
return rhythm {
  unit = "1/4",
  pattern = { 1 },
  emit = { "c4" }
}
```

A rhythm with a pattern with sub divisions.

```lua
-- sequence of 1/4th c4 and two 1/8 c5 notes.
return rhythm {
  unit = "1/4",
  pattern = { 1, { 1, 1 } },
  emit = { "c4", "c5", "c5" }
}
```

A rhythm emitting a Tidal Cycle.

```lua
-- emit a tidal cycle every bar
return rhythm {
  unit = "1/1",
  emit = cycle("a4 e4@2 <c4 c5>")
}
```

A rhythm, using a Lua function as dynamic pattern generator.

```lua
-- maybe trigger a c4 on every 2nd 1/4.
return rhythm {
  unit = "1/4",
  pattern = function(context) 
    if context.pulse_step % 2 == 1 then
      return math.random() > 0.5 and 1 or 0
    else
      return 1
    end 
  end,
  emit = "c4"
}
```

A rhythm with a static pattern and emitter and a dynamic seeded probablility gate.

```lua
-- change for other variations or set to nil to get *really* random behavior 
local seed = 12345678

return rhythm {
  unit = "1/4",
  pattern = { 1, { 1, 0.1, 0.2 }, 1, { 1, 0, 0, 0.3 } },
  gate = function(init_context)
    local rand = math.randomstate(seed)
    return function(context)
      -- use pulse values as trigger probabilities
      return context.pulse_value >= rand() 
    end
  end,
  emit = { "c4'-7 v0.6", "d#4'M v0.4", "g4'- v0.3" },
}
```

See [Examples](../examples/README.md) in this guide for more advanced and guided examples.
