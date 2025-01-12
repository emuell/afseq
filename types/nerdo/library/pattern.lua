---
--- Part of the afseq trait:
--- Exports Pattern, a table wrapper with array alike helper functions which are useful to
--- ease creating rhythmical patterns. Inspired from https://github.com/rick4stley/array
---
--- Copyright (c) 2024 Eduard Müller <mail@emuell.net>
--- Distributed under the MIT License.
---

----------------------------------------------------------------------------------------------------

---Valid pulse value in a pattern
---@alias PulseValue number|boolean|string|table

---Default empty pulse values for specific pulse types.
local empty_pulse_values = {
  ["userdata"] = {},
  ["table"] = {},
  ["string"] = "",
  ["boolean"] = false,
  ["number"] = 0
}

---Guess empty pulse values from an existing pattern (implementation detail).
local function empty_pulse_value(table)
  if type(table) == "table" then
    for _, v in pairs(table) do
      -- special case for numbers, which are more likely notes when > 1
      if type(v) == "number" then
        if v > 1 then
          return {}
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

---Array alike table with helper functions to ease creating rhythmic patterns.
---
---### examples:
---```lua
----- using + and * operators to combine patterns
---pattern.from{ 0, 1 } * 3 + { 1, 0 }
---```
---```lua
----- repeating, spreading and subsets
---pattern.from{ 0, 1, { 1, 1 } }:repeat_n(4):spread(1.25):take(16)
---```
---```lua
----- euclidean patterns
---pattern.euclidean(12, 16)
---pattern.from{ 1, 0.5, 1, 1 }:euclidean(12)
---```
---```lua
----- generate/init from functions
---pattern.new(8):init(1) --> 1,1,1,1,1,1,1,1
---pattern.new(12):init(function() return math.random(0.5, 1.0) end )
---pattern.new(16):init(scale("c", "minor").notes_iter())
---```
---```lua
----- generate note patterns
---pattern.from{ "c4", "g4", "a4" } * 7 + { "a4", "g4", "c4" }
---```
---```lua
----- generate chords from degree values
---pattern.from{ 1, 5, 6, 4 }:map(function(index, degree)
---  return scale("c", "minor"):chord(degree)
---end)
---```
---@class Pattern : table
---@operator add(Pattern|table): Pattern
---@operator mul(number): Pattern
pattern = {}

----------------------------------------------------------------------------------------------------
--- Pattern creation
----------------------------------------------------------------------------------------------------

