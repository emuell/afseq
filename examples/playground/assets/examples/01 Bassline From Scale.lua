--[[
  Create a note pattern from scales, distributed via euclidean rhythms. 
--]]

-- Scale we're creating the note pattern from
local s = scale("c3", "minor")

-- Create a note pattern from euclidean distributed chords 
local notes =
  pulse.from(s:chord("i", 3)):euclidean(8) + 
  pulse.from(s:chord("vi", 3)):euclidean(8, 1):reverse() +
  pulse.from(s:chord("v", 3)):euclidean(8) +
  pulse.from(s:chord("iii", 5)):euclidean(8):reverse()

-- The rhythm emits the notes as a sequence
return pattern {
  unit = "1/16",
  event = sequence(notes):amplify(0.8)
}