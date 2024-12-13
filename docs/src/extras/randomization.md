# Randomization

Controlled randomness can be a lot of fun when creating music algorithmically, so afseq supports a number of randomisation techniques to deal with *pseudo* randomness. 

### Random Number Generation

You can use the standard Lua [`math.random()`](https://www.lua.org/pil/18.html) to create pseudo-random numbers in afseq, and can use [`math.randomseed()`](https://www.lua.org/pil/18.html) to seed them. 

Note that the standard Lua random implementation is overridden by afseq, to use a [Xoshiro256PlusPlus](https://docs.rs/rand_xoshiro/latest/rand_xoshiro/struct.Xoshiro256PlusPlus.html) random number generator. This ensures that seeded random operations behave the same on all platforms and architectures.

Here's a simple example which creates a random melody line based on a scale.

```lua
-- create a scale to pick notes from
local cmin = scale("c", "minor")

-- pick 10 random notes from the scale
local random_notes = pattern.new(10, function()
  return cmin.notes[math.random(#cmin.notes)] 
end)

return rhythm {
  emit = random_notes
}
```

### Random Number Seeding

You can use `math.randomseed()` to seed the global random number generator.

```lua
-- create a scale to pick notes from
local cmin = scale("c", "minor")

-- pick the same random 10 notes from the scale every time
math.randomseed(1234)
local random_notes = pattern.new(10, function() 
  return cmin.notes[math.random(#cmin.notes)] 
end)

return rhythm {
  emit = random_notes
}
```

### Local Random Number Generators

When seeding the RNG, each time a rhythm is (re)started, an existing rhythm instance will continue to run. The global state of a rhythm script is not recreated each time the rhythm is played again. 

See [generators](./generators.md) for details of how afseq handles global and local states in general.

To create multiple separate local random states, use the non standard [`math.randomstate(seed)`](../API/modules/math.md#randomstate) function to create local, possibly seeded random number generators. 

```lua
local cmin = scale("c", "minor")
return rhythm {
  emit = function(context) 
    local rand = math.randomstate(1234) -- a local random number generator
    return function(context) 
      return note(cmin.notes[rand(#cmin.notes)])
    end
  end
}
```

In the example above, each newly triggered rhythm instance will result in the same sequence of *random* notes, and multiple running instances will not interfere with each other.
