# Quickstart

This guide will take you from basic patterns to advanced rhythmic sequencing through practical examples. Each example builds on previous concepts and includes comments to help you understand how things work.

> [!NOTE]
> afseq uses [Lua](https://www.lua.org/) as a scripting language. If you're not familiar with Lua, don't worry. Lua is very easy to pick up and fortunately there are great tutorials out there, such as [this one](https://w3schools.tech/tutorial/lua/index).


## Table of Contents
- [Basic Patterns](#basic-patterns)
- [Rhythm Variations](#rhythm-variations)
- [Using Tidal Cycles](#using-tidal-cycles)
- [Dynamic Patterns](#dynamic-patterns)
- [Advanced Techniques](#advanced-techniques)

## Basic Patterns

### Simple Quarter Note Pulse

```lua
-- The most basic rhythm: steady quarter notes
return rhythm {
    unit = "1/4", -- Quarter note timing grid
    emit = "c4"   -- Play middle C on each pulse
}
```
- TRY THIS: Change unit to `"1/8"` for eighth notes
- TRY THIS: Replace `"c3"` with `"e3"` or `"g3"` for different notes

This creates a steady pulse playing C4 on every quarter note. The `unit` parameter sets the timing grid, while `emit` defines what note to play.

### Alternating Notes

```lua
-- Create a pattern that alternates between notes
return rhythm {
    unit = "1/8",           -- Eighth note timing grid
    pattern = {1, 0, 1, 1}, -- Play-rest-play-play pattern
    emit = {"c3", "d3"}     -- Alternates between C3 and D3
}
```
- TRY THIS: Change pattern to `{1, 1, 0, 1}` for a different rhythm
- TRY THIS: Add more notes to emit like `{"c3", "d3", "e3", "g3"}`


Here, `pattern` controls when notes play (1) or rest (0). The `emit` parameter cycles through the provided notes when a step triggers.

### Simple Drum Pattern

```lua
-- Basic drum pattern with kick and hi-hat
return rhythm {
    unit = "1/16",        -- Sixteenth note grid
    pattern = {1, 0, 0, 0, 0, 0, 1, 0, 0, 0, 0, 0, 1, 0, 1, 0}, -- Kick pattern
    emit = {"c1", "f#3"}  -- C1 = kick, F#3 = hi-hat
}
```
- TRY THIS: Add a snare on beats 2 and 4 by adding "d2" to emit
- TRY THIS: Modify the pattern for different kick placements

This example creates a basic drum pattern using General MIDI notes where C1 is typically kick drum and F#3 is hi-hat.

## Rhythm Variations

### Subdivided Rhythm

```lua
-- Pattern with mixed note lengths
return rhythm {
    unit = "1/4",
    pattern = {1, {1, 1}},    -- One quarter note, then two eighth notes
    emit = {"c3", "d3", "e3"} -- C3 (quarter), D3, E3 (eighth notes)
}
```
- TRY THIS: Try more complex subdivisions like `{1, {1, {1, 1}}}`
- TRY THIS: Change the unit to `"1/8"` to make everything faster

Nested arrays in the pattern create subdivisions, allowing for more complex rhythms within the basic unit.

### Euclidean Rhythms

```lua
-- Distributes notes evenly across steps (common in many music traditions)
return rhythm {
    unit = "1/16",
    pattern = pattern.euclidean(3, 8),  -- 3 hits spread over 8 steps
    emit = "c1" -- Kick drum
}
```
- TRY THIS: Try different combinations like `(5, 8)` or `(7, 16)`
- TRY THIS: Use `pattern = pattern.euclidean(3, 8) + pattern.euclidean(5, 8)` to chain different pattern

Euclidean patterns distribute a number of notes evenly across steps, creating naturally pleasing rhythmic patterns found in many musical traditions. 

The [Pattern API](./API/pattern.md) contains various tools to programatically create patterns.

### Triplet Feel with Resolution

```lua
-- Create swing or triplet feel
return rhythm {
    unit = "1/8",
    resolution = 2/3, -- Triplet feel (3 notes in space of 2)
    emit = {"c3 v0.8", "e3 v0.1", "g3 v0.8"} -- v specifies volume, d delay, p panning
}
```
- TRY THIS: Change resolution to `"4/3"` for a different swing feel
- TRY THIS: Add `d` values between 0 and 1 to shuffle specific notes

The `resolution` parameter modifies the timing grid, enabling triplet feels, swing rhythms, and polyrhythms.

### Note Stacks and Chords

```lua
-- Polyphonic notes and chord notations
return rhythm {
    unit = "1/1",
    emit = {
      {"c4", "e4", "g4"},    -- c major by stacking notes
      "c4'M",                -- c major using ' chord notation
      note("c4'M"):transpose({12, 0, 0}), -- c major first inversion
      scale("c", "major"):chord(1)        -- c major from 1st degree of a c maj scale
    }
}
```
- TRY THIS: Add `v` values to chord strings to create more dynamics.
- TRY THIS: Use other chord modes than `M` such as `m`, `m5` or `+` or other degree values and scales.

A table of notes in a sequence `{}` allows emitting multiple notes at once. The `note()` function allows transforming existing note strings and chords can also be created from a `scale`. 

See [Notes and Scales](./guide/notes&scales.md) for more ways to create and manipulate notes and chords.

## Using Tidal Cycles

### Basic Cycle

```lua
-- Using tidal cycles notation for concise patterns
return rhythm {
    unit = "1/4", -- Emit a cycle every beat
    emit = cycle("c4 e4 g4") -- C major arpeggio
}
```

```lua
-- The simplified notation emits a cycle per bar
return cycle("c4 e4 g4") 
```

Tidal cycles provide a concise way to express patterns.

### Alternating Cycles

```lua
-- Switching between different patterns
return rhythm {
    unit = "1/4",
    emit = cycle("[c4 e4 g4]|[d4 f4 a4]") -- Randomly select one of two chords
}
```
- TRY THIS: Add more patterns with `|` like `[c4 e4 g4]|[d4 f4 a4]|[e4 g4 b4]`
- TRY THIS: Try simultaneous notes with square brackets `[c4 e4]`

The `|` operator in cycles randomly selects different patterns.

### Euclidean Rhythms in Cycles

```lua
-- Euclidean patterns in tidal cycles notation
return cycle("c4(3,8) e4(5,8) g4(7,8)")  -- Different Euclidean rhythms
```
- TRY THIS: Combine with alternation: `c4(3,8)|e4(5,8)`
- TRY THIS: Change the numbers for different distributions

Tidal cycles also support Euclidean patterns with the `(n,k)` notation, where `n` is the number of notes and `k` is the number of steps.

See [Cycles Guide](./guide/cycles.md) for more example and info about tidal cycles in afseq.

## Dynamic Patterns

### Random Note Selection

```lua
-- Randomly select notes from a list
local notes = {"c3", "d3", "e3", "g3"}
return rhythm {
    unit = "1/8",
    emit = function(context)
        return notes[math.random(#notes)] -- Pick random note from array
    end
}
```
- TRY THIS: Use notes from a specific scale with `local notes = scale("c4", "major").notes`
- TRY THIS: Add amplitude variation with `note(some_note):amplify(0.5 + math.random() * 0.5)`

Using functions for emitters enables dynamic behavior, like randomly selecting notes from a predefined set.

### Probability-Based Emitting

```lua
-- Emit notes with certain probability
return rhythm {
    unit = "1/8",
    pattern = {1, 1, 1, 1},  -- Regular pattern
    emit = function(context)
        if math.random() < 0.3 then  -- 30% chance to emit
            return "c4"
        end
    end
}
```
- TRY THIS: Vary probability by step position: `if math.random() < (context.step % 4) / 4 then`
- TRY THIS: Higher probability on downbeats: `if math.random() < ((context.pulse_step - 1) % 2 == 0 and 0.8 or 0.2) then`

This pattern uses a gate function to filter notes with a 30% probability, creating a sparse, probabilistic pattern. Gates are evaluated after patterns but before emission, giving you control over which triggered notes actually play.

### Stateful Arpeggiator

```lua
-- Create patterns that remember previous states
return rhythm {
    unit = "1/8",
    emit = function(init_context)
        local notes = {"c3", "e3", "g3", "b3"}
        local index = 1
        return function(context)
            local note = notes[index]
            index = (index % #notes) + 1 -- Cycle through notes
            return note
        end
    end
}
```
- TRY THIS: Add direction changes: `if index >= #notes or index <= 1 then direction = direction * -1 end`
- TRY THIS: Change notes based on time: `notes = scale("C4", {"major","minor"}[math.floor(context.time) % 2 + 1])`.notes

This example demonstrates stateful emitters that remember their position between calls, enabling sequences and other time-dependent behaviors.

## Advanced Techniques

### Conditional Gate

```lua
-- Filter which notes actually play using gates
return rhythm {
    unit = "1/8",
    pattern = {1, 0.1, 1, 0.5, 1, 0.2, 1, 0.1}, -- probability values
    gate = function(context)
        -- always play on even-numbered step values
        return (context.pulse_step - 1) % 2 == 0 or
            -- else use pulse values as probablities
            context.pulse_value >= math.random() 
    end,
    emit = "c3"
}
```
- TRY THIS: Create a threshold gate: `context.pulse_value > 0.5`
- TRY THIS: Only play when a specific MIDI note is held: `context.trigger.notes[1].key == "C4"`

Gates filter which triggered notes actually play, adding another layer of control to your patterns.

### Dynamic Cycles

```lua
-- Identifiers in cycles can be dynamically mapped to something else
local s = scale("C4", "minor")
return rhythm {
    unit = "1/4",
    emit = cycle("I III V VII"):map(function(context, value)
        -- value here is a single roman number from the cycle above
        local degree = value
        -- apply value as roman number chord degree
        return s:chord(degree)
    end)
}
```
- TRY THIS: Change scale to `"major", "dorian", or "pentatonic minor"`
- TRY THIS: Add parameters: `parameter.enum("scale", "minor", {"major", "minor", "pentatonic"})`

This example uses musical scale knowledge to generate chord progressions based on scale degrees.

See [Parameters](./guide/parameters.md) on how to add template parameters to rhythms.

### Generative Melody with Constraints

```lua
-- Create melodies that follow musical rules
return rhythm {
    unit = "1/8",
    emit = function(init_context)
        local pentatonic = scale("c4", "pentatonic minor").notes
        local last_note = 1
        return function(context)
            local next_note = math.random(#pentatonic)
            -- Prefer steps of 1 or 2 scale degrees (smoother melodies)
            while math.abs(next_note - last_note) > 2 do
                next_note = math.random(#pentatonic)
            end
            last_note = next_note
            return pentatonic[next_note]
        end
    end
}
```
- TRY THIS: Add occasional jumps: `if math.random() < 0.1 then ...` (allow larger intervals)
- TRY THIS: Change directions based on contour: add direction variable that occasionally flips

This generates melodies that follow musical constraints, like preferring small intervals for more natural-sounding melodies.

## Further Resources

- [Rhythm Guide](./guide/)
- [Advanced Topics](./extras/)
- [Guided Examples](./examples/)
- [Rhythm API Reference](./API/)

Remember to experiment by modifying these examples! The best way to learn is by tweaking parameters and seeing what happens.
