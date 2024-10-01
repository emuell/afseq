# Rhythm 

## What is a Rhythm?

A rhythm is driven by a configurable second or musical beat [**TimeBase**](./timebase.md).

Each rhythm is composed of 3 units: [**Pattern**](./pattern.md) -> [**Gate**](./gate.md) -> [**Emitter**](./emitter.md). 


```md
~ Timebase ~ 
Define basic time unit and step length of a pulse.
  e.g. 1 second or a quarter note or bar...

┌------------┐
│  Pattern   |
└------------┘
Define basic rhythmical pattern as pulse train.
  e.g. `{ 0, 0.5, 0, 1, { 1, 1, 1 } }` where the inner {} is a subdivision
  pulse that "crams" a subset of pulses into the duration of a single pulse.
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
Generates a single event for a single pulse.
  e.g. constantly trigger a `C-4` note for each pulse.
   - or emit a single note sequence of notes for each pulse -> an arpeggio.
   - or emit a sequence of chords -> a chord progression.
   - or emit a Tidal Cycle. 
```

> By separating the **rhythmic** from the **tonal** (or parameter value) part of a musical sequence, each part of the sequence can be freely modified, composed and (re)combined.

We're basically treating music in two dimensions here: the *rhythmic* part as one dimension, and the *tonal* part as another. However, it's also possible to use just the emitter part of afseq, which can be particularly useful when using tidal cycles. This can be done by using a simple never-ending 1-valued train pulse as the input pattern, which then defines the time grid for the emitter.

All content in rhythms can be either be fixed -> e.g. a fixed Lua table of events, or can be or dynamic -> a Lua function which [generates](./generators.md) events on the fly. 

Further, all dynamic functions or generators can also be controlled, automated via [parameters](./parameters.md) to change their behaviour at runtime.  

## Examples:

A rhythm with a fixed pattern and emitter, using the default gate:

```lua
-- sequence of 1/4th c4 and two 1/8 c5 notes.
rhythm {
  unit = "1/4",
  pattern = { 1, { 1, 1 } },
  emit = { "c4", "c5", "c5" }
}
```

A rhythm with default pattern and gate, emitting a Tidal Cycle:

```lua
-- emit a tidal cycle ever bar
rhythm {
  unit = "1/1",
  emit = cycle("a4 e4@2 <c4 c5>")
}
```

A rhythm with a dynamic pattern generator:

```lua
-- maybe trigger a c4 on every 2nd 1/4.
rhythm {
  unit = "1/4",
  pattern = function (context) 
    if (context.pulse_step % 2 == 1) then
      return math.random() > 0.5 and 1 or 0
    else
      return 1
    end 
  end,
  emit = "c4"
}
```

A rhythm with a fixed pattern, dynamic seeded probablility gate and dynamic emitter:

```lua
-- change for other veriations or set to nil to get really random behavior 
local seed = 2234

-- maybe emits events, using pulse values as probability
return rhythm {
  unit = "1/8",

  pattern = { 1, { 1, 0.1, 0.5 }, 1, { 1, 0, 0, 0.5 } },

  gate = function (context)
    -- create a local random number generator for the probability
    local rand = math.randomstate(seed)
    return function (context)
      -- use pulse value as trigger probability
      return context.pulse_value >= rand() 
    end
  end,
  
  emit = function (context)
    -- create a local random number generator for the humanizing delay
    local rand = math.randomstate(seed)
    return function (context)
      local volume, delay = 1.0, 0.0
      if context.pulse_time < 1 then
        -- lower volume and add a delay for events in sub patterns
        volume = context.pulse_time
        delay = rand() * 0.05
      end
      return { key = "c4", volume = volume, delay = delay }
    end
  end,
}
```

See [Examples](../examples/README.md) in this guide for more advanced and guided examples.
