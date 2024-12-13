# Gate

A rhythm's [`gate`](../API/rhythm.md#gate) is an optional filter unit that determines whether or not an event should be passed from the [pattern](./pattern.md) to the [emitter](./emitter.md). It can be used to dynamically filter out pulse events.

The default gate is a *threshold gate*, which passes all pulse values > 0. 


## Examples

Seeded probability gate, using the pattern pulse values as probability.

```lua
return rhythm {
  pattern = { 0, { 0.5, 1 }, 1, { 1, 0.8 } },
  gate = function(_init_context)
    local rand = math.randomstate(12366)
    return function(context)
      return context.pulse_value > rand()
    end
  end,
  emit = { "c4" }
}
```

A gate which filters out pulse values with on a configurable threshold.

```lua
return rhythm {
  inputs = { 
    parameter.number("threshold", 0.5, {0, 1}) 
  },
  pattern = { 
    0.2, { 0.5, 1 }, 0.9, { 1, 0.8 } 
  },
  gate = function(context)
    return context.pulse_value >= context.inputs.threshold
  end,
  emit = { "c4" }
}
```

---

See [generators](../extras/generators.md) for more info about stateful generators and [parameters](./parameters.md) about rhythm input parameters. 