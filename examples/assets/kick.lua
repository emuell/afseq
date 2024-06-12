return rhythm {
  unit = "1/4",
  resolution = 16,
  emit = cycle("bd [~ bd] ~ ~ bd [~ bd] _ ~ bd [~ bd] ~ ~ bd [~ bd] [_ bd2] [~ bd _ ~]"):map(
    function(context, value)
      -- print(context.channel, context.step, context.step_length, "->", value)
      if value == "bd" or value == "bd2" then
        return { key = 48, volume = value == "bd" and 0.9 or 0.5 }
      end
    end
  )
}
