-- Using tidal cycles notation for concise patterns
return pattern {
  unit = "1/4", -- Emit a cycle every beat
  event = cycle("c4 e4 g4") -- C major arpeggio
}

-- TRY THIS: The simplified notation emits a cycle per bar
-- TRY THIS: Add more patterns with | like [c4|c5 e4 g4]|[d4 f4|g5 a4]|[e4 g4 b4]
-- TRY THIS: Try simultaneous notes with square brackets [c4 e4]

-- See https://tidalcycles.org/docs/reference/mini_notation/ 
-- for more info about the Tidal Cycles mini-notation