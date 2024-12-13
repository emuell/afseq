# Generators

[Patterns](../guide/pattern.md), [Gates](../guide/gate.md) and [Emitters](../guide/emitter.md) can use Lua functions to dynamically generate or evaluate content.

Annonymous Lua functions, as used in rhythms, are actually [closures](https://www.lua.org/pil/6.1.html). They keep a record of their environment, so all (up)values which are declared outside of the annonymous function are accessible from within the function itself. 

We can use this in afseq scripts to keep track of a rhythm's *global* or *local* state.   

### Runtime

To better understand how local and global states are relevant here, we first need to understand how rhythms are evaluated.

Let's say we're in a DAW that supports afseq. This DAW triggers your rhythm script when a single note is triggered. If we now want to allow polyphonic playback of scripts, only *one script instance* is actually created *per instrument or track*. So a *single script* will be *triggered multiple times* with multiple notes. 

This means that all notes triggered by the DAW will share the same global state within a rhythm script. But this also means that in order to create local states for each individual note trigger, you'll need to keep track of a local state somehow.


### Functions

In the following example, an emitter function keeps track of its state by referencing a globally defined `counter` variable.

```lua
local counter = 0
return rhythm {
  emit = function(_context)
    local midi_note = counter 
    counter = (counter + 1) % 128 
    return note(midi_note) 
  end, 
}
```

When playing a single instance of this rhythm, you'll get an event stream of increasing note values. As expected. But when triggering this script multiple times polyphonically, each triggerd script instance increases the counter on its own, so you'll get multiple streams with note values increased by multiple note steps. 

### Contexts

The easiest way to deal with this, is using the function's passed context. Apart from global playback information such as the BPM or sample rate, the context also keeps track of the rhythm's internal playback state. 

A `context` passed to *pattern* functions only contains the global playback status. A `context` passed to *gate and emitter* functions contains the global playback status and status of the pattern.

See [pattern context API](../API/rhythm.md#PatternContext), [gate context API](../API/rhythm.md#GateContext), [emitter context API](../API/rhythm.md#EmitterContext) for details.

Contexts also may contain user controlled input variables. See [parameters](../guide/parameters.md) for more info about this. 

By making use of the context we can now rewrite the example above to:

```lua
return rhythm {
  emit = function(context)
    -- NB: pulse_step is an 1 based index, midi notes start with 0
    local midi_note = (context.pulse_step - 1) % 128
    return note(midi_note)
  end
}
```

Because the context is unique for each newly triggered rhythm instance, we now get multiple continously increasing note event streams again.


### Generators

Generators in afseq are pattern, gate or emit **functions**, that do **return another function**. This is similar to how iterators work in Lua. By returning a function from a function you can create a new local state that is valid for the returned function only. 

Let's use our counter example again with such a *generator*:

```lua
return rhythm {
  emit = function(_init_context)
    local counter = 0 -- local state!
    return function(_context)
      local midi_note = counter
      counter = (counter + 1) % 128 
      return note(midi_note) 
    end
  end, 
}
```

Here the outer function is called *once* when the rhythm is started - just to create the local state and to return the actual emit function. The returned function is then called repeatedly while the rhythm instance is running, operating on the local state it was initialised with.


### When to use what?

- If you have a function that does not depend on an (external) state, simply use a global or anonymous function.

- If you have a function which only depends on the rhythm playback context, use a global or anonymous function too and only make use of the passed context.

- If you need to keep track of local states separately for each new rhythm run, use a generator.

- If you need a mix of local and global state, use a generator which also reaches out to global and local variables. 

---

See also advanced topic about [randomization](./randomization.md), which makes use the the generator concept to keep track of local random states.