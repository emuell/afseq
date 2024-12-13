# Input Parameters

Rhythm [`inputs`](../API/rhythm.md#inputs) allow user controlled parameter values to be injected into a rhythm. This allows you to write more flexible rhythms that can be used as templates or to automate functions within the rhythm.

Input parameters can be accessed in dynamic pattern, gate, emitter or cycle function [`contexts`](../API/rhythm.md#EmitterContext).

## Parameter Types

Currenty available parameter types are:

- boolean - on/off switches - [`parameter.boolean`](../API/input.md#boolean)
- integer - integer value ranges - [`parameter.integer`](../API/input.md#integer)
- number - real number value ranges -[`parameter.number`](../API/input.md#number)
- string - enumeration value sets - [`parameter.enum`](../API/input.md#enum)

## Parameter access
 
When defining a parameter, each parameter has a unique string id set. This id can then be used to access the *actual* paramter value in the function contexts.

Definition:

» `inputs = { parameter.boolean("enabled", true) }`

Usage:

» `emit = function(context) return context.inputs.enabled and "c5" or nil }`

Usage, if you've got spaces in your ids (not recommended):

» `emit = function(context) return context.inputs["enabled"] and "c5" or nil }`


## Examples

Euclidean pattern generator with user configurable steps, pulses, offset value.

```lua
return rhythm {
  inputs = {
    parameter.integer('steps', 12, {1, 64}, "Steps", 
      "Number of on steps in the pattern"),
    parameter.integer('pulses', 16, {1, 64}, "Pulses", 
      "Total number of on & off pulses"),
    parameter.integer('offset', 0, {-16, 16}, "Offset", 
      "Rotates on pattern left (values > 0) or right (values < 0)"),
  },
  unit = "1/1",
  pattern = function(context)
    return pattern.euclidean(
      math.min(context.inputs.steps, context.inputs.pulses), 
      context.inputs.pulses, 
      context.inputs.offset)
  end,
  emit = "c4"
}
```


Random bass line generator with user defined custom scales and variations (seeds).
```lua
local scales = {"Chromatic", "Minor", "Major"}
return rhythm {
  inputs = {
    parameter.enum('scale', scales[1], scales, "Scale"),
    parameter.integer('notes', 7, {1, 12}, "#Notes"),
    parameter.integer('variation', 0, {0, 0xff}, "Variation"),
  },
  unit = "1/1",
  pattern = function(context)
    local rand = math.randomstate(2345 + context.inputs.variation)
    return pattern.euclidean(rand(3, 16), 16, 0)
  end,
  emit = function(context)
    local notes = scale("c4", context.inputs.scale).notes
    local rand = math.randomstate(127364 + context.inputs.variation)
    local notes = pattern.new(context.inputs.notes):map(function(_)
      return notes[rand(#notes)]
    end)
    return notes[math.imod(context.step, #notes)]
  end
}
```

Drum pattern cycle with configurable note values for each drumkit instrument. 
```lua
return rhythm {
  unit = "1/1",
  inputs = {
    parameter.integer("bd_note", 48, {0, 127}),
    parameter.integer("sn_note", 70, {0, 127}),
    parameter.integer("hh_note", 98, {0, 127})
  },
  emit = cycle([[
    [<hh1 hh2 hh2>*12],
    [bd1 ~]*2 ~ [~ bd2] ~,
    [~ sn1]*2,
    [~ sn2]*8
  ]]):map(function(context, value)
    for _, id in pairs{"bd", "sn", "hh"} do
      local number = value:match(id.."(%d+)")
      if number then
        return note(context.inputs[id.."_note"]):volume(
          number == "2" and 0.2 or 1.0)
      end
    end
  end)
}
```