# Emitter 

A rhythm's [`emitter`](../API/rhythm.md#emit) generates events for incoming pulse values. Just like the pattern, it can be made up of a static list of events or it can be a dynamic generator - a Lua function. 

In addition to dynamic Lua functions, you can also use a tidal [cycle](./cycles.md) as an emitter.

The default emitter generates a single middle C note value for each incoming pulse.    


## Event Types

Currently afseq only supports monophonic or polyphonic *note events* as emitter output. This is likely to change in the future to allow other musically interesting events to be emitted. 

Note values can be expressed as:
- Raw integer values like `48`, which are interpreted as MIDI note numbers. 
- Raw note strings such as `"c4"` (single notes) or `"d#4'maj"` (chords).
- Lua note tables like `{ key = 48, volume = 0.1 }`.
- Lua API note objects such as `note(48):volume(0.1)` or `note("c4", "g4")` or `note("c4'min"):transpose({-12, 0, 0})`.
- Lua `nil` values, empty table `{}` or `"-"` strings are interpreted as rests.
- The string `"off"` or `"~"` is interpreted as note off. 

See [notes & scales](./notes&scales.md) for more information about the different ways to create and manipulate notes and chords.


## Static Emitters

The simplest form of a emitter is a Lua table (array) of note or nil values (a rest).

Static emitter arrays define **note event sequences**. Each incoming, possibly filtered, [gated](./gate.md) pulse from the [pattern](./pattern.md) picks the next event in the sequence, then moves on in the sequence. Sequences are repeated as long as the pattern is running.  

» `emit = { "c4", "d#4", "g4" }` *arpeggio - sequence*

» `emit = { { "c4", "d#4", "g4" } }` *single chord - single event*

To ease distinguishing polyponic contents, use [`sequence`](../API/sequence.md) and [`note`](../API/note.md):

» `emit = sequence("c4", "d#4", "g4")` *arpeggio - sequence*

» `emit = note("c4", "d#4", "g4")` *single chord - single event*


## Dynamic Emitters

Dynamic emitter functions return **single note events**. Each incoming, possibly filtered, [gated](./gate.md) impulse from the [pattern](./pattern.md) will trigger the emit function to create the next event as long as the pattern is running.   

» `emit = function(context) return math.random() > 0.5 and "c4" or "c5" end` *randomly emit c4 or c5 notes*

» `emit = function(context) return context.pulse_count % 2 == 1 and "c4" or "c5" end` *alternate c4 and c5 notes*

See API docs for [context](../API/rhythm.md#EmitterContext) for more info about the context passed to dynamic functions. 


## Cycle Emitters

Cycle emitters emit **a whole cycle** for a single pulse. So any incoming, possibly filtered, [gated](./gate.md) pulse from the [pattern](./pattern.md) will trigger a full cycle as long as the pattern is running.   

» `emit = cycle("[c4, d#4, g4]")` *a single chord*

You probably won't use custom patterns or gate functions with cycles, but it's possible to sequence entire cycles with them, or use cycles as single note generators too:

» `emit = cycle("[c4 d#4 g4]")` *arpeggio*

» `emit = cycle("[c4 <d#4 d4> g4|g5]")` *arpeggio with variations*

See [cycles](./cycles.md) for more info about Tidal Cycles support in afseq. 

## Examples

Sequence of c4, g4 notes.

```lua
rhythm {
  emit = { "c4", "g4" }
}
```

Chord of c4, d#4, g4 notes.
```lua
rhythm {
  emit = sequence(
    { "c4",  "d#4",  "g4"  }, -- or "c4'min"
    { "---", "off",  "off" }
  ) 
}
```

Sequence of c4, g4 with volume 0.5.
```lua
rhythm {
  emit = sequence{"c4", "g4"}:volume(0.5)
}
```


Stateless function.
```lua
rhythm {
  emit = function(context)
    return 48 + math.random(1, 4) * 5
  end
}
```

Stateful generator.
```lua
rhythm {
  emit = function(initial_context)
    local count, step, notes = 1, 2, scale("c5", "minor").notes
    ---@param context EmitterContext
    return function(context)
      local key = notes[count]
      count = (count + step - 1) % #notes + 1
      return { key = key, volume = 0.5 }
    end
  end
}
```

Note pattern using the "pattern" lib.
```lua
local tritone = scale("c5", "tritone")
return rhythm {
  emit = pattern.from(tritone:chord(1, 4)):euclidean(6) +
    pattern.from(tritone:chord(5, 4)):euclidean(6)
}
```

Tidal cycle.
```lua
rhythm {
  emit = cycle("<[a3 c4 e4 a4]*3 [d4 g3 g4 c4]>")
}
```



See [generators](../extras/generators.md) for more info about stateful generators.

