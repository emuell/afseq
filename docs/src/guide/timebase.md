# Timebase

A rhythm's [`timebase`](../API/rhythm.md#unit) represents the unit of time for the rhythm, either in musical beats or wall-clock time (seconds, ms). It defines the unit and duration of a single step in patterns. The time base is static and thus can't be changed during runtime.

The default time unit of rhythm is a beat. 

The BPM and signature (beats per bar) settings are configured by the application which is running the rhythm. 

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

The [`resolution`](../API/rhythm.md#resolution) parameter acts as an additional multiplier to the time unit and can be any positive real number. You can use it to scale the unit or to create odd time signatures.

## Examples

A slightly off beat time unit.
```lua
rhythm {
  unit = "beats", 
  resolution = 1.01,
  emit = "c4"
}
```

Sixteenth tripplets
```lua
rhythm {
  unit = "1/16", 
  resolution = 4/3,
  emit = "c4"
}
```


2 Seconds
```lua
rhythm {
  unit = "seconds", 
  resolution = 2,
  emit = "c4"
}
```