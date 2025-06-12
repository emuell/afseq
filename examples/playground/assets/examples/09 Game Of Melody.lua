--[[
  Game of Melody: randomly creates melody populations from the C minor scale.
  Lets the melodies that best fit a specified target melody survive.

  Stop the playback and start it again for a new random population.
--]]

-- Define the unit for timing
local UNIT = "1/16"
-- Define the population size
local POPULATION_SIZE = 12
-- Define the maximum number of generations before we repeat
local MAX_GENERATIONS = math.huge

-- Define the minor scale in MIDI note values
local MINOR_SCALE = scale("c", "minor")
-- Define notes to choose from: 16 notes
local SOURCE_NOTES = pulse.new(16, MINOR_SCALE:notes_iter())
-- Define the optimal melody (best fitness): 16 notes
-- with ascending, then descending notes from the scale
local TARGET_MELODY = pulse.from(
    1, 2, 3, 4, 5, 6, 7, 8, 8, 7, 6, 5, 4, 4, 2, 1):map(
  function (_, value)
    return SOURCE_NOTES[value]
  end
)
-- Define a chord progression using the minor scale as background chords
local CHORD_PROGRESSION = {
  MINOR_SCALE:chord("i"),  -- C minor chord
  MINOR_SCALE:chord("i"),  -- C minor chord
  MINOR_SCALE:chord("iv"), -- F minor chord
  MINOR_SCALE:chord("v"),  -- G minor chord
}

-- Define the fitness function
local function evaluate_fitness(melody)
  -- Calculate the fitness based on how similar the
  -- melody is compared to the target chord
  local fitness = 0
  assert(#melody == #TARGET_MELODY)
  for i = 1, #TARGET_MELODY do
    if melody[i] == TARGET_MELODY[i] then
      fitness = fitness + 1
    end
  end
  return fitness
end

-- Generate an initial population of melodies
local function generate_population()
  local population = {}
  for i = 1, POPULATION_SIZE do
    local melody = {}
    for j = 1, #TARGET_MELODY do
      local note_index = math.random(1, #SOURCE_NOTES)
      table.insert(melody, SOURCE_NOTES[note_index])
    end
    table.insert(population, melody)
  end
  return population
end

-- Perform selection based on fitness values
local function select_parents(population)
  local parents = {}
  for i = 1, 2 do
    local best_melody = nil
    local best_fitness = 0
    for _, melody in ipairs(population) do
      local fitness = evaluate_fitness(melody)
      if fitness > best_fitness then
        best_fitness = fitness
        best_melody = melody
      end
    end
    table.insert(parents, best_melody)
  end
  return parents
end

-- Perform crossover to create new melodies
local function crossover(parents)
  local melody1 = parents[1]
  local melody2 = parents[2]
  local crossover_point = math.random(1, #melody1)
  local child_melody = {}
  for i = 1, crossover_point do
    table.insert(child_melody, melody1[i])
  end
  for i = crossover_point + 1, #melody2 do
    table.insert(child_melody, melody2[i])
  end
  return child_melody
end

-- Perform mutation to introduce randomness
local function mutate(melody, notes)
  local mutation_point = math.random(1, #melody)
  local new_note_index = math.random(1, #notes)
  melody[mutation_point] = notes[new_note_index]
end

-- Create nerdo pattern
return pattern {
  unit = UNIT,
  event = function(context)
    -- a newly triggered pattern starts a new world
    local world = {
      population = generate_population(),
      generation = 1,
      chord = nil,
      melody = nil,
    }
    -- and evolves with each new pattern pulse step
    return function(context)
      -- perform selection every 32 steps
      if (context.step) % 16 == 1 then
        -- Select parents for crossover
        local parents = select_parents(world.population)
        -- Perform crossover to create new melody
        world.melody = crossover(parents)
        -- Perform mutation on the child melody
        mutate(world.melody, SOURCE_NOTES)
        -- Replace a quarter of the population with random melodies
        for i = 1, math.max(1, POPULATION_SIZE / 4), 1 do
          local replace_index = math.random(1, POPULATION_SIZE)
          world.population[replace_index] = world.melody
        end
        -- Increment generation count
        world.generation = world.generation + 1
        -- Check if maximum generations reached
        if world.generation > MAX_GENERATIONS then
          -- Reset population for next iteration
          world.population = generate_population()
          world.generation = 1
        end
        -- Calculate the chord index based on the current step
        local chord_index = math.imod((context.step - 1) / 16 + 1, #CHORD_PROGRESSION)
        world.chord = note(CHORD_PROGRESSION[chord_index]):volume(0.5)
      else
        world.chord = note({ "---", "---", "---" })
      end
      -- Emit the melody note
      local note_index = math.imod(context.step, #world.melody)
      return note {
        note(world.chord.notes[1]),
        note(world.chord.notes[2]),
        note(world.chord.notes[3]),
        note(world.melody[note_index]):volume(0.7)
      }
    end
  end
}