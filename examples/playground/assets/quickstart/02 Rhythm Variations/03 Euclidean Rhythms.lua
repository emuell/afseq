-- Distributes notes evenly across steps (common in many music traditions)
return rhythm {
  unit = "1/16",
  pattern = pattern.euclidean(3, 8, 0),  -- 3 hits spread over 8 steps without offset
  emit = "c4"
}

-- TRY THIS: Try different combinations like (5, 8, -2) or (7, 16)
-- TRY THIS: Use pattern = pattern.euclidean(3, 8) + pattern.euclidean(5, 8) to chain different patterns
