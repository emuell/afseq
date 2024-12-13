# Cycles

In addition to static arrays of [notes](./notes&scales.md) or dynamic [generator functions](../extras/generators.md), emitters in afseq can also emit cycles using the tidal cycles [mini-notation](https://tidalcycles.org/docs/reference/mini_notation/).


[Tidal Cycles](https://tidalcycles.org/) allows you to make patterns with code using a custom functional approach. It includes a language for describing flexible (e.g. polyphonic, polyrhythmic, generative) sequences of sounds, notes, parameters, and all kind of information.

## Usage

To create cycles in afseq, use the [`cycle`](../API/cycle.md#cycle) function in the emitter and pass it a mini-notation as string.

\> `emit = cycle("c4 d#4 <a5 g5> _")`

> [!NOTE]
> Please see [Tidal Cycles Mini-Notation Reference](https://tidalcycles.org/docs/reference/mini_notation/) for a complete overview of the cycle notation.

### Limitations

There's no exact specification for how tidal cycles work, and it's constantly evolving, but at the moment we support the mini notation as it works in Tidal, with the following limitations and changes: 

* Stacks and random choices are valid without brackets (`a | b` is parsed as `[a | b]`)

* Operators currently only accept numbers on the right side (`a3*2` is valid, `a3*<1 2>` is not)

* `:` - Sets the instrument or remappable target instead of selecting samples

### Timing 

The base time of a pattern in tidal is specified as *cycles per second*. In afseq, the time of a cycle instead is given in *cycles per pattern pulse units*. 

```lua
-- emits an entire cycle every bar
return rhythm {
  unit = "bars",
  emit = cycle("c d e f")
}
```

### Sequencing

An emitter in afseq gets triggered for each incoming non-gated pattern pulse. This is true for cycles are well and allows you to sequence entire cycles too. 

```lua
-- emit an entire cycle's every bar, then pause for a bar, then repeat
return rhythm {
  unit = "bars",
  pattern = {1, 0},
  emit = cycle("c d e f")
}
```

You can also use the mini notation to emit single notes only, making use of tidal's note alternating and randomization features only: 

```lua
-- emit a single note from a cycle in an euclidean pattern
return rhythm {
  unit = "beats",
  pattern = pattern.euclidean(5, 8),
  emit = cycle("<c d e f g|b>")
}
```

### Seeding

afseq's general random number generator is also used in cycles. So when you seed the global number generator, you can also seed the cycle's random operations with `math.randomseed(12345)`.  


### Mapping

Notes and chords in cycles are expressed as [note strings](./notes&scales.md#note-strings) in afseq. But you can also dynamically evaluate and map cycle identifiers using the cycle [`map`](../API/cycle.md#map) function.

This allows you, for example, to inject [parameters](./parameters.md) into cycles or to use custom identifiers.

Using custom identifiers with a static map (a Lua table):

```lua
return rhythm {
  unit = "bars",
  emit = cycle("[bd*4], [_ sn]*2"):map({ 
    ["bd"] = note("c4 #0"), 
    ["sn"] = note("g4 #1") 
  })
}
```

Using custom identifiers with a dynamic map function (a Lua function):

```lua
return rhythm {
  unit = "bars",
  emit = cycle("[bd*4], [_ sn]*2"):map(function(context, value)
    if value == "bd" then
      return note("c4 #0")
    elseif value == "sn" then
      return note("g4 #1")
    end
  end)
}
```

## Examples

A simple polyrhythm

```lua
return rhythm {
  unit = "1/1",
  emit = cycle("[C3 D#4 F3 G#4], [[D#3?0.2 G4 F4]/64]*63")
}
```

Mapped multi channel beats

```lua
return rhythm {
  unit = "1/1",
  emit = cycle([[
    [<h1 h2 h2>*12],
    [kd ~]*2 ~ [~ kd] ~,
    [~ s1]*2,
    [~ s2]*8
  ]]):map({
    kd = "c4 #0", -- Kick
    s1 = "c4 #1", -- Snare
    s2 = "c4 #1 v0.1", -- Ghost snare
    h1 = "c4 #2", -- Hat
    h2 = "c4 #2 v0.2", -- Hat
  })
}
```

Dynamically mapped roman chord numbers with user defined scale

```lua
return rhythm {
  unit = "1/1",
  resolution = 4,
  inputs = {
    parameter.enum("mode", "major", {"major", "minor"})
  },
  emit = cycle("I V III VI"):map(
    function(context, value)
      local s = scale("c4", context.inputs.mode)
      return function(context, value)
        return value ~= "_" and s:chord(value) or value
      end
    end
  )
}
```
