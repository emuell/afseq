-- Advanced chord and scale operations
return rhythm {
  unit = "1/1",
  emit = {
    chord("c4", "major"),          -- C major via the chord function
    chord("c4", {0, 4, 7}),        -- C major via custom intervals
    scale("c", "major"):chord(1),  -- C major from 1st degree of C major scale
    scale("c", "major"):chord(5)   -- G major from 5th degree of C major scale
  }
}

-- TRY THIS: Use other scales like `"minor"`, `"dorian"`, or `"pentatonic"`
-- TRY THIS: Try different chord degrees: `scale("c", "major"):chord(2)` for D minor
