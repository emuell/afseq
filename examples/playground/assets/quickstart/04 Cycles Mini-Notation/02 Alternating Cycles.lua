-- Switching between different patterns
return rhythm {
  unit = "1/4",
  emit = cycle("[c4 e4 g4]|[d4 f4 a4]") -- Randomly select one of two chords
}

-- TRY THIS: Combine with alternation: c4(3,8)|e4(5,8)
-- TRY THIS: Change the numbers for different distributions

-- See https://tidalcycles.org/docs/reference/mini_notation/ 
-- for more info about the Tidal Cycles mini-notatio