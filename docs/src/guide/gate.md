# Gate

A pattern's [`gate`](../API/pattern.md#gate) is an optional filter unit that determines whether or not an event should be passed from the [pulse](./pulse.md) to the [event](./event.md) stage. It can be used to statically or dynamically filter out pulses.

The default gate is a *threshold gate*, which passes all pulse values > 0. 


## Examples

Seeded probability gate, using the pattern pulse values as probability.

```lua
return pattern {
  pulse = { 0, { 0.5, 1 }, 1, { 1, 0.8 } },
  gate = function(init_context)
    local rand = math.randomstate(12366)
    return function(context)
      return context.pulse_value > rand()
    end
  end,
  event = { "c4" }
}
```

A gate which filters out pulse values with on a configurable threshold.

```lua
return pattern {
  parameter = { 
    parameter.number("threshold", 0.5, {0, 1}) 
  },
  pulse = { 
    0.2, { 0.5, 1 }, 0.9, { 1, 0.8 } 
  },
  gate = function(context)
    return context.pulse_value >= context.parameter.threshold
  end,
  event = { "c4" }
}
```

---

See [generators](../extras/generators.md) for more info about stateful generators and [parameters](./parameters.md) about pattern parameters. 