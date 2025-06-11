--[[
  Create a note pattern from scales, distributed via euclidean rhythms. 
--]]

-- Scale we're creating the note pattern from
local s = scale("c3", "minor")

-- Create a notes pattern from euclidean distributed chords 
local notes =
  pattern.from(s:chord("i", 3)):euclidean(8) + 
  pattern.from(s:chord("vi", 3)):euclidean(8, 1):reverse() +
  pattern.from(s:chord("v", 3)):euclidean(8) +
  pattern.from(s:chord("iii", 5)):euclidean(8):reverse()

-- The rhythm emits the notes as a sequence
return rhythm {
  unit = "1/16",
  emit = sequence(notes):amplify(0.8)
}