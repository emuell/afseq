# CycleMapContext  
> Context passed to 'cycle:map` functions.  

<!-- toc -->
  

---  
## Properties
### beats_per_bar : [`integer`](../API/builtins/integer.md) {#beats_per_bar}
> --Project's beats per bar setting.

### beats_per_min : [`number`](../API/builtins/number.md) {#beats_per_min}
> --Project's tempo in beats per minutes.

### channel : [`integer`](../API/builtins/integer.md) {#channel}
> channel/voice index within the cycle. each channel in the cycle gets emitted and thus mapped
> separately, starting with the first channel index 1.

### inputs : table<[`string`](../API/builtins/string.md), [`boolean`](../API/builtins/boolean.md) | [`string`](../API/builtins/string.md) | [`number`](../API/builtins/number.md)> {#inputs}
> Current input parameter values, using parameter ids as keys
> and the actual parameter value as value.

### playback : [`PlaybackState`](#PlaybackState) {#playback}
> Specifies how the cycle currently is running.

### samples_per_sec : [`integer`](../API/builtins/integer.md) {#samples_per_sec}
> --Project's sample rate in samples per second.

### step : [`integer`](../API/builtins/integer.md) {#step}
> Continues step counter for each channel, incrementing with each new mapped value in the cycle.
> Starts from 1 when the cycle starts running or after it got reset.

### step_length : [`number`](../API/builtins/number.md) {#step_length}
> step length fraction within the cycle, where 1 is the total duration of a single cycle run.

### trigger_note : [`integer`](../API/builtins/integer.md)[`?`](../API/builtins/nil.md) {#trigger_note}
> Note value that triggered, started the rhythm, if any.

### trigger_offset : [`integer`](../API/builtins/integer.md)[`?`](../API/builtins/nil.md) {#trigger_offset}
> Note slice offset value that triggered, started the rhythm, if any.

### trigger_volume : [`number`](../API/builtins/number.md)[`?`](../API/builtins/nil.md) {#trigger_volume}
> Note volume that triggered, started the rhythm, if any.

  

