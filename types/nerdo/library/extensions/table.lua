---
--- Part of the afseq trait:
--- adds a few extra helper functions to the Lua default table lib
---

----------------------------------------------------------------------------------------------------

---Create a new empty table, or convert an exiting table to an object that uses the global
---'table.XXX' functions as methods, just like strings in Lua do.
---
---### examples:
---```lua
---t = table.new(); t:insert("a"); print(t[1]) -> "a";
---t = table.new{1,2,3}; print(t:concat("|")); -> "1|2|3";
---```
---@param t table?
---@nodiscard
function table.new(t)
  assert(not t or type(t) == 'table', ("bad argument #1 to 'table.new' " ..
    "(table or nil expected, got '%s')"):format(type(t)))
  return setmetatable(t or {}, { __index = _G.table })
end

---Test if the table contains an entry matching the given value,
---starting from element number start_index or 1.
---
---### examples:
---```lua
---t = {"a", "b"}; table.contains(t, "a") --> true
---t = {a=1, b=2}; table.contains(t, 2) --> true
---t = {"a", "b"}; table.contains(t, "c") --> false
---```
---@param t table
---@param value any
---@param start_index integer?
---@return boolean
---@nodiscard
function table.contains(t, value, start_index)
  return table.find(t, value, start_index) ~= nil
end

---Find first match of given value, starting from element
-- number start_index or 1.
---
---Returns the first *key* that matches the value or nil
---
---### examples:
---```lua
---t = {"a", "b"}; table.find(t, "a") --> 1
---t = {a=1, b=2}; table.find(t, 2) --> "b"
---t = {"a", "b", "a"}; table.find(t, "a", 2) --> "3"
---t = {"a", "b"}; table.find(t, "c") --> nil
---```
---@param t table
---@param value any
---@param start_index integer?
---@return any? key
---@nodiscard
function table.find(t, value, start_index)
  assert(type(t) == 'table', ("bad argument #1 to 'table.find' " ..
    "(table expected, got '%s')"):format(type(t)))
  if start_index == nil then
    for k, v in pairs(t) do
      if v == value then
        return k
      end
    end
  else
    for k, v in ipairs(t) do
      if k >= start_index and v == value then
        return k
      end
    end
  end
  return nil
end

---Serialize a table to a string for display/debugging purposes.
---@param t table
---@return string
---@nodiscard
function table.tostring(t)
  assert(type(t) == 'table', ("bad argument #1 to 'table.tostring' " ..
    "(table expected, got '%s')"):format(type(t)))
  local function _value_str(v)
    if "string" == type(v) then
      v = string.gsub(v, "\n", "\\n")
      if string.match(string.gsub(v, "[^'\"]", ""), '^"+$') then
        return "'" .. v .. "'"
      end
      return '"' .. string.gsub(v, '"', '\\"') .. '"'
    else
      return "table" == type(v) and table.tostring(v) or
          tostring(v)
    end
  end
  local function _key_str(k)
    if "string" == type(k) and string.match(k, "^[_%a][_%a%d]*$") then
      return k
    else
      return "[" .. _value_str(k) .. "]"
    end
  end
  local result, done = {}, {}
  for k, v in ipairs(t) do
    table.insert(result, _value_str(v))
    done[k] = true
  end
  for k, v in pairs(t) do
    if not done[k] then
      table.insert(result,
        _key_str(k) .. " = " .. _value_str(v))
    end
  end
  return "{" .. table.concat(result, ", ") .. "}"
end

---Copy the metatable and all elements non recursively into a new table.
---Creates a clone with shared references.
---@param t table
---@nodiscard
function table.copy(t)
  assert(type(t) == 'table', ("bad argument #1 to 'table.copy' " ..
    "(table expected, got '%s')"):format(type(t)))
  local new_table = {}
  for k, v in pairs(t) do
    new_table[k] = v
  end
  return setmetatable(new_table, getmetatable(t))
end

---Backwards compatibility with Lua 5.1.
---@diagnostic disable-next-line: deprecated
table.unpack = table.unpack or unpack
