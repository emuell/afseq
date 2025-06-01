--[[
  Multi channel drum pattern cycle with explicit instrument selection.
--]]

return cycle(
  "[[kd ~]*2 _ [_ kd] ~] ,"..
  "[[_ s1 [_? s2?] <s1 [s1 _ s2|_? s1]>]] ,"..
  "[_ _ cw <[_ cw:d0.08] _>] ,"..
  "[<oh:<v0.5 v0.8> hh hh>*12]"
):map({
  kd = "c4 #11",      -- Kick
  s1 = "c4 #5",       -- Snare
  s2 = "c4 #5 v0.2",  -- Snare
  cw = "c4 #12 v0.4", -- Cowbell
  oh = "c4 #7",       -- Open Hat
  hh = "c4 #6 v0.5",  -- Closed Hat
})