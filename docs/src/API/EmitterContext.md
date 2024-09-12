# EmitterContext  
> Context passed to 'emit' functions.  

<!-- toc -->
  

---  
## Properties
### beats_per_bar : [`integer`](../API/builtins/integer.md) {#beats_per_bar}
> --Project's beats per bar setting.

### beats_per_min : [`number`](../API/builtins/number.md) {#beats_per_min}
> --Project's tempo in beats per minutes.

### inputs : table<[`string`](../API/builtins/string.md), [`boolean`](../API/builtins/boolean.md) | [`string`](../API/builtins/string.md) | [`number`](../API/builtins/number.md)> {#inputs}
> Current input parameter values, using parameter ids as keys
> and the actual parameter value as value.

### playback : [`PlaybackState`](#PlaybackState) {#playback}
> Specifies how the emitter currently is running.

### pulse_step : [`integer`](../API/builtins/integer.md) {#pulse_step}
> Continues pulse counter, incrementing with each new **skipped or emitted pulse**.
> Unlike `step` in emitter this includes all pulses, so it also counts pulses which do
> not emit events. Starts from 1 when the rhythm starts running or is reset.

### pulse_time : [`number`](../API/builtins/number.md) {#pulse_time}
> Current pulse's step time as fraction of a full step in the pattern. For simple pulses this
> will be 1, for pulses in subdivisions this will be the reciprocal of the number of steps in the
> subdivision, relative to the parent subdivisions pulse step time.
> #### examples:
> ```lua
> {1, {1, 1}} --> step times: {1, {0.5, 0.5}}
> ```

### pulse_time_step : [`number`](../API/builtins/number.md) {#pulse_time_step}
> Continues pulse time counter, incrementing with each new **skipped or emitted pulse**.
> Starts from 0 and increases with each new pulse by the pulse's step time duration.

### pulse_value : [`number`](../API/builtins/number.md) {#pulse_value}
> Current pulse value. For binary pulses this will be 1, 0 pulse values will not cause the emitter
> to be called, so they never end up here.
> Values between 0 and 1 will be used as probabilities and thus are maybe emitted or skipped.

### samples_per_sec : [`integer`](../API/builtins/integer.md) {#samples_per_sec}
> --Project's sample rate in samples per second.

### step : [`integer`](../API/builtins/integer.md) {#step}
> Continues step counter, incrementing with each new *emitted* pulse.
> Unlike `pulse_step` this does not include skipped, zero values pulses so it basically counts
> how often the emit function already got called.
> Starts from 1 when the rhythm starts running or is reset.

### trigger_note : [`integer`](../API/builtins/integer.md)[`?`](../API/builtins/nil.md) {#trigger_note}
> Note value that triggered, started the rhythm, if any.

### trigger_offset : [`integer`](../API/builtins/integer.md)[`?`](../API/builtins/nil.md) {#trigger_offset}
> Note slice offset value that triggered, started the rhythm, if any.

### trigger_volume : [`number`](../API/builtins/number.md)[`?`](../API/builtins/nil.md) {#trigger_volume}
> Note volume that triggered, started the rhythm, if any.

  

