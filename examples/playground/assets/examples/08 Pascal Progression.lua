--[[
  Generates random notes and chord progressions using a pascal triangle as pattern.
--]]

-- Define the Pascal's Triangle pattern
local pascals_triangle = {
  { 1 },
  { 1, 1 },
  { 1, 2, 1 },
  { 1, 3, 3,  1 },
  { 1, 4, 6,  4,  1 },
  { 1, 5, 10, 10, 5,  1 },
  { 1, 6, 15, 20, 15, 6,  1 },
  { 1, 7, 21, 35, 35, 21, 7, 1 }
}

-- Define the scale we pick notes from (16 notes from a C pentatonic scale)
local relaxing_scale = pattern.new(16, scale("c5", "pentatonic major"):notes_iter())

-- Function to generate chord progressions from Pascal's Triangle values
local function generate_chord_progressions(triangle)
  local chord_progressions = {}
  for _, row in ipairs(triangle) do
    local progression = {}
    for _, value in ipairs(row) do
      local degree = math.imod(value, #relaxing_scale)
      table.insert(progression, relaxing_scale[degree] - 12)       -- Transpose an octave down
    end
    table.insert(chord_progressions, progression)
  end
  return chord_progressions
end

-- Random seed for reproducibility
math.randomseed(2323)

-- Generate chord progressions
local chord_progressions = generate_chord_progressions(pascals_triangle)

return rhythm {
  unit = "1/2",
  emit = function(context)
    local result = {}
    -- randomly add maybe notes
    if math.random() < 0.5 then
      -- add a single random note from scale
      local maybe_note = relaxing_scale[math.random(1, #relaxing_scale)] - 12
      table.insert(result, { key = maybe_note })
    end
    -- randomly add rests (off)
    if math.random() < 0.1 then
      table.insert(result, "off")
    else
      -- randomly select a chord progression
      local progression_index = math.random(1, #chord_progressions)
      local progression = chord_progressions[progression_index]
      -- Emit chord progression notes with random volume, panning, and delay
      local delays = { 0.0, 1 / 6, 2 / 6, 3 / 6, 4 / 6, 3 / 6 }
      for _, note in ipairs(progression) do
        local random_volume = 0.4 + 0.2 * math.random()
        local random_panning = math.random() * 2 - 1
        local random_delay = delays[math.random(#delays)]
        table.insert(result, {
          key = note,
          volume = random_volume,
          panning = random_panning,
          delay = random_delay
        })
      end
    end
    return result
  end
}
