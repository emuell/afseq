# Parameters

Pattern [`parameters`](../API/pattern.md#parameter) allow user controlled parameter values to be injected into a pattern. This allows you to write more flexible patterns that can be used as templates or to automate functions within the pattern.

Input parameters can be accessed in dynamic `pulse`, `gate`, `event` or `cycle` function [`contexts`](../API/pattern.md#EventContext).

## Parameter Types

Currenty available parameter types are:

- boolean - on/off switches - [`parameter.boolean`](../API/parameter.md#boolean)
- integer - integer value ranges - [`parameter.integer`](../API/parameter.md#integer)
- number - real number value ranges -[`parameter.number`](../API/parameter.md#number)
- string - enumeration value sets - [`parameter.enum`](../API/parameter.md#enum)

## Parameter access
 
When defining a parameter, each parameter has a unique string id set. This id is used to access the *actual* paramter value in the function contexts.

Definition:

» `parameter = { parameter.boolean("enabled", true) }`

Usage:

» `event = function(context) return context.parameter.enabled and "c5" or nil }`

Usage, if you've got spaces in your ids (not recommended):

» `event = function(context) return context.parameter["enabled"] and "c5" or nil }`


## Examples

Euclidean pattern generator with user configurable steps, pulses, offset value.

```lua
return pattern {
  parameter = {
    parameter.integer('steps', 12, {1, 64}, "Steps", 
      "Number of on steps in the pattern"),
    parameter.integer('pulses', 16, {1, 64}, "Pulses", 
      "Total number of on & off pulses"),
    parameter.integer('offset', 0, {-16, 16}, "Offset", 
      "Rotates on pattern left (values > 0) or right (values < 0)"),
  },
  unit = "1/1",
  pulse = function(context)
    return pulse.euclidean(
      math.min(context.parameter.steps, context.parameter.pulses), 
      context.parameter.pulses, 
      context.parameter.offset)
  end,
  event = "c4"
}
```


Random bass line generator with user defined custom scales and variations (seeds).
```lua
local scales = {"Chromatic", "Minor", "Major"}
return pattern {
  parameter = {
    parameter.enum('scale', scales[1], scales, "Scale"),
    parameter.integer('notes', 7, {1, 12}, "#Notes"),
    parameter.integer('variation', 0, {0, 0xff}, "Variation"),
  },
  unit = "1/1",
  pulse = function(context)
    local rand = math.randomstate(2345 + context.parameter.variation)
    return pulse.euclidean(rand(3, 16), 16, 0)
  end,
  event = function(context)
    local notes = scale("c4", context.parameter.scale).notes
    local rand = math.randomstate(127364 + context.parameter.variation)
    local notes = pulse.new(context.parameter.notes):map(function(_)
      return notes[rand(#notes)]
    end)
    return notes[math.imod(context.step, #notes)]
  end
}
```

Drum pattern cycle with configurable note values for each drumkit instrument. 
```lua
return pattern {
  unit = "1/1",
  parameter = {
    parameter.integer("bd_note", 48, {0, 127}),
    parameter.integer("sn_note", 70, {0, 127}),
    parameter.integer("hh_note", 98, {0, 127})
  },
  event = cycle([[
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