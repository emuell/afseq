# Cycles

In addition to static arrays of [notes](./notes&scales.md) or dynamic [generator functions](../extras/generators.md), emitters in afseq can also emit cycles using the [tidal cycles mini-notation](https://tidalcycles.org/docs/reference/mini_notation/)

## Introduction

Cycles let you create repeating musical patterns using a simple text notation. Think of it like writing drum patterns or melodies in a compact, readable way.


```lua
-- Basic pattern with notes and rests
return cycle("c4 d4 <e4 g4> _")
```

This plays:
1. C4
2. D4
3. Alternate between E4 and G4
4. play no event: `_` means rest

## Pattern Basics

Key symbols to know:

| Symbol | Meaning                          | Example             |
|--------|----------------------------------|---------------------|
| ` `    | Separates steps                  | `c4 d4`             |
| `,`    | Parallel patterns                | `[c4,e4], [g4,a4]`  |
| `< >`  | Alternates between values        | `<c4 e4 g4>`        |
| `\|`   | Random choice                    | `c4\|d4\|e4`        |
| `*`    | Repeat                           | `c4*4`              |
| `_`    | Elongate                         | `c4 _ d4`           |
| `~`    | Rest                             | `c4 ~ d4`           |

<br>

> [!TIP]
> Please see [Tidal Cycles Mini-Notation Reference](https://tidalcycles.org/docs/reference/mini_notation/) for a complete overview of the cycle notation.

## Basic Examples

Basic scale:
```lua
return cycle("c4 d4 e4 f4 g4 a4 b4 c5")
```

Drum pattern:
```lua
return cycle("[c1 ~ e1 ~]*2")
```

Random melody:
```lua
return cycle("[c4|d4|e4|f4|g4|a4]*8")
```

## Combining with Patterns

Control when patterns play using `rhythm`:

```lua
return rhythm {
  unit = "bars",       -- Timing unit
  pattern = {1, 0},    -- Play on odd bars only
  emit = cycle("c4 d4 e4 f4")
}
```

## Advanced Usage

### Limitations

There's no exact specification for how tidal cycles work, and it's constantly evolving, but at the moment we support the mini notation as it works in Tidal, with the following limitations and changes: 

* Stacks and random choices are valid without brackets (`a | b` is parsed as `[a | b]`)

* `:` sets the instrument or remappable target instead of selecting samples but also allows setting note attributes such as instrument/volume/pan/delay (e.g. `c4:v0.1:p0.5`)

* In bjorklund expressions, operators *within* and on the *right side* are not supported (e.g. `bd(<3 2>, 8)` and `bd(3, 8)*2` are *not* supported)

### Timing 

The base time of a pattern in tidal is specified as *cycles per second*. In afseq, the time of a cycle instead is given in *cycles per pattern pulse units*. 

```lua
-- emits an entire cycle every beat
return rhythm {
  unit = "beats",
  emit = cycle("c4 d4 e4") -- tripplet
}
```

### Sequencing

An emitter in afseq gets triggered for each incoming non-gated pattern pulse. This is true for cycles are well and allows you to sequence entire cycles too. 

```lua
-- emit an entire cycle's every bar, then pause for two bars, then repeat
return rhythm {
  unit = "bars",
  pattern = { 1, 0, 0 },
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

### Note Attributes

You can set note attributes in cycle patterns using chained `:` expressions:

```lua
-- Set instrument (2), panning (-0.5), and delay (0.25)
cycle("d4:2:p-0.5:d0.25")

-- Set instrument (1) with alternating volumes (0.1, 0.2)
cycle("c4:1:<v0.1 v0.2>")

-- Set multiple attributes with randomization
cycle("c4:[v0.5:d0.1|v0.8]")
```

Supported note attributes are:
- Instrument: `:#X` - same as `:X`, without the `#`
- Volume: `:vX` - with X \[0.0-1.0\]
- Panning: `:pX` - with X \[-1.0 to 1.0\] 
- Delay: `:dX` - with X \[0.0-1.0\)

Note that `X` must be written as *floating point number* for volume, panning and delay:</br> `c4:p-1.0` and `c4:p.8` is valid, while `c4:p-1` **is not valid**!

### Mapping

Notes and chords in cycles are expressed as [note strings](./notes&scales.md#note-strings) in afseq. But you can also dynamically evaluate and map cycle identifiers using the cycle [`map`](../API/cycle.md#map) function.

This allows you, for example, to inject [input parameters](./parameters.md) into cycles or to use custom identifiers.

Using custom identifiers with a static map (a Lua table):

```lua
return cycle("[bd*4], [_ sn]*2"):map{ 
  bd = note("c4 #0"), 
  sn = note("g4 #1") 
}
```

Using custom identifiers with a dynamic map function (a Lua function):

```lua
return cycle("[bd*4], [_ sn]*2"):map(function(context, value)
  if value == "bd" then
    return note("c4 #0")
  elseif value == "sn" then
    return note("g4 #1")
  end
end)
```

## Advanced Examples

Chord progression 

```lua
return cycle("[c'M g'M a'm f'M]/4")
```

A polyrhythm

```lua
return cycle("[C3 D#4 F3 G#4], [[D#3?0.2 G4 F4]/64]*63")
```

Alternate panning with note attributes

```lua
cycle("c4:<p-0.5 p0.0 p0.5>")
```

Mapped multi channel beats

```lua
-- use [=[ and ]=] as Lua multiline string
return cycle([=[
  [<h1 h2 h2>*12],
  [kd ~]*2 ~ [~ kd] ~,
  [~ s1]*2,
  [~ s2]*8
]=]):map({
  kd = "c4 #0", -- Kick
  s1 = "c4 #1", -- Snare
  s2 = "c4 #1 v0.1", -- Ghost snare
  h1 = "c4 #2", -- Hat
  h2 = "c4 #2 v0.2", -- Hat
})
```

Dynamically mapped roman chord numbers with user defined scale

```lua
return rhythm {
  unit = "1/1",
  resolution = 4,
  inputs = {
    parameter.enum("mode", "major", { "major", "minor" })
  },
  emit = cycle("I V III VI"):map(
    function(init_context, value)
      local s = scale("c4", init_context.inputs.mode)
      return function(context, value)
        return value ~= "_" and s:chord(value) or value
      end
    end
  )
}
```