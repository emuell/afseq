---
--- Part of the afseq trait:
--- Exports Mapping, a set of helper function which can be used to map cycle content.
---

----------------------------------------------------------------------------------------------------

---@alias NoteMapFunction fun(context: CycleMapContext, value: Note|string):Note|string

---@class NoteProperties
---@field transpose number? Note transpose step
---@field instrument number? Instrument/Sample/Patch >= 0
---@field volume number? Volume in range [0.0 - 1.0]
---@field panning number? Panning factor in range [-1.0 - 1.0] where 0 is center
---@field delay number? Delay factor in range [0.0 - 1.0]

----------------------------------------------------------------------------------------------------

---Set of helper functions to map cycle values.
---
---### examples:
---```lua
----- set volume values of notes in cycles
---cycle("c4 <a# g d#>"):map(
---  mappings.volume({1.0, 0.5})
---)
---
----- convert numbers in cycle to chords, using numbers in the cycle as scale degrees
---cycle("[1 5 6 <_ 4>]/4"):map(
---  mappings.chords(scale("a", "major"))
---)
---```
---@class CycleMappings : table
mappings = {}

----------------------------------------------------------------------------------------------------

---Applies given note properties to notes in the cycle.
---
---@param properties NoteProperties|(NoteProperties[])
---@return NoteMapFunction
mappings.note_properties = function(properties)
  properties = type(properties) == "table" and properties or { properties }
  return function(context, value)
    local property = properties[math.imod(context.step, #properties)]
    local result = note(tonumber(value) or value)
    if property.transpose then
      result = result:transpose(property.transpose)
    end
    if property.instrument then
      result = result:instrument(property.instrument)
    end
    if property.volume then
      result = result:volume(property.volume)
    end
    if property.panning then
      result = result:panning(property.panning)
    end
    if property.delay then
      result = result:delay(property.delay)
    end
    return result
  end
end

---@param steps number|number[]
mappings.transpose = function(steps)
  steps = type(steps) == "table" and steps or { steps }
  return function(context, value)
    local step = steps[math.imod(context.step, #steps)]
    local result = note(tonumber(value) or value)
    result = result:transpose(step)
    return result
  end
end

---@param instruments number|number[]
mappings.instrument = function(instruments)
  instruments = type(instruments) == "table" and instruments or { instruments }
  local properties = {}
  for i, instrument in ipairs(instruments) do
    properties[i] = { instrument = instrument }
  end
  return mappings.note_properties(properties)
end

---@param volumes number|number[]
mappings.volume = function(volumes)
  volumes = type(volumes) == "table" and volumes or { volumes }
  local properties = {}
  for i, volume in ipairs(volumes) do
    properties[i] = { volume = volume }
  end
  return mappings.note_properties(properties)
end

---@param pannings number|number[]
mappings.panning = function(pannings)
  pannings = type(pannings) == "table" and pannings or { pannings }
  local properties = {}
  for i, panning in ipairs(pannings) do
    properties[i] = { panning = panning }
  end
  return mappings.note_properties(properties)
end

---@param delays number|number[]
mappings.delay = function(delays)
  delays = type(delays) == "table" and delays or { delays }
  local properties = {}
  for i, delay in ipairs(delays) do
    properties[i] = { delay = delay }
  end
  return mappings.note_properties(properties)
end

----------------------------------------------------------------------------------------------------

---Maps numbers in cycle to chords as degree values, using the given scale.
---
---Optional note_counts can be a single value such as '3' or an array of numbers such as
---'{3, 4}' which will then be mapped to each nth item in the cycle.
---
---### examples:
---```
---cycle("[1 5 6 <_ 4>]/4"):map(
---  mappings.chords(scale("a", "major"))
---)
---```
---
---@param scale Scale
---@param note_counts (number|number[])?
---@return NoteMapFunction
mappings.chords = function(scale, note_counts)
  note_counts = (type(note_counts) == "table" and note_counts) or
      (type(note_counts) == "number" and { note_counts }) or { 3 }
  return function(context, value)
    local degree = type(value) == "string" and tonumber(value, 10) or nil
    if degree == nil then
      return value -- pass non number values as they are
    end
    assert(degree >= 1 and degree <= 7, "invalid degree value for chord: '" .. degree .. "'");
    local note_count = note_counts[math.imod(context.step, #note_counts)]
    return scale:chord(degree, note_count)
  end
end

----------------------------------------------------------------------------------------------------

---Maps numbers in cycle to intervals in the given scale.
---
---### examples:
---```
---cycle("1 3 5 7"):map(
---  mappings.intervals(scale("a4", "major"))
---)
---```
---
---@param scale Scale
---@return NoteMapFunction
mappings.intervals = function(scale)
  return function(context, value)
    local interval = type(value) == "string" and tonumber(value, 10) or nil
    if interval == nil then
      return value -- pass non number values as they are
    end
    return note(scale.notes[interval])
  end
end

----------------------------------------------------------------------------------------------------

---Combine multiple map functions into one.
---
---### examples:
---```lua
---return rhythm {
---  unit = "1/1",
---  emit = cycle("1 5 <_ 6>"):map(
---    mappings.combine(
---      mappings.chords( scale("c4", "minor") ),
---      mappings.delay({ 0.0, 0.1 }),
---      mappings.volume( 0.5 ))
---    )
---}
---```
---@param ... NoteMapFunction
---@return NoteMapFunction
function mappings.combine(...)
  local mappings = { ... }
  return function(context, value)
    for _, mapping in ipairs(mappings) do
      value = mapping(context, value)
    end
    return value
  end
end
