--[[
  Create a chord progression from a scale
--]]

local cmin = scale("c4", "major")

return rhythm {
  unit = "1/2",
  emit = sequence(
    cmin:chord("i"),
    cmin:chord("iv"),
    cmin:chord("vi"),
    cmin:chord("v"),
    cmin:chord("i"),
    cmin:chord("iv"),
    cmin:chord("iii"),
    note(cmin:chord("v")):transpose({-12, })
  ):amplify(0.6)
}
