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

---Array alike table with helper functions to ease creating rhythmic patterns.
---@class Pattern : table
---@operator add(Pattern|table): Pattern
---@operator mul(number): Pattern
local pattern = {}

----------------------------------------------------------------------------------------------------
--- Pattern creation
----------------------------------------------------------------------------------------------------

---Create a new empty pattern.
---@return Pattern
function pattern.new()
  local t = {}
  return setmetatable(t, {
    ---all table functions can be accessed as member functions
    __index = pattern,
    ---operator + adds two patterns
    __add = function(a, b)
      return a:copy():add(b)
    end,
    ---operator * creates a repeated pattern
    __mul = function(a, b)
      return a:copy():repeat_n(b)
    end
  })
end

---Create a new pattern from a set of values or tables.
---When passing tables, those will be flattened.
---@param ... PulseValue|(PulseValue[])
---@return Pattern
function pattern.from(...)
  return pattern.new():push_back(...)
end

-- create a shallow-copy of the given pattern (or self)
function pattern.copy(self)
  return pattern.from(self)
end

---Create an new pattern or spread and existing pattern evenly within the given length,
---using Bresenham’s line algorithm. Similar, but not exactly like "euclidean".
---@param steps table|integer Existing pattern or number of on steps in the pattern.
---@param length integer Number of total steps in the pattern.
---@param offset integer? Optional rotation offset.
---@param empty_value PulseValue? Value used as empty value (by default 0 or guessed from existing content).
---Shortcut for:
---```lua
---pattern.new():init(1, steps):spread(length / steps):rotate(offset) -- or
---pattern.from{1,1,1}:spread(length / #self):rotate(offset)
---```
function pattern.distributed(steps, length, offset, empty_value)
  assert(type(length) == "number" and length > 0, 
    "invalid length argument (must be an integer > 0)")
  local from
  if type(steps) == "table" then
    from = pattern.from(steps)
  else
    assert(type(steps) == "number" and steps > 0, 
      "invalid step argument (must be an integer > 0)")
    from = pattern.new():init(1, steps)
  end
  assert(length >= #from, 
    "Invalid length or steps arguments (length must be >= steps")
  return from:spread(length / #from, empty_value):rotate(offset or 0)
end

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

---Merge/add two tables (implementation detail).
local function join_tables(a, b)
  local c = { table.unpack(a) }
  for i = 1, #b do
    table.insert(c, b[i])
  end
  return c
end

---Recursive euclidean pattern impl (implementation detail).
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

---Create a new euclidean rhythm pattern with the given pulses or number of new pulses
---in the given length and optionally rotate the contents.
---[Euclidean Rhythm](https://en.wikipedia.org/wiki/Euclidean_rhythm)
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
    assert(type(steps) == "number" and steps > 0,
      "invalid steps argument (must be a table or an integer > 0)")
    for _ = 1, steps do
      table.insert(front, { 1 })
    end
  end
  assert(type(length) == "number" and length > 0, 
    "invalid length argument (expecting an integer > 0)")
  assert(length >= #front, 
    "invalid length or step (length must be >= #pulses)")
  assert(type(offset) == "number" or offset == nil, 
    "invalid offset argument (must be an integer or nil)")
  empty_value = empty_value == nil and empty_pulse_value(steps) or 0
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
    result:rotate(offset)
  end
  return result
end

----------------------------------------------------------------------------------------------------
--- Access sub ranges
----------------------------------------------------------------------------------------------------

---Shortcut for table.unpack(pattern): returns elements from this pattern as var args.
---@return (PulseValue)[]
function pattern.unpack(self)
  return table.unpack(self)
end

---Get sub range from the pattern as new pattern.
---When the given length is past end of this pattern its filled up with empty values.
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
  empty_value = empty_value == nil and empty_pulse_value(self) or 0
  for ii = i, len do
    a:push_back(self[ii] or empty_value)
  end
  return a
end

---Get first n items from the pattern as new pattern.
---@param length integer
function pattern.take(self, length)
  assert(type(length) == "number" and length > 0, 
    "invalid length argument (must be an integer > 0)")
  return self:subrange(1, length)
end

----------------------------------------------------------------------------------------------------
--- Modify contents
----------------------------------------------------------------------------------------------------

---Clear a pattern, remove all its contents.
function pattern.clear(self)
  while #self > 0 do
    table.remove(self)
  end
  return self
end

---Fill pattern with the given value or generator function in length.
---@param value PulseValue|fun(index: integer):PulseValue
---@param length integer
function pattern.init(self, value, length)
  assert(type(length) == "number" and length > 0, 
    "invalid length argument (must be an integer > 0)")
  self:clear()
  if type(value) == 'function' then
    for i = 1, length do
      table.insert(self, value(i))
    end
  else
    for _ = 1, length do
      table.insert(self, value)
    end
  end
  return self
end

---Invert the order of items.
function pattern.reverse(self)
  local num = #self
  for i = 1, math.floor(num / 2) do
    local j = num - (i - 1)
    self[j], self[i] = self[i], self[j]
  end
  return self
end

---Shift contents by the given amount to the left (negative amount) or right.
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

---Push any number of items or other pattern contents to the end of the pattern.
---When passing array alike tables or patterns, they will be unpacked.
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
  for i = 1, select("#", ...) do
    local v = select(i, ...)
    add_unpacked(v)
  end
  return self
end

---Alias for pattern.push_back.
pattern.add = pattern.push_back

---Remove an entry from the back of the pattern and returns the popped item.
---@return PulseValue
function pattern.pop_back(self)
  return table.remove(self)
end

---Duplicate the pattern n times.
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
---@param amount number Spread factor (2 = double, 0.5 = half the size).
---@param empty_value PulseValue? Value used as empty value (by default 0 or guessed from existing content).
function pattern.spread(self, amount, empty_value)
  assert(type(amount) == "number" and amount > 0, 
    "invalid amount argument (must be an integer > 0)")
  empty_value = empty_value == nil and empty_pulse_value(self) or 0
  local old_num = #self
  local old = self:copy()
  self:init(empty_value, old_num * amount)
  for i = 1, old_num do
    self[math.floor((i - 1) * amount + 0.5) + 1] = old[i]
  end
  return self
end

return pattern
