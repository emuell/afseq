---@meta
error("Do not try to execute this file. It's just a type definition file.")
---
---Part of the afseq trait: Defines LuaLS annotations for the afseq Rhythm class.
---

----------------------------------------------------------------------------------------------------

---Optional trigger context passed to `pattern`, `gate` and 'emit' functions.
---Specifies which keyboard note triggered, if any, started the rhythm.
---
---@class TriggerContext
---
---Note value that triggered, started the rhythm, if any.
---@field trigger_note integer?
---Note volume that triggered, started the rhythm, if any.
---@field trigger_volume number?
---Note slice offset value that triggered, started the rhythm, if any.
---@field trigger_offset integer?
---
---Current input parameter values, using parameter ids as keys
---and the actual parameter value as value.
---@see InputParameter
---@field inputs table<string, number|integer|boolean|string>

----------------------------------------------------------------------------------------------------

---Transport & playback time context passed to `pattern`, `gate` and `emit` functions.
---@class TimeContext : TriggerContext
---
---Project's tempo in beats per minutes.
---@field beats_per_min number
---Project's beats per bar setting.
---@field beats_per_bar integer
---Project's sample rate in samples per second.
---@field samples_per_sec integer

----------------------------------------------------------------------------------------------------

---Context passed to `pattern` and `gate` functions.
---@class PatternContext : TimeContext
---
---Continues pulse counter, incrementing with each new **skipped or emitted pulse**.
---Unlike `step` in emitter this includes all pulses, so it also counts pulses which do
---not emit events. Starts from 1 when the rhythm starts running or is reset.
---@field pulse_step integer
---Continues pulse time counter, incrementing with each new **skipped or emitted pulse**.
---Starts from 0 and increases with each new pulse by the pulse's step time duration.
---@field pulse_time_step number

----------------------------------------------------------------------------------------------------

---Context passed to `gate` functions.
---@class GateContext : PatternContext
---
---Current pulse's step time as fraction of a full step in the pattern. For simple pulses this
---will be 1, for pulses in subdivisions this will be the reciprocal of the number of steps in the
---subdivision, relative to the parent subdivisions pulse step time.
---### examples:
---```lua
---{1, {1, 1}} --> step times: {1, {0.5, 0.5}}
---```
---@field pulse_time number
---Current pulse value. For binary pulses this will be 1, 0 pulse values will not cause the emitter
---to be called, so they never end up here.
---Values between 0 and 1 will be used as probabilities and thus are maybe emitted or skipped.
---@field pulse_value number

----------------------------------------------------------------------------------------------------

---- *seeking*: The emitter is auto-seeked to a target time. All results are discarded. Avoid
---  unnecessary computations while seeking, and only maintain your generator's internal state.
---- *running*: The emitter is played back regularly. Results are audible.
---@alias PlaybackState "seeking"|"running"

---Context passed to 'emit' functions.
---@class EmitterContext : GateContext
---
---Specifies how the emitter currently is running.
---@field playback PlaybackState
---Continues step counter, incrementing with each new *emitted* pulse.
---Unlike `pulse_step` this does not include skipped, zero values pulses so it basically counts
---how often the emit function already got called.
---Starts from 1 when the rhythm starts running or is reset.
---@field step integer

----------------------------------------------------------------------------------------------------

---Single pulse value or a nested subdivision of pulses within a pattern.
---@alias Pulse (0|1|number|boolean|nil)|(Pulse)[]

----------------------------------------------------------------------------------------------------

