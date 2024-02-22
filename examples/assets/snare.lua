local fun = require "fun"

return emitter {
  unit = "1/16",
  pattern = fun.cycle { 0, 0, 0, 0, 1, 0, 0, 0 }:take(7 * 8):chain { 0, 0, 0, 1, 0, 0, 1, 0 }:to_table(),
  -- pattern = pattern.from(0, 1):spread(4) * 7 + { 0, 0, 0, 1 } + { 0, 0, 1, 0 },
  emit = note({ key = "C5" }):with_volume(1.4),
}
