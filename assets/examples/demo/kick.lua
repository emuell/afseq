local pattern = require "pattern"

return emitter {
  unit = "sixteenth",
  pattern = pattern.from({ 1, 0, 0, 0 }, { 0, 0, 1, 0}, {0, 0, 1, 0}, {0, 0, 0, 0} ),
  emit = {60, 60, {60, "C8 0.135"}},
}
