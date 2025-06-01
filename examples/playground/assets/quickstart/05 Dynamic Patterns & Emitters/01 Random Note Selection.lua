-- Randomly select notes from a list
local notes = {"c4", "d4", "e4", "g4"}
return rhythm {
  unit = "1/8",
  emit = function(context)
    return notes[math.random(#notes)] -- Pick random note from array
  end
}

-- TRY THIS: Use notes from a specific scale with local notes = scale("c4", "major").notes
-- TRY THIS: Add amplitude variation with note(some_note):amplify(0.5 + math.random() * 0.5)
