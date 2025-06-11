--[[
  Controlled randomness: Combines two seeded randomly generated euclidean
  rhythms with seeded randomly generated notes in a given scale.
--]]

-- CHANGE ME: adjust timing
local UNIT = "1/16"
local PATTERN_LEN = 16
local MELODY_LEN = PATTERN_LEN

-- CHANGE ME: adjust variations
local SCALE = "blues minor"
local RHYTHM_SEED = 1
local MELODY_SEED = 1

---

-- Create a random rhythm from two Euclidean rhythms
local function generate_rhythm(seed)
  -- Create a new local random number generator
  local rand = math.randomstate(seed)

  -- Generate the primary Euclidean rhythm pattern
  local primary = pattern.euclidean(
    rand(1, PATTERN_LEN / 2), PATTERN_LEN)
  -- Generate a secondary Euclidean rhythm pattern
  local secondary = pattern.euclidean(
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
  return combined
end

-- Create a function to map combined rhythm to the scale with randomness
local function generate_melody(pattern_len, scale, seed)
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
  return melody
end

-- return rhythm
return rhythm {
  unit = UNIT,
  pattern = function(context)
    -- Generate combined rhythm
    local rhythm = generate_rhythm(RHYTHM_SEED)
    -- Pick pulse from the rhythm for each new step
    return function(context)
      return rhythm[math.imod(context.pulse_step, #rhythm)]
    end
  end,
  emit = function(context)
    -- Define the notes that should be used for the melody
    local scale = scale("c", SCALE).notes
    -- Generate the melody based on the combined rhythm
    local melody = generate_melody(MELODY_LEN, scale, MELODY_SEED)
    -- Pick note from the melody and use the pulse value as volume
    ---@param context EmitterContext
    return function(context)
      local step = math.imod(context.pulse_step, #melody)
      return note(melody[step]):volume(context.pulse_value)
    end
  end
}
