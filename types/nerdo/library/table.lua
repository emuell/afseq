---
--- Part of the afseq trait:
--- adds a few extra helper functions to the Lua default tablelib
---

----------------------------------------------------------------------------------------------------

---Create a new empty table that uses the global 'table.XXX' functions as methods. 
---### examples:
---```lua
---t = table.new(); t:insert("a"); rprint(t) -> [1] = a;
---```
function table.new()
  return table.create({})
end

---Create a new, or convert an exiting table to an object that uses the global
---'table.XXX' functions as methods, just like strings in Lua do.
---@param t table?
---### examples:
---```lua
---t = table.create(); t:insert("a"); rprint(t) -> [1] = a;
---t = table.create{1,2,3}; print(t:concat("|")); -> "1|2|3";
---```
function table.create(t)
  assert(not t or type(t) == 'table')
  return setmetatable(t or {}, { __index = _G.table })
end

---Recursively clears and removes all table elements.
---@param t table
---@param cleared unknown?
function table.clear(t, cleared)
  assert(type(t) == 'table')
  cleared = cleared or {}
  for k,v in pairs(t) do
    if (type(v) == 'table' and not cleared[v]) then
      cleared[v] = true
      table.clear(v, cleared)
    end
    t[k] = nil
  end
end

---Returns true when the table is empty, else false and will also work
---for non indexed tables
---@param t table
---@return boolean
---### examples:
---```lua
---t = {};          print(table.is_empty(t)); -> true;
---t = {66};        print(table.is_empty(t)); -> false;
---t = {["a"] = 1}; print(table.is_empty(t)); -> false;
function table.is_empty(t)
  return t == nil or next(t) == nil
end

---Find first match of *value* in the given table, starting from element
---number *start_index*.<br>
---Returns the first *key* that matches the value or nil
---@param t table
---@param value any
---@param start_index integer?
---@return any key_or_nil
---### examples:
---```lua
---t = {"a", "b"}; table.find(t, "a") --> 1
---t = {a=1, b=2}; table.find(t, 2) --> "b"
---t = {"a", "b", "a"}; table.find(t, "a", 2) --> "3"
---t = {"a", "b"}; table.find(t, "c") --> nil
---```
function table.find(t, value, start_index)
  local start_index = start_index or 1
  local count = 1
  for k, v in pairs(t) do
    if (count >= start_index and v == value) then
      return k
    end
    count = count + 1
  end
  return nil
end

---Return an indexed table of all keys that are used in the table.
---@param t table
---@return table
---### examples:
---```lua
---t = {a="aa", b="bb"}; rprint(table.keys(t)); --> "a", "b"
---t = {"a", "b"};       rprint(table.keys(t)); --> 1, 2
---```
function table.keys(t)
  local u = {}
  for k,_ in pairs(t) do
    table.insert(u, k)
  end
  return u
end

---Return an indexed table of all values that are used in the table
---@param t table
---@return table
---### examples:
---```lua
--- t = {a="aa", b="bb"}; rprint(table.values(t)); --> "aa", "bb"
--- t = {"a", "b"};       rprint(table.values(t)); --> "a", "b"
---```
function table.values(t)
  local u = {}
  for _,v in pairs(t) do
    table.insert(u, v)
  end
  return u
end

---Deeply copy the metatable and all elements of the given table recursively
---into a new table - create a clone with unique references.
---@param t table
---@return table
function table.rcopy(t)
  assert(type(t) == 'table', ("bad argument #1 to 'table.copy' "..
    "(table expected, got '%s')"):format(type(t)))
  local lookup_table = {}
  local function _copy(object)
    if (type(object) ~= 'table') then
      return object
    elseif (lookup_table[object] ~= nil) then
      return lookup_table[object]
    else
      local new_table = {}
      lookup_table[object] = new_table
      for k, v in pairs(object) do
        new_table[_copy(k)] = _copy(v)
      end
      return setmetatable(new_table, getmetatable(object))
    end
  end
  return _copy(t)
end

---Deeply copy the metatable and all elements of the given table recursively
---into a new table - create a clone with unique references.
---@param t table
---@return table
function table.copy(t)
  assert(type(t) == 'table', ("bad argument #1 to 'table.copy' "..
    "(table expected, got '%s')"):format(type(t)))
  local new_table = {}
  for k, v in pairs(t) do
    new_table[k] = v
  end
  return setmetatable(new_table, getmetatable(t))
end

---Count the number of items of a table, also works for non index
---based tables (using pairs).
---@param t table
---@returns number
---### examples:
---```lua
---t = {["a"]=1, ["b"]=1}; print(table.count(t)) --> 2
---```
function table.count(t)
  assert(type(t) == 'table', ("bad argument #1 to 'table.copy' "..
    "(table expected, got '%s')"):format(type(t)))
  local count = 0
  for _,_ in pairs(t) do
    count = count + 1
  end
  return count
end

---Backwards compatibility with Lua 5.1.
table.unpack = table.unpack or unpack
