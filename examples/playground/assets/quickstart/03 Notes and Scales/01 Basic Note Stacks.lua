-- Create a chord by stacking notes
return rhythm {
  unit = "1/1",
  emit = { {"c4", "e4", "g4"}, "c4" } -- C major chord followed by a single C4
}

-- TRY THIS: Try different chord combinations like `{"d4", "f4", "a4"}` for D minor
-- TRY THIS: Add `v` values to create dynamics: `{"c4 v0.8", "e4 v0.6", "g4 v0.4"}
-- TRY THIS: Use `sequence` and `note` functions to group and modify notes:
--   sequence { 
--      note{"c4","e4","g4"}:transpose({12,}),
--      note{"c4"}:volume(0.8)
--   }
