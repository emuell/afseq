-- Note properties
return rhythm {
  unit = "1/8",
  -- `v` = volume [0 - 1] `p` = panning [-1 - 1] `d` = delay [0 - 1] `#` = instrument
  emit = { "c4 v0.2", "off d0.5", "g4 v0.8" }
}

-- TRY THIS: Play specific instruments with # such as `c4 #8`
-- TRY THIS: Add delays to some of the notes
