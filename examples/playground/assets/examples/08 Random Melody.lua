--[[
  Controlled randomness: Combines two seeded randomly generated euclidean
  rhythms with seeded randomly generated notes in a given scale.
--]]

-- base timing & generator lengths: change me, if you like
local UNIT = "1/16"
local PATTERN_LEN = 16
local MELODY_LEN = PATTERN_LEN

---

-- last created rhythm cache
last_rhythm_seed = nil
last_rhythm = nil

-- Create a random rhythm from two Euclidean rhythms
local function generate_rhythm(seed)
  -- Use last generated one just to avoid overhead
  if last_rhythm_seed == seed then
    return last_rhythm
  end 
  -- Create a new local random number generator
  local rand = math.randomstate(seed)

  -- Generate the primary Euclidean rhythm pattern
  local primary = pulse.euclidean(
    rand(1, PATTERN_LEN / 2), PATTERN_LEN)
  -- Generate a secondary Euclidean rhythm pattern
  local secondary = pulse.euclidean(
    rand(1, PATTERN_LEN / 2), PATTERN_LEN)

  -- Volume randomization state
  local volumes = { 0.25, 0.5 }
  local last_volume = 1.0
  local last_step = 1

  -- Combine the two rhythms using the Schillinger principle
  local combined = { 1 }
  for step = 2, #primary do
    if primary[step] == 1 or secondary[step] == 1 then
      -- randomize note volume
      local volume = (step % 4 == 1) and 1.0 or volumes[rand(#volumes)]
      if last_volume < 1.0 or step - last_step > 1 then
        volume = 1.0
      end 
      combined[step] = volume
      last_step = step
      last_volume = volume
    else
      combined[step] = 0
    end
  end
  last_rhythm_seed = seed
  last_rhythm = combined
  return combined
end

-- last created melody cache
last_melody_seed = nil
last_melody_scale = nil
last_melody = nil

-- Create a function to map combined rhythm to the scale with randomness
local function generate_melody(pattern_len, scale, seed)
  -- Use last generated one just to avoid overhead
  if last_melody_seed == seed and last_melody_scale == scale then
    return last_melody
  end
  -- Create a new local random number generator
  local rand = math.randomstate(seed)
  -- generate the melody, starting with the root note
  local melody = { scale[1] }
  local last_note_index = nil 
  for step = 2, pattern_len do
    -- pick a random note from the scale
    local note_index = last_note_index
    while note_index == last_note_index do 
      note_index = rand(#scale + 1)
    end
    last_note_index = note_index
    -- last note in scale is treated as an off
    if note_index == #scale + 1 then
      melody[#melody + 1] = "off"
    else
      melody[#melody + 1] = scale[note_index]
    end
  end
  last_melody_seed = seed
  last_melody_scale = scale
  last_melody = melody
  return melody
end

-- return pattern
return pattern {
  unit = UNIT,
  parameter = {
    parameter.integer("rhythm_seed", 1, {1, 9999}, "Rhythm Seed"),
    parameter.integer("melody_seed", 1, {1, 9999}, "Melody Seed"),
    parameter.enum("scale", scale_names()[8], scale_names(), "Melody Scale"),
  },
  pulse = function(context)
    -- Generate combined rhythm
    local rhythm = generate_rhythm(context.parameter.rhythm_seed)
    -- Pick pulse from the rhythm for each new step
    return rhythm[math.imod(context.pulse_step, #rhythm)]
  end,
  event = function(context)
    -- Define the notes that should be used for the melody
    local scale = scale("c", context.parameter.scale).notes
    -- Generate the melody based on the combined rhythm
    local melody = generate_melody(MELODY_LEN, scale, context.parameter.melody_seed)
    -- Pick note from the melody and use the pulse value as volume
    local step = math.imod(context.pulse_step, #melody)
    return note(melody[step]):volume(context.pulse_value)
  end
}