---Create a new empty pattern or pattern with the given length and pulse value.
---
---
---### examples:
---```lua
---pattern.new(4,1) --> {1,1,1,1}
---pattern.new(4, function() return math.random() end)
---```
---@param length integer? Initial length of the pattern. When undefined, an empty pattern is created.
---@param value (PulseValue|(fun(index: integer):PulseValue))? Value or generator function, which sets the initial values in the pattern.
---@return Pattern
---@nodiscard
function pattern.new(length, value)
  local t = setmetatable({}, {
    ---all pattern functions can be accessed as member functions
    __index = pattern,
    ---operator + adds two patterns
    __add = function(a, b)
      return a:copy():push_back(b)
    end,
    ---operator * creates a repeated pattern
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

---Create a new pattern from an existing set of values or tables.
---When passing tables, those will be flattened.
---
---### examples:
---```lua
---pattern.from(1,0,1,0) --> {1,0,1,0}
---pattern.from({1,0},{1,0}) --> {1,0,1,0}
---```
---@param ... PulseValue|(PulseValue[])
---@return Pattern
---@nodiscard
function pattern.from(...)
  return pattern.new():push_back(...)
end

-- create a shallow-copy of the given pattern (or self)
---
---### examples:
---```lua
---local p = pattern.from(1, 0)
---local p2 = p:copy() --> {1,0}
---```
---@return Pattern
---@nodiscard
function pattern.copy(self)
  return pattern.from(self:unpack())
end

---Create an new pattern or spread and existing pattern evenly within the given length.
---Similar, but not exactly like `euclidean`.
---
---Shortcut for `pattern.from{1,1,1}:spread(length / #self):rotate(offset)`
---
---### examples:
---```lua
---pattern.distributed(3, 8) --> {1,0,0,1,0,1,0}
---pattern.from{1,1}:distributed(4, 1) --> {0,1,0,1}
---```
---@param steps table|integer Existing pattern or number of on steps in the pattern.
---@param length integer Number of total steps in the pattern.
---@param offset integer? Optional rotation offset.
---@param empty_value PulseValue? Value used as empty value (by default 0 or guessed from existing content).
function pattern.distributed(steps, length, offset, empty_value)
  assert(type(length) == "number" and length > 0,
    "invalid length argument (must be an integer > 0)")
  local from
  if type(steps) == "table" then
    from = pattern.from(steps)
  else
    assert(type(steps) == "number" and steps > 0,
      "invalid step argument (must be an integer > 0)")
    from = pattern.new(steps, 1)
  end
  assert(length >= #from,
    "Invalid length or steps arguments (length must be >= steps")
  return from:spread(length / #from, empty_value):rotate(offset or 0)
end

---Create a new euclidean rhythm pattern with the given pulses or number of new pulses
---in the given length and optionally rotate the contents.
---[Euclidean Rhythm](https://en.wikipedia.org/wiki/Euclidean_rhythm)
---
---### examples:
---```lua
---pattern.euclidean(3, 8)
--- --> {1,0,0,1,0,0,1,0}
---pattern.from{"a", "b", "c"}:euclidean(8, 0, "-")
--- --> {"a","-","-","b","-","-","c","-"}
---```
---@param steps table|integer Existing pattern or number of on steps in the pattern.
---@param length integer Number of total steps in the pattern.
---@param offset integer? Optional rotation offset.
---@param empty_value PulseValue? Value used as off value (by default 0 or guessed from existing content).
function pattern.euclidean(steps, length, offset, empty_value)
  -- get or create initial pulse pattern
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
  -- recursive euclidean pattern impl
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
    local result = pattern.new();
    for _ = 1, length do
      result:push_back(empty_value)
    end
    return result
  elseif #front >= length then
    local result = pattern.new();
    for _ = 1, length do
      result:push_back(1)
    end
    return result
  else
    local back = {}
    for _ = 1, length - #front do
      table.insert(back, { empty_value })
    end
    -- spread
    local rhythms = euclidean_impl(front, back);
    -- convert to pattern and flatten
    local result = pattern.new();
    for _, g in ipairs(rhythms) do
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
--- Access sub ranges
----------------------------------------------------------------------------------------------------

---Shortcut for table.unpack(pattern): returns elements from this pattern as var args.
---
---### examples:
---```lua
---local p = pattern.from{1,2,3,4}
---local v1, v2, v3, v4 = p:unpack()
---```
---@return PulseValue ...
---@nodiscard
function pattern.unpack(self)
  return table.unpack(self)
end

---Get sub range from the pattern as new pattern.
---When the given length is past end of this pattern its filled up with empty values.
---
---### examples:
---```lua
---local p = pattern.from{1,2,3,4}
---p = p:subrange(2,3) --> {2,3}
---p = p:subrange(1,4,"X") --> {2,3,"X","X"}
---```
---@param i integer Subrange start
---@param j integer? Subrange end (defaults to pattern length)
---@param empty_value PulseValue? Value used as empty value (by default 0 or guessed from existing content).
function pattern.subrange(self, i, j, empty_value)
  assert(type(i) == "number" and i > 0,
    "invalid subrange start argument (must be an integer > 0)")
  assert(j == nil or (type(j) == "number" and j > 0),
    "invalid subrange end argument (must be an integer > 0)")
  local len = j or #self
  local a = pattern.new()
  empty_value = empty_value or empty_pulse_value(self)
  for ii = i, len do
    a:push_back(self[ii] or empty_value)
  end
  return a
end

---Get first n items from the pattern as new pattern.
---When the given length is past end of this pattern its filled up with empty values.
---
---### examples:
---```lua
---local p = pattern.from{1,2,3,4}
---p = p:take(2) --> {1,2}
---p = p:take(4, "") --> {1,2,"",""}
---```
---@param length integer
---@param empty_value PulseValue? Value used as empty value (by default 0 or guessed from existing content).
function pattern.take(self, length, empty_value)
  assert(type(length) == "number" and length > 0,
    "invalid length argument (must be an integer > 0)")
  return self:subrange(1, length, empty_value)
end

----------------------------------------------------------------------------------------------------
--- Modify contents
----------------------------------------------------------------------------------------------------

---Clear a pattern, remove all its contents.
---
---### examples:
---```lua
---local p = pattern.from{1,0}
---p:clear() --> {}
---```
function pattern.clear(self)
  while #self > 0 do
    table.remove(self)
  end
  return self
end

---Fill pattern with the given value or generator function in length.
---
---### examples:
---```lua
---local p = pattern.from{0,0}
---p:init(1) --> {1,1}
---p:init("X", 3) --> {"X","X", "X"}
---```
---@param value PulseValue|fun(index: integer):PulseValue
---@param length integer?
function pattern.init(self, value, length)
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

---Apply the given function to every item in the pattern.
---
---### examples:
---```lua
---local p = pattern.from{1,3,5}
---p:map(function(k, v)
---  return scale("c", "minor"):degree(v)
---end) --> {48, 51, 55}
---```
---@param fun fun(index: integer, value: PulseValue): PulseValue
function pattern.map(self, fun)
  local num = #self
  for i = 1, num do
    self[i] = fun(i, self[i])
  end
  return self
end

---Invert the order of items.
---
---### examples:
---```lua
---local p = pattern.from{1,2,3}
---p:reverse() --> {3,2,1}
---```
function pattern.reverse(self)
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
---local p = pattern.from{1,0,0}
---p:rotate(1) --> {0,1,0}
---p:rotate(-2) --> {0,0,1}
---```
---@param amount integer
function pattern.rotate(self, amount)
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

---Push a single or multiple number of items or other pattern contents to the end of the pattern.
---Note: When passing array alike tables or patterns, they will be *unpacked*.
---
---### examples:
---```lua
---local p = pattern.new()
---p:push_back(1) --> {1}
---p:push_back(2,3) --> {1,2,3}
---p:push_back{4} --> {1,2,3,4}
---p:push_back({5,{6,7}) --> {1,2,3,4,5,6,7}
---```
---@param ... PulseValue|(PulseValue)[]
function pattern.push_back(self, ...)
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

---Remove an entry from the back of the pattern. returns the popped item.
---
---### examples:
---```lua
---local p = pattern.from({1,2})
---p:pop_back() --> {1}
---p:pop_back() --> {}
---p:pop_back() --> {}
---```
---@return PulseValue
function pattern.pop_back(self)
  return table.remove(self)
end

---Duplicate the pattern n times.
---
---### examples:
---```lua
---local p = pattern.from{1,2,3}
---patterns:repeat_n(2) --> {1,2,3,1,2,3}
---```
---@param count integer
function pattern.repeat_n(self, count)
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

---Expand (with amount > 1) or shrink (amount < 1) the length of the pattern by the
---given factor, spreading allowed content evenly and filling gaps with 0 or the
---given empty value.
---
---### examples:
---```lua
---local p = pattern.from{1,1}
---p:spread(2) --> {1,0,1,0}
---p:spread(1/2) --> {1,1}
---```
---@param amount number Spread factor (2 = double, 0.5 = half the size).
---@param empty_value PulseValue? Value used as empty value (by default 0 or guessed from existing content).
function pattern.spread(self, amount, empty_value)
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

---Serialze a pattern for display/debugging purposes.
---
---### examples:
---```lua
---pattern.euclidean(3, 8):tostring() --> "{1, 1, 1, 0}"
---```
---@return string
---@nodiscard
pattern.tostring = function(self)
  return table.tostring(self)
end
