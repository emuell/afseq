# Timebase

A pattern's [`timebase`](../API/pattern.md#unit) represents the unit of time for the pattern, either in musical beats or wall-clock time (seconds, ms). It defines the unit and duration of a single step in patterns. The time base is static and thus can't be changed during runtime.

The default time unit of pattern is a beat. 

The BPM and signature (beats per bar) settings are configured by the application which is running the pattern. 

## Supported Time Units

### Beat-Time

- `"bars"`  *using the host's beats per bar setting*
- `"beats"` *alias for 1/4*
- `"1/1"` *4 * 1/4*
- `"1/2"` *2 * 1/4*
- `"1/4"` *a beat*
- `"1/8"` *0.5 * 1/4*
- `"1/16"` *0.25 * 1/4*
- `"1/32"` *0.125 * 1/4*
- `"1/64"` *0.0625 * 1/4*

### Wallclock-Time

 - `"ms"` *millisecond*
 - `"seconds"` *whole seconds*

## Resolution

The [`resolution`](../API/pattern.md#resolution) property acts as an additional multiplier to the time unit and can be any positive real number. You can use it to scale the unit or to create odd time signatures.

## Examples

A slightly off beat time unit.
```lua
return pattern {
  unit = "beats", 
  resolution = 1.01,
  event = "c4"
}
```

Sixteenth tripplets.
```lua
return pattern {
  unit = "1/16", 
  resolution = 2/3,
  event = "c4"
}
```

A beat expressed with resolution.
```lua
return pattern {
  unit = "1/1", 
  resolution = 1/4,
  event = "c4"
}
```

2 Seconds.
```lua
return pattern {
  unit = "seconds", 
  resolution = 2,
  event = "c4"
}
```