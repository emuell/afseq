---
--- Part of the afseq trait:
--- Exports pulse, a table wrapper with helper functions to ease creating rhythmical
--- pulse or note patterns. Inspired from https://github.com/rick4stley/array
---

---@diagnostic disable: need-check-nil

----------------------------------------------------------------------------------------------------

---Valid pulse value in a pulse table
---@alias PulseTableValue number|boolean|string|table

---Default empty pulse values for specific pulse types.
local empty_pulse_values = {
  ["userdata"] = {},
  ["table"] = {},
  ["string"] = "",
  ["boolean"] = false,
  ["number"] = 0
}

---Guess empty pulse values from an existing pulse table (implementation detail).
local function empty_pulse_value(table)
  if type(table) == "table" then
    for _, v in pairs(table) do
      -- special case for numbers, which are likely notes when > 1
      if type(v) == "number" then
        if v > 1 then
          return 0xff -- empty note value
        end
      end
      local value = empty_pulse_values[type(v)]
      if value ~= nil then
        return value
      end
    end
  end
  return 0
end

---Table with helper functions to ease creating rhythmic patterns.
---
---### examples:
---```lua
----- using + and * operators to combine patterns
---pulse.from{ 0, 1 } * 3 + { 1, 0 }
---```
---```lua
----- repeating, spreading and subsets
---pulse.from{ 0, 1, { 1, 1 } }:repeat_n(4):spread(1.25):take(16)
---```
---```lua
----- euclidean patterns
---pulse.euclidean(12, 16)
---pulse.from{ 1, 0.5, 1, 1 }:euclidean(12)
---```
---```lua
----- generate/init from functions
---pulse.new(8):init(1) --> 1,1,1,1,1,1,1,1
---pulse.new(12):init(function() return math.random(0.5, 1.0) end )
---pulse.new(16):init(scale("c", "minor").notes_iter())
---```
---```lua
----- generate pulses with note values
---pulse.from{ "c4", "g4", "a4" } * 7 + { "a4", "g4", "c4" }
---```
---```lua
----- generate chords from degree values
---pulse.from{ 1, 5, 6, 4 }:map(function(index, degree)
---  return scale("c", "minor"):chord(degree)
---end)
---```
---@class Pulse : table
---@operator add(Pulse|table): Pulse
---@operator mul(number): Pulse
pulse = {}

----------------------------------------------------------------------------------------------------
--- Pulse table creation
----------------------------------------------------------------------------------------------------

---Create a new empty pulse table or a pulse table with the given length and value.
---
---
---### examples:
---```lua
---pulse.new(4) --> {0,0,0,0}
---pulse.new(4, 1) --> {1,1,1,1}
---pulse.new(4, function() return math.random() end)
---```
---@param length integer? Initial length of the pulse. When undefined, an empty pulse is created.
---@param value (PulseTableValue|(fun(index: integer):PulseTableValue))? Value or generator function, which sets the initial values in the pulse.
---@return Pulse
---@nodiscard
function pulse.new(length, value)
  local t = setmetatable({}, {
    ---all pulse functions can be accessed as member functions
    __index = pulse,
    ---operator + adds two patterns
    __add = function(a, b)
      return a:copy():push_back(b)
    end,
    ---operator * creates a repeated pulse
    __mul = function(a, b)
      return a:copy():repeat_n(b)
    end
  })
  -- initialize
  if length ~= nil then
    value = value or 0
    t:init(value, length)
  end
  return t
end

---Create a new pulse table from an existing set of values or tables.
---When passing tables, those will be flattened.
---
---### examples:
---```lua
---pulse.from(1,0,1,0) --> {1,0,1,0}
---pulse.from({1,0},{1,0}) --> {1,0,1,0}
---```
---@param ... PulseTableValue|(PulseTableValue[])
---@return Pulse
---@nodiscard
function pulse.from(...)
  return pulse.new():push_back(...)
end

