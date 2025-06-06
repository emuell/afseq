--[[
  Create a note pattern from scales, distributed via euclidean. 
--]]

-- Scale we're creating the note pattern from
local s = scale("c3", "minor")

 -- Create a notes pattern from euclidean distributed chords 
local notes =
  pattern.from(s:chord("I", 3)):euclidean(8) + 
  pattern.from(s:chord("VI", 3)):euclidean(8, 1):reverse() +
  pattern.from(s:chord("V", 3)):euclidean(8) +
  pattern.from(s:chord("III", 5)):euclidean(8):reverse()

return rhythm {
  unit = "1/16",
  emit = function(context)
    -- Cycle through notes with every new step
    local step = math.imod(context.step, #notes)
    return note(notes[step])
  end
}