---Construction options for a new rhythm.
---@class RhythmOptions
---
---Base time unit of the emitter. Use `resolution` to apply an additional factor, in order to
---create other less common rhythm bases.
---### examples:
---```lua
---unit = "beats", resolution = 1.01 --> slightly off beat pulse
---unit = "1/16", resolution = 4/3 --> triplet
---```
---@field unit "ms"|"seconds"|"bars"|"beats"|"1/1"|"1/2"|"1/4"|"1/8"|"1/16"|"1/32"|"1/64"
---Factor which is applied on `unit` to specify the final time resolution of the emitter.
---### examples:
---```lua
---unit = "beats", resolution = 1.01 --> slightly off beat pulse
---unit = "1/16", resolution = 4/3 --> triplet
---```
---@field resolution number?
---
---Optional offset in `unit * resolution` time units. By default 0.
---When set, the rhythm's event output will be delayed by the given offset value.
---### examples:
---```lua
---unit = "1/4",
---resolution = 4,
---offset = 4 -- start emitting after 4*4 beats
---```
---@field offset number?
---
---Define optional input parameters for the rhythm. Input parameters can dynamically
---change a rhythms behavior everywhere where `context`s are passed, e.g. in pattern,
---gate, emitter or cycle map generator functions.
---
---### examples:
---```lua
----- trigger a single note as specified by input parameter 'note'
----- when input parameter 'enabled' is true, else triggers nothing.
---  inputs = {
---    parameter.boolean("enabled", true),
---    parameter.integer("note", 48, { 0, 127 })
---  },
----- [...]
---  emit = function(context)
---    if context.inputs.enabled then -- boolean value
---      return note(context.inputs.note) -- integer value
---    else
---      return nil
---    end
---  end
---```
---@field inputs? InputParameter[]
---
---Specify the rhythmical pattern of the emitter. Each pulse with a value of 1 or true
---will cause an event from the `emitter` property to be triggered in the emitters
---time unit. 0 or nil values never trigger, and values in-between do *maybe* trigger.
---
---To create deterministic random patterns, seed the random number generator before
---creating the rhythm via `math.randomseed(some_seed)`
---
---Patterns can contains subdivisions, sub tables of pulses, to "cram" multiple pulses
---into a single pulse's time interval. This way more complex rhythmical patterns can
---be created.
---
---When no pattern is defined, a constant pulse of `1` is triggered by the rhythm.
---
---Just like the `emitter` property, patterns can either be a static array of values
---or a function or generators which produces values dynamically.
---
---### examples:
---```lua
----- static pattern
---pattern = { 1, 0, 0, 1 }
----- "cram" pulses into a single pulse slot via subdivisions
---pattern = { 1, { 1, 1, 1 } }
---
----- patterns created via the "patterns" lib
---pattern = pattern.from{ 1, 0 } * 3 + { 1, 1 }
---pattern = pattern.euclidean(7, 16, 2)
---
----- stateless pattern function
---pattern = function(_context)
---  return math.random(0, 1)
---end
---
----- stateful generator function
---pattern = function(_init_context)
---  local triggers = table.new{ 0, 6, 10 }
---  return function(context)
---    local step = (context.step - 1) % 16
---    return triggers:contains(step)
---  end
---end
---
---```
---@field pattern Pulse[]|(fun(context: PatternContext):Pulse)|(fun(context: PatternContext):fun(context: PatternContext):Pulse)?
---
---If and how many times a pattern should repeat. When 0 or false, the pattern does not repeat
---and plays back only once. When true, the pattern repeats endlessly, which is the default.
---When a number > 0, this specifies the number of times the pattern repeats until it stops.
---
---Note: When `pattern` is a function or iterator, the repeat count is the number of
---*function calls or iteration steps*. When the pattern is a pulse array, this is the number of
---times the whole pattern gets repeated.
---
---### examples:
---```lua
---repeat = 0 -- one-shot
---repeat = false -- also a one-shot
---repeat = 3 -- play the pattern 4 times
---repeat = true -- play & repeat forever
---```
---@field repeats (integer|boolean)?
---
---Optional pulse train filter function or generator function which filters events between
---the pattern and emitter. By default a threshold gate, which passes all pulse values
---greater than zero. 
---
---Custom function should returns true when a pattern pulse value should be passed, 
---and false when the emitter should be skipped.  
---
---### examples:
---```lua
----- probability gate: skips all 0s, passes all 1s. pulse alues in range (0, 1) are
----- maybe passed, using the pulse value as probablility.
---gate = function(context)
---  return context.pulse_value > math.random()
---end
---```
---@field gate (fun(context: GateContext):boolean)|(fun(context: GateContext):fun(context: GateContext):boolean)?
---
---Specify the melodic pattern of the rhythm. For every pulse in the rhythmical pattern, the event
---from the specified emit sequence. When the end of the sequence is reached, it starts again from
---the beginning.
---
---To generate notes dynamically, you can pass a function or a function iterator, instead of a
---static array or sequence of notes.
---
---Events can also be generated using the tidal cycle mini-notation. Cycles are repeated endlessly
---by default, and have the duration of a single pulse in the pattern. Patterns can be used to
---sequence cycles too.
---
---### examples:
---```lua
----- a sequence of c4, g4
---emit = {"c4", "g4"}
----- a chord of c4, d#4, g4
---emit = {{"c4", "d#4", "g4"}} -- or {"c4'min"}
----- a sequence of c4, g4 with volume 0.5
---emit = sequence{"c4", "g4"}:volume(0.5)
---
----- stateless generator function
---emit = function(_context)
---  return 48 + math.random(1, 4) * 5
---end
---
----- stateful generator function
---emit = function(_init_context)
---  local count, step, notes = 1, 2, scale("c5", "minor").notes
---  return function(_context)
---    local key = notes[count]
---    count = (count + step - 1) % #notes + 1
---    return { key = key, volume = 0.5 }
---  end
---end
---
----- a note pattern
---local tritone = scale("c5", "tritone")
---.. -- instrument #1,5,7 will be set as specified.
---emit = pattern.from(tritone:chord(1, 4)):euclidean(6) +
---  pattern.from(tritone:chord(5, 4)):euclidean(6)
---
----- a tidal cycle
---emit = cycle("<[a3 c4 e4 a4]*3 [d4 g3 g4 c4]>")
-----
---```
---@field emit Cycle|Sequence|Note|NoteValue|(NoteValue|Note)[]|(fun(context: EmitterContext):NoteValue)|(fun(context: EmitterContext):fun(context: EmitterContext):NoteValue)


