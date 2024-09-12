# Cycle  

<!-- toc -->
  

---  
## Functions
### map([*self*](../API/builtins/self.md), map : {  }) {#map}
`->`[`Cycle`](../API/Cycle.md)  

> Map names in in the cycle to custom note events.
> 
> By default, strings in cycles are interpreted as notes, and integer values as MIDI note
> values. Custom identifiers such as "bd" are undefined and will result into a rest, when
> they are not mapped explicitly.
> 
> #### examples:
> ```lua
> --Using a fixed mapping table
> cycle("bd [bd, sn]"):map({
>   bd = "c4",
>   sn = "e4 #1 v0.2"
> })
> --Using a fixed mapping table with targets
> cycle("bd:1 <bd:5, bd:7>"):map({
>   bd = { key = "c4", volume = 0.5 }, -- instrument #1,5,7 will be set as specified
> })
> --Using a dynamic map function
> cycle("4 5 4 <5 [4|6]>"):map(function(context, value)
>   -- emit a random note with 'value' as octave
>   return math.random(0, 11) + value * 12
> end)
> --Using a dynamic map function generator
> cycle("4 5 4 <4 [5|7]>"):map(function(context)
>   local notes = scale("c", "minor").notes
>   return function(context, value)
>     -- emit a 'cmin' note arp with 'value' as octave
>     local note = notes[math.imod(context.step, #notes)]
>     local octave = tonumber(value)
>     return { key = note + octave * 12 }
>   end
> end)
> --Using a dynamic map function to map values to chord degrees
> cycle("1 5 1 [6|7]"):map(function(context)
>   local cmin = scale("c", "minor")
>   return function(context, value)
>     return note(cmin:chord(tonumber(value)))
>   end
> end)
> ```  

