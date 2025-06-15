# Generators

[Pulse](../guide/pulse.md), [Gate](../guide/gate.md) and [Event](../guide/event.md) can be specified as Lua functions to dynamically generate or evaluate content.

Anonymous Lua functions, as used in patterns, are actually [closures](https://www.lua.org/pil/6.1.html). They keep a record of their environment, so all (up)values which are declared outside of the anonymous function are accessible from within the function itself. 

We can use this in pattrns scripts to keep track of a pattern's *global* or *local* state.   

### Runtime

To better understand how local and global states are relevant here, we first need to understand how patterns are evaluated.

Let's say we're in a DAW that supports pattrns. This DAW triggers your pattern script when a single note is triggered. If we now want to allow polyphonic playback of scripts, only *one script instance* is actually created *per instrument or track*. So a *single script* will be *triggered multiple times* with multiple notes. 

This means that all notes triggered by the DAW will share the same global state within a pattern script. But this also means that in order to create local states for each individual note trigger, you'll need to keep track of a local state somehow.


### Functions

In the following example, an event function keeps track of its state by referencing a globally defined `counter` variable.

```lua
local counter = 0
return pattern {
  event = function(context)
    local midi_note = counter 
    counter = (counter + 1) % 128 
    return note(midi_note) 
  end, 
}
```

When playing a single instance of this pattern, you'll get an event stream of increasing note values. As expected. But when triggering this script multiple times polyphonically, each triggerd script instance increases the counter on its own, so you'll get multiple streams with note values increased by multiple note steps. 

### Contexts

The easiest way to deal with this, is using the function's passed context. Apart from global playback information such as the BPM or sample rate, the context also keeps track of the pattern's internal playback state. 

A `context` passed to *pattern* functions only contains the global playback status. A `context` passed to *gate and event* functions contains the global playback status and status of the pulse.

See [pattern context API](../API/pattern.md#PulseContext), [gate context API](../API/pattern.md#GateContext), [event context API](../API/pattern.md#EventContext) for details.

Contexts also may contain user controlled input variables. See [parameters](../guide/parameters.md) for more info about this. 

By making use of the context we can now rewrite the example above to:

```lua
return pattern {
  event = function(context)
    -- NB: pulse_step is an 1 based index, midi notes start with 0
    local midi_note = (context.pulse_step - 1) % 128
    return note(midi_note)
  end
}
```

Because the context is unique for each newly triggered pattern instance, we now get multiple continously increasing note event streams again.


### Generators

Generators in pattrns are pattern, gate or emit **functions**, that do **return another function**. This is similar to how iterators work in Lua. By returning a function from a function you can create a new local state that is valid for the returned function only. 

Let's use our counter example again with such a *generator*:

```lua
return pattern {
  event = function(init_context)
    local counter = 0 -- local state!
    return function(context)
      local midi_note = counter
      counter = (counter + 1) % 128 
      return note(midi_note) 
    end
  end, 
}
```

Here the outer function is called *once* when the pattern is started - just to create the local state and to return the actual emit function. The returned function is then called repeatedly while the pattern instance is running, operating on the local state it was initialised with.


### When to use what?

- If you have a function that does not depend on an (external) state, simply use a global or anonymous function.

- If you have a function which only depends on the pattern playback context, use a global or anonymous function too and only make use of the passed context.

- If you need to keep track of local states separately for each new pattern run, use a generator.

- If you need a mix of local and global state, use a generator which also reaches out to global and local variables. 

---

See also advanced topic about [randomization](./randomization.md), which makes use the the generator concept to keep track of local random states.