----------------------------------------------------------------------------------------------------

---Create a new rhythm with the given configuration.
---
---### examples:
---```lua
----- trigger a chord sequence every 4 bars after 4 bars
---return rhythm {
---  unit = "bars",
---  resolution = 4,
---  offset = 1,
---  emit = sequence("c4'm", note("g3'm7"):transpose({0, 12, 0, 0}))
---}
---
-----trigger notes in an euclidean triplet pattern
---return rhythm {
---  unit = "1/8",
---  resolution = 3/2,
---  pattern = pattern.euclidean(6, 16, 2),
---  emit = sequence("c3", "c3", note{ "c4", "a4" }:volume(0.75))
---}
---
-----trigger notes in a seeded, random subdivision pattern
---math.randomseed(23498)
---return rhythm {
---  unit = "1/8",
---  pattern = { 1, { 0, 1 }, 0, 0.3, 0.2, 1, { 0.5, 0.1, 1 }, 0.5 },
---  emit = { "c4" },
---}
---
-----trigger random notes in a random pattern from a pentatonic scale
---return rhythm {
---  unit = "1/16",
---  pattern = function(context)
---    return (context.pulse_step % 4 == 1) or (math.random() > 0.8)
---  end,
---  emit = function(_init_context)
---    local cmin = scale("c5", "pentatonic minor").notes
---    return function(_context)
---      return { key = cmin[math.random(#cmin)], volume = 0.7 }
---    end
---  end
---}
---
-----play a seeded tidal cycle
---math.randomseed(9347565)
---return rhythm {
---  unit = "bars", -- emit one cycle per bar
---  emit = cycle("[c4 [f5 f4]*2]|[c4 [g5 g4]*3]")
---}
-----
---```
---@param options RhythmOptions
---@return userdata
---@nodiscard
function rhythm(options) end