-- create a shallow-copy of the given pulse table (or self)
---
---### examples:
---```lua
---local p = pulse.from(1, 0)
---local p2 = p:copy() --> {1,0}
---```
---@return Pulse
---@nodiscard
function pulse.copy(self)
  return pulse.from(self:unpack())
end

---Create an new pulse table or spread and existing pulse evenly within the given length.
---Similar, but not exactly like `euclidean`.
---
---Shortcut for `pulse.from{1,1,1}:spread(length / #self):rotate(offset)`
---
---### examples:
---```lua
---pulse.distributed(3, 8) --> {1,0,0,1,0,1,0}
---pulse.from{1,1}:distributed(4, 1) --> {0,1,0,1}
---```
---@param steps table|integer Existing pulse or number of on steps in the pulse.
---@param length integer Number of total steps in the pulse.
---@param offset integer? Optional rotation offset.
---@param empty_value PulseTableValue? Value used as empty value (by default 0 or guessed from existing content).
function pulse.distributed(steps, length, offset, empty_value)
  assert(type(length) == "number" and length > 0,
    "invalid length argument (must be an integer > 0)")
  local from
  if type(steps) == "table" then
    from = pulse.from(steps)
  else
    assert(type(steps) == "number" and steps > 0,
      "invalid step argument (must be an integer > 0)")
    from = pulse.new(steps, 1)
  end
  assert(length >= #from,
    "Invalid length or steps arguments (length must be >= steps")
  return from:spread(length / #from, empty_value):rotate(offset or 0)
end

---Create a new euclidean rhythm pulse table with the given pulses or number of new pulses
---in the given length. Optionally rotate the contents too.
---[Euclidean Rhythm](https://en.wikipedia.org/wiki/Euclidean_rhythm)
---
---### examples:
---```lua
---pulse.euclidean(3, 8)
--- --> {1,0,0,1,0,0,1,0}
---pulse.from{"x", "x", "x"}:euclidean(8, 0, "-")
--- --> {"x","-","-","x","-","-","x","-"}
---```
---@param steps table|integer Existing pulse or number of on steps in the pulse.
---@param length integer Number of total steps in the pulse.
---@param offset integer? Optional rotation offset.
---@param empty_value PulseTableValue? Value used as off value (by default 0 or guessed from existing content).
function pulse.euclidean(steps, length, offset, empty_value)
  -- get or create initial pulse pulse
  local front = {}
  if type(steps) == "table" then
    for _, v in ipairs(steps) do
      table.insert(front, { v })
    end
  else
    assert(type(steps) == "number" and steps >= 0,
      "invalid steps argument (must be a table or an integer > 0)")
    for _ = 1, steps do
      table.insert(front, { 1 })
    end
  end
  assert(type(length) == "number" and length > 0,
    "invalid length argument (expecting an integer > 0)")
  assert(type(offset) == "number" or offset == nil,
    "invalid offset argument (must be an integer or nil)")
  empty_value = empty_value or empty_pulse_value(steps)
  -- merge/add two tables
  local function join_tables(a, b)
    local c = { table.unpack(a) }
    for i = 1, #b do
      table.insert(c, b[i])
    end
    return c
  end
  -- recursive euclidean pulse impl
  local function euclidean_impl(front, back)
    if #back < 2 then
      return join_tables(front, back)
    end
    local newFront = {}
    while #front > 0 and #back > 0 do
      table.insert(newFront, join_tables(table.remove(front), table.remove(back)))
    end
    return euclidean_impl(newFront, join_tables(front, back))
  end
  -- apply
  if #front == 0 then
    local result = pulse.new();
    for _ = 1, length do
      result:push_back(empty_value)
    end
    return result
  elseif #front >= length then
    local result = pulse.from(front);
    while #result > length do
      result:pop_back()
    end
    return result
  else
    local back = {}
    for _ = 1, length - #front do
      table.insert(back, { empty_value })
    end
    -- spread
    local rhythm = euclidean_impl(front, back);
    -- convert to pulse and flatten
    local result = pulse.new();
    for _, g in ipairs(rhythm) do
      result:push_back(g);
    end
    -- rotate
    if offset then
      result:rotate(-offset)
    end
    return result
  end
end

----------------------------------------------------------------------------------------------------
--- Access pulse table content
----------------------------------------------------------------------------------------------------

---Shortcut for table.unpack(pulse): returns elements from this pulse as var args.
---
---### examples:
---```lua
---local p = pulse.from{1,2,3,4}
---local v1, v2, v3, v4 = p:unpack()
---```
---@return PulseTableValue ...
---@nodiscard
function pulse.unpack(self)
  return table.unpack(self)
end

---Fetch a sub-range from the pulse table as new pulse table.
---When the given length is past end of this pulse it is filled up with empty values.
---
---### examples:
---```lua
---local p = pulse.from{1,2,3,4}
---p = p:subrange(2,3) --> {2,3}
---p = p:subrange(1,4,"X") --> {2,3,"X","X"}
---```
---@param i integer Subrange start
---@param j integer? Subrange end (defaults to pulse length)
---@param empty_value PulseTableValue? Value used as empty value (by default 0 or guessed from existing content).
function pulse.subrange(self, i, j, empty_value)
  assert(type(i) == "number" and i > 0,
    "invalid subrange start argument (must be an integer > 0)")
  assert(j == nil or (type(j) == "number" and j > 0),
    "invalid subrange end argument (must be an integer > 0)")
  local len = j or #self
  local a = pulse.new()
  empty_value = empty_value or empty_pulse_value(self)
  for ii = i, len do
    a:push_back(self[ii] or empty_value)
  end
  return a
end

---Get first n items from the pulse as new pulse table.
---When the given length is past end of this pulse its filled up with empty values.
---
---### examples:
---```lua
---local p = pulse.from{1,2,3,4}
---p = p:take(2) --> {1,2}
---p = p:take(4, "") --> {1,2,"",""}
---```
---@param length integer
---@param empty_value PulseTableValue? Value used as empty value (by default 0 or guessed from existing content).
function pulse.take(self, length, empty_value)
  assert(type(length) == "number" and length > 0,
    "invalid length argument (must be an integer > 0)")
  return self:subrange(1, length, empty_value)
end

----------------------------------------------------------------------------------------------------
--- Modify contents
----------------------------------------------------------------------------------------------------

---Clear a pulse table, remove all its contents.
---
---### examples:
---```lua
---local p = pulse.from{1,0}
---p:clear() --> {}
---```
function pulse.clear(self)
  while #self > 0 do
    table.remove(self)
  end
  return self
end

---Fill pulse table with the given value or generator function in the given length.
---
---### examples:
---```lua
---local p = pulse.from{0,0}
---p:init(1) --> {1,1}
---p:init("X", 3) --> {"X","X", "X"}
---p:init(function(i) return math.random() end, 3)
---```
---@param value PulseTableValue|fun(index: integer):PulseTableValue
---@param length integer?
function pulse.init(self, value, length)
  assert(type(length) == "nil" or (type(length) == "number" and length > 0),
    "invalid length argument (must be an integer > 0)")
  length = length or #self
  if type(value) == 'function' then
    for i = 1, length do
      self[i] = value(i)
    end
  else
    for i = 1, length do
      self[i] = value
    end
  end
  while #self > length do
    table.remove(self, #self)
  end
  return self
end

---Apply the given function to every item in the pulse table.
---
---### examples:
---```lua
---local p = pulse.from{1,3,5}
---p:map(function(k, v)
---  return scale("c", "minor"):degree(v)
---end) --> {48, 51, 55}
---```
---@param fun fun(index: integer, value: PulseTableValue): PulseTableValue
function pulse.map(self, fun)
  local num = #self
  for i = 1, num do
    self[i] = fun(i, self[i])
  end
  return self
end

---Invert the order of items in the pulse table.
---
---### examples:
---```lua
---local p = pulse.from{1,2,3}
---p:reverse() --> {3,2,1}
---```
function pulse.reverse(self)
  local num = #self
  for i = 1, math.floor(num / 2) do
    local j = num - (i - 1)
    self[j], self[i] = self[i], self[j]
  end
  return self
end

---Shift contents by the given amount to the left (negative amount) or right.
---
---### examples:
---```lua
---local p = pulse.from{1,0,0}
---p:rotate(1) --> {0,1,0}
---p:rotate(-2) --> {0,0,1}
---```
---@param amount integer
function pulse.rotate(self, amount)
  assert(type(amount) == "number",
    "invalid amount argument (must be an integer)")
  local n = #self
  amount = amount % n
  if amount == 0 then return self end
  for i = n, 1, -1 do
    self[i + amount] = self[i]
  end
  for i = 1, amount do
    self[i], self[i + n] = self[i + n], nil
  end
  return self
end

----------------------------------------------------------------------------------------------------
--- Add/remove contents
----------------------------------------------------------------------------------------------------

---Push a single or multiple items or other pulse contents to the end of the pulse.
---Note: When passing array alike tables or patterns, they will be *unpacked*.
---
---### examples:
---```lua
---local p = pulse.new()
---p:push_back(1) --> {1}
---p:push_back(2,3) --> {1,2,3}
---p:push_back{4} --> {1,2,3,4}
---p:push_back({5,{6,7}) --> {1,2,3,4,5,6,7}
---```
---@param ... PulseTableValue|(PulseTableValue)[]
function pulse.push_back(self, ...)
  local function add_unpacked(v)
    if type(v) == 'table' and #v > 0 then
      for i = 1, #v do
        if v[i] ~= nil then
          add_unpacked(v[i])
        end
      end
    else
      table.insert(self, v)
    end
  end
  local len = select("#", ...)
  for i = 1, len do
    local v = select(i, ...)
    add_unpacked(v)
  end
  return self
end

---Remove an entry from the back of the pulse table. returns the removed item.
---
---### examples:
---```lua
---local p = pulse.from({1,2})
---p:pop_back() --> {1}
---p:pop_back() --> {}
---p:pop_back() --> {}
---```
---@return PulseTableValue
function pulse.pop_back(self)
  return table.remove(self)
end

---Repeat contents of the pulse table n times.
---
---### examples:
---```lua
---local p = pulse.from{1,2,3}
---patterns:repeat_n(2) --> {1,2,3,1,2,3}
---```
---@param count integer
function pulse.repeat_n(self, count)
  assert(type(count) == "number" and count > 0,
    "invalid count argument (must be an integer > 0)")
  local num = #self
  for _ = 1, count - 1 do
    for i = 1, num do
      self:push_back(self[i])
    end
  end
  return self
end

---Expand (with amount > 1) or shrink (amount < 1) the length of the pulse table by 
---the given factor, spreading allowed content evenly and filling gaps with 0 or the
---given empty value.
---
---### examples:
---```lua
---local p = pulse.from{1,1}
---p:spread(2) --> {1,0,1,0}
---p:spread(1/2) --> {1,1}
---```
---@param amount number Spread factor (2 = double, 0.5 = half the size).
---@param empty_value PulseTableValue? Value used as empty value (by default 0 or guessed from existing content).
function pulse.spread(self, amount, empty_value)
  assert(type(amount) == "number" and amount > 0,
    "invalid amount argument (must be an integer > 0)")
  empty_value = empty_value or empty_pulse_value(self)
  local old_num = #self
  local new_num = math.floor(old_num * amount + 0.5)
  local old = self:copy()
  self:init(empty_value, new_num)
  for i = 1, old_num do
    local j = math.floor((i - 1) * amount + 0.5) + 1
    if j <= new_num then
      self[j] = old[i]
    end
  end
  return self
end

----------------------------------------------------------------------------------------------------
--- Conversion
----------------------------------------------------------------------------------------------------

---Serialze a pulse table for display/debugging purposes.
---
---### examples:
---```lua
---pulse.euclidean(3, 8):tostring() --> "{1, 0, 0, 1, 0, 0, 1, 0}"
---```
---@return string
---@nodiscard
pulse.tostring = function(self)
  return table.tostring(self)
end